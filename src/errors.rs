use serde::{Deserialize, Serialize};

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