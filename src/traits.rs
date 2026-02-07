use std::collections::HashMap;

/// Trait for runtime schema operations.
///
pub trait RuntimeSchema {
    fn pointer_and_timestamp_fields(&self) -> Vec<(&'static str, &'static str)>;
    fn field_names_and_types(&self) -> Vec<(&'static str, &'static str)>;
    fn schema_params() -> (HashMap<&'static str, &'static str>, &'static str);
}
