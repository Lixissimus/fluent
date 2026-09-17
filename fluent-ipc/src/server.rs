use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{
    io::{AsyncBufReadExt, BufReader, ReadHalf},
    net::{UnixListener, UnixStream},
};

use crate::{Error, Message, Result};

pub struct Aggregator {
    listener: UnixListener,
    path: Arc<PathBuf>,
}

impl Aggregator {
    pub async fn bind(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_owned();
        remove_stale_socket(&path).await?;
        let listener = UnixListener::bind(&path)?;
        Ok(Self {
            listener,
            path: Arc::new(path),
        })
    }

    pub async fn accept(&self) -> Result<Connection> {
        let (stream, _) = self.listener.accept().await?;
        Ok(Connection::new(stream))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

async fn remove_stale_socket(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    match UnixStream::connect(path).await {
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "another aggregator is already listening",
        )
        .into()),
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
            std::fs::remove_file(path)?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

impl Drop for Aggregator {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.path.as_ref());
    }
}

pub struct Connection {
    reader: BufReader<ReadHalf<UnixStream>>,
}

impl Connection {
    fn new(stream: UnixStream) -> Self {
        let (read_half, _) = tokio::io::split(stream);
        Self {
            reader: BufReader::new(read_half),
        }
    }

    pub async fn next_message(&mut self) -> Result<Option<Message>> {
        let mut frame = String::new();
        let bytes_read = self.reader.read_line(&mut frame).await?;
        if bytes_read == 0 {
            return Ok(None);
        }
        if !frame.ends_with('\n') {
            return Err(Error::UnterminatedMessage);
        }
        Ok(Some(serde_json::from_str(
            frame.trim_end_matches(['\r', '\n']),
        )?))
    }
}
