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
    /// Only `semantic_search` honors min_score; skipped when None so the wire
    /// is unchanged for every other command (the server defaults it to None).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f32>,
    /// Hybrid metadata pre-filter for `semantic_search` — a JSON-encoded
    /// criteria object in the same shape `lookup_keys_where` takes (a hard
    /// AND constraint; ranking stays pure cosine). Skipped when None so the
    /// wire is unchanged for every other command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_filter: Option<String>,
    /// Optional precomputed vector for a single write or semantic query.
    /// Supplying it bypasses server-side embedding for that operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_vector: Option<Vec<f32>>,
    /// Existing item keys mapped to precomputed vectors. Used only by
    /// `semantic_upsert_vectors`.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub semantic_vectors: HashMap<String, Vec<f32>>,
    /// Precomputed vectors paired by position with the values in a bulk insert.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub semantic_vector_list: Vec<Vec<f32>>,
    /// Per-request override for synchronous index waiting on persistent writes.
    /// `Some(true)` → the write returns only after its indexes update
    /// (read-your-writes); `Some(false)` → fire-and-forget; `None` → use the
    /// server-wide default. Skipped when None so the wire is unchanged for
    /// callers that don't set it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_index: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::StoreRequestClient;
    use std::collections::HashMap;

    #[test]
    fn omits_empty_semantic_vector_fields() {
        let value = serde_json::to_value(StoreRequestClient::default()).unwrap();

        assert!(value.get("semantic_vector").is_none());
        assert!(value.get("semantic_vectors").is_none());
        assert!(value.get("semantic_vector_list").is_none());
    }

    #[test]
    fn serializes_precomputed_vectors_on_the_wire() {
        let mut semantic_vectors = HashMap::new();
        semantic_vectors.insert("42".to_owned(), vec![0.1, 0.2]);
        let request = StoreRequestClient {
            semantic_vector: Some(vec![0.3, 0.4]),
            semantic_vectors,
            semantic_vector_list: vec![vec![0.5, 0.6]],
            ..Default::default()
        };

        let value = serde_json::to_value(request).unwrap();

        assert_eq!(
            serde_json::from_value::<Vec<f32>>(value["semantic_vector"].clone()).unwrap(),
            vec![0.3, 0.4]
        );
        assert_eq!(
            serde_json::from_value::<HashMap<String, Vec<f32>>>(value["semantic_vectors"].clone())
                .unwrap(),
            HashMap::from([("42".to_owned(), vec![0.1, 0.2])])
        );
        assert_eq!(
            serde_json::from_value::<Vec<Vec<f32>>>(value["semantic_vector_list"].clone()).unwrap(),
            vec![vec![0.5, 0.6]]
        );
    }
}
