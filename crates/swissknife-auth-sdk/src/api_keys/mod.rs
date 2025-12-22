use crate::{Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub key_id: String,
    pub secret: String,
    pub full_key: String,
    pub hash: String,
    pub prefix: String,
}

#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    pub prefix: String,
    pub key_id_length: usize,
    pub secret_length: usize,
    pub separator: char,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            prefix: "sk".into(),
            key_id_length: 8,
            secret_length: 32,
            separator: '_',
        }
    }
}

impl ApiKeyConfig {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            ..Default::default()
        }
    }

    pub fn key_id_length(mut self, length: usize) -> Self {
        self.key_id_length = length;
        self
    }

    pub fn secret_length(mut self, length: usize) -> Self {
        self.secret_length = length;
        self
    }

    pub fn separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }
}

pub struct ApiKeyGenerator {
    config: ApiKeyConfig,
}

impl ApiKeyGenerator {
    pub fn new(config: ApiKeyConfig) -> Self {
        Self { config }
    }

    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self::new(ApiKeyConfig::new(prefix))
    }

    pub fn generate(&self) -> ApiKey {
        let mut rng = rand::thread_rng();

        let key_id_bytes: Vec<u8> = (0..self.config.key_id_length / 2 + 1)
            .map(|_| rng.gen())
            .collect();
        let key_id = hex::encode(&key_id_bytes)[..self.config.key_id_length].to_string();

        let secret_bytes: Vec<u8> = (0..self.config.secret_length).map(|_| rng.gen()).collect();
        let secret = URL_SAFE_NO_PAD.encode(&secret_bytes);

        let full_key = format!(
            "{}{}{}{}{}",
            self.config.prefix, self.config.separator, key_id, self.config.separator, secret
        );

        let prefix = format!(
            "{}{}{}",
            self.config.prefix, self.config.separator, key_id
        );

        let hash = self.hash_key(&full_key);

        ApiKey {
            key_id,
            secret,
            full_key,
            hash,
            prefix,
        }
    }

    pub fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn verify(&self, key: &str, stored_hash: &str) -> bool {
        let computed_hash = self.hash_key(key);
        constant_time_compare(&computed_hash, stored_hash)
    }

    pub fn parse(&self, key: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = key.split(self.config.separator).collect();
        if parts.len() != 3 {
            return Err(Error::InvalidToken("Invalid API key format".into()));
        }

        if parts[0] != self.config.prefix {
            return Err(Error::InvalidToken("Invalid API key prefix".into()));
        }

        Ok((parts[1].to_string(), parts[2].to_string()))
    }

    pub fn extract_key_id(&self, key: &str) -> Result<String> {
        let (key_id, _) = self.parse(key)?;
        Ok(key_id)
    }
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[derive(Debug, Clone)]
pub struct ScopedApiKey {
    pub key: ApiKey,
    pub scopes: Vec<String>,
    pub expires_at: Option<u64>,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct ScopedApiKeyGenerator {
    generator: ApiKeyGenerator,
}

impl ScopedApiKeyGenerator {
    pub fn new(config: ApiKeyConfig) -> Self {
        Self {
            generator: ApiKeyGenerator::new(config),
        }
    }

    pub fn generate(&self, scopes: Vec<String>, expires_in: Option<std::time::Duration>) -> ScopedApiKey {
        let key = self.generator.generate();

        let expires_at = expires_in.map(|duration| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + duration.as_secs()
        });

        ScopedApiKey {
            key,
            scopes,
            expires_at,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn verify(&self, key: &str, stored_hash: &str) -> bool {
        self.generator.verify(key, stored_hash)
    }

    pub fn is_expired(&self, scoped_key: &ScopedApiKey) -> bool {
        scoped_key.expires_at.map_or(false, |exp| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > exp
        })
    }

    pub fn has_scope(&self, scoped_key: &ScopedApiKey, scope: &str) -> bool {
        scoped_key.scopes.iter().any(|s| s == scope || s == "*")
    }

    pub fn has_any_scope(&self, scoped_key: &ScopedApiKey, scopes: &[&str]) -> bool {
        scopes.iter().any(|scope| self.has_scope(scoped_key, scope))
    }

    pub fn has_all_scopes(&self, scoped_key: &ScopedApiKey, scopes: &[&str]) -> bool {
        scopes.iter().all(|scope| self.has_scope(scoped_key, scope))
    }
}

pub fn generate_random_token(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn generate_secure_token() -> String {
    generate_random_token(32)
}
