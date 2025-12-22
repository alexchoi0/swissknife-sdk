use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl IntrospectionResponse {
    pub fn inactive() -> Self {
        Self {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: None,
            iat: None,
            nbf: None,
            sub: None,
            aud: None,
            iss: None,
            jti: None,
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.exp.map_or(false, |exp| {
            let now = chrono::Utc::now().timestamp();
            now > exp
        })
    }

    pub fn scopes(&self) -> Vec<&str> {
        self.scope
            .as_ref()
            .map(|s| s.split_whitespace().collect())
            .unwrap_or_default()
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes().contains(&scope)
    }

    pub fn audiences(&self) -> Vec<String> {
        match &self.aud {
            Some(serde_json::Value::String(s)) => vec![s.clone()],
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenTypeHint {
    AccessToken,
    RefreshToken,
    Custom(String),
}

impl TokenTypeHint {
    pub fn as_str(&self) -> &str {
        match self {
            TokenTypeHint::AccessToken => "access_token",
            TokenTypeHint::RefreshToken => "refresh_token",
            TokenTypeHint::Custom(s) => s,
        }
    }
}

pub struct IntrospectionClient {
    http: reqwest::Client,
    endpoint: String,
    client_id: String,
    client_secret: Option<String>,
}

impl IntrospectionClient {
    pub fn new(
        endpoint: impl Into<String>,
        client_id: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            endpoint: endpoint.into(),
            client_id: client_id.into(),
            client_secret: None,
        }
    }

    pub fn with_client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    pub async fn introspect(&self, token: &str) -> Result<IntrospectionResponse> {
        self.introspect_with_hint(token, None).await
    }

    pub async fn introspect_with_hint(
        &self,
        token: &str,
        token_type_hint: Option<TokenTypeHint>,
    ) -> Result<IntrospectionResponse> {
        let mut form = vec![("token", token.to_string())];

        if let Some(hint) = token_type_hint {
            form.push(("token_type_hint", hint.as_str().to_string()));
        }

        let mut request = self.http.post(&self.endpoint);

        if let Some(secret) = &self.client_secret {
            request = request.basic_auth(&self.client_id, Some(secret));
        } else {
            form.push(("client_id", self.client_id.clone()));
        }

        let response = request.form(&form).send().await?;

        if !response.status().is_success() {
            return Err(Error::OAuth(format!(
                "Introspection failed: {}",
                response.status()
            )));
        }

        let introspection: IntrospectionResponse = response.json().await?;
        Ok(introspection)
    }

    pub async fn is_active(&self, token: &str) -> Result<bool> {
        let response = self.introspect(token).await?;
        Ok(response.active)
    }
}

pub struct TokenRevocationClient {
    http: reqwest::Client,
    endpoint: String,
    client_id: String,
    client_secret: Option<String>,
}

impl TokenRevocationClient {
    pub fn new(
        endpoint: impl Into<String>,
        client_id: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            endpoint: endpoint.into(),
            client_id: client_id.into(),
            client_secret: None,
        }
    }

    pub fn with_client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    pub async fn revoke(&self, token: &str) -> Result<()> {
        self.revoke_with_hint(token, None).await
    }

    pub async fn revoke_with_hint(
        &self,
        token: &str,
        token_type_hint: Option<TokenTypeHint>,
    ) -> Result<()> {
        let mut form = vec![("token", token.to_string())];

        if let Some(hint) = token_type_hint {
            form.push(("token_type_hint", hint.as_str().to_string()));
        }

        let mut request = self.http.post(&self.endpoint);

        if let Some(secret) = &self.client_secret {
            request = request.basic_auth(&self.client_id, Some(secret));
        } else {
            form.push(("client_id", self.client_id.clone()));
        }

        let response = request.form(&form).send().await?;

        if !response.status().is_success() && response.status().as_u16() != 200 {
            return Err(Error::OAuth(format!(
                "Revocation failed: {}",
                response.status()
            )));
        }

        Ok(())
    }

    pub async fn revoke_access_token(&self, token: &str) -> Result<()> {
        self.revoke_with_hint(token, Some(TokenTypeHint::AccessToken)).await
    }

    pub async fn revoke_refresh_token(&self, token: &str) -> Result<()> {
        self.revoke_with_hint(token, Some(TokenTypeHint::RefreshToken)).await
    }
}

use async_trait::async_trait;

#[async_trait]
pub trait TokenValidator: Send + Sync {
    async fn validate(&self, token: &str) -> Result<IntrospectionResponse>;
}

#[async_trait]
impl TokenValidator for IntrospectionClient {
    async fn validate(&self, token: &str) -> Result<IntrospectionResponse> {
        self.introspect(token).await
    }
}

pub struct CachingTokenValidator<V: TokenValidator> {
    inner: V,
    cache: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, (IntrospectionResponse, std::time::Instant)>>>,
    cache_duration: std::time::Duration,
}

impl<V: TokenValidator> CachingTokenValidator<V> {
    pub fn new(inner: V, cache_duration: std::time::Duration) -> Self {
        Self {
            inner,
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            cache_duration,
        }
    }
}

#[async_trait]
impl<V: TokenValidator> TokenValidator for CachingTokenValidator<V> {
    async fn validate(&self, token: &str) -> Result<IntrospectionResponse> {
        let token_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            hex::encode(hasher.finalize())
        };

        {
            let cache = self.cache.read().await;
            if let Some((response, cached_at)) = cache.get(&token_hash) {
                if cached_at.elapsed() < self.cache_duration {
                    return Ok(response.clone());
                }
            }
        }

        let response = self.inner.validate(token).await?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(token_hash, (response.clone(), std::time::Instant::now()));
        }

        Ok(response)
    }
}
