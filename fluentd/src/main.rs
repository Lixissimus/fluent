use std::{
    collections::HashSet,
    eprintln,
    path::PathBuf,
    println,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use clap::Parser;
use fluent_ipc::{protocol::ctrl, protocol::inst};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Socket for communication with fluentctl
    ///
    /// The default value is fine as long as you don't also change the socket when running fluentctl
    #[arg(long, default_value = "/tmp/fluent-if.sock")]
    if_socket: PathBuf,

    /// Socket for communication with the fluent instances
    ///
    /// The default value is fine as long as you don't also change the socket when running the instances
    #[arg(long, default_value = "/tmp/fluent.sock")]
    inst_socket: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let instance_server = inst::Server::bind(&args.inst_socket)
        .await
        .with_context(|| format!("could not bind aggregator socket at {:?}", args.inst_socket))?;

    let ctrl_server = ctrl::Server::bind(&args.if_socket)
        .await
        .with_context(|| format!("could not bind interface socket at {:?}", args.if_socket))?;

    let instances = Arc::new(Mutex::new(HashSet::new()));
    loop {
        tokio::select! {
            result = instance_server.accept() => {
                let connection = result?;
                tokio::spawn(handle_instance_connection(connection, instances.clone()));
            }
            result = ctrl_server.accept() => {
                let connection = result?;
                tokio::spawn(handle_ctrl_connection(connection, instances.clone()));
            }
            result = tokio::signal::ctrl_c() => {
                result.context("could not listen for Ctrl-C")?;
                eprintln!("shutting down");
                return Ok(());
            }
        }
    }
}

async fn handle_instance_connection(
    mut connection: inst::ServerConnection,
    instances: Arc<Mutex<HashSet<u32>>>,
) {
    loop {
        match connection.next_message().await {
            Ok(Some(message)) => match message.kind {
                inst::ClientMessageKind::Status { pid } => {
                    instances.lock().unwrap().insert(pid);
                }
            },
            Ok(None) => return,
            Err(error) => {
                eprintln!("could not read client message: {error}");
                return;
            }
        }
    }
}

async fn handle_ctrl_connection(
    mut connection: ctrl::ServerConnection,
    instances: Arc<Mutex<HashSet<u32>>>,
) {
    loop {
        match connection.next_message().await {
            Ok(Some(message)) => match message.kind {
                ctrl::ClientMessageKind::GetStatus => {
                    let pids: Vec<_> = instances.lock().unwrap().iter().map(|pid| *pid).collect();
                    if let Err(e) = connection.send(&ctrl::ServerMessage::status(&pids)).await {
                        eprintln!("error sending ctrl message: {e}")
                    }
                }
            },
            Ok(None) => {
                println!("ctrl connection closed");
                break;
            }
            Err(e) => eprintln!("error receiving ctrl message: {e}"),
        }
    }
}
