use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcDiscoveryDocument {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userinfo_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_session_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_authorization_endpoint: Option<String>,
    #[serde(default)]
    pub scopes_supported: Vec<String>,
    #[serde(default)]
    pub response_types_supported: Vec<String>,
    #[serde(default)]
    pub response_modes_supported: Vec<String>,
    #[serde(default)]
    pub grant_types_supported: Vec<String>,
    #[serde(default)]
    pub subject_types_supported: Vec<String>,
    #[serde(default)]
    pub id_token_signing_alg_values_supported: Vec<String>,
    #[serde(default)]
    pub token_endpoint_auth_methods_supported: Vec<String>,
    #[serde(default)]
    pub claims_supported: Vec<String>,
    #[serde(default)]
    pub code_challenge_methods_supported: Vec<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

struct CachedDocument {
    document: OidcDiscoveryDocument,
    fetched_at: Instant,
}

pub struct OidcDiscoveryClient {
    http: reqwest::Client,
    cache: Arc<RwLock<Option<CachedDocument>>>,
    cache_duration: Duration,
    issuer: String,
}

impl OidcDiscoveryClient {
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
            cache_duration: Duration::from_secs(3600),
            issuer: issuer.into(),
        }
    }

    pub fn with_cache_duration(mut self, duration: Duration) -> Self {
        self.cache_duration = duration;
        self
    }

    fn discovery_url(&self) -> String {
        let issuer = self.issuer.trim_end_matches('/');
        format!("{}/.well-known/openid-configuration", issuer)
    }

    pub async fn discover(&self) -> Result<OidcDiscoveryDocument> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.cache_duration {
                    return Ok(cached.document.clone());
                }
            }
        }

        let url = self.discovery_url();
        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(Error::OAuth(format!(
                "Failed to fetch OIDC discovery document: {}",
                response.status()
            )));
        }

        let document: OidcDiscoveryDocument = response.json().await?;

        if document.issuer != self.issuer && document.issuer != self.issuer.trim_end_matches('/') {
            return Err(Error::OAuth(format!(
                "Issuer mismatch: expected {}, got {}",
                self.issuer, document.issuer
            )));
        }

        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedDocument {
                document: document.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(document)
    }

    pub async fn authorization_endpoint(&self) -> Result<String> {
        Ok(self.discover().await?.authorization_endpoint)
    }

    pub async fn token_endpoint(&self) -> Result<String> {
        Ok(self.discover().await?.token_endpoint)
    }

    pub async fn userinfo_endpoint(&self) -> Result<Option<String>> {
        Ok(self.discover().await?.userinfo_endpoint)
    }

    pub async fn jwks_uri(&self) -> Result<Option<String>> {
        Ok(self.discover().await?.jwks_uri)
    }

    pub async fn revocation_endpoint(&self) -> Result<Option<String>> {
        Ok(self.discover().await?.revocation_endpoint)
    }

    pub async fn introspection_endpoint(&self) -> Result<Option<String>> {
        Ok(self.discover().await?.introspection_endpoint)
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.try_write() {
            *cache = None;
        }
    }
}

pub struct OidcProviderMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
}

impl OidcProviderMetadata {
    pub fn new(issuer: impl Into<String>, jwks_uri: impl Into<String>) -> Self {
        let issuer = issuer.into();
        Self {
            authorization_endpoint: format!("{}/authorize", issuer),
            token_endpoint: format!("{}/token", issuer),
            userinfo_endpoint: Some(format!("{}/userinfo", issuer)),
            jwks_uri: jwks_uri.into(),
            issuer,
            scopes_supported: vec!["openid".into(), "email".into(), "profile".into()],
            response_types_supported: vec!["code".into(), "token".into(), "id_token".into()],
            grant_types_supported: vec!["authorization_code".into(), "refresh_token".into()],
            subject_types_supported: vec!["public".into()],
            id_token_signing_alg_values_supported: vec!["RS256".into()],
        }
    }

    pub fn to_discovery_document(&self) -> OidcDiscoveryDocument {
        OidcDiscoveryDocument {
            issuer: self.issuer.clone(),
            authorization_endpoint: self.authorization_endpoint.clone(),
            token_endpoint: self.token_endpoint.clone(),
            userinfo_endpoint: self.userinfo_endpoint.clone(),
            jwks_uri: Some(self.jwks_uri.clone()),
            registration_endpoint: None,
            revocation_endpoint: None,
            introspection_endpoint: None,
            end_session_endpoint: None,
            device_authorization_endpoint: None,
            scopes_supported: self.scopes_supported.clone(),
            response_types_supported: self.response_types_supported.clone(),
            response_modes_supported: vec!["query".into(), "fragment".into()],
            grant_types_supported: self.grant_types_supported.clone(),
            subject_types_supported: self.subject_types_supported.clone(),
            id_token_signing_alg_values_supported: self.id_token_signing_alg_values_supported.clone(),
            token_endpoint_auth_methods_supported: vec!["client_secret_basic".into(), "client_secret_post".into()],
            claims_supported: vec!["sub".into(), "iss".into(), "aud".into(), "exp".into(), "iat".into(), "email".into(), "name".into()],
            code_challenge_methods_supported: vec!["S256".into()],
            extra: std::collections::HashMap::new(),
        }
    }
}
