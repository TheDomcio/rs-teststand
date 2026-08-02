//! The engine thread: owns the engine, answers panels, forwards what runs.
//!
//! This is the loop a real host is built around, and the reason the example
//! exists. It does three things in turn, forever:
//!
//! 1. **Answer commands.** Whatever the panels asked for since the last pass.
//! 2. **Pump.** Dispatch this thread's window messages, or COM cannot deliver
//!    to this apartment.
//! 3. **Drain and publish.** Take the engine's messages, flatten each one, and
//!    broadcast it.
//!
//! All of it on one thread, which owns the engine for its whole life. The
//! server is elsewhere; only data crosses between them.

use std::time::{Duration, Instant};

use rs_teststand::{
    ConflictHandler, Engine, Execution, GetSeqFileOptions, SequenceFile, UIMessageCode,
    pump_thread_messages,
};
use rs_teststand_bridge::{ClientTimeout, ClientWatch, Command, MessageEvent, PayloadPolicy, Response, WatchState};
use rs_teststand_websocket::{Request, WebSocketBridge};
use rs_teststand_serde::PropertyObjectValue as _;

use crate::demo_sequence;

/// What a panel wants out of a context. "Everything" is the one request that
/// cannot be served, so the host answers a named list instead.
const REQUESTED: [&str; 3] = ["Locals", "Parameters", "FileGlobals"];

/// How long the host waits for a run to unwind after it has been called off.
///
/// Bounded, because this is the path taken when nobody is left to complain: an
/// unbounded wait here would leave the very process the timeout exists to end.
const UNWIND_TIMEOUT: Duration = Duration::from_secs(20);

/// How long the loop sleeps when there is nothing to do.
///
/// Short enough that a panel sees progress promptly, long enough that an idle
/// station is not spinning a core.
const IDLE: Duration = Duration::from_millis(30);

/// One run in flight, with the file it came from.
///
/// Held together because releasing the file before the run ends leaves the
/// engine waiting on a thread nobody is pumping for.
struct Running {
    execution: Execution,
    sequence_file: SequenceFile,
}

/// The host loop.
pub struct Orchestrator {
    engine: Engine,
    bridge: WebSocketBridge,
    running: Option<Running>,
    /// A file opened by `load_file` and not yet started.
    ///
    /// Held between the two commands so a panel can load, see whether it
    /// worked, and only then offer a run button.
    loaded: Option<SequenceFile>,
    /// The dead-man's switch.
    ///
    /// If the orchestrator dies, its socket closes and nothing else happens.
    /// Without this the host would keep a test running with nobody able to stop
    /// it, which on a station wired to hardware is the failure that matters.
    watch: ClientWatch,
}

impl Orchestrator {
    /// Creates the engine and binds the bridge.
    ///
    /// # Errors
    /// Any failure creating the engine or binding the socket.
    pub fn new(address: &str, timeout: ClientTimeout) -> Result<Self, Box<dyn std::error::Error>> {
        let bridge = WebSocketBridge::bind(address)?;
        let engine = Engine::new()?;
        engine.set_ui_message_polling_enabled(true)?;
        Ok(Self {
            engine,
            bridge,
            running: None,
            loaded: None,
            // The clock runs from now, not from the first connection: an
            // orchestrator that dies before it ever connects must not leave the
            // host waiting for a client that is never coming.
            watch: ClientWatch::new(timeout, Instant::now()),
        })
    }

    /// Where panels should connect.
    #[must_use]
    pub fn address(&self) -> String {
        format!("ws://{}", self.bridge.address())
    }

    /// Runs until a panel asks the host to stop.
    ///
    /// # Errors
    /// Any failure the engine reports that the loop cannot answer locally.
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            while let Some(request) = self.bridge.next_command() {
                println!("command: {}", request.command.name());
                if matches!(request.command, Command::Shutdown) {
                    self.bridge.reply(
                        &request,
                        &Response::Done {
                            command: "shutdown".to_owned(),
                        },
                    );
                    return Ok(());
                }
                let response = self.answer(&request);
                self.bridge.reply(&request, &response);
            }

