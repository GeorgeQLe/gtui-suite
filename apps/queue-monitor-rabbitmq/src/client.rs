use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;

use crate::config::RabbitConfig;
use crate::models::*;

pub struct RabbitClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl RabbitClient {
    pub fn new(config: &RabbitConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let auth = format!("{}:{}", config.user, config.password);
        let auth_header = format!("Basic {}", STANDARD.encode(auth));

        Ok(Self {
            client,
            base_url: config.url.trim_end_matches('/').to_string(),
            auth_header,
        })
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}/api{}", self.base_url, path);
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .context("Failed to connect to RabbitMQ")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("RabbitMQ API error {}: {}", status, text);
        }

        response.json().await.context("Failed to parse response")
    }

    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}/api{}", self.base_url, path);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .json(body)
            .send()
            .await
            .context("Failed to connect to RabbitMQ")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("RabbitMQ API error {}: {}", status, text);
        }

        response.json().await.context("Failed to parse response")
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}/api{}", self.base_url, path);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .context("Failed to connect to RabbitMQ")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("RabbitMQ API error {}: {}", status, text);
        }

        Ok(())
    }

    pub async fn get_overview(&self) -> Result<Overview> {
        self.get("/overview").await
    }

    pub async fn get_queues(&self, vhost: &str) -> Result<Vec<Queue>> {
        let vhost_encoded = urlencoding::encode(vhost);
        self.get(&format!("/queues/{}", vhost_encoded)).await
    }

    pub async fn get_all_queues(&self) -> Result<Vec<Queue>> {
        self.get("/queues").await
    }

    pub async fn get_exchanges(&self, vhost: &str) -> Result<Vec<Exchange>> {
        let vhost_encoded = urlencoding::encode(vhost);
        self.get(&format!("/exchanges/{}", vhost_encoded)).await
    }

    pub async fn get_all_exchanges(&self) -> Result<Vec<Exchange>> {
        self.get("/exchanges").await
    }

    pub async fn get_bindings(&self, vhost: &str) -> Result<Vec<Binding>> {
        let vhost_encoded = urlencoding::encode(vhost);
        self.get(&format!("/bindings/{}", vhost_encoded)).await
    }

    pub async fn get_connections(&self) -> Result<Vec<Connection>> {
        self.get("/connections").await
    }

    pub async fn get_channels(&self) -> Result<Vec<Channel>> {
        self.get("/channels").await
    }

    pub async fn get_consumers(&self) -> Result<Vec<Consumer>> {
        self.get("/consumers").await
    }

    pub async fn get_vhosts(&self) -> Result<Vec<Vhost>> {
        self.get("/vhosts").await
    }

    pub async fn get_messages(
        &self,
        vhost: &str,
        queue: &str,
        count: u32,
        ack_mode: &str,
    ) -> Result<Vec<Message>> {
        let vhost_encoded = urlencoding::encode(vhost);
        let queue_encoded = urlencoding::encode(queue);

        #[derive(serde::Serialize)]
        struct GetRequest {
            count: u32,
            ackmode: String,
            encoding: String,
        }

        let request = GetRequest {
            count,
            ackmode: ack_mode.to_string(),
            encoding: "auto".to_string(),
        };

        self.post(
            &format!("/queues/{}/{}/get", vhost_encoded, queue_encoded),
            &request,
        )
        .await
    }

    pub async fn publish_message(
        &self,
        vhost: &str,
        exchange: &str,
        routing_key: &str,
        payload: &str,
    ) -> Result<()> {
        let vhost_encoded = urlencoding::encode(vhost);
        let exchange_encoded = urlencoding::encode(exchange);

        #[derive(serde::Serialize)]
        struct PublishRequest {
            routing_key: String,
            payload: String,
            payload_encoding: String,
            properties: PublishProperties,
        }

        #[derive(serde::Serialize)]
        struct PublishProperties {
            delivery_mode: u8,
        }

        let request = PublishRequest {
            routing_key: routing_key.to_string(),
            payload: payload.to_string(),
            payload_encoding: "string".to_string(),
            properties: PublishProperties { delivery_mode: 2 },
        };

        #[derive(serde::Deserialize)]
        struct PublishResponse {
            routed: bool,
        }

        let response: PublishResponse = self
            .post(
                &format!("/exchanges/{}/{}/publish", vhost_encoded, exchange_encoded),
                &request,
            )
            .await?;

        if !response.routed {
            anyhow::bail!("Message was not routed to any queue");
        }

        Ok(())
    }

    pub async fn purge_queue(&self, vhost: &str, queue: &str) -> Result<()> {
        let vhost_encoded = urlencoding::encode(vhost);
        let queue_encoded = urlencoding::encode(queue);
        self.delete(&format!(
            "/queues/{}/{}/contents",
            vhost_encoded, queue_encoded
        ))
        .await
    }

    pub async fn delete_queue(&self, vhost: &str, queue: &str) -> Result<()> {
        let vhost_encoded = urlencoding::encode(vhost);
        let queue_encoded = urlencoding::encode(queue);
        self.delete(&format!("/queues/{}/{}", vhost_encoded, queue_encoded))
            .await
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::urlencoding;

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("/"), "%2F");
        assert_eq!(urlencoding::encode("test"), "test");
        assert_eq!(urlencoding::encode("my/vhost"), "my%2Fvhost");
    }
}
