//! Owning the engine on its own thread so many callers can share it.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tokio::sync::{broadcast, oneshot};

use rs_teststand::Engine;

use crate::event::{MessageEvent, PayloadPolicy};

use crate::Error;

/// How many messages the broadcast channel holds before the slowest subscriber
/// starts missing them.
///
/// A subscriber that falls this far behind is told how many it lost rather than
/// being silently starved.
const MESSAGE_BACKLOG: usize = 1024;

/// How long the engine thread waits between queue polls when idle.
///
/// Short enough that a message is forwarded promptly, long enough that an idle
/// host is not spinning a core.
const IDLE_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// How long to let the engine finish shutting down before giving up on it.
///
/// Bounded on purpose: a host that cannot be stopped is worse than one that
/// stops untidily.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// A unit of work to run against the engine, on the engine's own thread.
type Job = Box<dyn FnOnce(&Engine) + Send>;

/// A shared, thread-safe front end to one engine.
///
/// The engine is a single-threaded-apartment COM object: its wrappers are
/// deliberately neither [`Send`] nor [`Sync`], and driving one from several
/// threads does not work. A server, though, is inherently multi-threaded.
///
/// This resolves that by giving the engine a thread of its own. Callers submit
/// work; the engine thread runs it and sends the answer back. Because the work
/// is a closure rather than a fixed command list, the whole API is reachable
/// without enumerating it:
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use rs_teststand_bridge::EngineHost;
///
/// let host = EngineHost::start()?;
/// let version = host.with_engine(|engine| engine.version_string()).await??;
/// println!("TestStand {version}");
/// # Ok(())
/// # }
/// ```
///
/// The same thread polls the message queue and broadcasts what it finds, so a
/// host can serve requests and forward progress at once — which is what an IPC
/// layer needs and what a single-threaded caller cannot do.
#[derive(Debug)]
pub struct EngineHost {
    jobs: mpsc::Sender<Job>,
    messages: broadcast::Sender<MessageEvent>,
    /// Kept so the thread is joined when the host is dropped.
    worker: Option<thread::JoinHandle<()>>,
}

