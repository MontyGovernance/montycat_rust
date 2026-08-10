//! Response framing — the guarantees connection pooling depends on.
//!
//! Responses are newline-delimited. The reader must return exactly one frame,
//! retain any bytes past that newline *with the connection*, and cost O(n)
//! rather than rescanning its accumulated buffer per chunk. See
//! `montycat_semantic/CLIENT_CONNECTION_POOLING_CONTRACT.md` §7.
//!
//! These are stub-server tests; no live engine is required.

use montycat::{Engine, MontycatResponse};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};

/// A stub that accepts one connection and replies with `script`, written as
/// separate `write_all` calls so the client is forced across multiple reads.
async fn engine_replying_with(script: Vec<Vec<u8>>) -> (Engine, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = vec![0; 16 * 1024];
        let _ = socket.read(&mut buffer).await.unwrap();
        for piece in script {
            socket.write_all(&piece).await.unwrap();
            socket.flush().await.unwrap();
            // Force the client to observe a partial frame before the rest lands.
            sleep(Duration::from_millis(10)).await;
        }
        // Hold the socket open briefly so a client that (incorrectly) waits for
        // EOF rather than the newline would hang and fail the test by timeout
        // rather than passing accidentally.
        sleep(Duration::from_millis(50)).await;
    });

    (
        Engine::new(
            "127.0.0.1".into(),
            port,
            "owner".into(),
            "secret".into(),
            Some("orders".into()),
            false,
        ),
        server,
    )
}

#[tokio::test]
async fn response_larger_than_chunk_size_is_read_whole() {
    // CHUNK_SIZE is 256 KiB; go well past it so the read spans many fills.
    let filler = "x".repeat(600 * 1024);
    let body = format!("{{\"status\":true,\"payload\":\"{filler}\"}}\n");
    let (engine, server) = engine_replying_with(vec![body.clone().into_bytes()]).await;

    let bytes = engine.list_owners().await.unwrap();
    let parsed = MontycatResponse::<Value>::parse_response(Ok(bytes)).unwrap();

    assert!(parsed.status);
    assert_eq!(
        parsed.payload.as_str().map(str::len),
        Some(600 * 1024),
        "payload truncated — a response larger than CHUNK_SIZE was not read whole"
    );
    server.await.unwrap();
}

#[tokio::test]
async fn response_split_mid_json_across_reads_is_reassembled() {
    let body = b"{\"status\":true,\"payload\":\"split-across-reads\"}\n";
    let (head, tail) = body.split_at(20);
    let (engine, server) = engine_replying_with(vec![head.to_vec(), tail.to_vec()]).await;

    let bytes = engine.list_owners().await.unwrap();
    let parsed = MontycatResponse::<Value>::parse_response(Ok(bytes)).unwrap();

    assert!(parsed.status);
    assert_eq!(parsed.payload.as_str(), Some("split-across-reads"));
    server.await.unwrap();
}

#[tokio::test]
async fn reader_stops_at_the_first_newline_and_does_not_absorb_the_next_frame() {
    // Both frames arrive in a single write. A reader that stops only when its
    // accumulated buffer *contains* a newline would swallow frame two into
    // frame one — on a pooled connection that is the next caller's response.
    let two_frames = b"{\"status\":true,\"payload\":\"first\"}\n\
                       {\"status\":true,\"payload\":\"second\"}\n";
    let (engine, server) = engine_replying_with(vec![two_frames.to_vec()]).await;

    let bytes = engine.list_owners().await.unwrap();
    let raw = bytes.expect("a response");

    let text = String::from_utf8(raw).unwrap();
    assert_eq!(
        text.matches('\n').count(),
        1,
        "read more than one frame: {text:?}"
    );
    assert!(
        !text.contains("second"),
        "the following frame leaked into this response: {text:?}"
    );

    let parsed = MontycatResponse::<Value>::parse_response(Ok(Some(text.into_bytes()))).unwrap();
    assert_eq!(parsed.payload.as_str(), Some("first"));
    server.await.unwrap();
}

#[tokio::test]
async fn trailing_bytes_after_the_newline_are_not_appended_to_the_response() {
    // A frame followed by the *start* of the next one. The response must be the
    // first frame exactly — the partial tail belongs to the connection.
    let script = vec![b"{\"status\":true,\"payload\":\"only\"}\n{\"status\":tr".to_vec()];
    let (engine, server) = engine_replying_with(script).await;

    let bytes = engine.list_owners().await.unwrap();
    let text = String::from_utf8(bytes.expect("a response")).unwrap();

    assert_eq!(
        text.trim_end(),
        r#"{"status":true,"payload":"only"}"#,
        "response carried bytes belonging to the next frame"
    );
    server.await.unwrap();
}
