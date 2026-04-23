//! Shared HTTP client utilities

use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Request failed: {0}")]
    Request(String),
    #[error("Timeout")]
    Timeout,
    #[error("Not Modified")]
    NotModified,
    #[error("Invalid response")]
    InvalidResponse,
}

#[derive(Clone, Default)]
pub struct HttpCache {
    etag: Option<HashMap<String, String>>,
    last_modified: Option<HashMap<String, String>>,
}

impl HttpCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_etag(&self, url: &str) -> Option<String> {
        self.etag.as_ref()?.get(url).cloned()
    }

    pub fn get_last_modified(&self, url: &str) -> Option<String> {
        self.last_modified.as_ref()?.get(url).cloned()
    }

    pub fn store(&mut self, url: String, etag: Option<String>, last_modified: Option<String>) {
        if let Some(etag) = etag {
            self.etag.get_or_insert_with(HashMap::new).insert(url.clone(), etag);
        }
        if let Some(lm) = last_modified {
            self.last_modified.get_or_insert_with(HashMap::new).insert(url, lm);
        }
    }
}

pub struct HttpClient {
    client: reqwest::Client,
    cache: Arc<RwLock<HttpCache>>,
}

impl HttpClient {
    pub fn new(user_agent: &str, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(timeout_secs))
            .tcp_keepalive(Duration::from_secs(30))
            .pool_max_idle_per_host(5)
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            cache: Arc::new(RwLock::new(HttpCache::new())),
        }
    }

    pub async fn get(&self, url: &str) -> Result<String, HttpError> {
        let cache = self.cache.read().await;
        let etag = cache.get_etag(url);
        let last_modified = cache.get_last_modified(url);
        drop(cache);

        let mut request = self.client.get(url);

        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        if let Some(lm) = last_modified {
            request = request.header("If-Modified-Since", lm);
        }

        let response = request
            .send()
            .await
            .map_err(|e| HttpError::Request(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Err(HttpError::NotModified);
        }

        if !response.status().is_success() {
            return Err(HttpError::Request(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }

        let etag = response
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let last_modified = response
            .headers()
            .get("Last-Modified")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        if etag.is_some() || last_modified.is_some() {
            let mut cache = self.cache.write().await;
            cache.store(url.to_string(), etag, last_modified);
        }

        response
            .text()
            .await
            .map_err(|e| HttpError::Request(e.to_string()))
    }

    pub async fn get_uncached(&self, url: &str) -> Result<String, HttpError> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| HttpError::Request(e.to_string()))?;

        if !response.status().is_success() {
            return Err(HttpError::Request(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| HttpError::Request(e.to_string()))
    }
}

pub async fn http_get(url: &str, user_agent: &str, timeout_secs: u64) -> Result<String, HttpError> {
    let client = HttpClient::new(user_agent, timeout_secs);
    client.get(url).await
}
