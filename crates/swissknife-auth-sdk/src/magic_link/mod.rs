use crate::{Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct MagicLinkConfig {
    pub secret: Vec<u8>,
    pub expiry: Duration,
    pub base_url: String,
    pub path: String,
}

impl MagicLinkConfig {
    pub fn new(secret: impl Into<Vec<u8>>, base_url: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            expiry: Duration::from_secs(15 * 60),
            base_url: base_url.into(),
            path: "/auth/magic".into(),
        }
    }

    pub fn expiry(mut self, expiry: Duration) -> Self {
        self.expiry = expiry;
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

#[derive(Debug, Clone)]
pub struct MagicLink {
    pub token: String,
    pub url: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct MagicLinkData {
    pub email: String,
    pub user_id: Option<String>,
    pub nonce: String,
    pub expires_at: u64,
}

pub struct MagicLinkGenerator {
    config: MagicLinkConfig,
}

impl MagicLinkGenerator {
    pub fn new(config: MagicLinkConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, email: &str, user_id: Option<&str>) -> Result<MagicLink> {
        let mut rng = rand::thread_rng();
        let nonce: [u8; 16] = rng.gen();
        let nonce_b64 = URL_SAFE_NO_PAD.encode(nonce);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Other(e.to_string()))?
            .as_secs();
        let expires_at = now + self.config.expiry.as_secs();

        let payload = format!(
            "{}:{}:{}:{}",
            email,
            user_id.unwrap_or(""),
            nonce_b64,
            expires_at
        );

        let mut mac = HmacSha256::new_from_slice(&self.config.secret)
            .map_err(|e| Error::Other(e.to_string()))?;
        mac.update(payload.as_bytes());
        let signature = mac.finalize().into_bytes();
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature);

        let token = format!(
            "{}.{}.{}.{}.{}",
            URL_SAFE_NO_PAD.encode(email),
            URL_SAFE_NO_PAD.encode(user_id.unwrap_or("")),
            nonce_b64,
            expires_at,
            signature_b64
        );

        let url = format!(
            "{}{}?token={}",
            self.config.base_url, self.config.path, token
        );

        Ok(MagicLink {
            token,
            url,
            expires_at,
        })
    }

    pub fn verify(&self, token: &str) -> Result<MagicLinkData> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 5 {
            return Err(Error::InvalidToken("Invalid magic link format".into()));
        }

        let email = String::from_utf8(
            URL_SAFE_NO_PAD
                .decode(parts[0])
                .map_err(|_| Error::InvalidToken("Invalid email encoding".into()))?,
        )
        .map_err(|_| Error::InvalidToken("Invalid email UTF-8".into()))?;

        let user_id_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| Error::InvalidToken("Invalid user_id encoding".into()))?;
        let user_id = if user_id_bytes.is_empty() {
            None
        } else {
            Some(
                String::from_utf8(user_id_bytes)
                    .map_err(|_| Error::InvalidToken("Invalid user_id UTF-8".into()))?,
            )
        };

        let nonce = parts[2].to_string();
        let expires_at: u64 = parts[3]
            .parse()
            .map_err(|_| Error::InvalidToken("Invalid expiry".into()))?;
        let provided_signature = parts[4];

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Other(e.to_string()))?
            .as_secs();

        if now > expires_at {
            return Err(Error::TokenExpired);
        }

        let payload = format!(
            "{}:{}:{}:{}",
            email,
            user_id.as_deref().unwrap_or(""),
            nonce,
            expires_at
        );

        let mut mac = HmacSha256::new_from_slice(&self.config.secret)
            .map_err(|e| Error::Other(e.to_string()))?;
        mac.update(payload.as_bytes());
        let expected_signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        if expected_signature != provided_signature {
            return Err(Error::InvalidToken("Invalid signature".into()));
        }

        Ok(MagicLinkData {
            email,
            user_id,
            nonce,
            expires_at,
        })
    }
}

pub struct OtpGenerator {
    secret: Vec<u8>,
    expiry: Duration,
    length: usize,
}

impl OtpGenerator {
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
            expiry: Duration::from_secs(10 * 60),
            length: 6,
        }
    }

    pub fn expiry(mut self, expiry: Duration) -> Self {
        self.expiry = expiry;
        self
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn generate(&self, identifier: &str) -> Result<(String, u64)> {
        let mut rng = rand::thread_rng();
        let max = 10u64.pow(self.length as u32);
        let code: u64 = rng.gen_range(0..max);
        let code_str = format!("{:0>width$}", code, width = self.length);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Other(e.to_string()))?
            .as_secs();
        let expires_at = now + self.expiry.as_secs();

        Ok((code_str, expires_at))
    }

    pub fn hash(&self, code: &str, identifier: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|e| Error::Other(e.to_string()))?;
        mac.update(format!("{}:{}", identifier, code).as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
    }

    pub fn verify(&self, code: &str, identifier: &str, stored_hash: &str) -> Result<bool> {
        let computed = self.hash(code, identifier)?;
        Ok(computed == stored_hash)
    }
}
