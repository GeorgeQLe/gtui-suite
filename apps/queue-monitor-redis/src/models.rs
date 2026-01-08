use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub name: String,
    pub length: u64,
    pub first_entry_id: Option<String>,
    pub last_entry_id: Option<String>,
    pub groups: Vec<ConsumerGroup>,
}

impl Stream {
    pub fn new(name: &str, length: u64) -> Self {
        Self {
            name: name.to_string(),
            length,
            first_entry_id: None,
            last_entry_id: None,
            groups: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerGroup {
    pub name: String,
    pub pending: u64,
    pub last_delivered_id: String,
    pub consumers: Vec<Consumer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumer {
    pub name: String,
    pub pending: u64,
    #[serde(with = "serde_duration")]
    pub idle: Duration,
}

mod serde_duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEntry {
    pub id: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisKey {
    pub name: String,
    pub key_type: KeyType,
    pub ttl: Option<i64>,
    pub memory: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    String,
    List,
    Set,
    ZSet,
    Hash,
    Stream,
    Unknown,
}

impl KeyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyType::String => "string",
            KeyType::List => "list",
            KeyType::Set => "set",
            KeyType::ZSet => "zset",
            KeyType::Hash => "hash",
            KeyType::Stream => "stream",
            KeyType::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "string" => KeyType::String,
            "list" => KeyType::List,
            "set" => KeyType::Set,
            "zset" => KeyType::ZSet,
            "hash" => KeyType::Hash,
            "stream" => KeyType::Stream,
            _ => KeyType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubChannel {
    pub name: String,
    pub subscribers: u32,
    pub pattern: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisInfo {
    pub version: String,
    pub connected_clients: u64,
    pub used_memory: u64,
    pub used_memory_human: String,
    pub total_commands_processed: u64,
    pub uptime_in_seconds: u64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
}

impl Default for RedisInfo {
    fn default() -> Self {
        Self {
            version: "Unknown".to_string(),
            connected_clients: 0,
            used_memory: 0,
            used_memory_human: "0B".to_string(),
            total_commands_processed: 0,
            uptime_in_seconds: 0,
            keyspace_hits: 0,
            keyspace_misses: 0,
        }
    }
}

impl RedisInfo {
    pub fn hit_ratio(&self) -> f64 {
        let total = self.keyspace_hits + self.keyspace_misses;
        if total == 0 {
            0.0
        } else {
            self.keyspace_hits as f64 / total as f64 * 100.0
        }
    }

    pub fn uptime_display(&self) -> String {
        let days = self.uptime_in_seconds / 86400;
        let hours = (self.uptime_in_seconds % 86400) / 3600;
        let minutes = (self.uptime_in_seconds % 3600) / 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_type_from_str() {
        assert_eq!(KeyType::from_str("string"), KeyType::String);
        assert_eq!(KeyType::from_str("LIST"), KeyType::List);
        assert_eq!(KeyType::from_str("unknown"), KeyType::Unknown);
    }

    #[test]
    fn test_redis_info_hit_ratio() {
        let info = RedisInfo {
            keyspace_hits: 80,
            keyspace_misses: 20,
            ..Default::default()
        };
        assert!((info.hit_ratio() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_redis_info_uptime_display() {
        let info = RedisInfo {
            uptime_in_seconds: 90061,
            ..Default::default()
        };
        assert_eq!(info.uptime_display(), "1d 1h 1m");
    }

    #[test]
    fn test_stream_new() {
        let stream = Stream::new("test-stream", 100);
        assert_eq!(stream.name, "test-stream");
        assert_eq!(stream.length, 100);
    }
}
