use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a store request to be sent to the Montycat server from the client side.
///
/// # Fields
/// - `schema: Option<String>` : The schema to be used.
/// - `username: String` : The username for authentication.
/// - `password: String` : The password for authentication.
/// - `keyspace: String` : The keyspace to be used.
/// - `store: String` : The store to be used.
/// - `persistent: bool` : Indicates if the store is persistent.
/// - `distributed: bool` : Indicates if the store is distributed.
/// - `limit_output: HashMap<String, usize>` : Limits for output.
/// - `key: Option<String>` : The key for the operation.
/// - `value: String` : The value for the operation.
/// - `command: String` : The command to be executed.
/// - `expire: u64` : Expiration time for the key.
/// - `bulk_values: Vec<String>` : Bulk values for the operation.
/// - `bulk_keys: Vec<String>` : Bulk keys for the operation.
/// - `bulk_keys_values: HashMap<String, String>` : Bulk key-value pairs for the operation.
/// - `search_criteria: String` : Criteria for searching.
/// - `with_pointers: bool` : Indicates if pointers should be included.
/// - `key_included: bool` : Indicates if the key is included in the response.
/// - `volumes: Vec<String>` : Volumes to be used.
/// - `latest_volume: bool` : Indicates if the latest volume should be used.
/// - `pointers_metadata: bool` : Indicates if pointers metadata should be included.
///
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct StoreRequestClient {
    pub schema: Option<String>,
    pub username: String,
    pub password: String,
    pub keyspace: String,
    pub store: String,
    pub persistent: bool,
    pub distributed: bool,
    pub limit_output: HashMap<String, usize>,
    pub key: Option<String>,
    pub value: String,
    pub command: String,
    pub expire: u64,
    pub bulk_values: Vec<String>,
    pub bulk_keys: Vec<String>,
    pub bulk_keys_values: HashMap<String, String>,
    pub search_criteria: String,
    pub with_pointers: bool,

    pub key_included: bool,
    pub volumes: Vec<String>,
    pub latest_volume: bool,
    pub pointers_metadata: bool,
}
