use std::{
    io,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Serialize, de::DeserializeOwned};
use tokio::net::{UnixListener, UnixStream};

use crate::{Connection, error::ConnectError};

pub struct Socket<R, S>
where
    R: Serialize + DeserializeOwned,
    S: Serialize + DeserializeOwned,
{
    listener: UnixListener,
    path: Arc<PathBuf>,
    _phantom_data1: PhantomData<R>,
    _phantom_data2: PhantomData<S>,
}

impl<R, S> Socket<R, S>
where
    R: Serialize + DeserializeOwned,
    S: Serialize + DeserializeOwned,
{
    pub async fn bind(path: impl AsRef<Path>) -> Result<Self, ConnectError> {
        let path = path.as_ref().to_owned();
        remove_stale_socket(&path).await?;
        let listener = UnixListener::bind(&path)?;
        Ok(Self {
            listener,
            path: Arc::new(path),
            _phantom_data1: PhantomData,
            _phantom_data2: PhantomData,
        })
    }

    pub async fn accept(&self) -> Result<Connection<R, S>, ConnectError> {
        let (stream, _) = self.listener.accept().await?;
        Ok(Connection::new(stream))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

async fn remove_stale_socket(path: &Path) -> Result<(), io::Error> {
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

impl<R, S> Drop for Socket<R, S>
where
    R: Serialize + DeserializeOwned,
    S: Serialize + DeserializeOwned,
{
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.path.as_ref());
    }
}
