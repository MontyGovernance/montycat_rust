use crate::MontycatClientError;
use crate::engine::structure::Engine;
#[cfg(feature = "tls")]
use rustls_pki_types::ServerName;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::watch::Receiver;
use tokio::time::timeout;
#[cfg(feature = "tls")]
use tokio_rustls::TlsConnector;
#[cfg(feature = "tls")]
use tokio_rustls::{
    client::TlsStream,
    rustls::{ClientConfig, RootCertStore},
};

pub(crate) type StreamCallback = Arc<dyn Fn(&mut [u8]) + Send + Sync>;

const CHUNK_SIZE: usize = 1024 * 256;

/// Represents a connection, either plain TCP or TLS.
/// This enum is used internally to abstract over the connection type.
///
/// # Variants
/// - `Plain(TcpStream)`: Represents a plain TCP connection.
/// - `Tls(TlsStream<TcpStream>)`: Represents a TLS-encrypted connection.
///
/// # Methods
/// - `split(self) -> (Box<dyn AsyncRead + Unpin + Send>, Box<dyn AsyncWrite + Unpin + Send>)`:
///   Splits the connection into a reader and writer.
///
pub(crate) enum Connection {
    Plain(TcpStream),
    #[cfg(feature = "tls")]
    Tls(Box<TlsStream<TcpStream>>),
}

impl Connection {
    /// Splits the connection into a reader and writer.
    /// This is useful for concurrently reading from and writing to the connection.
    ///
    /// Consumes the connection, so it cannot be used by a pool, which must hold
    /// the connection whole alongside its `BufReader`. Retained for the
    /// subscription path, which reads and writes concurrently and is never
    /// pooled (see the pooling contract §5).
    ///
    /// # Returns
    ///
    /// - `(Box<dyn AsyncRead + Unpin + Send>, Box<dyn AsyncWrite + Unpin + Send>)`:
    ///   A tuple containing the reader and writer.
    ///
    pub(crate) fn split(
        self,
    ) -> (
        Box<dyn AsyncRead + Unpin + Send>,
        Box<dyn AsyncWrite + Unpin + Send>,
    ) {
        match self {
            Connection::Plain(stream) => {
                let (r, w) = tokio::io::split(stream);
                (Box::new(r), Box::new(w))
            }
            #[cfg(feature = "tls")]
            Connection::Tls(stream) => {
                let (r, w) = tokio::io::split(stream);
                (Box::new(r), Box::new(w))
            }
        }
    }

    /// Closes the connection, sending TLS `close_notify` where applicable.
    ///
    /// Dropping a `TlsStream` closes abruptly, which the server logs as a TLS
    /// error. Every path that discards a pooled connection while still in async
    /// context must call this rather than relying on `Drop`.
    pub(crate) async fn shutdown(&mut self) -> Result<(), MontycatClientError> {
        match self {
            Connection::Plain(stream) => stream.shutdown().await,
            #[cfg(feature = "tls")]
            Connection::Tls(stream) => stream.shutdown().await,
        }
        .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))
    }
}

// Direct `AsyncRead`/`AsyncWrite` so a pool can hold the connection whole. The
// enum cannot be a trait object here because the TLS variant only exists under
// a feature, so both impls delegate per variant.
impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Connection::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            #[cfg(feature = "tls")]
            Connection::Tls(stream) => Pin::new(stream.as_mut()).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Connection::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            #[cfg(feature = "tls")]
            Connection::Tls(stream) => Pin::new(stream.as_mut()).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Connection::Plain(stream) => Pin::new(stream).poll_flush(cx),
            #[cfg(feature = "tls")]
            Connection::Tls(stream) => Pin::new(stream.as_mut()).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Connection::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
            #[cfg(feature = "tls")]
            Connection::Tls(stream) => Pin::new(stream.as_mut()).poll_shutdown(cx),
        }
    }
}

