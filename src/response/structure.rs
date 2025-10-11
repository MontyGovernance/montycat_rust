// use core::fmt;

// use serde::{Deserialize, Serialize};
// use crate::errors::MontycatClientError;
// use simd_json;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct MontycatResponse<T = serde_json::Value> {
//     pub status: bool,
//     #[serde(default)]
//     pub payload: T,
//     pub error: Option<String>,
// }

// impl<T> MontycatResponse<T>
// where
//     for<'de> T: Deserialize<'de> + Clone + 'static + fmt::Debug,
// {
//     /// Generic implementation to parse MontycatResponse with special handling for payloads that are strings
//     /// and may contain JSON or primitive types
//     ///
//     /// # Examples
//     ///
//     /// ```rust,no_run
//     /// let response: Result<Option<Vec<u8>>, MontycatClientError> = ...; // Assume this is obtained from some operation
//     ///
//     /// let parsed_response = MontycatResponse::<YourType>::parse_response(response);
//     /// //where YourType is the expected type of the payload
//     /// // or alternatively use MontycatResponse::<String> or MontycatResponse::<serde_json::Value>
//     /// let parsed_response = MontycatResponse::<serde_json::Value>::parse_response(response);
//     /// ```
//     ///
//     /// # Errors
//     ///
//     /// Returns MontycatClientError if parsing fails or no data is received
//     ///
//     pub fn parse_response(
//         bytes: Result<Option<Vec<u8>>, MontycatClientError>,
//     ) -> Result<Self, MontycatClientError> {
//         let mut bytes_unwrapped: Vec<u8> = bytes?
//             .ok_or_else(|| MontycatClientError::ValueParsingError("No data received".into()))?;
//         let slice: &mut [u8] = bytes_unwrapped.as_mut_slice();

//         let mut response: MontycatResponse<simd_json::OwnedValue> =
//             simd_json::from_slice(slice).map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?;

//         let payload: T = match &mut response.payload {
//             simd_json::OwnedValue::String(s) => {

//                 let mut bytes = s.as_bytes().to_vec();
//                 let slice = bytes.as_mut_slice();

//                 match simd_json::from_slice::<T>(slice) {
//                     Ok(parsed) => { parsed },
//                     Err(_) => {
//                         match serde_json::from_str::<T>(s) {
//                             Ok(parsed) => parsed,
//                             Err(e) => {

//                                 if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Option<String>>() {

//                                    return Ok(MontycatResponse {
//                                        status: response.status,
//                                        payload: T::clone(unsafe { &*(&Some(s.to_string()) as *const Option<String> as *const T) }),
//                                        error: response.error.take(),
//                                    });

//                                 }

//                                 return Err(MontycatClientError::ValueParsingError(format!("Failed to parse payload {} as target type: {}", s, e)));
//                             }
//                         }
//                     }
//                 }
//             }

//             _ => {
//                 let s = simd_json::to_string(&response.payload)
//                     .map_err(|e| MontycatClientError::ValueParsingError(format!("Failed to stringify payload: {}", e)))?;
//                 serde_json::from_str::<T>(&mut s.as_str())
//                     .map_err(|e| MontycatClientError::ValueParsingError(format!("Failed to parse non-string payload: {}", e)))?
//             }

//         };

//         Ok(MontycatResponse {
//             status: response.status,
//             payload,
//             error: response.error.take(),
//         })
//     }

// }

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

impl<T> MontycatResponse<T>
where
    for<'de> T: Deserialize<'de> + Clone + 'static + fmt::Debug,
{
    pub fn parse_response(
        bytes: Result<Option<Vec<u8>>, MontycatClientError>,
    ) -> Result<Self, MontycatClientError> {
        let mut bytes_unwrapped: Vec<u8> = bytes?
            .ok_or_else(|| MontycatClientError::ValueParsingError("No data received".into()))?;
        let slice: &mut [u8] = bytes_unwrapped.as_mut_slice();

        let mut response: MontycatResponse<simd_json::OwnedValue> =
            simd_json::from_slice(slice)
                .map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?;

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
            .map_err(|e| MontycatClientError::ValueParsingError(format!("{}", e)))?;

        let payload: T = serde_json::from_str(&s)
            .map_err(|e| MontycatClientError::ValueParsingError(format!("{}", e)))?;

        Ok(MontycatResponse {
            status: response.status,
            payload,
            error: response.error.take(),
        })

    }

}