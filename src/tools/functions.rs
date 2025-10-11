use crate::errors::MontycatClientError;
use crate::traits::RuntimeSchema;
use serde::Serialize;
use serde_json::{Value, Map};


pub fn process_json_value<T>(value: &T) -> Result<String, MontycatClientError>
where T: Serialize,
{
    let value_to_send: String = simd_json::to_string(value)
        .map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?;

    Ok(value_to_send)
}

pub fn process_value<T>(value: T) -> Result<String, MontycatClientError>
where
    T: Serialize + RuntimeSchema,
{
    let pointer_and_timestamp_fields: Vec<(&'static str, &'static str)> = value.pointer_and_timestamp_fields();
    let mut val_as_map: Map<String, Value> = Map::new();

    if !pointer_and_timestamp_fields.is_empty() {

        let mut pointers: Map<String, Value> = Map::new();
        let mut timestamps: Map<String, Value> = Map::new();

        let val_as_serde = serde_json::to_value(&value)
            .map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?;

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
                            return Err(MontycatClientError::NoValidInputProvided);
                        }
                    }

                    removal.push(field_name);
                } else if field_type == "Timestamp" {

                    let timestamp_value: &Value = field_value.get("timestamp").ok_or(MontycatClientError::NoValidInputProvided)?;

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
                .map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?
        } else {
            simd_json::to_string(&val_as_map)
                .map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?
        }
    };

    Ok(value_to_send)
}

pub fn define_type(field_type: &str) -> Result<&'static str, MontycatClientError> {
    match  field_type.replace(' ', "").as_str() {
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

        // Option / Result
        s if s.starts_with("Option<") => Ok("Option"),
        s if s.starts_with("Result<") => Ok("Result"),

        // Custom types
        "Pointer" => Ok("Pointer"),
        "Timestamp" => Ok("Timestamp"),

        // Fallback
        _ => Err(MontycatClientError::UnsupportedFieldType(field_type.to_owned())),
    }
}