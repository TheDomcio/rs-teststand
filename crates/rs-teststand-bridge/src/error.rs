//! What can go wrong when the engine lives on someone else's thread.

/// A failure from the host, as distinct from a failure from the engine.
///
/// The two are worth telling apart. An [`Engine`](Self::Engine) error means the
/// engine refused the work and the host is still serving; the other variants
/// mean the host itself is unavailable, and retrying the same request will not
/// help.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The engine refused the work. The host is unaffected.
    #[error(transparent)]
    Engine(#[from] rs_teststand::Error),

    /// The engine thread could not be started.
    #[error("the engine thread could not be started: {reason}")]
    ThreadNotStarted {
        /// What the operating system reported.
        reason: String,
    },

    /// The engine thread has stopped, so no further work can be run.
    ///
    /// Terminal for this host: build a new one.
    #[error("the engine thread has stopped")]
    HostStopped,

    /// The work was submitted but the thread ended before answering.
    #[error("the engine thread ended before returning a result")]
    ResultLost,

    /// A message's object payload could not be turned into data.
    ///
    /// Separate from [`Engine`](Self::Engine) because the engine answered
    /// perfectly well: what failed was rendering the tree it handed over, and
    /// the rest of the message is still worth delivering.
    #[error("a message payload could not be serialized: {0}")]
    Payload(#[from] serde_json::Error),

    /// The socket carrying events failed.
    ///
    /// Separate again from [`Engine`](Self::Engine): the engine is fine and the
    /// run is unaffected, only the listener is unreachable. A host may well
    /// choose to keep testing and stop reporting.
    #[error("the event transport failed: {0}")]
    Transport(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn an_engine_failure_keeps_its_own_message() {
        // The host must not bury what the engine said behind wording of its
        // own; a caller debugging a refused call needs the engine's text.
        let engine = rs_teststand::Error::UnexpectedType {
            expected: "a number",
            actual: "a string",
        };
        let expected = engine.to_string();
        let wrapped = Error::from(engine);
        assert_eq!(wrapped.to_string(), expected);
    }

    #[test]
    fn a_payload_failure_is_not_reported_as_an_engine_failure() {
        // The distinction matters to a host deciding whether to retry: the
        // engine is fine, only this message's object could not be rendered.
        let parsed = serde_json::from_str::<serde_json::Value>("{");
        assert!(parsed.is_err(), "an unterminated object is not valid JSON");
        if let Err(broken) = parsed {
            let wrapped = Error::from(broken);
            assert!(matches!(wrapped, Error::Payload(_)), "got {wrapped:?}");
            assert!(wrapped.to_string().starts_with("a message payload"));
        }
    }

    #[test]
    fn a_stopped_host_says_so_without_blaming_the_engine() {
        assert_eq!(
            Error::HostStopped.to_string(),
            "the engine thread has stopped"
        );
    }
}
