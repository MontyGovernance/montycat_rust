use crate::errors::MontycatClientError;
use core::fmt;
use serde::{Deserialize, Serialize};
use simd_json;

/// Represents a response from the Montycat server.
///
/// # Fields
/// - `status: bool` : Indicates if the request was successful.
/// - `payload: T` : The payload of the response, generic over type T.
/// - `error: Option<String>` : An optional error message if the request failed.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MontycatResponse<T = serde_json::Value> {
    pub status: bool,
    #[serde(default)]
    pub payload: T,
    pub error: Option<String>,
}

/// Represents a streaming response from the Montycat server.
///
/// # Fields
/// - `message: Option<String>` : An optional message from the server.
/// - `status: bool` : Indicates if the request was successful.
/// - `payload: T` : The payload of the response, generic over type T.
/// - `error: Option<String>` : An optional error message if the request failed.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MontycatStreamResponse<T = serde_json::Value> {
    pub message: Option<String>,
    pub status: bool,
    #[serde(default)]
    pub payload: T,
    pub error: Option<String>,
}

impl<T> MontycatResponse<T>
where
    for<'de> T: Deserialize<'de> + Clone + 'static + fmt::Debug,
{
    /// Parses the response bytes into a MontycatResponse<T>.
    ///
    /// This function handles nested JSON strings by recursively parsing them.
    /// If the payload contains JSON strings, they will be parsed into their respective structures.
    ///
    /// # Errors
    ///
    /// - Returns `MontycatClientError::ClientValueParsingError` if parsing fails at any step.
    ///
    /// # Example
    ///
    /// ```rust
    /// let response_bytes: Result<Option<Vec<u8>>, MontycatClientError> = ...;
    /// let parsed_response: MontycatResponse<Option<MyStruct>> = MontycatResponse::parse_response(response_bytes);
    /// ```
    ///
    pub fn parse_response(
        bytes: Result<Option<Vec<u8>>, MontycatClientError>,
    ) -> Result<Self, MontycatClientError> {
        let mut bytes_unwrapped: Vec<u8> = bytes?.ok_or_else(|| {
            MontycatClientError::ClientValueParsingError("No data received".into())
        })?;
        let slice: &mut [u8] = bytes_unwrapped.as_mut_slice();

        let mut response: MontycatResponse<simd_json::OwnedValue> = simd_json::from_slice(slice)
            .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

        fn recursively_parse_json(v: simd_json::OwnedValue) -> simd_json::OwnedValue {
            match v {
                simd_json::OwnedValue::String(s) => {
                    if (s.starts_with('{') && s.ends_with('}'))
                        || (s.starts_with('[') && s.ends_with(']'))
                    {
                        let mut bytes = s.as_bytes().to_vec();
                        if let Ok(inner) =
                            simd_json::from_slice::<simd_json::OwnedValue>(bytes.as_mut_slice())
                        {
                            return recursively_parse_json(inner);
                        }
                    }

                    simd_json::OwnedValue::String(s)
                }

                simd_json::OwnedValue::Array(boxed_vec) => {
                    let vec = *boxed_vec;
                    let new_vec = vec
                        .into_iter()
                        .map(recursively_parse_json)
                        .collect::<Vec<_>>();
                    simd_json::OwnedValue::Array(Box::new(new_vec))
                }

                simd_json::OwnedValue::Object(boxed_map) => {
                    let map = *boxed_map;
                    let new_map = map
                        .into_iter()
                        .map(|(k, v)| (k, recursively_parse_json(v)))
                        .collect::<_>();
                    simd_json::OwnedValue::Object(Box::new(new_map))
                }

                other => other,
            }
        }

        let normalized_payload: simd_json::OwnedValue =
            recursively_parse_json(response.payload.clone());

        let s = simd_json::to_string(&normalized_payload)
            .map_err(|e| MontycatClientError::ClientValueParsingError(format!("{}", e)))?;

        let payload: T = serde_json::from_str(&s)
            .map_err(|e| MontycatClientError::ClientValueParsingError(format!("{}", e)))?;

        Ok(MontycatResponse {
            status: response.status,
            payload,
            error: response.error.take(),
        })
    }
}

