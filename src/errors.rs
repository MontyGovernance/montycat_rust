use serde::{Deserialize, Serialize};

/// Represents various client-side errors that can occur in the Montycat Rust client.
///
/// # Variants
/// - `ClientEngineError(String)` : Represents errors related to the client engine.
/// - `ClientValueParsingError(String)` : Represents errors that occur during value parsing.
/// - `ClientGenericError(String)` : Represents generic client errors.
/// - `ClientSelectedBothKeyAndCustomKey` : Error when both key and custom key
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
            MontycatClientError::ClientSelectedBothKeyAndCustomKey => {
                "You selected both key and custom key. Choose one".to_owned()
            }
            MontycatClientError::ClientNoValidInputProvided => "No valid input provided".to_owned(),
            MontycatClientError::ClientAsyncRuntimeError(msg) => msg.to_owned(),
            MontycatClientError::ClientMultipleSchemasFound => {
                "Multiple schemas found. Bulk values must have a single schema".to_owned()
            }
            MontycatClientError::ClientUnsupportedFieldType(ty) => {
                format!("Unsupported field type: {}", ty)
            }
        }
    }
}

impl std::fmt::Display for MontycatClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

/// Lets this error compose with `?`, `Box<dyn Error>`, `anyhow`, and
/// `thiserror`'s `#[from]`.
///
/// Without it a caller could not write
/// `Engine::from_uri(..)?` inside a `Result<_, Box<dyn Error>>` function — the
/// usual shape of an `axum` or `tokio` `main` — and had to reach for
/// `.map_err(|e| e.message())` at every call site.
///
/// No `source()` is reported: the variants carry rendered strings rather than
/// an underlying error value, so there is no cause to chain to.
impl std::error::Error for MontycatClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_engine_error_message() {
        let error = MontycatClientError::ClientEngineError("Connection failed".to_string());
        assert_eq!(error.message(), "Connection failed");
    }

    #[test]
    fn test_client_value_parsing_error_message() {
        let error = MontycatClientError::ClientValueParsingError("Invalid JSON".to_string());
        assert_eq!(error.message(), "Invalid JSON");
    }

    #[test]
    fn test_client_generic_error_message() {
        let error = MontycatClientError::ClientGenericError("Something went wrong".to_string());
        assert_eq!(error.message(), "Something went wrong");
    }

    #[test]
    fn test_client_store_not_set_message() {
        let error = MontycatClientError::ClientStoreNotSet;
        assert_eq!(error.message(), "Store is not set in the engine");
    }

    #[test]
    fn test_client_selected_both_key_and_custom_key_message() {
        let error = MontycatClientError::ClientSelectedBothKeyAndCustomKey;
        assert_eq!(
            error.message(),
            "You selected both key and custom key. Choose one"
        );
    }

    #[test]
    fn test_client_no_valid_input_provided_message() {
        let error = MontycatClientError::ClientNoValidInputProvided;
        assert_eq!(error.message(), "No valid input provided");
    }

    #[test]
    fn test_client_async_runtime_error_message() {
        let error = MontycatClientError::ClientAsyncRuntimeError("Tokio error".to_string());
        assert_eq!(error.message(), "Tokio error");
    }

    #[test]
    fn test_client_multiple_schemas_found_message() {
        let error = MontycatClientError::ClientMultipleSchemasFound;
        assert_eq!(
            error.message(),
            "Multiple schemas found. Bulk values must have a single schema"
        );
    }

    #[test]
    fn test_client_unsupported_field_type_message() {
        let error = MontycatClientError::ClientUnsupportedFieldType("ComplexType".to_string());
        assert_eq!(error.message(), "Unsupported field type: ComplexType");
    }

    #[test]
    fn display_matches_message() {
        let error = MontycatClientError::ClientStoreNotSet;
        assert_eq!(error.to_string(), error.message());
        assert_eq!(format!("{}", error), "Store is not set in the engine");
    }

    #[test]
    fn composes_with_the_question_mark_operator() {
        // The shape an axum/tokio `main` uses. Before `Display` + `Error` this
        // did not compile: `Box<dyn Error>` had no `From<MontycatClientError>`.
        fn fallible() -> Result<(), Box<dyn std::error::Error>> {
            Err(MontycatClientError::ClientNoValidInputProvided)?
        }
        let err = fallible().unwrap_err();
        assert_eq!(err.to_string(), "No valid input provided");
    }

    #[test]
    fn is_usable_as_a_std_error_trait_object() {
        let error: Box<dyn std::error::Error> =
            Box::new(MontycatClientError::ClientEngineError("boom".into()));
        assert_eq!(error.to_string(), "boom");
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_serialization() {
        let error = MontycatClientError::ClientStoreNotSet;
        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("ClientStoreNotSet"));
    }

    #[test]
    fn test_error_deserialization() {
        let json = r#""ClientStoreNotSet""#;
        let error: MontycatClientError = serde_json::from_str(json).unwrap();
        assert_eq!(error.message(), "Store is not set in the engine");
    }

    #[test]
    fn test_error_with_string_serialization() {
        let error = MontycatClientError::ClientEngineError("test error".to_string());
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: MontycatClientError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.message(), "test error");
    }
}