/// Sends data to the Montycat server and handles the response.
/// Supports both plain TCP and TLS connections based on the `use_tls` flag.
/// Can handle both standard requests and subscription requests.
///
/// # Arguments
///
/// - `engine: &Engine`: Supplies host, port, and TLS setting, and — once pooling
///   lands — the connection pool itself. Taking the engine rather than a
///   `(host, port, use_tls)` triplet is what gives this function a route to a pool.
/// - `query: &[u8]`: The query to be sent to the server as a byte slice.
/// - `callback: Option<StreamCallback>`: An optional callback invoked once per
///   response frame. Supplying one is what makes this a subscription — the mode
///   is never inferred from the payload.
/// - `stop_event: Option<&mut Receiver<bool>>`: An optional stop event to terminate subscriptions.
/// - `port_override: Option<u16>`: Connect to this port instead of `engine.port`.
///   Only subscriptions use it, for the `port + 1` subscription port.
///
/// # Returns
///
/// - `Result<Option<Vec<u8>>, MontycatClientError>`:
///   - For standard requests, returns `Ok(Some(response_bytes))` containing the server's response.
///   - For subscription requests, returns `Ok(None)` after the subscription is terminated.
///   - Returns an error of type `MontycatClientError` if any issues occur during the process.
///
pub(crate) async fn send_data(
    engine: &Engine,
    query: &[u8],
    callback: Option<StreamCallback>,
    stop_event: Option<&mut Receiver<bool>>,
    port_override: Option<u16>,
) -> Result<Option<Vec<u8>>, MontycatClientError> {
    let port: u16 = port_override.unwrap_or(engine.port);

    // A subscription is the call that supplies a callback — never inferred from
    // the payload. Scanning the request for "subscribe" misread any record whose
    // value merely contained that word, routing it into the streaming branch,
    // which has no read timeout and therefore hung forever.
    match callback {
        Some(cb) => subscription(engine, port, query, cb, stop_event).await,
        None => request(engine, port, query).await,
    }
}

/// Open a fresh connection, performing the TLS handshake when configured.
async fn connect(engine: &Engine, port: u16) -> Result<Connection, MontycatClientError> {
    let use_tls: bool = engine.use_tls;
    let host: String = engine.host.clone();
    let plain_stream: TcpStream = TcpStream::connect((host.as_ref(), port))
        .await
        .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
    let connection = if use_tls {
        #[cfg(feature = "tls")]
        {
            let mut root_cert_store = RootCertStore::empty();
            root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();

            let connector = TlsConnector::from(Arc::new(config));
            let server_name = ServerName::try_from(host)
                .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

            let tls_stream = match timeout(
                Duration::from_secs(10),
                connector.connect(server_name, plain_stream),
            )
            .await
            {
                Ok(Ok(stream)) => stream,
                Ok(Err(e)) => {
                    return Err(MontycatClientError::ClientEngineError(format!(
                        "TLS handshake failed: {}",
                        e
                    )));
                }
                Err(_) => {
                    return Err(MontycatClientError::ClientEngineError(
                        "TLS handshake timed out".to_string(),
                    ));
                }
            };
            Connection::Tls(Box::new(tls_stream))
        }

        #[cfg(not(feature = "tls"))]
        {
            return Err(MontycatClientError::ClientEngineError(
                "TLS feature not enabled".to_string(),
            ));
        }
    } else {
        Connection::Plain(plain_stream)
    };

    Ok(connection)
}

/// Streaming path. Never pooled (pooling contract §5): a subscription is
/// long-lived, streams many responses to one request, and lives on the
/// `port + 1` subscription port.
async fn subscription(
    engine: &Engine,
    port: u16,
    query: &[u8],
    callback: StreamCallback,
    stop_event: Option<&mut Receiver<bool>>,
) -> Result<Option<Vec<u8>>, MontycatClientError> {
    // `split()` here because this path reads and writes concurrently.
    let (reader, mut writer) = connect(engine, port).await?.split();
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, reader);

    writer
        .write_all(query)
        .await
        .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
    writer
        .flush()
        .await
        .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

    let mut buf = vec![];
    loop {
        if let Some(ref stop) = stop_event
            && let Ok(true) = stop.has_changed()
            && *stop.borrow()
        {
            break;
        }

        buf.clear();
        let n = reader
            .read_until(b'\n', &mut buf)
            .await
            .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
        if n == 0 {
            break;
        }

        // One frame per callback. The previous code handed the callback whatever
        // had accumulated — two frames arriving in one chunk were delivered
        // concatenated as a single event, and a partial third frame in the same
        // chunk was dropped by the following `clear()`.
        callback(buf.as_mut_slice());
    }

    // Unconditional close: the server's watchers tear down on EOF, and leaving
    // them alive deadlocks later remove_keyspace/remove_store on the same store.
    writer
        .shutdown()
        .await
        .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
    Ok(None)
}