impl<T> MontycatStreamResponse<T>
where
    for<'de> T: Deserialize<'de> + Clone + 'static + fmt::Debug,
{
    /// Parses the response bytes into a MontycatStreamResponse<T>.
    ///
    /// This function handles nested JSON strings by recursively parsing them.
    /// If the payload contains JSON strings, they will be parsed into their respective structures.
    ///
    /// # Errors
    ///
    /// If the response cannot be parsed, an error will be returned.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// let response_bytes: &Vec<u8> = ...;
    /// let parsed_response: MontycatStreamResponse<Option<MyStruct>> = MontycatStreamResponse::parse_response(response_bytes);
    /// ```
    ///
    pub fn parse_response(bytes: &Vec<u8>) -> Result<Self, MontycatClientError> {
        let mut bytes_unwrapped: Vec<u8> = bytes.clone();

        let mut response: MontycatStreamResponse<simd_json::OwnedValue> =
            simd_json::from_slice(bytes_unwrapped.as_mut_slice())
                .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

        fn recursively_parse_json(v: simd_json::OwnedValue) -> simd_json::OwnedValue {
            match v {
                simd_json::OwnedValue::String(s) => {
                    if (s.starts_with('{') && s.ends_with('}'))
                        || (s.starts_with('[') && s.ends_with(']'))
                    {
                        let mut bytes = s.as_bytes().to_vec();
                        if let Ok(inner) =
                            simd_json::from_slice::<simd_json::OwnedValue>(bytes.as_mut_slice())
                        {
                            return recursively_parse_json(inner);
                        }
                    }

                    simd_json::OwnedValue::String(s)
                }

                simd_json::OwnedValue::Array(boxed_vec) => {
                    let vec = *boxed_vec;
                    let new_vec = vec
                        .into_iter()
                        .map(recursively_parse_json)
                        .collect::<Vec<_>>();
                    simd_json::OwnedValue::Array(Box::new(new_vec))
                }

                simd_json::OwnedValue::Object(boxed_map) => {
                    let map = *boxed_map;
                    let new_map = map
                        .into_iter()
                        .map(|(k, v)| (k, recursively_parse_json(v)))
                        .collect::<_>();
                    simd_json::OwnedValue::Object(Box::new(new_map))
                }

                other => other,
            }
        }

        let normalized_payload: simd_json::OwnedValue =
            recursively_parse_json(response.payload.clone());

        let s = simd_json::to_string(&normalized_payload)
            .map_err(|e| MontycatClientError::ClientValueParsingError(format!("{}", e)))?;

        let payload: T = serde_json::from_str(&s)
            .map_err(|e| MontycatClientError::ClientValueParsingError(format!("{}", e)))?;

        Ok(MontycatStreamResponse {
            status: response.status,
            message: response.message.take(),
            payload,
            error: response.error.take(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        id: u32,
        name: String,
    }

    // ===== MontycatResponse Tests =====

    #[test]
    fn test_montycat_response_parse_simple_success() {
        let json_str = r#"{"status":true,"payload":"test_value","error":null}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<String> = MontycatResponse::parse_response(bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.payload, "test_value");
        assert_eq!(response.error, None);
    }

    #[test]
    fn test_montycat_response_parse_with_error() {
        let json_str = r#"{"status":false,"payload":null,"error":"Something went wrong"}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<Option<String>> =
            MontycatResponse::parse_response(bytes).unwrap();

        assert!(!response.status);
        assert_eq!(response.payload, None);
        assert_eq!(response.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_montycat_response_parse_struct() {
        let json_str = r#"{"status":true,"payload":{"id":1,"name":"test"},"error":null}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<TestStruct> =
            MontycatResponse::parse_response(bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.payload.id, 1);
        assert_eq!(response.payload.name, "test");
    }

    #[test]
    fn test_montycat_response_parse_nested_json_string() {
        let json_str =
            r#"{"status":true,"payload":"{\"id\":42,\"name\":\"nested\"}","error":null}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<TestStruct> =
            MontycatResponse::parse_response(bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.payload.id, 42);
        assert_eq!(response.payload.name, "nested");
    }

    #[test]
    fn test_montycat_response_parse_array() {
        let json_str = r#"{"status":true,"payload":[{"id":1,"name":"first"},{"id":2,"name":"second"}],"error":null}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<Vec<TestStruct>> =
            MontycatResponse::parse_response(bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.payload.len(), 2);
        assert_eq!(response.payload[0].id, 1);
        assert_eq!(response.payload[1].name, "second");
    }

    #[test]
    fn test_montycat_response_parse_option_some() {
        let json_str = r#"{"status":true,"payload":{"id":99,"name":"optional"},"error":null}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<Option<TestStruct>> =
            MontycatResponse::parse_response(bytes).unwrap();

        assert!(response.status);
        assert!(response.payload.is_some());
        assert_eq!(response.payload.unwrap().id, 99);
    }

    #[test]
    fn test_montycat_response_parse_option_none() {
        let json_str = r#"{"status":true,"payload":null,"error":null}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<Option<TestStruct>> =
            MontycatResponse::parse_response(bytes).unwrap();

        assert!(response.status);
        assert!(response.payload.is_none());
    }

    #[test]
    fn test_montycat_response_parse_error_no_data() {
        let bytes: Result<Option<Vec<u8>>, MontycatClientError> = Ok(None);

        let result: Result<MontycatResponse<String>, MontycatClientError> =
            MontycatResponse::parse_response(bytes);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.message().contains("No data received"));
        }
    }

    #[test]
    fn test_montycat_response_parse_error_invalid_json() {
        let invalid_json = b"not valid json";
        let bytes = Ok(Some(invalid_json.to_vec()));

        let result: Result<MontycatResponse<String>, MontycatClientError> =
            MontycatResponse::parse_response(bytes);

        assert!(result.is_err());
    }

    #[test]
    fn test_montycat_response_parse_error_propagation() {
        let bytes: Result<Option<Vec<u8>>, MontycatClientError> = Err(
            MontycatClientError::ClientEngineError("Connection failed".to_string()),
        );

        let result: Result<MontycatResponse<String>, MontycatClientError> =
            MontycatResponse::parse_response(bytes);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.message(), "Connection failed");
        }
    }

    // ===== MontycatStreamResponse Tests =====

    #[test]
    fn test_montycat_stream_response_parse_simple() {
        let json_str =
            r#"{"message":"Processing","status":true,"payload":"stream_data","error":null}"#;
        let bytes = json_str.as_bytes().to_vec();

        let response: MontycatStreamResponse<String> =
            MontycatStreamResponse::parse_response(&bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.message, Some("Processing".to_string()));
        assert_eq!(response.payload, "stream_data");
        assert_eq!(response.error, None);
    }

    #[test]
    fn test_montycat_stream_response_parse_with_error() {
        let json_str = r#"{"message":null,"status":false,"payload":null,"error":"Stream error"}"#;
        let bytes = json_str.as_bytes().to_vec();

        let response: MontycatStreamResponse<Option<String>> =
            MontycatStreamResponse::parse_response(&bytes).unwrap();

        assert!(!response.status);
        assert_eq!(response.message, None);
        assert_eq!(response.error, Some("Stream error".to_string()));
    }

    #[test]
    fn test_montycat_stream_response_parse_struct() {
        let json_str = r#"{"message":"Data ready","status":true,"payload":{"id":123,"name":"streamed"},"error":null}"#;
        let bytes = json_str.as_bytes().to_vec();

        let response: MontycatStreamResponse<TestStruct> =
            MontycatStreamResponse::parse_response(&bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.message, Some("Data ready".to_string()));
        assert_eq!(response.payload.id, 123);
        assert_eq!(response.payload.name, "streamed");
    }

    #[test]
    fn test_montycat_stream_response_parse_nested_json() {
        let json_str = r#"{"message":"Nested data","status":true,"payload":"{\"id\":77,\"name\":\"nested_stream\"}","error":null}"#;
        let bytes = json_str.as_bytes().to_vec();

        let response: MontycatStreamResponse<TestStruct> =
            MontycatStreamResponse::parse_response(&bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.payload.id, 77);
        assert_eq!(response.payload.name, "nested_stream");
    }

    #[test]
    fn test_montycat_stream_response_parse_invalid_json() {
        let invalid_json = b"not valid json".to_vec();

        let result: Result<MontycatStreamResponse<String>, MontycatClientError> =
            MontycatStreamResponse::parse_response(&invalid_json);

        assert!(result.is_err());
    }

    #[test]
    fn test_montycat_stream_response_no_message() {
        let json_str = r#"{"status":true,"payload":"data","error":null}"#;
        let bytes = json_str.as_bytes().to_vec();

        let response: MontycatStreamResponse<String> =
            MontycatStreamResponse::parse_response(&bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.message, None);
        assert_eq!(response.payload, "data");
    }

    #[test]
    fn test_recursive_json_parsing_deeply_nested() {
        let json_str = r#"{"status":true,"payload":"[{\"id\":1,\"name\":\"item1\"},{\"id\":2,\"name\":\"item2\"}]","error":null}"#;
        let bytes = Ok(Some(json_str.as_bytes().to_vec()));

        let response: MontycatResponse<Vec<TestStruct>> =
            MontycatResponse::parse_response(bytes).unwrap();

        assert!(response.status);
        assert_eq!(response.payload.len(), 2);
        assert_eq!(response.payload[0].id, 1);
        assert_eq!(response.payload[1].name, "item2");
    }
}
