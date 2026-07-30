//! A request is a subscription because the caller asked for a callback, never
//! because its payload happens to contain the word "subscribe".
//!
//! Regression: subscription mode used to be detected by scanning the serialized
//! request for `b"subscribe"`, so inserting a record whose value contained that
//! word routed the call into the streaming branch — which has no read timeout
//! and never returns.

use montycat::{Engine, PersistentKeyspace};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// Stub engine: read a newline-framed request, answer it, keep the connection
/// open — the same loop as the server's `main_server/connection.rs`.
async fn stub_engine() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut reader = BufReader::new(socket);
                let mut line = String::new();
                while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                    let _ = reader
                        .get_mut()
                        .write_all(b"{\"status\":true,\"payload\":\"1\",\"error\":null}\n")
                        .await;
                    line.clear();
                }
            });
        }
    });
    port
}

async fn insert(port: u16, note: &str) -> bool {
    let engine = Engine::new(
        "127.0.0.1".into(),
        port,
        "u".into(),
        "p".into(),
        Some("s".into()),
        false,
    );
    let keyspace = PersistentKeyspace::new("k", &engine);

    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        keyspace.insert_value_no_schema(None, serde_json::json!({ "note": note })),
    )
    .await
    .is_ok()
}

#[tokio::test]
async fn a_value_containing_subscribe_is_not_treated_as_a_subscription() {
    let port = stub_engine().await;

    assert!(insert(port, "hello").await, "control insert must return");
    assert!(
        insert(port, "please subscribe").await,
        "a value containing 'subscribe' must not be routed into the streaming branch"
    );
    assert!(
        insert(port, "subscribe").await,
        "a value equal to 'subscribe' must not be routed into the streaming branch"
    );
    assert!(
        insert(port, "unsubscribe from the newsletter").await,
        "a value containing 'subscribe' as a substring must still return"
    );
}
