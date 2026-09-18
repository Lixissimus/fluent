use fluent_ipc::Connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut connection = Connection::connect("/tmp/fluent-if.sock").await?;

    loop {
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
    }

    Ok(())
}
