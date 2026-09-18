use std::path::Path;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf},
    net::UnixStream,
};

use crate::{
    ConnectError, Message,
    error::{ReceiveError, SendError},
};

pub struct Connection {
    reader: BufReader<ReadHalf<UnixStream>>,
    writer: BufWriter<WriteHalf<UnixStream>>,
}

impl Connection {
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, ConnectError> {
        Ok(Self::new(UnixStream::connect(path).await?))
    }

    pub fn new(stream: UnixStream) -> Self {
        let (read_half, write_half) = tokio::io::split(stream);
        Self {
            reader: BufReader::new(read_half),
            writer: BufWriter::new(write_half),
        }
    }

    pub async fn next_message(&mut self) -> Result<Option<Message>, ReceiveError> {
        let mut frame = String::new();
        let bytes_read = self.reader.read_line(&mut frame).await?;
        if bytes_read == 0 {
            return Ok(None);
        }
        if !frame.ends_with('\n') {
            return Err(ReceiveError::UnterminatedMessage);
        }
        Ok(Some(serde_json::from_str(
            frame.trim_end_matches(['\r', '\n']),
        )?))
    }

    pub async fn send(&mut self, message: &Message) -> Result<(), SendError> {
        let mut frame = serde_json::to_vec(message).unwrap();
        frame.push(b'\n');
        self.writer.write_all(&frame).await?;
        self.writer.flush().await?;

        Ok(())
    }
}
