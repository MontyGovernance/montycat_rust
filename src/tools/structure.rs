use std::{collections::HashMap, hash::Hash};
use serde::{Deserialize, Serialize};

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
        let mut map: HashMap<String,  usize> = HashMap::new();
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
/// ```rust
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
    /// ```rust
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
/// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
    /// let range_map = Timestamp::range("2024-01-01T00:00:00Z", "2024-12-31T23:59:59Z");
    /// ```
    ///
    /// # Notes
    /// Method to be used when only the HashMap representation is needed such as in lookups.
    ///
    pub fn range(start: &str, stop: &str) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::with_capacity(1);
        map.insert("range_timestamp".to_string(), vec![start.to_owned(), stop.to_owned()]);
        map
    }

}
