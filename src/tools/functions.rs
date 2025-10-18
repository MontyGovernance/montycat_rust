use crate::{errors::MontycatClientError};
use crate::traits::RuntimeSchema;
use serde::Serialize;
use serde_json::{Value, Map};
use crate::request::utis::functions::is_custom_type;
use std::collections::HashSet;
use std::{any::type_name};
use rayon::prelude::*;

/// Processes a JSON-serializable value into a JSON string.
/// 
/// # Arguments
/// - `value: &T` : A reference to the value to be serialized.
///
/// # Returns
/// - `Result<String, MontycatClientError>` : The serialized JSON string or an error if serialization fails.
///
pub(crate) fn process_json_value<T>(value: &T) -> Result<String, MontycatClientError>
where T: Serialize,
{
    let value_to_send: String = simd_json::to_string(value)
        .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

    Ok(value_to_send)
}

/// Processes a value into a JSON string, handling special fields for pointers and timestamps.
/// 
/// # Arguments
/// - `value: T` : The value to be processed.
/// 
/// # Returns
/// - `Result<String, MontycatClientError>` : The processed JSON string or an error if processing fails.
///
pub(crate) fn process_value<T>(value: T) -> Result<String, MontycatClientError>
where
    T: Serialize + RuntimeSchema,
{
    let pointer_and_timestamp_fields: Vec<(&'static str, &'static str)> = value.pointer_and_timestamp_fields();
    let mut val_as_map: Map<String, Value> = Map::new();

    if !pointer_and_timestamp_fields.is_empty() {

        let mut pointers: Map<String, Value> = Map::new();
        let mut timestamps: Map<String, Value> = Map::new();

        let val_as_serde = serde_json::to_value(&value)
            .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

        if let Some(obj) = val_as_serde.as_object() {
            val_as_map = obj.to_owned();
        }

        let mut removal: Vec<&str> = Vec::new();

        for (field_name, field_type) in pointer_and_timestamp_fields {
            if let Some(field_value) = val_as_map.get(field_name) {
                if field_type == "Pointer" {
                    let pointing_key = field_value.get("key");
                    let pointing_keyspace = field_value.get("keyspace");

                    match (pointing_key, pointing_keyspace) {
                        (Some(key), Some(keyspace)) => {

                            let content: Value = serde_json::json!([keyspace, key]);

                            pointers.insert(field_name.to_string(), content);
                        },
                        _ => {
                            return Err(MontycatClientError::ClientNoValidInputProvided);
                        }
                    }

                    removal.push(field_name);
                } else if field_type == "Timestamp" {

                    let timestamp_value: &Value = field_value.get("timestamp").ok_or(MontycatClientError::ClientNoValidInputProvided)?;

                    timestamps.insert(field_name.to_string(), timestamp_value.clone());
                    removal.push(field_name);
                }
            }
        }

        for field in removal {
            val_as_map.remove(field);
        }

        if !pointers.is_empty() {
            val_as_map.insert("pointers".into(), pointers.into());
        }

        if !timestamps.is_empty() {
            val_as_map.insert("timestamps".into(), timestamps.into());
        }

    }

    let value_to_send: String = {
        if val_as_map.is_empty() {
            simd_json::to_string(&value)
                .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?
        } else {
            simd_json::to_string(&val_as_map)
                .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?
        }
    };

    Ok(value_to_send)
}

/// Determines the Montycat field type for a given Rust type name.
/// 
/// # Arguments
/// * `field_type: &str` - The name of the Rust type.
///
/// # Returns
/// * `Result<&'static str, MontycatClientError>` - The corresponding Montycat field type or an error if the type is unsupported.
///
pub(crate) fn define_type(field_type: &str) -> Result<&'static str, MontycatClientError> {

    match field_type.replace(' ', "").as_str() {
        // Strings
        "String" | "&str" | "char" => Ok("String"),

        // Numbers
        "i8" | "i16" | "i32" | "i64" | "i128"
        | "u8" | "u16" | "u32" | "u64" | "u128"
        | "isize" | "usize" => Ok("Number"),

        // Floating points
        "f32" | "f64" => Ok("Float"),

        // Boolean
        "bool" => Ok("Boolean"),

        // Collections
        s if s.starts_with("Vec<") => Ok("Array"),
        s if s.starts_with("HashMap<") => Ok("Object"),
        s if s.starts_with("BTreeMap<") => Ok("Object"),
        s if s.starts_with("HashSet<") => Ok("Array"),
        s if s.starts_with("BTreeSet<") => Ok("Array"),

        // Custom types
        "Pointer" => Ok("Pointer"),
        "Timestamp" => Ok("Timestamp"),

        // Fallback
        _ => Err(MontycatClientError::ClientUnsupportedFieldType(field_type.to_owned())),
    }
}

/// Processes a bulk of JSON-serializable values into a single JSON string and optional schema.
/// 
/// # Arguments
/// - `values: Vec<T>` : A vector of values to be processed.
///
/// # Returns
/// - `Result<(String, Option<String>), MontycatClientError>` : A result containing the processed JSON string and an optional schema, or an error if processing fails.
///
pub(crate) async fn process_bulk_values<T>(values: Vec<T>) -> Result<(String, Option<String>), MontycatClientError>
where T: Serialize + RuntimeSchema + Send + 'static,
{
    let res: (String, Option<String>) = tokio::task::spawn_blocking(move || {

        let serialized_and_schemas: Result<Vec<(String, Option<String>)>, MontycatClientError> =
            values
                .into_par_iter()
                .map(|val| {
                    let serialized = process_value(val)?;
                    let type_name_retrieved: &str = type_name::<T>();
                    let schema = is_custom_type(type_name_retrieved).map(|s| s.to_string());
                    Ok((serialized, schema))
                })
                .collect();

        let serialized_and_schemas = serialized_and_schemas?;

        let serialized_values: Vec<String> = serialized_and_schemas.iter().map(|(s, _)| s.clone()).collect();
        let schemas: HashSet<String> = serialized_and_schemas.iter().filter_map(|(_, s)| s.clone()).collect();

        let schema = match schemas.len() {
            0 => None,
            1 => Some(schemas.into_iter().next().unwrap()),
            _ => return Err(MontycatClientError::ClientMultipleSchemasFound),
        };

        let value_to_send: String = process_json_value(&serialized_values)?;

        Ok((value_to_send, schema))

    }).await.map_err(|e| MontycatClientError::ClientAsyncRuntimeError(e.to_string()))??;

    Ok(res)

}

