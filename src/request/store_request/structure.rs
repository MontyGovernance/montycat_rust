use std::collections::HashMap;
use serde::{Serialize, Deserialize};

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
