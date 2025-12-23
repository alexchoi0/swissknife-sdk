use crate::{KeyValueProvider, Result};
use async_trait::async_trait;

pub struct RedisClient {
    base_url: String,
    auth_token: Option<String>,
}

impl RedisClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: None,
        }
    }

    pub fn with_auth(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    async fn request(&self, command: &str, args: &[&str]) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "command": command,
            "args": args,
        });

        let mut request = client
            .post(&format!("{}/execute", self.base_url))
            .header("Content-Type", "application/json")
            .json(&body);

        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let resp = request.send().await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        resp.json().await
            .map_err(|e| crate::Error::Query(e.to_string()))
    }
}

#[async_trait]
impl KeyValueProvider for RedisClient {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let result = self.request("GET", &[key]).await?;
        Ok(result.get("value").and_then(|v| v.as_str()).map(String::from))
    }

    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()> {
        if let Some(ttl) = ttl_seconds {
            let ttl_str = ttl.to_string();
            self.request("SET", &[key, value, "EX", &ttl_str]).await?;
        } else {
            self.request("SET", &[key, value]).await?;
        };
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let result = self.request("DEL", &[key]).await?;
        Ok(result.get("deleted").and_then(|v| v.as_u64()).map(|v| v > 0).unwrap_or(false))
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let result = self.request("EXISTS", &[key]).await?;
        Ok(result.get("exists").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        let result = self.request("KEYS", &[pattern]).await?;
        Ok(result.get("keys")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default())
    }

    async fn ttl(&self, key: &str) -> Result<Option<i64>> {
        let result = self.request("TTL", &[key]).await?;
        let ttl = result.get("ttl").and_then(|v| v.as_i64()).unwrap_or(-2);
        if ttl < 0 {
            Ok(None)
        } else {
            Ok(Some(ttl))
        }
    }

    async fn expire(&self, key: &str, seconds: u64) -> Result<bool> {
        let seconds_str = seconds.to_string();
        let result = self.request("EXPIRE", &[key, &seconds_str]).await?;
        Ok(result.get("result").and_then(|v| v.as_i64()).map(|v| v == 1).unwrap_or(false))
    }

    async fn incr(&self, key: &str) -> Result<i64> {
        let result = self.request("INCR", &[key]).await?;
        Ok(result.get("value").and_then(|v| v.as_i64()).unwrap_or(0))
    }

    async fn decr(&self, key: &str) -> Result<i64> {
        let result = self.request("DECR", &[key]).await?;
        Ok(result.get("value").and_then(|v| v.as_i64()).unwrap_or(0))
    }
}
