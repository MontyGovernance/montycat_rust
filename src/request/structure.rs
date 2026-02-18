use super::store_request::structure::StoreRequestClient;
use crate::errors::MontycatClientError;
use indexmap::IndexMap;
use simd_json;

/// Represents a request to be sent to the Montycat server.
///
/// # Variants
/// - `Raw(IndexMap<String, Vec<String>>)` : A raw command represented as a map.
/// - `Store(StoreRequestClient)` : A store command represented by a `StoreRequestClient`.
///
/// Methods:
/// - `new_raw_command(command: Vec<String>, credentials: Vec<String>) -> Self` : Creates a new raw command request.
/// - `new_store_command(store_request: StoreRequestClient) -> Self` : Creates a new store command request.
/// - `byte_down(&self) -> Result<Vec<u8>, MontycatClientError>` : Serializes the request into a byte vector.   
///
/// Errors:
/// - `MontycatClientError::ClientEngineError(String)` : Returned if serialization fails.
///
#[derive(Debug, Clone)]
pub(crate) enum Req {
    Raw(IndexMap<String, Vec<String>>),
    Store(Box<StoreRequestClient>),
}

impl Req {
    /// Creates a new raw command request.
    ///
    /// # Arguments
    /// - `command: Vec<String>` : The command to be sent.
    /// - `credentials: Vec<String>` : The credentials for authentication.
    ///
    /// # Returns
    /// - `Self` : A new instance of `Req` representing the raw command.
    ///
    pub(crate) fn new_raw_command(command: Vec<String>, credentials: Vec<String>) -> Self {
        let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
        map.insert("raw".to_string(), command);
        map.insert("credentials".to_string(), credentials);
        Req::Raw(map)
    }

    /// Creates a new store command request.
    ///
    /// # Arguments
    /// - `store_request: StoreRequestClient` : The store request to be sent.
    ///
    /// # Returns
    /// - `Self` : A new instance of `Req` representing the store command.
    ///
    pub(crate) fn new_store_command(store_request: StoreRequestClient) -> Self {
        Req::Store(Box::new(store_request))
    }

    /// Serializes the request into a byte vector.
    ///
    /// # Returns
    /// - `Result<Vec<u8>, MontycatClientError>` : The serialized byte vector or an error if serialization fails.
    ///
    pub(crate) fn byte_down(&self) -> Result<Vec<u8>, MontycatClientError> {
        match self {
            Req::Raw(map) => {
                let json_str: String = simd_json::to_string(map)
                    .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
                let mut bytes: Vec<u8> = json_str.into_bytes();
                bytes.push(b'\n');
                Ok(bytes)
            }
            Req::Store(map) => {
                let json_str: String = simd_json::to_string(map)
                    .map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;
                let mut bytes: Vec<u8> = json_str.into_bytes();
                bytes.push(b'\n');
                Ok(bytes)
            }
        }
    }
}
