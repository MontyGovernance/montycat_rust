use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

/// Represents a limit with start and stop values.
///
/// # Fields
/// - `start: usize` : The starting index of the limit.
/// - `stop: usize` : The stopping index of the limit.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Limit {
    pub start: usize,
    pub stop: usize,
}

impl Limit {
    /// Creates a default limit with start and stop set to 0.
    ///
    /// # Returns
    /// - `Self` : A new instance of `Limit` with default values.
    ///
    pub(crate) fn default_limit() -> Self {
        Self { start: 0, stop: 0 }
    }

    /// Converts the limit into a HashMap representation.
    ///
    /// # Returns
    /// - `HashMap<String, usize>` : A HashMap with "start" and "stop" keys.
    ///
    pub(crate) fn to_map(&self) -> HashMap<String, usize> {
        let mut map: HashMap<String, usize> = HashMap::new();
        map.insert("start".to_string(), self.start);
        map.insert("stop".to_string(), self.stop);
        map
    }

    /// Creates a new limit with specified start and stop values.
    ///
    /// # Arguments
    /// - `start: usize` : The starting index of the limit.
    /// - `stop: usize` : The stopping index of the limit.
    ///
    /// # Returns
    /// - `Self` : A new instance of `Limit` with the specified values.
    ///
    pub fn new(start: usize, stop: usize) -> Self {
        Self { start, stop }
    }
}

/// Represents a pointer with keyspace and key.
///
/// # Fields
/// - `keyspace: String` : The keyspace of the pointer.
/// - `key: String` : The key of the pointer.
///
/// # Examples
/// ```rust, ignore
/// let pointer = Pointer::new("my_keyspace", "my_key");
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Pointer {
    pub keyspace: String,
    pub key: String,
}

impl Pointer {
    /// Creates a new pointer with specified keyspace and key values.
    ///
    /// # Arguments
    /// - `keyspace: &str` : The keyspace of the pointer.
    /// - `key: &str` : The key of the pointer.
    ///
    /// # Returns
    /// - `Self` : A new instance of `Pointer` with the specified values.
    ///
    pub fn new(keyspace: &str, key: &str) -> Self {
        Self {
            keyspace: keyspace.to_owned(),
            key: key.to_owned(),
        }
    }

    /// Sets the pointer values and returns them as a tuple.
    ///
    /// # Arguments
    /// - `keyspace: &str` : The keyspace of the pointer.
    /// - `key: &str` : The key of the pointer.
    ///
    /// # Returns
    /// - `(String, String)` : A tuple containing the keyspace and key.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let (keyspace, key) = Pointer::set_pointer("my_keyspace", "my_key");
    /// ```
    ///
    /// # Notes
    /// Method to be used when only the tuple representation is needed such as in update operations, lookups, etc.
    ///
    pub fn using(keyspace: &str, key: &str) -> (String, String) {
        (keyspace.to_owned(), key.to_owned())
    }
}

/// Represents a timestamp with an optional timestamp string.
///
/// # Fields
/// - `timestamp: Option<String>` : The timestamp string.
///
/// # Examples
/// ```rust, ignore
/// let ts = Timestamp::new("2024-01-01T00:00:00Z");
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Timestamp {
    timestamp: Option<String>,
}

impl Timestamp {
    /// Creates a new timestamp with the specified timestamp string.
    ///
    /// # Arguments
    /// - `timestamp: &str` : The timestamp string.
    ///
    /// # Returns
    /// - `Self` : A new instance of `Timestamp` with the specified value.
    ///
    pub fn new(timestamp: &str) -> Self {
        Self {
            timestamp: Some(timestamp.to_owned()),
        }
    }

    /// Sets the timestamp value and returns it as a string.
    ///
    /// # Arguments
    /// - `timestamp: &str` : The timestamp string.
    ///
    /// # Returns
    /// - `String` : The timestamp string.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let ts_str = Timestamp::using("2024-01-01T00:00:00Z");
    /// ```
    ///
    /// # Notes
    /// Method to be used when only the string representation is needed such as in update operations.
    ///
    pub fn using(timestamp: &str) -> String {
        timestamp.to_owned()
    }

