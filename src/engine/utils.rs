use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch::Receiver;
use tokio::time::timeout;
use serde_json::Value;
use crate::MontycatClientError;
use std::{sync::Arc, time::Duration};

const CHUNK_SIZE: usize = 1024 * 256;

pub async fn send_data(
    host: &str,
    port: u16,
    query: &[u8],
    callback: Option<Arc<dyn Fn(Value) + Send + Sync>>,
    stop_event: Option<&mut Receiver<bool>>,
) -> Result<Option<Vec<u8>>, MontycatClientError> {

    let mut stream = TcpStream::connect((host, port)).await.map_err(|e| MontycatClientError::EngineError(e.to_string()))?;

    stream.write_all(query).await.map_err(|e| MontycatClientError::EngineError(e.to_string()))?;
    stream.flush().await.map_err(|e| MontycatClientError::EngineError(e.to_string()))?;

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
            let n = stream.read(&mut chunk).await.map_err(|e| MontycatClientError::EngineError(e.to_string()))?;
            if n == 0 {
                break;
            }

            buf.extend_from_slice(&chunk[..n]);

            if buf.contains(&b'\n') {
                if let Ok(text) = std::str::from_utf8(&buf) {
                    let parsed = serde_json::from_str::<Value>(text.trim()).map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?;
                    if let Some(ref cb) = callback {
                        cb(parsed.clone());
                    }
                }
                buf.clear();
            }
        }

        stream.shutdown().await.map_err(|e| MontycatClientError::EngineError(e.to_string()))?;
        Ok(None)

    } else {

        loop {

            let mut chunk = vec![0u8; CHUNK_SIZE];

            let n = timeout(
                Duration::from_secs(120),
                stream.read(&mut chunk),
            ).await
            .map_err(|e| MontycatClientError::EngineError(e.to_string()))?
            .map_err(|e| MontycatClientError::EngineError(e.to_string()))?;

            if n == 0 {
                break;
            }

            buf.extend_from_slice(&chunk[..n]);
            if buf.contains(&b'\n') {
                break;
            }
        }

        stream.shutdown().await.map_err(|e| MontycatClientError::EngineError(e.to_string()))?;

        Ok(Some(buf))

    }
}

// Recursive JSON parser
// fn recursive_parse_json(value: &Value) -> Result<Value, Box<dyn Error>> {
//     Ok(match value {
//         Value::Object(map) => {
//             let mut new_map = serde_json::Map::new();
//             for (k, v) in map {
//                 new_map.insert(k.clone(), recursive_parse_json(v)?);
//             }
//             Value::Object(new_map)
//         }
//         Value::Array(arr) => {
//             let new_arr: Vec<Value> = arr.iter().map(recursive_parse_json).collect::<Result<_, _>>()?;
//             Value::Array(new_arr)
//         }
//         Value::String(s) => {
//             if let Ok(inner) = serde_json::from_str::<Value>(s) {
//                 recursive_parse_json(&inner)?
//             } else {
//                 Value::String(s.clone())
//             }
//         }
//         _ => value.clone(),
//     })
// }
