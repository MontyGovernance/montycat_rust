//! Connection pooling behaviour.
//!
//! Covers the required matrix in
//! `montycat_semantic/CLIENT_CONNECTION_POOLING_CONTRACT.md` §9. Stub servers
//! throughout; no live engine required.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use montycat::{Engine, PoolConfig};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

const OK: &[u8] = b"{\"status\":true,\"payload\":null,\"error\":null}\n";

/// Serves newline-framed requests, many per connection, counting accepts.
async fn counting_server() -> (u16, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepts = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&accepts);

    let handle = tokio::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                // Mirrors the engine: read a request, write a response, repeat
                // until the client hangs up.
                let mut reader = BufReader::new(socket);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => return,
                        Ok(_) => {}
                    }
                    if reader.get_mut().write_all(OK).await.is_err() {
                        return;
                    }
                }
            });
        }
    });

    (port, accepts, handle)
}

fn engine_for(port: u16) -> Engine {
    Engine::new(
        "127.0.0.1".into(),
        port,
        "owner".into(),
        "secret".into(),
        Some("orders".into()),
        false,
    )
}

#[tokio::test]
async fn pooling_is_off_by_default_and_opens_a_connection_per_request() {
    let (port, accepts, _srv) = counting_server().await;
    let engine = engine_for(port);

    for _ in 0..5 {
        engine.list_owners().await.unwrap();
    }

    assert_eq!(
        accepts.load(Ordering::SeqCst),
        5,
        "unpooled engine must not reuse connections"
    );
    assert!(engine.pool().is_none());
}

#[tokio::test]
async fn sequential_requests_reuse_one_pooled_connection() {
    let (port, accepts, _srv) = counting_server().await;
    let engine = engine_for(port).with_pool(PoolConfig::default());

    for _ in 0..10 {
        engine.list_owners().await.unwrap();
    }

    assert_eq!(
        accepts.load(Ordering::SeqCst),
        1,
        "10 sequential requests should share one connection"
    );
    assert_eq!(engine.pool().unwrap().idle_len().await, 1);
    engine.close_pool().await;
}

#[tokio::test]
async fn a_cloned_engine_shares_the_same_pool() {
    // Keyspaces hold the engine by value and clone it per operation, so a clone
    // that pooled separately would defeat pooling entirely.
    let (port, accepts, _srv) = counting_server().await;
    let engine = engine_for(port).with_pool(PoolConfig::default());

    engine.list_owners().await.unwrap();
    let clone = engine.clone();
    clone.list_owners().await.unwrap();

    assert_eq!(
        accepts.load(Ordering::SeqCst),
        1,
        "a clone opened its own connection — the pool is not shared"
    );
    engine.close_pool().await;
}

#[tokio::test]
async fn idle_connections_never_exceed_max_idle() {
    let (port, _accepts, _srv) = counting_server().await;
    let engine = engine_for(port).with_pool(PoolConfig {
        max_idle: 2,
        idle_timeout: Duration::from_secs(30),
    });

    // Concurrency forces several live connections at once; only max_idle of
    // them may be retained afterwards.
    let mut set = Vec::new();
    for _ in 0..8 {
        let e = engine.clone();
        set.push(tokio::spawn(async move { e.list_owners().await }));
    }
    for task in set {
        task.await.unwrap().unwrap();
    }

    assert!(
        engine.pool().unwrap().idle_len().await <= 2,
        "pool grew past max_idle"
    );
    engine.close_pool().await;
}

#[tokio::test]
async fn connections_older_than_the_idle_timeout_are_discarded() {
    let (port, accepts, _srv) = counting_server().await;
    let engine = engine_for(port).with_pool(PoolConfig {
        max_idle: 4,
        idle_timeout: Duration::from_millis(50),
    });

    engine.list_owners().await.unwrap();
    assert_eq!(accepts.load(Ordering::SeqCst), 1);

    sleep(Duration::from_millis(120)).await;

    engine.list_owners().await.unwrap();
    assert_eq!(
        accepts.load(Ordering::SeqCst),
        2,
        "an expired connection was reused instead of being discarded"
    );
    engine.close_pool().await;
}

