use std::{env, ffi::OsString, path::PathBuf};

use anyhow::{Context, bail};
use fluent_ipc::{Connection, Socket};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket_path = socket_path_from_args()?;
    let aggregator = Socket::bind(&socket_path).await.with_context(|| {
        format!(
            "could not bind aggregator socket at {}",
            socket_path.display()
        )
    })?;

    let interface = Socket::bind("/tmp/fluent-if.sock")
        .await
        .with_context(|| "could not bind interface socket at /tmp/fluent-if.sock")?;

    loop {
        tokio::select! {
            result = aggregator.accept() => {
                let connection = result?;
                tokio::spawn(handle_aggregator_connection(connection));
            }
            result = interface.accept() => {
                let connection = result?;
                tokio::spawn(handle_interface_connection(connection));
            }
            result = tokio::signal::ctrl_c() => {
                result.context("could not listen for Ctrl-C")?;
                eprintln!("shutting down");
                return Ok(());
            }
        }
    }
}

fn socket_path_from_args() -> anyhow::Result<PathBuf> {
    let mut args = env::args_os();
    let program = args
        .next()
        .unwrap_or_else(|| OsString::from("fluent-aggregator"));
    let Some(socket_path) = args.next() else {
        bail!("usage: {} SOCKET_PATH", program.to_string_lossy());
    };
    Ok(PathBuf::from(socket_path))
}

async fn handle_aggregator_connection(mut connection: Connection) {
    loop {
        match connection.next_message().await {
            Ok(Some(message)) => match serde_json::to_string(&message) {
                Ok(message) => println!("{message}"),
                Err(error) => eprintln!("could not encode received message: {error}"),
            },
            Ok(None) => return,
            Err(error) => {
                eprintln!("could not read client message: {error}");
                return;
            }
        }
    }
}

async fn handle_interface_connection(mut connection: Connection) {
    loop {
        match connection.next_message().await {
            Ok(Some(message)) => match serde_json::to_string(&message) {
                Ok(message) => println!("{message}"),
                Err(error) => eprintln!("could not encode received message: {error}"),
            },
            Ok(None) => return,
            Err(error) => {
                eprintln!("could not read client message: {error}");
                return;
            }
        }
    }
}
