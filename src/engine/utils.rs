use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch::Receiver;
use tokio::time::timeout;
use crate::MontycatClientError;
use std::{sync::Arc, time::Duration};

#[cfg(feature = "tls")]
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
#[cfg(feature = "tls")]
use tokio_rustls::TlsConnector;
#[cfg(feature = "tls")]
use rustls_pki_types::ServerName;

const CHUNK_SIZE: usize = 1024 * 256;

pub async fn send_data(
    host: &str,
    port: u16,
    query: &[u8],
    callback: Option<Arc<dyn Fn(&Vec<u8>) + Send + Sync>>,
    stop_event: Option<&mut Receiver<bool>>,
    use_tls: bool,
) -> Result<Option<Vec<u8>>, MontycatClientError> {

    let mut stream: TcpStream = TcpStream::connect((host, port)).await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

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
            let tls_stream = connector.connect(server_name, stream).await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
            stream = TcpStream::from_std(tls_stream.get_ref().try_clone().map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?).map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
        }
        #[cfg(not(feature = "tls"))]
        {
            return Err(MontycatClientError::ClientEngineError("TLS feature not enabled".to_string()));
        }
    }

    stream.write_all(query).await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
    stream.flush().await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

    let mut buf = vec![];

    let is_subscription = query.windows(9).any(|w| w == b"subscribe");

    if is_subscription {
        loop {

            if let Some(ref stop) = stop_event {
                if *stop.borrow() {
                    break;
                }
            }

            let mut chunk = vec![0u8; CHUNK_SIZE];
            let n = stream.read(&mut chunk).await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
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

        stream.shutdown().await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
        Ok(None)

    } else {

        loop {

            let mut chunk = vec![0u8; CHUNK_SIZE];

            let n = timeout(
                Duration::from_secs(120),
                stream.read(&mut chunk),
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

        stream.shutdown().await.map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
        Ok(Some(buf))

    }
}