#[tokio::test]
async fn a_server_closed_idle_connection_is_retried_once_and_succeeds() {
    // The stale-socket case. Note the mechanism is *not* a failing write: writing
    // to a peer-closed socket normally succeeds, since the bytes just land in the
    // send buffer. The pool detects the dead connection when it checks it out and
    // finds the socket readable-at-EOF, discards it, and opens a fresh one — so
    // the request is sent exactly once and never replayed.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepts = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&accepts);

    tokio::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                return;
            };
            let n = counter.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let mut reader = BufReader::new(socket);
                let mut line = String::new();
                // The first connection serves exactly one request then hangs up,
                // mimicking a server-side idle reaper.
                let budget = if n == 0 { 1 } else { usize::MAX };
                for _ in 0..budget {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => return,
                        Ok(_) => {}
                    }
                    if reader.get_mut().write_all(OK).await.is_err() {
                        return;
                    }
                }
                let _ = reader.get_mut().shutdown().await;
            });
        }
    });

    let engine = engine_for(port).with_pool(PoolConfig::default());

    engine.list_owners().await.unwrap();
    // Give the stub time to close the first connection while it sits idle.
    sleep(Duration::from_millis(80)).await;

    let second = engine.list_owners().await;
    assert!(
        second.is_ok(),
        "a stale pooled connection was not retried: {second:?}"
    );
    assert_eq!(
        accepts.load(Ordering::SeqCst),
        2,
        "retry should have opened exactly one fresh connection"
    );
    engine.close_pool().await;
}

#[tokio::test]
async fn a_read_failure_is_returned_and_never_retried() {
    // The rule whose violation duplicates user data: the engine may have applied
    // the write already and only the response was lost. Replaying is a data bug,
    // not resilience. The stub accepts the write, then dies without replying.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepts = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&accepts);

    tokio::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let mut reader = BufReader::new(socket);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
                // Read the request, then drop without responding.
                let _ = reader.get_mut().shutdown().await;
            });
        }
    });

    let engine = engine_for(port).with_pool(PoolConfig::default());
    let result = engine.list_owners().await;

    assert_eq!(
        accepts.load(Ordering::SeqCst),
        1,
        "a read-phase failure was retried — contract §4 forbids this"
    );
    assert!(
        result.is_err(),
        "EOF before a response must surface as an error, not as a successful \
         empty response that callers would then try to parse: {result:?}"
    );
    engine.close_pool().await;
}

#[tokio::test]
async fn no_bytes_leak_between_two_requests_on_one_pooled_connection() {
    // The framing guarantee pooling depends on: distinct payloads, so a leaked
    // byte from response one would corrupt response two visibly.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(socket);
        let mut line = String::new();
        for n in 0..2 {
            line.clear();
            if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                return;
            }
            let body = format!("{{\"status\":true,\"payload\":\"response-{n}\"}}\n");
            reader.get_mut().write_all(body.as_bytes()).await.unwrap();
        }
        sleep(Duration::from_millis(50)).await;
    });

    let engine = engine_for(port).with_pool(PoolConfig::default());

    let first = String::from_utf8(engine.list_owners().await.unwrap().unwrap()).unwrap();
    let second = String::from_utf8(engine.list_owners().await.unwrap().unwrap()).unwrap();

    assert!(first.contains("response-0"), "first was {first:?}");
    assert!(
        !first.contains("response-1"),
        "first absorbed the next frame"
    );
    assert!(second.contains("response-1"), "second was {second:?}");
    assert!(
        !second.contains("response-0"),
        "second carried leftovers from the first"
    );
    engine.close_pool().await;
}

#[tokio::test]
async fn close_pool_drains_every_idle_connection() {
    let (port, _accepts, _srv) = counting_server().await;
    let engine = engine_for(port).with_pool(PoolConfig::default());

    engine.list_owners().await.unwrap();
    assert_eq!(engine.pool().unwrap().idle_len().await, 1);

    engine.close_pool().await;
    assert_eq!(
        engine.pool().unwrap().idle_len().await,
        0,
        "close_pool left connections behind"
    );
}

#[tokio::test]
async fn a_deserialized_engine_has_no_pool() {
    // `pool` is `#[serde(skip)]`, so a round-tripped engine must fall back to
    // connect-per-request rather than carrying a dangling pool.
    let (port, _accepts, _srv) = counting_server().await;
    let engine = engine_for(port).with_pool(PoolConfig::default());
    assert!(engine.pool().is_some());

    let json = serde_json::to_string(&engine).unwrap();
    let restored: Engine = serde_json::from_str(&json).unwrap();

    assert!(restored.pool().is_none());
    restored.list_owners().await.unwrap();
    let _ = TcpStream::connect(("127.0.0.1", port)).await;
}