    /// Creates a HashMap representing an "after" timestamp criteria.
    ///
    /// # Arguments
    /// - `after: &str` : The "after" timestamp string.
    ///
    /// # Returns
    /// - `HashMap<String, String>` : A HashMap with the "after" key and its corresponding value.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let after_map = Timestamp::after("2024-01-01T00:00:00Z");
    /// ```
    ///
    /// # Notes
    /// Method to be used when only the HashMap representation is needed such as in lookups.
    ///
    pub fn after(after: &str) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::with_capacity(1);
        map.insert("after".to_string(), after.to_owned());
        map
    }

    /// Creates a HashMap representing a "before" timestamp criteria.
    ///
    /// # Arguments
    /// - `before: &str` : The "before" timestamp string.
    ///
    /// # Returns
    /// - `HashMap<String, String>` : A HashMap with the "before" key and its corresponding value.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let before_map = Timestamp::before("2024-01-01T00:00:00Z");
    /// ```
    /// # Notes
    /// Method to be used when only the HashMap representation is needed such as in lookups.
    ///
    pub fn before(before: &str) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::with_capacity(1);
        map.insert("before".to_string(), before.to_owned());
        map
    }

    /// Creates a HashMap representing a range of timestamps.
    ///
    /// # Arguments
    /// - `start: &str` : The starting timestamp string.
    /// - `stop: &str` : The stopping timestamp string.
    ///
    /// # Returns
    /// - `HashMap<String, Vec<String>>` : A HashMap with the "range_timestamp" key and its corresponding start and stop values.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let range_map = Timestamp::range("2024-01-01T00:00:00Z", "2024-12-31T23:59:59Z");
    /// ```
    ///
    /// # Notes
    /// Method to be used when only the HashMap representation is needed such as in lookups.
    ///
    pub fn range(start: &str, stop: &str) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::with_capacity(1);
        map.insert(
            "range_timestamp".to_string(),
            vec![start.to_owned(), stop.to_owned()],
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Limit Tests =====

    #[test]
    fn test_limit_new() {
        let limit = Limit::new(10, 20);
        assert_eq!(limit.start, 10);
        assert_eq!(limit.stop, 20);
    }

    #[test]
    fn test_limit_default_limit() {
        let limit = Limit::default_limit();
        assert_eq!(limit.start, 0);
        assert_eq!(limit.stop, 0);
    }

    #[test]
    fn test_limit_to_map() {
        let limit = Limit::new(5, 15);
        let map = limit.to_map();
        assert_eq!(map.get("start"), Some(&5));
        assert_eq!(map.get("stop"), Some(&15));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_limit_serialization() {
        let limit = Limit::new(1, 100);
        let serialized = serde_json::to_string(&limit).unwrap();
        let deserialized: Limit = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.start, 1);
        assert_eq!(deserialized.stop, 100);
    }

    #[test]
    fn test_limit_equality() {
        let limit1 = Limit::new(10, 20);
        let limit2 = Limit::new(10, 20);
        let limit3 = Limit::new(10, 21);
        assert_eq!(limit1, limit2);
        assert_ne!(limit1, limit3);
    }

    #[test]
    fn test_limit_default_trait() {
        let limit: Limit = Default::default();
        assert_eq!(limit.start, 0);
        assert_eq!(limit.stop, 0);
    }

    // ===== Pointer Tests =====

    #[test]
    fn test_pointer_new() {
        let pointer = Pointer::new("users", "user123");
        assert_eq!(pointer.keyspace, "users");
        assert_eq!(pointer.key, "user123");
    }

    #[test]
    fn test_pointer_using() {
        let (keyspace, key) = Pointer::using("products", "prod456");
        assert_eq!(keyspace, "products");
        assert_eq!(key, "prod456");
    }

    #[test]
    fn test_pointer_serialization() {
        let pointer = Pointer::new("orders", "order789");
        let serialized = serde_json::to_string(&pointer).unwrap();
        let deserialized: Pointer = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.keyspace, "orders");
        assert_eq!(deserialized.key, "order789");
    }

    #[test]
    fn test_pointer_equality() {
        let pointer1 = Pointer::new("test", "key1");
        let pointer2 = Pointer::new("test", "key1");
        let pointer3 = Pointer::new("test", "key2");
        assert_eq!(pointer1, pointer2);
        assert_ne!(pointer1, pointer3);
    }

    #[test]
    fn test_pointer_default() {
        let pointer: Pointer = Default::default();
        assert_eq!(pointer.keyspace, "");
        assert_eq!(pointer.key, "");
    }

    #[test]
    fn test_pointer_with_empty_strings() {
        let pointer = Pointer::new("", "");
        assert_eq!(pointer.keyspace, "");
        assert_eq!(pointer.key, "");
    }

    #[test]
    fn test_pointer_with_special_characters() {
        let pointer = Pointer::new("my-keyspace_123", "key:with:colons");
        assert_eq!(pointer.keyspace, "my-keyspace_123");
        assert_eq!(pointer.key, "key:with:colons");
    }

    // ===== Timestamp Tests =====

    #[test]
    fn test_timestamp_new() {
        let ts = Timestamp::new("2024-01-01T00:00:00Z");
        assert_eq!(ts.timestamp, Some("2024-01-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_timestamp_using() {
        let ts_str = Timestamp::using("2024-12-31T23:59:59Z");
        assert_eq!(ts_str, "2024-12-31T23:59:59Z");
    }

    #[test]
    fn test_timestamp_after() {
        let map = Timestamp::after("2024-06-01T00:00:00Z");
        assert_eq!(map.get("after"), Some(&"2024-06-01T00:00:00Z".to_string()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_timestamp_before() {
        let map = Timestamp::before("2024-06-30T23:59:59Z");
        assert_eq!(map.get("before"), Some(&"2024-06-30T23:59:59Z".to_string()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_timestamp_range() {
        let map = Timestamp::range("2024-01-01T00:00:00Z", "2024-12-31T23:59:59Z");
        let range = map.get("range_timestamp").unwrap();
        assert_eq!(range.len(), 2);
        assert_eq!(range[0], "2024-01-01T00:00:00Z");
        assert_eq!(range[1], "2024-12-31T23:59:59Z");
    }

    #[test]
    fn test_timestamp_serialization() {
        let ts = Timestamp::new("2024-03-15T12:30:00Z");
        let serialized = serde_json::to_string(&ts).unwrap();
        let deserialized: Timestamp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.timestamp,
            Some("2024-03-15T12:30:00Z".to_string())
        );
    }

    #[test]
    fn test_timestamp_equality() {
        let ts1 = Timestamp::new("2024-01-01T00:00:00Z");
        let ts2 = Timestamp::new("2024-01-01T00:00:00Z");
        let ts3 = Timestamp::new("2024-01-02T00:00:00Z");
        assert_eq!(ts1, ts2);
        assert_ne!(ts1, ts3);
    }

    #[test]
    fn test_timestamp_default() {
        let ts: Timestamp = Default::default();
        assert_eq!(ts.timestamp, None);
    }

    #[test]
    fn test_timestamp_with_different_formats() {
        let ts1 = Timestamp::new("2024-01-01");
        let ts2 = Timestamp::new("1704067200");
        let ts3 = Timestamp::new("2024-01-01T00:00:00.000Z");

        assert_eq!(ts1.timestamp, Some("2024-01-01".to_string()));
        assert_eq!(ts2.timestamp, Some("1704067200".to_string()));
        assert_eq!(ts3.timestamp, Some("2024-01-01T00:00:00.000Z".to_string()));
    }
}
