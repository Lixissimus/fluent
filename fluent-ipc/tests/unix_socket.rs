use std::time::{SystemTime, UNIX_EPOCH};

use fluent_ipc::{Aggregator, Client, Error, Message};
use tokio::io::AsyncWriteExt;

fn socket_path() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("fluent-ipc-{nonce}.sock"))
}

#[tokio::test]
async fn client_reports_pid_to_aggregator() {
    let path = socket_path();
    let aggregator = Aggregator::bind(&path).await.expect("bind socket");
    let mut client = Client::connect(&path).await.expect("connect client");
    let mut connection = aggregator.accept().await.expect("accept client");

    client.send_status().await.expect("send status");
    let message = connection
        .next_message()
        .await
        .expect("read message")
        .expect("message");
    assert_eq!(message, Message::status(std::process::id()));
}

#[tokio::test]
async fn aggregator_accepts_multiple_clients() {
    let path = socket_path();
    let aggregator = Aggregator::bind(&path).await.expect("bind socket");
    let mut first = Client::connect(&path).await.expect("connect first client");
    let mut second = Client::connect(&path).await.expect("connect second client");

    let mut first_connection = aggregator.accept().await.expect("accept first client");
    let mut second_connection = aggregator.accept().await.expect("accept second client");

    first.send_status().await.expect("send first status");
    second.send_status().await.expect("send second status");

    assert_eq!(
        first_connection
            .next_message()
            .await
            .expect("read first")
            .expect("first message"),
        Message::status(std::process::id())
    );
    assert_eq!(
        second_connection
            .next_message()
            .await
            .expect("read second")
            .expect("second message"),
        Message::status(std::process::id())
    );
}

#[tokio::test]
async fn connection_reports_clean_disconnect() {
    let path = socket_path();
    let aggregator = Aggregator::bind(&path).await.expect("bind socket");
    let client = Client::connect(&path).await.expect("connect client");
    let mut connection = aggregator.accept().await.expect("accept client");
    drop(client);
    assert_eq!(
        connection.next_message().await.expect("read disconnect"),
        None
    );
}

#[tokio::test]
async fn connection_rejects_unterminated_message() {
    let path = socket_path();
    let aggregator = Aggregator::bind(&path).await.expect("bind socket");
    let mut client = tokio::net::UnixStream::connect(&path)
        .await
        .expect("connect client");
    let mut connection = aggregator.accept().await.expect("accept client");

    client
        .write_all(br#"{"version":1,"type":"status","pid":42}"#)
        .await
        .expect("write unterminated message");
    drop(client);

    assert!(matches!(
        connection.next_message().await,
        Err(Error::UnterminatedMessage)
    ));
}
