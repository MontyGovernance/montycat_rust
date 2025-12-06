use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::watch::Receiver;
use tokio::time::timeout;
use crate::MontycatClientError;
use std::{sync::Arc, time::Duration};
#[cfg(feature = "tls")]
use tokio_rustls::{rustls::{ClientConfig, RootCertStore}, client::TlsStream};
#[cfg(feature = "tls")]
use tokio_rustls::TlsConnector;
#[cfg(feature = "tls")]
use rustls_pki_types::ServerName;

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
    #[cfg(not(feature = "tls"))]
    Plain(TcpStream),
    #[cfg(feature = "tls")]
    Tls(TlsStream<TcpStream>),
}

impl Connection {
    /// Splits the connection into a reader and writer.
    /// This is useful for concurrently reading from and writing to the connection.
    /// 
    /// # Returns
    /// 
    /// - `(Box<dyn AsyncRead + Unpin + Send>, Box<dyn AsyncWrite + Unpin + Send>)`:
    ///   A tuple containing the reader and writer.
    ///
    pub(crate) fn split(self) -> (Box<dyn AsyncRead + Unpin + Send>, Box<dyn AsyncWrite + Unpin + Send>) {
        match self {
            #[cfg(not(feature = "tls"))]
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
}

/// Sends data to the Montycat server and handles the response.
/// Supports both plain TCP and TLS connections based on the `use_tls` flag.
/// Can handle both standard requests and subscription requests.
///
/// # Arguments
/// 
/// - `host: &str`: The hostname of the Montycat server.
/// - `port: u16`: The port number of the Montycat server.
/// - `query: &[u8]`: The query to be sent to the server as a byte slice.
/// - `callback: Option<Arc<dyn Fn(&Vec<u8>) + Send + Sync>>`: An optional callback function to handle incoming data for subscriptions.
/// - `stop_event: Option<&mut Receiver<bool>>`: An optional stop event to terminate subscriptions.
/// - `use_tls: bool`: A flag indicating whether to use TLS for the connection.
///
/// # Returns
/// 
/// - `Result<Option<Vec<u8>>, MontycatClientError>`:
///   - For standard requests, returns `Ok(Some(response_bytes))` containing the server's response.
///   - For subscription requests, returns `Ok(None)` after the subscription is terminated.
///   - Returns an error of type `MontycatClientError` if any issues occur during the process.
///
pub(crate) async fn send_data(
    host: &str,
    port: u16,
    query: &[u8],
    callback: Option<Arc<dyn Fn(&Vec<u8>) + Send + Sync>>,
    stop_event: Option<&mut Receiver<bool>>,
    use_tls: bool,
) -> Result<Option<Vec<u8>>, MontycatClientError> {

    let host: String = host.to_string();
    let plain_stream: TcpStream = TcpStream::connect((host.as_ref(), port)).await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
    #[cfg(feature = "tls")]
    let mut tls_stream: Option<tokio_rustls::client::TlsStream<TcpStream>> = None;

    if use_tls {
        #[cfg(feature = "tls")]
        {
            let mut root_cert_store = RootCertStore::empty();
            root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();

            let connector = TlsConnector::from(Arc::new(config));
            let server_name = ServerName::try_from(host).map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

            match timeout(
                Duration::from_secs(10),
                connector.connect(server_name, plain_stream)
            ).await {
                Ok(Ok(stream)) => tls_stream = Some(stream),
                Ok(Err(e)) => return Err(MontycatClientError::ClientEngineError(format!("TLS handshake failed: {}", e))),
                Err(_) => return Err(MontycatClientError::ClientEngineError("TLS handshake timed out".to_string())),
            };
        }

        #[cfg(not(feature = "tls"))]
        {
            return Err(MontycatClientError::ClientEngineError("TLS feature not enabled".to_string()));
        }
    }

    #[cfg(feature = "tls")]
    let connection  = if let Some(tls) = tls_stream {
        Connection::Tls(tls)
    } else {
        return Err(MontycatClientError::ClientEngineError("TLS stream not initialized".to_string()));
    };

    #[cfg(not(feature = "tls"))]
    let connection = Connection::Plain(plain_stream);

    let (mut reader, mut writer) = connection.split();

    writer.write_all(query).await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
    writer.flush().await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

    let mut buf = vec![];

    let is_subscription = query.windows(9).any(|w| w == b"subscribe");

    if is_subscription {
        loop {

            if let Some(ref stop) = stop_event {
                // if *stop.borrow() {
                //     break;
                // }
                if let Ok(true) = stop.has_changed()
                    && *stop.borrow() {
                        break;
                    }
            }

            let mut chunk = vec![0u8; CHUNK_SIZE];
            let n = reader.read(&mut chunk).await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
            if n == 0 {
                break;
            }

            buf.extend_from_slice(&chunk[..n]);

            if buf.contains(&b'\n') {
                if let Some(ref cb) = callback {
                    cb(&buf);
                }
                buf.clear();
            }
        }

        writer.shutdown().await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
        Ok(None)

    } else {

        loop {

            let mut chunk = vec![0u8; CHUNK_SIZE];

            let n = timeout(
                Duration::from_secs(120),
                reader.read(&mut chunk),
            ).await
            .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?
            .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

            if n == 0 {
                break;
            }

            buf.extend_from_slice(&chunk[..n]);
            if buf.contains(&b'\n') {
                break;
            }
        }

        writer.shutdown().await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
        Ok(Some(buf))

    }
}