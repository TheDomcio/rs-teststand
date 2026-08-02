//! Deciding whether a handshake's `Origin` is one the host will talk to.
//!
//! RFC 6455 section 10.2 asks a server meant for certain sites, rather than for
//! any web page, to verify the header and answer 403 when it does not like the
//! answer. A station host qualifies. Binding to loopback is not a defense: a
//! page the operator visits can open a socket to the loopback address and drive
//! the station from a script nobody reviewed.

use std::net::SocketAddr;

/// Whether a handshake carrying `origin` may proceed.
///
/// Three rules, in order of how often they decide:
///
/// A request with no `Origin` is served. Browsers always send one and native
/// clients never do, so this is the orchestrator, not the threat the header
/// exists to describe. Refusing it would lock out every non-browser caller to
/// stop an attack that needs a browser to run.
///
/// An origin on `allowed` is served. This is the operator saying which panel
/// they run.
///
/// The address the host itself serves the panel from is served, when it serves
/// one. A host bound to port 0 cannot know its own origin before binding, so it
/// would otherwise have to be told an address it just chose. `served_from` is
/// `None` when no page is served, and then this rule does not apply: without a
/// page the host has vouched for nothing, and its address is just a string
/// someone else can type too.
///
/// Everything else is refused. Matching is exact, so a scheme, a port or a
/// longer domain that merely starts the same is a different origin.
pub(super) fn is_allowed(
    origin: Option<&str>,
    allowed: &[String],
    served_from: Option<SocketAddr>,
) -> bool {
    let Some(origin) = origin else {
        return true;
    };
    if allowed.iter().any(|listed| listed == origin) {
        return true;
    }
    served_from.is_some_and(|address| {
        // Both schemes: the host serves plain HTTP today, and a deployment
        // behind TLS termination presents the same authority over https.
        origin == format!("http://{address}") || origin == format!("https://{address}")
    })
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "panicking is how a unit test reports a failure"
)]
mod tests {
    use super::is_allowed;

    fn address() -> std::net::SocketAddr {
        "127.0.0.1:50751".parse().expect("a literal address")
    }

    #[test]
    fn an_absent_origin_is_served() {
        assert!(is_allowed(None, &[], None));
    }

    #[test]
    fn an_unlisted_origin_is_refused() {
        assert!(!is_allowed(Some("http://evil.example"), &[], None));
    }

    #[test]
    fn a_listed_origin_is_served() {
        let allowed = vec!["http://panel.example".to_owned()];
        assert!(is_allowed(Some("http://panel.example"), &allowed, None));
    }

    #[test]
    fn matching_is_exact_rather_than_a_prefix() {
        // Registering a domain that extends an allowed one is cheap, so a
        // `starts_with` check would be worth nothing.
        let allowed = vec!["http://panel.example".to_owned()];
        for lookalike in [
            "http://panel.example.evil.test",
            "http://panel.example:8080",
            "https://panel.example",
            "http://not-panel.example",
        ] {
            assert!(!is_allowed(Some(lookalike), &allowed, None), "{lookalike}");
        }
    }

    #[test]
    fn the_host_trusts_the_origin_it_serves_its_own_page_from() {
        assert!(is_allowed(
            Some("http://127.0.0.1:50751"),
            &[],
            Some(address())
        ));
        assert!(is_allowed(
            Some("https://127.0.0.1:50751"),
            &[],
            Some(address())
        ));
    }

    #[test]
    fn without_a_page_the_hosts_own_address_earns_nothing() {
        assert!(!is_allowed(Some("http://127.0.0.1:50751"), &[], None));
    }

    #[test]
    fn a_different_port_on_the_same_host_is_a_different_origin() {
        assert!(!is_allowed(
            Some("http://127.0.0.1:50752"),
            &[],
            Some(address())
        ));
    }
}
