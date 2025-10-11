use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MontycatClientError {
    EngineError(String),
    ValueParsingError(String),
    GenericError(String),
    SelectedBothKeyAndCustomKey,
    SelectedBothPointersValueAndMetadata,
    StoreNotSet,
    NoValidInputProvided,
    AsyncRuntimeError(String),
    UnsupportedFieldType(String),
}

impl MontycatClientError {
    pub fn message(&self) -> String {
        match self {
            MontycatClientError::EngineError(err) => err.to_owned(),
            MontycatClientError::ValueParsingError(msg) => msg.to_owned(),
            MontycatClientError::GenericError(msg) => msg.to_owned(),
            MontycatClientError::StoreNotSet => "Store is not set in the engine".to_owned(),
            MontycatClientError::SelectedBothKeyAndCustomKey => "You selected both key and custom key. Choose one".to_owned(),
            MontycatClientError::SelectedBothPointersValueAndMetadata => "You selected both pointers value and pointers metadata. Choose one".to_owned(),
            MontycatClientError::NoValidInputProvided => "No valid input provided".to_owned(),
            MontycatClientError::AsyncRuntimeError(msg) => msg.to_owned(),
            MontycatClientError::UnsupportedFieldType(ty) => {
                format!("Unsupported field type: {}", ty)
            },
        }
    }
}