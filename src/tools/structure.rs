use hashbrown::HashMap;
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Timestamp {
    pub timestamp: Option<String>,
    pub start: Option<String>,
    pub stop: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

impl Timestamp {
    pub fn set_timestamp(timestamp: &str) -> Self {
        Self {
            timestamp: Some(timestamp.to_owned()),
            start: None,
            stop: None,
            before: None,
            after: None,
        }
    }

    pub fn range(start: &str, stop: &str) -> Self {
        Self {
            timestamp: None,
            start: Some(start.to_owned()),
            stop: Some(stop.to_owned()),
            before: None,
            after: None,
        }
    }

    pub fn before(before: &str) -> Self {
        Self {
            timestamp: None,
            start: None,
            stop: None,
            before: Some(before.to_owned()),
            after: None,
        }
    }

    pub fn after(after: &str) -> Self {
        Self {
            timestamp: None,
            start: None,
            stop: None,
            before: None,
            after: Some(after.to_owned()),
        }
    }

}
