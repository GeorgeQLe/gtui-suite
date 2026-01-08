use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub partitions: u32,
    pub replication_factor: u16,
    #[serde(default)]
    pub configs: HashMap<String, String>,
    #[serde(default)]
    pub internal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub topic: String,
    pub partition: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
    pub high_watermark: i64,
    pub low_watermark: i64,
}

impl Partition {
    pub fn message_count(&self) -> i64 {
        self.high_watermark - self.low_watermark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerGroup {
    pub name: String,
    pub state: GroupState,
    pub members: Vec<GroupMember>,
    pub protocol_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupState {
    Stable,
    PreparingRebalance,
    CompletingRebalance,
    Empty,
    Dead,
    Unknown,
}

impl GroupState {
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupState::Stable => "Stable",
            GroupState::PreparingRebalance => "Rebalancing",
            GroupState::CompletingRebalance => "Completing",
            GroupState::Empty => "Empty",
            GroupState::Dead => "Dead",
            GroupState::Unknown => "Unknown",
        }
    }
}

impl Default for GroupState {
    fn default() -> Self {
        GroupState::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub member_id: String,
    pub client_id: String,
    pub client_host: String,
    pub assignments: Vec<MemberAssignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAssignment {
    pub topic: String,
    pub partitions: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerLag {
    pub group: String,
    pub topic: String,
    pub partition: i32,
    pub current_offset: i64,
    pub log_end_offset: i64,
    pub lag: i64,
}

impl ConsumerLag {
    pub fn new(group: &str, topic: &str, partition: i32, current: i64, end: i64) -> Self {
        Self {
            group: group.to_string(),
            topic: topic.to_string(),
            partition,
            current_offset: current,
            log_end_offset: end,
            lag: end - current,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Broker {
    pub id: i32,
    pub host: String,
    pub port: u16,
    pub rack: Option<String>,
    pub is_controller: bool,
}

impl Broker {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub key: Option<String>,
    pub value: String,
    pub timestamp: Option<i64>,
    pub headers: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_message_count() {
        let partition = Partition {
            topic: "test".to_string(),
            partition: 0,
            leader: 1,
            replicas: vec![1, 2, 3],
            isr: vec![1, 2, 3],
            high_watermark: 1000,
            low_watermark: 100,
        };
        assert_eq!(partition.message_count(), 900);
    }

    #[test]
    fn test_consumer_lag_new() {
        let lag = ConsumerLag::new("group1", "topic1", 0, 500, 1000);
        assert_eq!(lag.lag, 500);
    }

    #[test]
    fn test_group_state() {
        assert_eq!(GroupState::Stable.as_str(), "Stable");
        assert_eq!(GroupState::PreparingRebalance.as_str(), "Rebalancing");
    }

    #[test]
    fn test_broker_address() {
        let broker = Broker {
            id: 1,
            host: "localhost".to_string(),
            port: 9092,
            rack: None,
            is_controller: true,
        };
        assert_eq!(broker.address(), "localhost:9092");
    }
}
