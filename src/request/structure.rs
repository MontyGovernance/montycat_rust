use indexmap::IndexMap;
use crate::errors::MontycatClientError;
use super::store_request::structure::StoreRequestClient;
use simd_json;

#[derive(Debug, Clone)]
pub(crate) enum Req {
    Raw(IndexMap<String, Vec<String>>),
    Store(StoreRequestClient),
}

impl Req {

    pub fn new_raw_command(command: Vec<String>, credentials: Vec<String>) -> Self {
        let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
        map.insert("raw".to_string(), command);
        map.insert("credentials".to_string(), credentials);
        Req::Raw(map)
    }

    pub fn new_store_command(store_request: StoreRequestClient) -> Self {
        Req::Store(store_request)
    }

    pub fn byte_down(&self) -> Result<Vec<u8>, MontycatClientError> {
        match self {
            Req::Raw(map) => {
                let json_str: String = simd_json::to_string(map).map_err(|e| MontycatClientError::EngineError(e.to_string()))?;
                let mut bytes: Vec<u8> = json_str.into_bytes();
                bytes.push(b'\n');
                Ok(bytes)
            },
            Req::Store(map) => {
                let json_str: String = simd_json::to_string(map).map_err(|e| MontycatClientError::EngineError(e.to_string()))?;
                let mut bytes: Vec<u8> = json_str.into_bytes();
                bytes.push(b'\n');
                Ok(bytes)
            },
        }
    }

}