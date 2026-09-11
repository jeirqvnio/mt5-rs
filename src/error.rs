use std::time::Duration;

/// The result of every call in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// What can go wrong between here and the terminal.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("`{method}` timed out after {elapsed:?}")]
    Timeout {
        method: &'static str,
        elapsed: Duration,
    },

    /// The terminal answered status 0, or an empty body where a record was
    /// required (unknown symbol, no tick).
    #[error("terminal refused: {0}")]
    Refused(String),

    /// The bytes do not match the layout this crate was derived from.
    #[error("protocol: {0}")]
    Protocol(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("not connected")]
    NotConnected,

    #[error("login {login} on {server} did not connect within {waited:?}")]
    LoginFailed {
        login: i64,
        server: String,
        waited: Duration,
    },

    /// A trade command failed before a verdict came back. The order may have
    /// reached the server: reconcile against history before resending.
    #[error("`{method}` failed with the outcome UNKNOWN: {source}")]
    OutcomeUnknown {
        method: &'static str,
        #[source]
        source: Box<Error>,
    },
}

impl Error {
    /// Whether re-issuing the same read is safe.
    pub(crate) fn is_transient(&self) -> bool {
        matches!(self, Error::Io(_) | Error::Timeout { .. })
    }
}
