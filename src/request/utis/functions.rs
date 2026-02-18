use crate::{MontycatClientError, global::PRIMITIVE_TYPES, tools::functions::process_json_value};
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use xxhash_rust::xxh32::xxh32;

/// Converts a custom key (integer or string) into a hashed value using xxHash.
///
/// # Arguments
///
/// * `key` - A key that implements `Display` (e.g., integer, string, etc.)
///
/// # Returns
///
/// * `String` - The xxHash digest of the provided key, returned as a string.
///
/// This function ensures consistent hashing for any key type.
pub(crate) fn convert_custom_key<T: Display>(key: T) -> String {
    let key_str = key.to_string();
    xxh32(key_str.as_bytes(), 0).to_string()
}

/// Determines if a given type name is a custom type (not a primitive type).
///
/// Arguments
/// * `type_name: &str` - The name of the type to check.
///
/// Returns
/// * `Option<&str>` - Returns `Some(&str)` with the type name if it is a custom type, otherwise returns `None` if it is a primitive type.
///
pub(crate) fn is_custom_type(type_name: &str) -> Option<&str> {
    let parsed_type_name: &str = type_name.rsplit("::").next().unwrap_or(type_name);
    if !PRIMITIVE_TYPES.contains(&parsed_type_name) {
        Some(parsed_type_name)
    } else {
        None
    }
}

/// Merges bulk keys and custom keys into a single vector of keys.
///
/// # Arguments
/// * `bulk_keys: Option<Vec<String>>` - A vector of bulk keys.
/// * `bulk_custom_keys: Option<Vec<String>>` - A vector of custom keys.
///
/// # Returns
/// * `Result<Vec<String>, MontycatClientError>` - A result containing the merged vector of keys or an error if no valid input is provided.
///
pub(crate) async fn merge_keys(
    bulk_keys: Option<Vec<String>>,
    bulk_custom_keys: Option<Vec<String>>,
) -> Result<Vec<String>, MontycatClientError> {
    if bulk_keys.is_none() && bulk_custom_keys.is_none() {
        return Err(MontycatClientError::ClientNoValidInputProvided);
    }

    let bulk_keys_clone: Option<Vec<String>> = bulk_keys.clone();
    let custom_keys_clone: Option<Vec<String>> = bulk_custom_keys.clone();

    let keys_processed: Vec<String> = tokio::task::spawn_blocking(move || {
        let mut keys_merged: Vec<String> = Vec::with_capacity(
            bulk_keys_clone.as_ref().map_or(0, |v| v.len())
                + custom_keys_clone.as_ref().map_or(0, |v| v.len()),
        );

        if let Some(bulk_keys) = bulk_keys_clone {
            keys_merged.extend(bulk_keys);
        }

        if let Some(custom) = custom_keys_clone {
            keys_merged.extend(custom.into_iter().map(convert_custom_key));
        }

        keys_merged
    })
    .await
    .map_err(|e| MontycatClientError::ClientAsyncRuntimeError(e.to_string()))?;

    if keys_processed.is_empty() {
        return Err(MontycatClientError::ClientNoValidInputProvided);
    }

    Ok(keys_processed)
}

/// Merges bulk key-value pairs and custom key-value pairs into a single HashMap.
///
/// # Arguments
/// * `bulk_keys_values: Vec<HashMap<String, T>>` - A vector of HashMaps containing bulk key-value pairs.
/// * `bulk_custom_keys_values: Vec<HashMap<String, T>>` - A vector of HashMaps containing custom key-value pairs.
///
/// # Returns
/// * `Result<HashMap<String, String>, MontycatClientError>` - A result containing the merged HashMap of key-value pairs or an error if serialization fails.
///
pub(crate) async fn merge_bulk_keys_values<T>(
    bulk_keys_values: Vec<HashMap<String, T>>,
    bulk_custom_keys_values: Vec<HashMap<String, T>>,
) -> Result<HashMap<String, String>, MontycatClientError>
where
    T: Serialize + Send + 'static,
{
    let res: HashMap<String, String> = tokio::task::spawn_blocking(move || {
        let mut bulk_keys_values = bulk_keys_values;

        if !bulk_custom_keys_values.is_empty() {
            for custom_key_value in bulk_custom_keys_values {
                for (custom_key, value) in custom_key_value {
                    let internal_key = convert_custom_key(&custom_key);
                    let mut map = HashMap::new();
                    map.insert(internal_key, value);
                    bulk_keys_values.push(map);
                }
            }
        }

        let merged: HashMap<String, T> = bulk_keys_values
            .into_par_iter()
            .flat_map(|map| map.into_par_iter())
            .collect();

        let serialized: HashMap<String, String> = merged
            .into_par_iter()
            .map(|(k, v)| process_json_value(&v).map(|val| (k, val)))
            .collect::<Result<HashMap<_, _>, MontycatClientError>>()?;

        Ok::<_, MontycatClientError>(serialized)
    })
    .await
    .map_err(|e| MontycatClientError::ClientAsyncRuntimeError(e.to_string()))??;

    Ok(res)
}

/// Fulfills a subscription request by creating the appropriate byte vector to be sent to the Montycat server.
///
/// # Arguments
/// - `store: &str` : The store to subscribe to.
/// - `keyspace: &str` : The keyspace to subscribe to.
/// - `key: Option<String>` : An optional key to subscribe to.
/// - `username: &str` : The username for authentication.
/// - `password: &str` : The password for authentication.
///
/// # Returns
/// - `Result<Vec<u8>, MontycatClientError>` : A result containing the byte vector for the subscription request or an error if serialization fails.
///
pub(crate) fn fulfil_subscription_request(
    store: &str,
    keyspace: &str,
    key: Option<String>,
    username: &str,
    password: &str,
) -> Result<Vec<u8>, MontycatClientError> {
    let mut indexmap = IndexMap::new();

    indexmap.insert("subscribe".to_string(), serde_json::Value::Bool(true));
    indexmap.insert(
        "store".to_string(),
        serde_json::Value::String(store.to_owned()),
    );
    indexmap.insert(
        "keyspace".to_string(),
        serde_json::Value::String(keyspace.to_owned()),
    );
    if let Some(k) = key {
        indexmap.insert("key".to_string(), serde_json::Value::String(k));
    }
    indexmap.insert(
        "username".to_string(),
        serde_json::Value::String(username.to_owned()),
    );
    indexmap.insert(
        "password".to_string(),
        serde_json::Value::String(password.to_owned()),
    );

    let bytes = serde_json::to_vec(&indexmap)
        .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

    Ok(bytes)
}