/// Request/response path — the only one that may use a pool.
async fn request(
    engine: &Engine,
    port: u16,
    query: &[u8],
) -> Result<Option<Vec<u8>>, MontycatClientError> {
    let pool = engine.pool.clone();

    // A connection handed back by the pool may have been closed by the peer
    // while idle. That is discovered on write, having transmitted nothing, so
    // replaying is safe — contract §4 permits exactly one retry, and only here.
    if let Some(pool) = &pool
        && let Some(reader) = pool.checkout().await
    {
        match exchange(reader, query).await {
            Ok((reader, response)) => {
                pool.checkin(reader).await;
                return Ok(Some(response));
            }
            Err(Exchange::Write(_)) => {
                // Stale socket. Fall through to a fresh connection below.
            }
            Err(Exchange::Read(e)) => {
                // NEVER retry a read failure. The engine may have already applied
                // an insert/update/delete and only the response was lost; these
                // commands are not idempotent and the wire has no request IDs, so
                // replaying would duplicate user data (contract §4).
                return Err(e);
            }
        }
    }

    let reader = BufReader::with_capacity(CHUNK_SIZE, connect(engine, port).await?);
    match exchange(reader, query).await {
        Ok((mut reader, response)) => {
            match &pool {
                Some(pool) => pool.checkin(reader).await,
                // Unpooled: close as before, so behaviour is unchanged when
                // pooling is disabled.
                None => {
                    reader.get_mut().shutdown().await?;
                }
            }
            Ok(Some(response))
        }
        Err(Exchange::Write(e) | Exchange::Read(e)) => Err(e),
    }
}

/// Which half of the exchange failed. The distinction is what makes the §4
/// retry rule expressible: a write failure on a pooled connection is safe to
/// replay, a read failure never is.
enum Exchange {
    Write(MontycatClientError),
    Read(MontycatClientError),
}

/// Write one request and read exactly one newline-framed response, returning
/// the connection so a caller may return it to the pool.
async fn exchange(
    mut reader: BufReader<Connection>,
    query: &[u8],
) -> Result<(BufReader<Connection>, Vec<u8>), Exchange> {
    let write = async {
        reader.get_mut().write_all(query).await?;
        reader.get_mut().flush().await
    }
    .await;

    if let Err(e) = write {
        return Err(Exchange::Write(MontycatClientError::ClientEngineError(
            e.to_string(),
        )));
    }

    let mut buf = vec![];
    // Anything the socket delivered past the newline stays buffered in `reader`
    // and travels with the connection, rather than being appended here.
    let read = timeout(Duration::from_secs(120), reader.read_until(b'\n', &mut buf)).await;

    match read {
        Err(e) => Err(Exchange::Read(MontycatClientError::ClientEngineError(
            e.to_string(),
        ))),
        Ok(Err(e)) => Err(Exchange::Read(MontycatClientError::ClientEngineError(
            e.to_string(),
        ))),
        // EOF before any response byte. The peer hung up mid-exchange; returning
        // `Ok` with an empty buffer would present that as a successful empty
        // response, and callers would parse garbage.
        Ok(Ok(0)) => Err(Exchange::Read(MontycatClientError::ClientEngineError(
            "connection closed before a response was received".to_string(),
        ))),
        Ok(Ok(_)) => Ok((reader, buf)),
    }
}
