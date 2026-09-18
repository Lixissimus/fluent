use std::io;

#[derive(Debug, thiserror::Error)]
pub enum ReceiveError {
    #[error("Unix socket I/O failed: {0}")]
    Io(#[from] io::Error),

    #[error("could not decode IPC message: {0}")]
    Json(#[from] serde_json::Error),

    #[error("received an unterminated IPC message")]
    UnterminatedMessage,
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Unix socket I/O failed: {0}")]
    Io(#[from] io::Error),

    #[error("could not encode IPC message: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Unix socket I/O failed: {0}")]
    Io(#[from] io::Error),
}
