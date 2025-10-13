use std::{collections::HashMap, hash::Hash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Limit {
    pub start: usize,
    pub stop: usize,
}

impl Limit {
    pub fn default_limit() -> Self {
        Self { start: 0, stop: 0 }
    }

    pub fn to_map(&self) -> HashMap<String, usize> {
        let mut map: HashMap<String,  usize> = HashMap::new();
        map.insert("start".to_string(), self.start);
        map.insert("stop".to_string(), self.stop);
        map
    }

    pub fn new(start: usize, stop: usize) -> Self {
        Self { start, stop }
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Pointer {
    pub keyspace: String,
    pub key: String,
}

impl Pointer {

    pub fn new(keyspace: &str, key: &str) -> Self {
        Self {
            keyspace: keyspace.to_owned(),
            key: key.to_owned(),
        }
    }

    pub fn set_pointer(keyspace: &str, key: &str) -> (String, String) {
        (keyspace.to_owned(), key.to_owned())
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Timestamp {
    pub timestamp: Option<String>,
}

impl Timestamp {

    pub fn new(timestamp: &str) -> Self {
        Self {
            timestamp: Some(timestamp.to_owned()),
        }
    }

    pub fn set_timestamp(timestamp: &str) -> String {
        timestamp.to_owned()
    }

    pub fn after(after: &str) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::with_capacity(1);
        map.insert("after".to_string(), after.to_owned());
        map
    }

    pub fn before(before: &str) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::with_capacity(1);
        map.insert("before".to_string(), before.to_owned());
        map
    }

    pub fn range(start: &str, stop: &str) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::with_capacity(1);
        map.insert("range_timestamp".to_string(), vec![start.to_owned(), stop.to_owned()]);
        map
    }

}
