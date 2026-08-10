//! Committed benchmark for connection reuse.
//!
//! Contract §9 requires a reproducible benchmark so the pooling ratio is tracked
//! rather than anecdotal. `#[ignore]`d because it is a timing measurement, not a
//! pass/fail assertion — timings vary by machine and build profile.
//!
//! Against a live engine:
//!
//! ```text
//! cargo test --all-features --test pooling_benchmark -- --ignored --nocapture
//! ```
//!
//! The reference figures in the contract (live `montycat_bin`, debug build, 300 ×
//! `list-owners`, loopback, no TLS) are:
//!
//! ```text
//! connect-per-request :    160.1 us/op
//! reused connection   :     50.3 us/op
//! handshake overhead  :    109.8 us/op   = 3.18x slower
//! ```
//!
//! Two things that make 3.18x a conservative floor rather than a ceiling: a
//! debug engine spends longer per request, shrinking the handshake's share of
//! the total; and loopback without TLS is the best case for connect-per-request,
//! since over a network the handshake costs a full round trip before the query
//! is even sent, and TLS adds one or two more.

use std::time::Instant;

use montycat::{Engine, PoolConfig};

const ITERATIONS: usize = 300;
const HOST: &str = "127.0.0.1";
const PORT: u16 = 21210;

fn engine() -> Engine {
    Engine::new(
        HOST.into(),
        PORT,
        "EUGENE".into(),
        "12345".into(),
        Some("playground_store".into()),
        false,
    )
}

#[tokio::test]
#[ignore = "requires a live engine on 127.0.0.1:21210"]
async fn connection_reuse_versus_connect_per_request() {
    // Warm up so neither figure pays one-off costs.
    let warm = engine();
    for _ in 0..10 {
        warm.list_owners().await.expect("engine reachable");
    }

    let unpooled = engine();
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        unpooled.list_owners().await.unwrap();
    }
    let per_connect = start.elapsed() / ITERATIONS as u32;

    let pooled = engine().with_pool(PoolConfig::default());
    // First call opens the connection every later call reuses.
    pooled.list_owners().await.unwrap();
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        pooled.list_owners().await.unwrap();
    }
    let per_reuse = start.elapsed() / ITERATIONS as u32;
    pooled.close_pool().await;

    let ratio = per_connect.as_secs_f64() / per_reuse.as_secs_f64();
    println!(
        "\nconnect-per-request : {:>10.1} us/op",
        per_connect.as_secs_f64() * 1e6
    );
    println!(
        "reused connection   : {:>10.1} us/op",
        per_reuse.as_secs_f64() * 1e6
    );
    println!(
        "handshake overhead  : {:>10.1} us/op   = {ratio:.2}x slower\n",
        (per_connect.as_secs_f64() - per_reuse.as_secs_f64()) * 1e6
    );

    assert!(
        per_reuse < per_connect,
        "reuse ({per_reuse:?}) was not faster than connect-per-request ({per_connect:?})"
    );
}
