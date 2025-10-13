use core::fmt;
use serde::{Deserialize, Serialize};
use crate::errors::MontycatClientError;
use simd_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MontycatResponse<T = serde_json::Value> {
    pub status: bool,
    #[serde(default)]
    pub payload: T,
    pub error: Option<String>,
}

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
        let mut bytes_unwrapped: Vec<u8> = bytes?
            .ok_or_else(|| MontycatClientError::ClientValueParsingError("No data received".into()))?;
        let slice: &mut [u8] = bytes_unwrapped.as_mut_slice();

        let mut response: MontycatResponse<simd_json::OwnedValue> =
            simd_json::from_slice(slice)
                .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

        fn recursively_parse_json(v: simd_json::OwnedValue) -> simd_json::OwnedValue {
            match v {
                simd_json::OwnedValue::String(s) => {
                    if (s.starts_with('{') && s.ends_with('}'))
                        || (s.starts_with('[') && s.ends_with(']')) {
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

        let normalized_payload: simd_json::OwnedValue = recursively_parse_json(response.payload.clone());

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
    pub fn parse_response(
        bytes: &Vec<u8>,
    ) -> Result<Self, MontycatClientError> {
        let mut bytes_unwrapped: Vec<u8> = bytes.clone();

        let mut response: MontycatStreamResponse<simd_json::OwnedValue> =
            simd_json::from_slice(bytes_unwrapped.as_mut_slice())
                .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

        fn recursively_parse_json(v: simd_json::OwnedValue) -> simd_json::OwnedValue {
            match v {
                simd_json::OwnedValue::String(s) => {
                    if (s.starts_with('{') && s.ends_with('}'))
                        || (s.starts_with('[') && s.ends_with(']')) {
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

        let normalized_payload: simd_json::OwnedValue = recursively_parse_json(response.payload.clone());

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