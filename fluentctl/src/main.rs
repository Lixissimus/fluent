use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand};
use fluent_ipc::protocol::ctrl::{ClientConnection, ClientMessage};
use tokio::time;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Socket for communication with fluent process
    ///
    /// The default value is fine as long as you don't also change the socket when running the fluent aggregator.
    #[arg(short, long, default_value = "/tmp/fluent-if.sock")]
    socket: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the status of all running fluent instances
    Status {
        /// Print the status continuously
        #[arg(long, default_value_t = false)]
        watch: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let connection = ClientConnection::connect(args.socket).await?;

    match args.command {
        Command::Status { watch } => run_status(connection, watch).await,
    }
}

async fn run_status(mut connection: ClientConnection, watch: bool) -> anyhow::Result<()> {
    loop {
        connection.send(&ClientMessage::get_status()).await?;
        match connection.next_message().await {
            Ok(Some(message)) => match serde_json::to_string(&message) {
                Ok(message) => println!("{message}"),
                Err(error) => eprintln!("could not encode received message: {error}"),
            },
            Ok(None) => break,
            Err(error) => {
                eprintln!("could not read client message: {error}");
                break;
            }
        }
        if !watch {
            break;
        }
        time::sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
