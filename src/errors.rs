use serde::{Deserialize, Serialize};

/// Represents various client-side errors that can occur in the Montycat Rust client.
/// 
/// # Variants
/// - `ClientEngineError(String)` : Represents errors related to the client engine.
/// - `ClientValueParsingError(String)` : Represents errors that occur during value parsing.
/// - `ClientGenericError(String)` : Represents generic client errors.
/// - `ClientSelectedBothKeyAndCustomKey` : Error when both key and custom key
/// - `ClientSelectedBothPointersValueAndMetadata` : Error when both pointers value and metadata are selected.
/// - `ClientStoreNotSet` : Error when the store is not set in the engine
/// - `ClientNoValidInputProvided` : Error when no valid input is provided.
/// - `ClientAsyncRuntimeError(String)` : Represents errors related to the async runtime.
/// - `ClientUnsupportedFieldType(String)` : Error for unsupported field types.
/// - `ClientMultipleSchemasFound` : Error when multiple schemas are found in bulk values.
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MontycatClientError {
    ClientEngineError(String),
    ClientValueParsingError(String),
    ClientGenericError(String),
    ClientSelectedBothKeyAndCustomKey,
    ClientSelectedBothPointersValueAndMetadata,
    ClientStoreNotSet,
    ClientNoValidInputProvided,
    ClientAsyncRuntimeError(String),
    ClientUnsupportedFieldType(String),
    ClientMultipleSchemasFound,
}

impl MontycatClientError {
    /// Retrieves the error message associated with the MontycatClientError.
    /// 
    /// # Returns
    /// - `String` : The error message.
    ///
    pub fn message(&self) -> String {
        match self {
            MontycatClientError::ClientEngineError(err) => err.to_owned(),
            MontycatClientError::ClientValueParsingError(msg) => msg.to_owned(),
            MontycatClientError::ClientGenericError(msg) => msg.to_owned(),
            MontycatClientError::ClientStoreNotSet => "Store is not set in the engine".to_owned(),
            MontycatClientError::ClientSelectedBothKeyAndCustomKey => "You selected both key and custom key. Choose one".to_owned(),
            MontycatClientError::ClientSelectedBothPointersValueAndMetadata => "You selected both pointers value and pointers metadata. Choose one".to_owned(),
            MontycatClientError::ClientNoValidInputProvided => "No valid input provided".to_owned(),
            MontycatClientError::ClientAsyncRuntimeError(msg) => msg.to_owned(),
            MontycatClientError::ClientMultipleSchemasFound => "Multiple schemas found. Bulk values must have a single schema".to_owned(),
            MontycatClientError::ClientUnsupportedFieldType(ty) => {
                format!("Unsupported field type: {}", ty)
            },
        }
    }
}