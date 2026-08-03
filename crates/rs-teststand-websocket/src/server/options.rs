//! What a host decides before it binds.

/// How a host serves: what it hands a browser, and who it will talk to.
///
/// Built by chaining, and every field has a default that works, so a host only
/// names what it cares about.
///
/// ```
/// use rs_teststand_websocket::Options;
///
/// let options = Options::default()
///     .page(include_str!("../../examples/panel.html"))
///     .allow_origin("http://192.0.2.10:50751");
/// ```
#[derive(Debug, Default, Clone)]
pub struct Options {
    /// Served to a browser asking for the root, when set.
    pub(super) page: Option<String>,
    /// Origins allowed to open a socket, beyond the two rules in
    /// [`super::origin::is_allowed`].
    pub(super) allowed_origins: Vec<String>,
}

impl Options {
    /// Serves `html` to a browser that asks for the root.
    ///
    /// One address for the panel and the socket, so opening the host's address
    /// is the whole setup. It also gives them the same origin, which is what
    /// lets the host trust its own panel without being told an address it picks
    /// at bind time. A page loaded from disk instead has an origin of `null`
    /// and gets no such trust.
    #[must_use]
    pub fn page(mut self, html: impl Into<String>) -> Self {
        self.page = Some(html.into());
        self
    }

    /// Allows `origin` to open a socket.
    ///
    /// Written the way a browser sends it, scheme and authority with no
    /// trailing slash, such as `http://192.0.2.10:50751`. Matching is exact.
    ///
    /// Needed only for a panel the host does not serve itself, since a page it
    /// serves already shares its origin and a native client sends no origin at
    /// all. Call it more than once to allow several.
    #[must_use]
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }
}
