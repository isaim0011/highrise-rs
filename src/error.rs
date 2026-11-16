use thiserror::Error;

/// Errors returned by the Highrise SDK transport and API calls.
#[derive(Debug, Error)]
pub enum Error {
    #[error("websocket error: {0}")]
    WebSocket(#[from] Box<tokio_tungstenite::tungstenite::Error>),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("web api rate limited (retry after {retry_after:?}s)")]
    RateLimited { retry_after: Option<u64> },

    #[error("web api error ({status}): {message}")]
    WebApi { status: u16, message: String },

    #[error("invalid json payload: {0}")]
    Json(#[from] serde_json::Error),

    #[error("request timed out waiting for server ack")]
    Timeout,

    #[error("highrise server error: {0}")]
    Server(String),

    #[error("not connected to room")]
    NotConnected,

    #[error("fatal auth or room permission error: {0}")]
    Fatal(String),

    #[error("invalid websocket configuration: {0}")]
    Config(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(err))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that cannot be fixed by reconnecting (bad token, missing designer rights, etc.).
pub fn is_fatal_server_error(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("invalid token")
        || lower.contains("unauthorized")
        || lower.contains("not allowed")
        || lower.contains("designer")
        || lower.contains("room not found")
}