impl EngineHost {
    /// Starts an engine on a dedicated thread.
    ///
    /// Message polling is enabled, so anything a sequence posts reaches
    /// [`subscribe`](Self::subscribe) without further setup.
    ///
    /// # Errors
    /// [`Error`] if the engine cannot be created.
    pub fn start() -> Result<Self, Error> {
        let (job_sender, job_receiver) = mpsc::channel::<Job>();
        let (message_sender, _) = broadcast::channel(MESSAGE_BACKLOG);
        let (ready_sender, ready_receiver) = mpsc::channel::<Result<(), Error>>();

        let messages = message_sender.clone();
        let worker = thread::Builder::new()
            .name("rs-teststand-engine".to_owned())
            .spawn(move || Self::run(&job_receiver, &messages, &ready_sender))
            .map_err(|source| Error::ThreadNotStarted {
                reason: source.to_string(),
            })?;

        // Surface a construction failure to the caller rather than leaving a
        // dead thread behind a healthy-looking handle.
        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                jobs: job_sender,
                messages: message_sender,
                worker: Some(worker),
            }),
            Ok(Err(error)) => Err(error),
            // The thread dropped its side without reporting: it is not running.
            Err(_) => Err(Error::HostStopped),
        }
    }

    /// Runs a closure against the engine and waits for its result.
    ///
    /// The closure runs on the engine's thread, so it may use any part of the
    /// API. Its result must be [`Send`] to come back.
    ///
    /// The outer error means the engine thread is gone; the inner one is
    /// whatever the closure produced.
    ///
    /// # Errors
    /// [`Error`] if the engine thread has stopped.
    pub async fn with_engine<F, T>(&self, work: F) -> Result<T, Error>
    where
        F: FnOnce(&Engine) -> T + Send + 'static,
        T: Send + 'static,
    {
        let (result_sender, result_receiver) = oneshot::channel();
        let job: Job = Box::new(move |engine| {
            // A closed receiver only means the caller stopped waiting.
            let _ = result_sender.send(work(engine));
        });
        self.jobs.send(job).map_err(|_| Error::HostStopped)?;
        result_receiver.await.map_err(|_| Error::ResultLost)
    }

    /// Receives every message posted from now on.
    ///
    /// Each subscriber gets its own copy. One that falls further behind than
    /// the backlog is told how many it missed rather than quietly losing them.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<MessageEvent> {
        self.messages.subscribe()
    }

    /// How many subscribers are currently listening.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.messages.receiver_count()
    }

    /// The engine thread: create, then serve work and drain the queue.
    fn run(
        jobs: &mpsc::Receiver<Job>,
        messages: &broadcast::Sender<MessageEvent>,
        ready: &mpsc::Sender<Result<(), Error>>,
    ) {
        let engine = match Engine::new() {
            Ok(engine) => engine,
            Err(error) => {
                let _ = ready.send(Err(error.into()));
                return;
            }
        };
        // Nothing reaches the queue until this is on.
        if let Err(error) = engine.set_ui_message_polling_enabled(true) {
            let _ = ready.send(Err(error.into()));
            return;
        }
        if ready.send(Ok(())).is_err() {
            return;
        }

        loop {
            // The engine is a single-threaded-apartment object and this is a
            // background thread, where nothing pumps by default. Without this
            // the apartment cannot service the calls an execution makes and the
            // process aborts rather than failing cleanly.
            if rs_teststand_sys::pump_thread_messages() {
                Self::close_apartment(engine);
                return;
            }

            // Messages next: a synchronous one blocks the sequence that posted
            // it until acknowledged, so draining beats serving a request.
            Self::drain_messages(&engine, messages);

            match jobs.recv_timeout(IDLE_POLL_INTERVAL) {
                Ok(job) => job(&engine),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                // Every handle is gone: drain once more so a final message is
                // not lost, then let the engine drop with the thread.
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    Self::drain_messages(&engine, messages);
                    // ShutDown is asynchronous and must be waited on: dropping
                    // the engine while it is still closing files and
                    // terminating executions tears COM down underneath work in
                    // progress. Bounded, so a stuck engine cannot wedge the
                    // host thread.
                    Self::close_apartment(engine);
                    return;
                }
            }
        }
    }

    /// Releases the engine and leaves the apartment, in that order.
    ///
    /// This thread initialized a single-threaded apartment and is about to
    /// exit. Unlike the process's main thread it genuinely detaches, so the
    /// apartment has to be closed or the COM runtime is left believing a live
    /// thread still owns it.
    ///
    /// Order is the whole point: the engine is dropped first, because
    /// uninitializing COM while an object is still alive aborts the process.
    fn close_apartment(engine: Engine) {
        let _ = engine.close(SHUTDOWN_TIMEOUT);
    }

    /// Takes everything currently queued and publishes it.
    ///
    /// Every message is acknowledged even when nobody is subscribed: the
    /// acknowledgement is what releases a synchronous poster, so skipping it
    /// would stall the sequence rather than merely drop a notification.
    fn drain_messages(engine: &Engine, messages: &broadcast::Sender<MessageEvent>) {
        while matches!(engine.is_ui_message_queue_empty(), Ok(false)) {
            let Ok(message) = engine.get_ui_message() else {
                return;
            };
            // A conversion failure must not skip the acknowledgement below: an
            // unacknowledged message stalls a synchronous poster.
            if let Ok(event) = MessageEvent::from_ui_message(&message, PayloadPolicy::default()) {
                // A send failure only means nobody is listening yet.
                let _ = messages.send(event);
            }
            let _ = message.acknowledge();
        }
    }
}

impl Drop for EngineHost {
    fn drop(&mut self) {
        // Dropping the sender ends the loop; joining lets the engine release
        // its COM references on the thread that created them.
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand::UIMessageCode;

    use crate::event::MessageEvent;

    fn event(code: i32) -> MessageEvent {
        MessageEvent {
            code,
            numeric: 0.0,
            text: String::new(),
            payload: None,
            synchronous: false,
            execution_id: None,
        }
    }

    #[test]
    fn a_sequence_posted_event_is_distinguishable_from_an_engine_one() {
        assert!(event(UIMessageCode::USER_MESSAGE_BASE + 1).is_from_sequence());
        assert!(!event(UIMessageCode::EndExecution.bits()).is_from_sequence());
    }

    #[test]
    fn an_engine_event_resolves_to_its_name() {
        assert_eq!(
            event(UIMessageCode::EndExecution.bits()).engine_code(),
            Some(UIMessageCode::EndExecution)
        );
        // A sequence's own code has no engine name, and that is not a failure.
        assert_eq!(event(UIMessageCode::USER_MESSAGE_BASE).engine_code(), None);
    }
}
