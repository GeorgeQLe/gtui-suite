#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Method::GET => Method::POST,
            Method::POST => Method::PUT,
            Method::PUT => Method::PATCH,
            Method::PATCH => Method::DELETE,
            Method::DELETE => Method::HEAD,
            Method::HEAD => Method::OPTIONS,
            Method::OPTIONS => Method::GET,
        }
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl Header {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    ApiKey,
}

impl Default for AuthType {
    fn default() -> Self {
        AuthType::None
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub api_key: Option<String>,
    pub api_key_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRequest {
    pub id: Uuid,
    pub name: String,
    pub method: Method,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub auth: AuthConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SavedRequest {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            method: Method::GET,
            url: String::new(),
            headers: Vec::new(),
            body: None,
            auth: AuthConfig::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub requests: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            requests: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: u64,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub request_id: Option<Uuid>,
    pub method: Method,
    pub url: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_as_str() {
        assert_eq!(Method::GET.as_str(), "GET");
        assert_eq!(Method::POST.as_str(), "POST");
        assert_eq!(Method::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn test_method_next_cycle() {
        let mut method = Method::GET;
        let methods = [
            Method::POST, Method::PUT, Method::PATCH,
            Method::DELETE, Method::HEAD, Method::OPTIONS, Method::GET,
        ];

        for expected in methods {
            method = method.next();
            assert_eq!(method, expected);
        }
    }

    #[test]
    fn test_method_default() {
        assert_eq!(Method::default(), Method::GET);
    }

    #[test]
    fn test_header_new() {
        let header = Header::new("Content-Type".to_string(), "application/json".to_string());
        assert_eq!(header.key, "Content-Type");
        assert_eq!(header.value, "application/json");
        assert!(header.enabled);
    }

    #[test]
    fn test_auth_config_default() {
        let auth = AuthConfig::default();
        assert_eq!(auth.auth_type, AuthType::None);
        assert!(auth.username.is_none());
        assert!(auth.password.is_none());
        assert!(auth.token.is_none());
    }

    #[test]
    fn test_saved_request_new() {
        let request = SavedRequest::new("Test Request".to_string());
        assert_eq!(request.name, "Test Request");
        assert_eq!(request.method, Method::GET);
        assert!(request.url.is_empty());
        assert!(request.headers.is_empty());
        assert!(request.body.is_none());
    }

    #[test]
    fn test_collection_new() {
        let collection = Collection::new("My Collection".to_string());
        assert_eq!(collection.name, "My Collection");
        assert!(collection.requests.is_empty());
    }

    #[test]
    fn test_method_serialization() {
        let json = serde_json::to_string(&Method::POST).unwrap();
        assert_eq!(json, "\"POST\"");

        let method: Method = serde_json::from_str("\"DELETE\"").unwrap();
        assert_eq!(method, Method::DELETE);
    }
}
