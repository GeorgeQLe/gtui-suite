use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overview {
    pub cluster_name: String,
    pub rabbitmq_version: String,
    pub erlang_version: String,
    pub message_stats: Option<MessageStats>,
    pub queue_totals: Option<QueueTotals>,
    pub object_totals: ObjectTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    #[serde(default)]
    pub publish: u64,
    #[serde(default)]
    pub deliver: u64,
    #[serde(default)]
    pub deliver_get: u64,
    #[serde(default)]
    pub ack: u64,
    #[serde(default)]
    pub redeliver: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueTotals {
    #[serde(default)]
    pub messages: u64,
    #[serde(default)]
    pub messages_ready: u64,
    #[serde(default)]
    pub messages_unacknowledged: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTotals {
    #[serde(default)]
    pub connections: u64,
    #[serde(default)]
    pub channels: u64,
    #[serde(default)]
    pub exchanges: u64,
    #[serde(default)]
    pub queues: u64,
    #[serde(default)]
    pub consumers: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub name: String,
    pub vhost: String,
    #[serde(default)]
    pub durable: bool,
    #[serde(default)]
    pub auto_delete: bool,
    #[serde(default)]
    pub messages: u64,
    #[serde(default)]
    pub messages_ready: u64,
    #[serde(default)]
    pub messages_unacknowledged: u64,
    #[serde(default)]
    pub consumers: u32,
    #[serde(default)]
    pub memory: u64,
    #[serde(default)]
    pub state: String,
}

impl Queue {
    pub fn state_display(&self) -> &str {
        match self.state.as_str() {
            "running" => "running",
            "idle" => "idle",
            "flow" => "flow",
            _ => &self.state,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
    pub name: String,
    pub vhost: String,
    #[serde(rename = "type")]
    pub exchange_type: String,
    #[serde(default)]
    pub durable: bool,
    #[serde(default)]
    pub auto_delete: bool,
    #[serde(default)]
    pub internal: bool,
}

impl Exchange {
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            "(AMQP default)"
        } else {
            &self.name
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub source: String,
    pub vhost: String,
    pub destination: String,
    pub destination_type: String,
    pub routing_key: String,
    #[serde(default)]
    pub properties_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub name: String,
    pub vhost: String,
    #[serde(default)]
    pub user: String,
    pub state: String,
    #[serde(default)]
    pub channels: u32,
    #[serde(default)]
    pub recv_oct: u64,
    #[serde(default)]
    pub send_oct: u64,
    pub peer_host: Option<String>,
    pub peer_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub vhost: String,
    pub connection_details: ConnectionDetails,
    #[serde(default)]
    pub number: u32,
    #[serde(default)]
    pub user: String,
    pub state: String,
    #[serde(default)]
    pub prefetch_count: u32,
    #[serde(default)]
    pub consumer_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDetails {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumer {
    pub consumer_tag: String,
    pub queue: ConsumerQueue,
    pub channel_details: ChannelDetails,
    #[serde(default)]
    pub ack_required: bool,
    #[serde(default)]
    pub prefetch_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerQueue {
    pub name: String,
    pub vhost: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDetails {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vhost {
    pub name: String,
    #[serde(default)]
    pub messages: u64,
    #[serde(default)]
    pub tracing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub payload: String,
    pub payload_encoding: String,
    #[serde(default)]
    pub message_count: u64,
    pub properties: MessageProperties,
    #[serde(default)]
    pub redelivered: bool,
    pub routing_key: String,
    pub exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub delivery_mode: Option<u8>,
    pub priority: Option<u8>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    pub expiration: Option<String>,
    pub message_id: Option<String>,
    pub timestamp: Option<u64>,
    pub type_field: Option<String>,
    pub user_id: Option<String>,
    pub app_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_state_display() {
        let queue = Queue {
            name: "test".to_string(),
            vhost: "/".to_string(),
            durable: true,
            auto_delete: false,
            messages: 0,
            messages_ready: 0,
            messages_unacknowledged: 0,
            consumers: 0,
            memory: 0,
            state: "running".to_string(),
        };
        assert_eq!(queue.state_display(), "running");
    }

    #[test]
    fn test_exchange_display_name() {
        let exchange = Exchange {
            name: "".to_string(),
            vhost: "/".to_string(),
            exchange_type: "direct".to_string(),
            durable: true,
            auto_delete: false,
            internal: false,
        };
        assert_eq!(exchange.display_name(), "(AMQP default)");
    }
}
