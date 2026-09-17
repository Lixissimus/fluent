use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unix socket I/O failed: {0}")]
    Io(#[from] io::Error),

    #[error("could not encode or decode IPC message: {0}")]
    Json(#[from] serde_json::Error),

    #[error("received an unterminated IPC message")]
    UnterminatedMessage,
}

pub type Result<T> = std::result::Result<T, Error>;
