#![allow(dead_code)]

use anyhow::Result;
use std::time::{Duration, Instant};

use crate::request::{AuthType, Method, Response, SavedRequest};

pub async fn send_request(request: &SavedRequest) -> Result<Response> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let method = match request.method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::PATCH => reqwest::Method::PATCH,
        Method::DELETE => reqwest::Method::DELETE,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
    };

    let mut req_builder = client.request(method, &request.url);

    // Add headers
    for header in &request.headers {
        if header.enabled {
            req_builder = req_builder.header(&header.key, &header.value);
        }
    }

    // Add authentication
    match request.auth.auth_type {
        AuthType::Basic => {
            if let (Some(username), Some(password)) = (&request.auth.username, &request.auth.password) {
                req_builder = req_builder.basic_auth(username, Some(password));
            }
        }
        AuthType::Bearer => {
            if let Some(token) = &request.auth.token {
                req_builder = req_builder.bearer_auth(token);
            }
        }
        AuthType::ApiKey => {
            if let (Some(name), Some(key)) = (&request.auth.api_key_name, &request.auth.api_key) {
                req_builder = req_builder.header(name, key);
            }
        }
        AuthType::None => {}
    }

    // Add body
    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    let start = Instant::now();
    let response = req_builder.send().await?;
    let duration = start.elapsed();

    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("").to_string();

    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response.text().await?;
    let size_bytes = body.len();

    Ok(Response {
        status,
        status_text,
        headers,
        body,
        duration_ms: duration.as_millis() as u64,
        size_bytes,
    })
}

pub fn generate_curl(request: &SavedRequest) -> String {
    let mut parts = vec![format!("curl -X {}", request.method.as_str())];

    for header in &request.headers {
        if header.enabled {
            parts.push(format!("-H '{}: {}'", header.key, header.value));
        }
    }

    match request.auth.auth_type {
        AuthType::Basic => {
            if let (Some(user), Some(pass)) = (&request.auth.username, &request.auth.password) {
                parts.push(format!("-u '{}:{}'", user, pass));
            }
        }
        AuthType::Bearer => {
            if let Some(token) = &request.auth.token {
                parts.push(format!("-H 'Authorization: Bearer {}'", token));
            }
        }
        AuthType::ApiKey => {
            if let (Some(name), Some(key)) = (&request.auth.api_key_name, &request.auth.api_key) {
                parts.push(format!("-H '{}: {}'", name, key));
            }
        }
        AuthType::None => {}
    }

    if let Some(body) = &request.body {
        parts.push(format!("-d '{}'", body.replace('\'', "'\\''")));
    }

    parts.push(format!("'{}'", request.url));
    parts.join(" \\\n  ")
}
