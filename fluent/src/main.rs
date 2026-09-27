use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Context, anyhow};
use fluent_ipc::protocol::inst::{ClientConnection, ClientMessage, ServerMessageKind};

const AGGREGATOR_SOCKET: &str = "/tmp/fluent.sock";
const STATUS_INTERVAL: Duration = Duration::from_secs(5);

fn main() -> anyhow::Result<()> {
    let active = Arc::new(Mutex::new(true));

    start_status_reporter(active.clone());

    let configs = [
        Path::new("/etc/interception/fluent.d/fluent.conf"),
        Path::new("/etc/interception/fluent.conf"),
    ];

    let config = configs.iter().find(|path| path.is_file());
    let Some(config) = config else {
        return Err(anyhow!("no config found, tried: {:?}", configs));
    };
    let input = fs::read_to_string(config).context("could not read config file")?;
    let config = fluent::config::parse(&input).context("could not parse config file")?;

    fluent::run(
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        &config,
        active,
    )?;
    Ok(())
}

fn start_status_reporter(active: Arc<Mutex<bool>>) {
    let socket_path = PathBuf::from(AGGREGATOR_SOCKET);

    thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .expect("could not start status reporter")
            .block_on(report_status(socket_path, active));
    });
}

async fn report_status(socket_path: PathBuf, active: Arc<Mutex<bool>>) {
    loop {
        let mut client = loop {
            match ClientConnection::connect(&socket_path).await {
                Ok(client) => break client,
                Err(error) => {
                    eprintln!("could not connect to aggregator: {error}");
                    tokio::time::sleep(STATUS_INTERVAL).await;
                }
            }
        };

        if let Err(error) = client
            .send(&ClientMessage::status(
                std::process::id(),
                *active.lock().unwrap(),
            ))
            .await
        {
            eprintln!("could not report initial Fluent status: {error}");
            break;
        }

        loop {
            match client.next_message().await {
                Ok(message) => match message {
                    Some(message) => match message.kind {
                        ServerMessageKind::GetStatus => {
                            if let Err(error) = client
                                .send(&ClientMessage::status(
                                    std::process::id(),
                                    *active.lock().unwrap(),
                                ))
                                .await
                            {
                                eprintln!("could not report Fluent status: {error}");
                                break;
                            }
                        }
                        ServerMessageKind::SetActive(val) => {
                            *active.lock().unwrap() = val;
                            if let Err(error) = client
                                .send(&ClientMessage::status(
                                    std::process::id(),
                                    *active.lock().unwrap(),
                                ))
                                .await
                            {
                                eprintln!("could not report Fluent status: {error}");
                                break;
                            }
                        }
                    },
                    None => {
                        eprintln!("fluentd closed socket");
                        return;
                    }
                },
                Err(error) => {
                    eprintln!("error receiving message from fluentd: {error}")
                }
            }

            tokio::time::sleep(STATUS_INTERVAL).await;
        }
    }
}
