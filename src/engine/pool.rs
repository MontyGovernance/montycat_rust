//! Connection pooling for request/response traffic.
//!
//! Implements the client half of
//! `montycat_semantic/CLIENT_CONNECTION_POOLING_CONTRACT.md`. The rules that
//! matter most here:
//!
//! - **§3** — pooling by `(host, port, tls)` is safe because credentials travel
//!   in every request payload and the engine re-authenticates per request.
//!   Connections carry no identity, so one may serve different users.
//! - **§4** — retry a request **only** when the write failed on a connection
//!   taken from the pool. Never retry after a read failure: the engine may have
//!   already applied the write and only the response was lost. These commands
//!   are not idempotent and the wire has no request IDs.
//! - **§5** — subscriptions are never pooled.
//! - **§6** — pooling is opt-in and bounded; an idle pooled connection still
//!   holds a server permit.
//! - **§7** — the `BufReader` doing the line splitting must live and die with
//!   the connection, or lookahead bytes are lost between requests.

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Waker};
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use crate::engine::utils::Connection;

/// How many idle connections to keep, and how long to keep them.
///
/// Defaults are deliberately conservative. An idle pooled connection holds one
/// of the engine's connection permits (`num_workers * 200`, of which the main
/// listener gets 35%), so a large pool across many client processes can starve
/// the server while mostly idle. Raise these only after measuring with the
/// `queue-depths` command under production-like load.
#[derive(Debug, Clone, Copy)]
pub struct PoolConfig {
    /// Maximum idle connections retained. Never unbounded.
    pub max_idle: usize,
    /// Discard an idle connection older than this.
    ///
    /// Must stay shorter than any server or firewall idle reaper so the client
    /// drops a connection before the peer does — that is what keeps the
    /// stale-connection retry path rare rather than routine.
    pub idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle: 8,
            idle_timeout: Duration::from_secs(30),
        }
    }
}

/// A checked-in connection and the moment it went idle.
pub(crate) struct PooledConn {
    pub(crate) reader: BufReader<Connection>,
    idle_since: Instant,
}

/// A bounded set of idle connections guarded by a mutex.
///
/// Deliberately not `bb8`/`deadpool`: this is a `Vec` with a timestamp per
/// entry, and a generic pool crate would impose a trait model that fights the
/// feature-gated `Connection` enum for no benefit.
#[derive(Debug)]
pub struct ConnectionPool {
    idle: Mutex<Vec<PooledConn>>,
    config: PoolConfig,
}

impl std::fmt::Debug for PooledConn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledConn")
            .field("idle_since", &self.idle_since)
            .finish_non_exhaustive()
    }
}

impl ConnectionPool {
    pub(crate) fn new(config: PoolConfig) -> Self {
        Self {
            idle: Mutex::new(Vec::new()),
            config,
        }
    }

    /// Take a healthy idle connection, discarding any that aged out or died.
    ///
    /// Returns `None` when nothing usable is left, in which case the caller
    /// opens a fresh connection. Discarded connections are shut down properly
    /// rather than dropped, so TLS peers see `close_notify`.
    pub(crate) async fn checkout(&self) -> Option<BufReader<Connection>> {
        let mut idle = self.idle.lock().await;
        while let Some(entry) = idle.pop() {
            let mut reader = entry.reader;
            if entry.idle_since.elapsed() >= self.config.idle_timeout || !is_healthy(&mut reader) {
                // Aged out or the peer hung up. Close here, in async context —
                // `Drop` cannot await a TLS shutdown.
                let _ = reader.get_mut().shutdown().await;
                continue;
            }
            return Some(reader);
        }
        None
    }

    /// Return a healthy connection. Connections that errored must never come
    /// back here — the caller discards those instead.
    pub(crate) async fn checkin(&self, reader: BufReader<Connection>) {
        let mut idle = self.idle.lock().await;
        if idle.len() >= self.config.max_idle {
            // At capacity: close rather than grow past the configured bound.
            drop(idle);
            let mut reader = reader;
            let _ = reader.get_mut().shutdown().await;
            return;
        }
        idle.push(PooledConn {
            reader,
            idle_since: Instant::now(),
        });
    }

    /// Drain and shut down every idle connection.
    ///
    /// `Drop` cannot be async, so a long-lived process should call this before
    /// exit; otherwise TLS connections close without `close_notify` and the
    /// server logs an error for each.
    pub async fn close(&self) {
        let mut idle = self.idle.lock().await;
        let entries: Vec<PooledConn> = idle.drain(..).collect();
        drop(idle);
        for entry in entries {
            let mut reader = entry.reader;
            let _ = reader.get_mut().shutdown().await;
        }
    }

    /// Number of idle connections currently held. Test and diagnostic use.
    pub async fn idle_len(&self) -> usize {
        self.idle.lock().await.len()
    }
}

/// Is this connection still usable for a fresh request/response exchange?
///
/// Checked at checkout rather than relying on the write failing, because a
/// write to a peer-closed socket usually *succeeds* — the bytes land in the
/// send buffer and the reset arrives later. Without this check a stale
/// connection swallows the request and then reads EOF, which is indistinguishable
/// from "the engine applied the write and the response was lost" and therefore
/// cannot be retried under contract §4.
///
/// A quiet socket is healthy. Readable means either EOF (peer hung up) or
/// unexpected leftover bytes; both disqualify the connection.
fn is_healthy(reader: &mut BufReader<Connection>) -> bool {
    // Anything already buffered means a previous response was not fully
    // consumed; that would corrupt the next caller's read.
    if !reader.buffer().is_empty() {
        return false;
    }

    // Poll the read exactly once with a no-op waker. Deliberately NOT
    // `tokio::time::timeout(Duration::ZERO, ..)`: that arms a real timer, and
    // tokio's timer granularity is ~1ms, which costs a millisecond on every
    // checkout and makes a pooled request slower than a fresh connection.
    let mut future = pin!(reader.fill_buf());
    let mut cx = Context::from_waker(Waker::noop());
    // Pending means nothing to read — a correctly-drained idle connection, and
    // the only healthy state. Ready is disqualifying either way: empty means EOF
    // (the peer hung up), non-empty means unconsumed bytes from a previous
    // response that would corrupt the next caller's read.
    future.as_mut().poll(&mut cx).is_pending()
}
