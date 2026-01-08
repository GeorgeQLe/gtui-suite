use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub mailboxes: Vec<Mailbox>,
}

impl Account {
    pub fn new(name: &str, email: &str, imap_host: &str, smtp_host: &str, username: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            email: email.to_string(),
            imap_host: imap_host.to_string(),
            imap_port: 993,
            smtp_host: smtp_host.to_string(),
            smtp_port: 587,
            username: username.to_string(),
            mailboxes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    pub name: String,
    pub path: String,
    pub total: u32,
    pub unread: u32,
    pub mailbox_type: MailboxType,
}

impl Mailbox {
    pub fn new(name: &str, path: &str, mailbox_type: MailboxType) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            total: 0,
            unread: 0,
            mailbox_type,
        }
    }

    pub fn display_icon(&self) -> &'static str {
        match self.mailbox_type {
            MailboxType::Inbox => "📥",
            MailboxType::Sent => "📤",
            MailboxType::Drafts => "📝",
            MailboxType::Trash => "🗑️",
            MailboxType::Spam => "⚠️",
            MailboxType::Archive => "📦",
            MailboxType::Custom => "📁",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MailboxType {
    Inbox,
    Sent,
    Drafts,
    Trash,
    Spam,
    Archive,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: Uuid,
    pub message_id: String,
    pub subject: String,
    pub from: Address,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub date: DateTime<Utc>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<Attachment>,
    pub flags: EmailFlags,
}

impl Email {
    pub fn new(subject: &str, from: Address, to: Vec<Address>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id: format!("<{}>", Uuid::new_v4()),
            subject: subject.to_string(),
            from,
            to,
            cc: Vec::new(),
            date: Utc::now(),
            body_text: None,
            body_html: None,
            attachments: Vec::new(),
            flags: EmailFlags::default(),
        }
    }

    pub fn preview(&self, max_len: usize) -> String {
        self.body_text
            .as_deref()
            .map(|s| {
                let cleaned: String = s
                    .lines()
                    .filter(|l| !l.starts_with('>'))
                    .collect::<Vec<_>>()
                    .join(" ");
                if cleaned.len() > max_len {
                    format!("{}...", &cleaned[..max_len])
                } else {
                    cleaned
                }
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub name: Option<String>,
    pub email: String,
}

impl Address {
    pub fn new(email: &str) -> Self {
        Self {
            name: None,
            email: email.to_string(),
        }
    }

    pub fn with_name(name: &str, email: &str) -> Self {
        Self {
            name: Some(name.to_string()),
            email: email.to_string(),
        }
    }

    pub fn display(&self) -> String {
        match &self.name {
            Some(name) => format!("{} <{}>", name, self.email),
            None => self.email.clone(),
        }
    }

    pub fn short_display(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.email)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmailFlags {
    pub seen: bool,
    pub answered: bool,
    pub flagged: bool,
    pub deleted: bool,
    pub draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
}

impl Attachment {
    pub fn size_display(&self) -> String {
        if self.size >= 1024 * 1024 {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        } else if self.size >= 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else {
            format!("{} B", self.size)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComposeEmail {
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<String>,
    pub in_reply_to: Option<String>,
}

impl ComposeEmail {
    pub fn reply(email: &Email, reply_all: bool) -> Self {
        let to = if reply_all {
            let mut addrs: Vec<String> = vec![email.from.email.clone()];
            addrs.extend(email.to.iter().map(|a| a.email.clone()));
            addrs.join(", ")
        } else {
            email.from.email.clone()
        };

        let subject = if email.subject.starts_with("Re:") {
            email.subject.clone()
        } else {
            format!("Re: {}", email.subject)
        };

        let quoted = email
            .body_text
            .as_deref()
            .map(|body| {
                body.lines()
                    .map(|l| format!("> {}", l))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        let body = format!(
            "\n\nOn {}, {} wrote:\n{}",
            email.date.format("%Y-%m-%d %H:%M"),
            email.from.short_display(),
            quoted
        );

        Self {
            to,
            cc: String::new(),
            subject,
            body,
            attachments: Vec::new(),
            in_reply_to: Some(email.message_id.clone()),
        }
    }

    pub fn forward(email: &Email) -> Self {
        let subject = if email.subject.starts_with("Fwd:") {
            email.subject.clone()
        } else {
            format!("Fwd: {}", email.subject)
        };

        let body = format!(
            "\n\n---------- Forwarded message ----------\nFrom: {}\nDate: {}\nSubject: {}\n\n{}",
            email.from.display(),
            email.date.format("%Y-%m-%d %H:%M"),
            email.subject,
            email.body_text.as_deref().unwrap_or("")
        );

        Self {
            to: String::new(),
            cc: String::new(),
            subject,
            body,
            attachments: Vec::new(),
            in_reply_to: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_display() {
        let addr = Address::with_name("John Doe", "john@example.com");
        assert_eq!(addr.display(), "John Doe <john@example.com>");

        let addr2 = Address::new("jane@example.com");
        assert_eq!(addr2.display(), "jane@example.com");
    }

    #[test]
    fn test_attachment_size_display() {
        let small = Attachment {
            filename: "test.txt".to_string(),
            mime_type: "text/plain".to_string(),
            size: 512,
        };
        assert_eq!(small.size_display(), "512 B");

        let medium = Attachment {
            filename: "doc.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            size: 50 * 1024,
        };
        assert_eq!(medium.size_display(), "50.0 KB");
    }

    #[test]
    fn test_email_preview() {
        let mut email = Email::new(
            "Test",
            Address::new("test@example.com"),
            vec![Address::new("to@example.com")],
        );
        email.body_text = Some("This is a test email body.".to_string());

        assert_eq!(email.preview(10), "This is a ...");
    }

    #[test]
    fn test_compose_reply() {
        let email = Email::new(
            "Hello",
            Address::with_name("Sender", "sender@example.com"),
            vec![Address::new("me@example.com")],
        );

        let reply = ComposeEmail::reply(&email, false);
        assert_eq!(reply.to, "sender@example.com");
        assert_eq!(reply.subject, "Re: Hello");
    }
}
