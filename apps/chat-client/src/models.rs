use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Irc,
    Matrix,
    Discord,
    Slack,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Irc => "IRC",
            Protocol::Matrix => "Matrix",
            Protocol::Discord => "Discord",
            Protocol::Slack => "Slack",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
    pub channels: Vec<Channel>,
}

impl Server {
    pub fn new(name: &str, protocol: Protocol, host: &str, port: u16, username: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            protocol,
            host: host.to_string(),
            port,
            username: username.to_string(),
            connected: false,
            channels: Vec::new(),
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.protocol.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub topic: Option<String>,
    pub users: Vec<User>,
    pub messages: Vec<Message>,
    pub unread_count: u32,
    pub is_private: bool,
}

impl Channel {
    pub fn new(name: &str, is_private: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            topic: None,
            users: Vec::new(),
            messages: Vec::new(),
            unread_count: 0,
            is_private,
        }
    }

    pub fn display_name(&self) -> String {
        if self.is_private {
            self.name.clone()
        } else {
            format!("#{}", self.name)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub status: UserStatus,
    pub is_op: bool,
    pub is_voice: bool,
}

impl User {
    pub fn new(username: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            display_name: None,
            status: UserStatus::Online,
            is_op: false,
            is_voice: false,
        }
    }

    pub fn nick_prefix(&self) -> &'static str {
        if self.is_op {
            "@"
        } else if self.is_voice {
            "+"
        } else {
            ""
        }
    }

    pub fn name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Online,
    Away,
    Busy,
    Offline,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Online => "online",
            UserStatus::Away => "away",
            UserStatus::Busy => "busy",
            UserStatus::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub is_mention: bool,
}

impl Message {
    pub fn new(sender: &str, content: &str, message_type: MessageType) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            message_type,
            is_mention: false,
        }
    }

    pub fn chat(sender: &str, content: &str) -> Self {
        Self::new(sender, content, MessageType::Chat)
    }

    pub fn system(content: &str) -> Self {
        Self::new("*", content, MessageType::System)
    }

    pub fn action(sender: &str, content: &str) -> Self {
        Self::new(sender, content, MessageType::Action)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Chat,
    System,
    Action,
    Notice,
    Join,
    Part,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_display_name() {
        let server = Server::new("Libera", Protocol::Irc, "irc.libera.chat", 6697, "user");
        assert_eq!(server.display_name(), "Libera (IRC)");
    }

    #[test]
    fn test_channel_display_name() {
        let public = Channel::new("rust", false);
        assert_eq!(public.display_name(), "#rust");

        let private = Channel::new("friend", true);
        assert_eq!(private.display_name(), "friend");
    }

    #[test]
    fn test_user_prefix() {
        let mut user = User::new("nick");
        assert_eq!(user.nick_prefix(), "");

        user.is_op = true;
        assert_eq!(user.nick_prefix(), "@");

        user.is_op = false;
        user.is_voice = true;
        assert_eq!(user.nick_prefix(), "+");
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::chat("user", "Hello!");
        assert_eq!(msg.sender, "user");
        assert_eq!(msg.message_type, MessageType::Chat);
    }
}
