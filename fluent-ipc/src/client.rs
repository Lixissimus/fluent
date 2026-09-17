use std::path::Path;

use tokio::{io::AsyncWriteExt, net::UnixStream};

use crate::{Message, Result};

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            stream: UnixStream::connect(path).await?,
        })
    }

    pub async fn send(&mut self, message: &Message) -> Result<()> {
        let mut frame = serde_json::to_vec(message)?;
        frame.push(b'\n');
        self.stream.write_all(&frame).await?;
        Ok(())
    }

    pub async fn send_status(&mut self) -> Result<()> {
        self.send(&Message::status(std::process::id())).await
    }
}
