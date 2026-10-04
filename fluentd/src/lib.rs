use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use fluent_ipc::protocol::{
    ctrl::{self, InstanceStatus},
    inst,
};
use tokio::sync::mpsc;

use crate::args::Args;

pub mod args;

pub async fn run(args: Args) -> anyhow::Result<()> {
    let instance_server = inst::Server::bind(&args.inst_socket)
        .await
        .with_context(|| format!("could not bind aggregator socket at {:?}", args.inst_socket))?;

    let ctrl_server = ctrl::Server::bind(&args.if_socket)
        .await
        .with_context(|| format!("could not bind interface socket at {:?}", args.if_socket))?;

    let instances = Arc::new(Mutex::new(HashMap::new()));
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

struct Instance {
    pid: u32,
    active: bool,
    command_tx: mpsc::UnboundedSender<bool>,
}

impl From<&Instance> for InstanceStatus {
    fn from(value: &Instance) -> Self {
        Self {
            pid: value.pid,
            active: value.active,
        }
    }
}

type Instances = HashMap<u32, Instance>;

async fn handle_instance_connection(
    connection: inst::ServerConnection,
    instances: Arc<Mutex<Instances>>,
) {
    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let mut conn = InstanceConnection {
        connection,
        registry: instances,
        command_tx,
        command_rx,
        pid: None,
    };
    conn.run().await
}

struct InstanceConnection {
    connection: inst::ServerConnection,
    registry: Arc<Mutex<Instances>>,
    command_tx: mpsc::UnboundedSender<bool>,
    command_rx: mpsc::UnboundedReceiver<bool>,
    pid: Option<u32>,
}

impl InstanceConnection {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                result = self.connection.next_message() => {
                    match result {
                        Ok(Some(message)) => match message.kind {
                            inst::ClientMessageKind::Status { pid, active } => {
                                self.pid = Some(pid);
                                self.registry
                                    .lock()
                                    .unwrap()
                                    .insert(pid, Instance {
                                        pid,
                                        active,
                                        command_tx: self.command_tx.clone(),
                                    });
                            }
                        },
                        Ok(None) => return,
                        Err(error) => {
                            eprintln!("could not read client message: {error}");
                            return;
                        }
                    }
                }
                command = self.command_rx.recv() => {
                    match command {
                        Some(active) => {
                            if let Err(error) = self.connection.send(&inst::ServerMessage::active(active)).await {
                                eprintln!("could not send command to instance: {error}");
                                return;
                            }
                        }
                        None => return,
                    }
                }
            }
        }
    }
}

impl Drop for InstanceConnection {
    fn drop(&mut self) {
        if let Some(pid) = self.pid {
            self.registry.lock().unwrap().remove(&pid);
        }
    }
}

async fn handle_ctrl_connection(
    mut connection: ctrl::ServerConnection,
    instances: Arc<Mutex<Instances>>,
) {
    loop {
        match connection.next_message().await {
            Ok(Some(message)) => match message.kind {
                ctrl::ClientMessageKind::GetStatus => {
                    let instances = instances
                        .lock()
                        .unwrap()
                        .values()
                        .map(|inst| inst.into())
                        .collect();
                    if let Err(e) = connection
                        .send(&ctrl::ServerMessage::status(instances))
                        .await
                    {
                        eprintln!("error sending ctrl message: {e}")
                    }
                }
                ctrl::ClientMessageKind::SetActive { pid, val } => {
                    let command_tx = instances
                        .lock()
                        .unwrap()
                        .get(&pid)
                        .map(|instance| instance.command_tx.clone());
                    match command_tx {
                        Some(command_tx) => {
                            if command_tx.send(val).is_err() {
                                eprintln!("instance {pid} is no longer connected");
                            }
                        }
                        None => eprintln!("no instance found with pid {pid}"),
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