            let _ = pump_thread_messages();
            self.forward_messages()?;

            if self
                .watch
                .observe(self.bridge.client_count(), Instant::now())
                == WatchState::Expired
            {
                println!("no client for the configured timeout; stopping the station");
                self.stop_everything()?;
                return Ok(());
            }
            std::thread::sleep(IDLE);
        }
    }

    /// Stops every run and waits for it, so nothing is left mid-operation.
    ///
    /// Terminate rather than abort: cleanup groups still run, so a fixture is
    /// powered down and hardware is left safe. That is the whole point of doing
    /// this instead of just exiting the process.
    fn stop_everything(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.engine.terminate_all()?;

        // Wait for the engine to say the runs ended. Releasing a file or
        // dropping the engine mid-unwind leaves it waiting on a thread nobody
        // is pumping for, and the process never exits.
        let deadline = Instant::now() + UNWIND_TIMEOUT;
        while Instant::now() < deadline && self.running.is_some() {
            let _ = pump_thread_messages();
            self.forward_messages()?;
            std::thread::sleep(IDLE);
        }
        if self.running.is_some() {
            println!("a run did not finish within {UNWIND_TIMEOUT:?}; exiting anyway");
        }
        if let Some(file) = self.loaded.take() {
            let _ = self.engine.release_sequence_file_ex(file, 0);
        }
        Ok(())
    }

    /// Answers one command, on the engine's own thread.
    fn answer(&mut self, request: &Request) -> Response {
        let name = request.command.name().to_owned();
        match &request.command {
            Command::Hello => self.hello(),
            Command::Login {
                user_name,
                password,
            } => self.login(user_name, password),
            Command::Logout => self.logout(),
            Command::LoadFile { path } => self.load_file(path),
            Command::Start { sequence } => self.start_loaded(sequence),
            Command::Run {
                sequence_file,
                sequence,
            } => self.start(sequence_file, sequence),
            Command::Terminate { execution_id } => self.terminate(*execution_id),
            Command::ReadValue {
                execution_id,
                lookup,
            } => self.read_value(*execution_id, lookup),
            // Handled before this is called.
            Command::Shutdown => Response::Done { command: name },
            // The command set is non-exhaustive, so a newer panel talking to an
            // older host is told plainly rather than ignored.
            _ => Response::Failed {
                command: name,
                reason: "this host does not implement that command".to_owned(),
            },
        }
    }

    fn hello(&self) -> Response {
        match (self.engine.version_string(), self.engine.is_64bit()) {
            (Ok(engine), Ok(is_64bit)) => Response::Hello { engine, is_64bit },
            (Err(error), _) | (_, Err(error)) => Response::Failed {
                command: "hello".to_owned(),
                reason: error.to_string(),
            },
        }
    }

    /// Logs a user in by name, after checking the password.
    ///
    /// The engine's login is a property write and checks nothing, so the check
    /// happens here: look the account up, validate, and only then make it
    /// current. An account with no password takes an empty string, which is
    /// ordinary on a station where the operator is identified but not
    /// authenticated.
    fn login(&self, user_name: &str, password: &str) -> Response {
        let failed = |reason: String| Response::Failed {
            command: "login".to_owned(),
            reason,
        };
        let found = match self.engine.get_user(user_name) {
            Ok(found) => found,
            Err(error) => return failed(error.to_string()),
        };
        let Some(user) = found else {
            return failed(format!("no account named {user_name:?} on this station"));
        };
        match user.validate_password(password) {
            Ok(true) => {}
            Ok(false) => return failed("the password does not match".to_owned()),
            Err(error) => return failed(error.to_string()),
        }
        if let Err(error) = self.engine.set_current_user(Some(&user)) {
            return failed(error.to_string());
        }
        Response::LoggedIn {
            user_name: user.login_name().unwrap_or_default(),
            full_name: user.full_name().unwrap_or_default(),
        }
    }

    /// Clears the current user, which the engine treats as logging out.
    fn logout(&self) -> Response {
        self.engine.set_current_user(None).map_or_else(
            |error| Response::Failed {
                command: "logout".to_owned(),
                reason: error.to_string(),
            },
            |()| Response::Done {
                command: "logout".to_owned(),
            },
        )
    }

    /// Opens a file and holds it, without running anything.
    fn load_file(&mut self, path: &str) -> Response {
        let failed = |reason: String| Response::Failed {
            command: "load_file".to_owned(),
            reason,
        };
        if self.running.is_some() {
            return failed("a run is in flight; terminate it first".to_owned());
        }
        let opened = if path.is_empty() {
            demo_sequence::build(&self.engine)
        } else {
            self.engine.get_sequence_file_ex(
                path,
                GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
                ConflictHandler::Error,
            )
        };
        match opened {
            Ok(file) => {
                let resolved = file
                    .path()
                    .unwrap_or_else(|_| "<built in memory>".to_owned());
                let sequences = file.num_sequences().unwrap_or(0);
                // Replacing a previously loaded file releases it, or the engine
                // keeps a load reference nobody will ever drop.
                if let Some(previous) = self.loaded.replace(file) {
                    let _ = self.engine.release_sequence_file_ex(previous, 0);
                }
                Response::Loaded {
                    path: resolved,
                    sequences,
                }
            }
            Err(error) => failed(error.to_string()),
        }
    }

    /// Starts the file already loaded.
    fn start_loaded(&mut self, sequence: &str) -> Response {
        let failed = |reason: String| Response::Failed {
            command: "start".to_owned(),
            reason,
        };
        if self.running.is_some() {
            return failed("a run is already in flight; terminate it first".to_owned());
        }
        let Some(sequence_file) = self.loaded.take() else {
            return failed("load a file first".to_owned());
        };
        match self
            .engine
            .new_execution(&sequence_file, sequence, None, false, 0)
        {
            Ok(execution) => {
                let id = execution.id().unwrap_or(-1);
                self.running = Some(Running {
                    execution,
                    sequence_file,
                });
                Response::Started { execution_id: id }
            }
            Err(error) => {
                // Put it back, so a failed start does not lose the file.
                self.loaded = Some(sequence_file);
                failed(error.to_string())
            }
        }
    }

    /// Starts a run. An empty path means the sequence this example builds.
    fn start(&mut self, path: &str, sequence: &str) -> Response {
        let failed = |reason: String| Response::Failed {
            command: "run".to_owned(),
            reason,
        };
        if self.running.is_some() {
            return failed("a run is already in flight; terminate it first".to_owned());
        }
        let opened = if path.is_empty() {
            demo_sequence::build(&self.engine)
        } else {
            self.engine.get_sequence_file_ex(
                path,
                GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
                ConflictHandler::Error,
            )
        };
        let sequence_file = match opened {
            Ok(file) => file,
            Err(error) => return failed(error.to_string()),
        };
        match self
            .engine
            .new_execution(&sequence_file, sequence, None, false, 0)
        {
            Ok(execution) => {
                let id = execution.id().unwrap_or(-1);
                self.running = Some(Running {
                    execution,
                    sequence_file,
                });
                Response::Started { execution_id: id }
            }
            Err(error) => failed(error.to_string()),
        }
    }

    fn terminate(&mut self, execution_id: i32) -> Response {
        let failed = |reason: String| Response::Failed {
            command: "terminate".to_owned(),
            reason,
        };
        let Some(running) = &self.running else {
            return failed("nothing is running".to_owned());
        };
        if running.execution.id().unwrap_or(-1) != execution_id {
            return failed(format!("execution {execution_id} is not the one running"));
        }
        running.execution.terminate().map_or_else(
            |error| failed(error.to_string()),
            |()| Response::Done {
                command: "terminate".to_owned(),
            },
        )
    }

    /// Resolves a property path in the running execution and answers with data.
    ///
    /// The reference itself could never leave the process, so what goes back is
    /// the subtree serialized.
    fn read_value(&self, execution_id: i32, lookup: &str) -> Response {
        let failed = |reason: String| Response::Failed {
            command: "read_value".to_owned(),
            reason,
        };
        let Some(running) = &self.running else {
            return failed("nothing is running".to_owned());
        };
        if running.execution.id().unwrap_or(-1) != execution_id {
            return failed(format!("execution {execution_id} is not the one running"));
        }
        // Read through the thread's sequence context, not the execution's own
        // property tree. `Locals` and `FileGlobals` live in the context; the
        // execution root has neither, and asking it for one comes back as
        // TS_Err_UnknownVariableOrProperty.
        let resolved = running
            .execution
            .get_thread(0)
            .and_then(|thread| thread.get_sequence_context(0))
            .and_then(|context| context.as_property_object())
            .and_then(|root| root.get_property_object(lookup, 0))
            .and_then(|subtree| subtree.to_value());
        match resolved {
            Ok(value) => serde_json::to_string(&value).map_or_else(
                |error| failed(error.to_string()),
                |value| Response::Value {
                    lookup: lookup.to_owned(),
                    value,
                },
            ),
            Err(error) => failed(error.to_string()),
        }
    }

    /// Takes everything queued and broadcasts it, then tidies a finished run.
    fn forward_messages(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while !self.engine.is_ui_message_queue_empty()? {
            let message = self.engine.get_ui_message()?;
            let code = message.event()?;
            let event = self.to_event(&message, code)?;
            self.bridge.publish(&event);

            if matches!(
                UIMessageCode::from_bits(code),
                Ok(UIMessageCode::EndExecution)
            ) {
                self.finish()?;
            }
            message.acknowledge()?;
        }
        Ok(())
    }

    /// Flattens one message, resolving a context into named subtrees.
    ///
    /// The crate does the flattening. The only thing left here is the context
    /// case: it arrives with no payload, because walking it whole is refused
    /// rather than fatal.
    fn to_event(
        &self,
        message: &rs_teststand::UIMessage,
        code: i32,
    ) -> Result<MessageEvent, Box<dyn std::error::Error>> {
        let mut event = MessageEvent::from_ui_message(message, PayloadPolicy::default())?;
        if code != demo_sequence::WHOLE_CONTEXT || event.payload.is_some() {
            return Ok(event);
        }
        if let Some(context) = message.activex_data()? {
            let mut resolved = serde_json::Map::new();
            for name in REQUESTED {
                let rendered = context
                    .get_property_object(name, 0)
                    .and_then(|subtree| subtree.to_value())
                    .map_or_else(
                        // Reported rather than hidden: a panel asking for
                        // something absent should be told.
                        |error| serde_json::Value::String(error.to_string()),
                        |value| serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
                    );
                resolved.insert(name.to_owned(), rendered);
            }
            event.payload = Some(serde_json::to_string(&resolved)?);
            event.text = "resolved subtrees of the sequence context".to_owned();
        }
        Ok(event)
    }

    /// Publishes the result table and releases the finished run.
    fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(running) = self.running.take() else {
            return Ok(());
        };
        let results: Vec<serde_json::Value> = running
            .execution
            .result_list()
            .and_then(|list| list.parse())
            .map(|steps| {
                steps
                    .iter()
                    .map(|step| {
                        serde_json::json!({
                            "name": step.name,
                            "type": step.step_type,
                            "status": step.status,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let status = running.execution.result_status().unwrap_or_default();

        self.bridge.publish(&MessageEvent {
            code: demo_sequence::SUMMARY,
            numeric: 100.0,
            text: format!("run finished: {status}"),
            payload: serde_json::to_string(&serde_json::json!({ "results": results })).ok(),
            synchronous: false,
            execution_id: running.execution.id().ok(),
        });
        println!("run finished: {status}");

        // Released only now the engine has reported the end.
        self.engine
            .release_sequence_file_ex(running.sequence_file, 0)?;
        Ok(())
    }
}
