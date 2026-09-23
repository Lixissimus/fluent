use std::time::{SystemTime, UNIX_EPOCH};

use fluent_ipc::{
    ConnectError, ReceiveError,
    protocol::ctrl::{ClientConnection, ClientMessage, Server},
};
use tokio::io::AsyncWriteExt;

fn socket_path() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("fluent-ipc-{nonce}.sock"))
}

#[tokio::test]
async fn aggregator_accepts_multiple_clients() {
    let path = socket_path();
    let aggregator = Server::bind(&path).await.expect("bind socket");
    let mut first = ClientConnection::connect(&path)
        .await
        .expect("connect first client");
    let mut second = ClientConnection::connect(&path)
        .await
        .expect("connect second client");

    let mut first_connection = aggregator.accept().await.expect("accept first client");
    let mut second_connection = aggregator.accept().await.expect("accept second client");

    first
        .send(&ClientMessage::get_status())
        .await
        .expect("send first status");
    second
        .send(&ClientMessage::get_status())
        .await
        .expect("send second status");

    assert_eq!(
        first_connection
            .next_message()
            .await
            .expect("read first")
            .expect("first message"),
        ClientMessage::get_status()
    );
    assert_eq!(
        second_connection
            .next_message()
            .await
            .expect("read second")
            .expect("second message"),
        ClientMessage::get_status()
    );
}

#[tokio::test]
async fn connection_reports_clean_disconnect() {
    let path = socket_path();
    let aggregator = Server::bind(&path).await.expect("bind socket");
    let client = ClientConnection::connect(&path)
        .await
        .expect("connect client");
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
    let aggregator = Server::bind(&path).await.expect("bind socket");
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
        Err(ReceiveError::UnterminatedMessage)
    ));
}

#[tokio::test]
async fn aggregator_removes_stale_socket() {
    let path = socket_path();
    let aggregator = Server::bind(&path).await.expect("bind stale socket path");
    drop(aggregator);
    let _ = std::os::unix::net::UnixListener::bind(&path)
        .expect("socket should be removed and ready to bind again");
}

#[tokio::test]
async fn aggregator_rejects_an_active_socket() {
    let path = socket_path();
    let aggregator = Server::bind(&path).await.expect("bind socket");

    let result = Server::bind(&path).await;
    assert!(
        matches!(result, Err(ConnectError::Io(error)) if error.kind() == std::io::ErrorKind::AddrInUse)
    );

    drop(aggregator);
}
