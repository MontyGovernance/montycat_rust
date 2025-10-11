
use std::{fmt::Display};
use xxhash_rust::xxh32::xxh32;
use crate::{MontycatClientError, global::PRIMITIVE_TYPES};

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
pub fn convert_custom_key<T: Display>(key: T) -> String {
    let key_str = key.to_string();
    xxh32(key_str.as_bytes(), 0).to_string()
}

pub fn is_custom_type(type_name: &str) -> Option<&str> {
    let parsed_type_name: &str = type_name.rsplit("::").next().unwrap_or(type_name);
    if !PRIMITIVE_TYPES.contains(&parsed_type_name) {
        Some(parsed_type_name)
    } else {
        None
    }
}

pub async fn merge_keys(bulk_keys: Option<Vec<String>>, bulk_custom_keys: Option<Vec<String>>) -> Result<Vec<String>, MontycatClientError> {

    if bulk_keys.is_none() && bulk_custom_keys.is_none() {
        return Err(MontycatClientError::NoValidInputProvided);
    }

    let bulk_keys_clone: Option<Vec<String>> = bulk_keys.clone();
    let custom_keys_clone: Option<Vec<String>> = bulk_custom_keys.clone();

    let keys_processed: Vec<String> = tokio::task::spawn_blocking(move || {

        let mut keys_merged: Vec<String> = Vec::with_capacity(
            bulk_keys_clone.as_ref().map_or(0, |v| v.len())
            + custom_keys_clone.as_ref().map_or(0, |v| v.len())
        );

        if let Some(bulk_keys) = bulk_keys_clone {
            keys_merged.extend(bulk_keys);
        }

        if let Some(custom) = custom_keys_clone {
            keys_merged.extend(custom.into_iter().map(convert_custom_key));
        }

        keys_merged

    }).await.map_err(|e| MontycatClientError::AsyncRuntimeError(e.to_string()))?;

    if keys_processed.is_empty() {
        return Err(MontycatClientError::NoValidInputProvided);
    }

    Ok(keys_processed)

}
