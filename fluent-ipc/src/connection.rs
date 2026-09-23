use std::{marker::PhantomData, path::Path};

use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf},
    net::UnixStream,
};

use crate::{
    ConnectError,
    error::{ReceiveError, SendError},
};

pub struct Connection<R, S>
where
    R: Serialize + DeserializeOwned,
    S: Serialize + DeserializeOwned,
{
    reader: BufReader<ReadHalf<UnixStream>>,
    writer: BufWriter<WriteHalf<UnixStream>>,
    _phantom_data1: PhantomData<R>,
    _phantom_data2: PhantomData<S>,
}

impl<R, S> Connection<R, S>
where
    R: Serialize + DeserializeOwned,
    S: Serialize + DeserializeOwned,
{
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, ConnectError> {
        Ok(Self::new(UnixStream::connect(path).await?))
    }

    pub fn new(stream: UnixStream) -> Self {
        let (read_half, write_half) = tokio::io::split(stream);
        Self {
            reader: BufReader::new(read_half),
            writer: BufWriter::new(write_half),
            _phantom_data1: PhantomData,
            _phantom_data2: PhantomData,
        }
    }

    pub async fn next_message(&mut self) -> Result<Option<R>, ReceiveError> {
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

    pub async fn send(&mut self, message: &S) -> Result<(), SendError> {
        let mut frame = serde_json::to_vec(message)?;
        frame.push(b'\n');
        self.writer.write_all(&frame).await?;
        self.writer.flush().await?;

        Ok(())
    }
}
