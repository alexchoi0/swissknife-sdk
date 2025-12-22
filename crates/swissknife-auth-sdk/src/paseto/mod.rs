use crate::{Error, Result};
use rusty_paseto::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

pub fn build_v4_local_token<T: Serialize>(claims: &T, key: &[u8; 32]) -> Result<String> {
    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(key));
    let claims_json = serde_json::to_string(claims).map_err(|e| Error::Token(e.to_string()))?;

    let token = rusty_paseto::prelude::PasetoBuilder::<V4, Local>::default()
        .set_claim(CustomClaim::try_from(("data", claims_json.as_str())).map_err(|e| Error::Token(e.to_string()))?)
        .build(&key)
        .map_err(|e| Error::Token(e.to_string()))?;

    Ok(token)
}

pub fn parse_v4_local_token<T: DeserializeOwned>(token: &str, key: &[u8; 32]) -> Result<T> {
    let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(key));

    let value = rusty_paseto::prelude::PasetoParser::<V4, Local>::default()
        .parse(token, &key)
        .map_err(|e| Error::Token(e.to_string()))?;

    let data = value.get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Token("Missing data claim".into()))?;

    serde_json::from_str(data).map_err(|e| Error::Token(e.to_string()))
}

pub fn build_v4_public_token<T: Serialize>(claims: &T, secret_key: &[u8; 64]) -> Result<String> {
    let key = PasetoAsymmetricPrivateKey::<V4, Public>::from(secret_key.as_slice());
    let claims_json = serde_json::to_string(claims).map_err(|e| Error::Token(e.to_string()))?;

    let token = rusty_paseto::prelude::PasetoBuilder::<V4, Public>::default()
        .set_claim(CustomClaim::try_from(("data", claims_json.as_str())).map_err(|e| Error::Token(e.to_string()))?)
        .build(&key)
        .map_err(|e| Error::Token(e.to_string()))?;

    Ok(token)
}

pub fn parse_v4_public_token<T: DeserializeOwned>(token: &str, public_key: &[u8; 32]) -> Result<T> {
    let key_obj = Key::<32>::from(public_key);
    let key = PasetoAsymmetricPublicKey::<V4, Public>::from(&key_obj);

    let value = rusty_paseto::prelude::PasetoParser::<V4, Public>::default()
        .parse(token, &key)
        .map_err(|e| Error::Token(e.to_string()))?;

    let data = value.get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Token("Missing data claim".into()))?;

    serde_json::from_str(data).map_err(|e| Error::Token(e.to_string()))
}

pub fn generate_v4_local_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn generate_v4_keypair() -> ([u8; 32], [u8; 64]) {
    use rand::RngCore;
    let mut seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);

    let secret = ed25519_dalek::SigningKey::from_bytes(&seed);
    let public = secret.verifying_key();

    let mut secret_bytes = [0u8; 64];
    secret_bytes[..32].copy_from_slice(&seed);
    secret_bytes[32..].copy_from_slice(public.as_bytes());

    (*public.as_bytes(), secret_bytes)
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct PasetoClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl PasetoClaims {
    pub fn new() -> Self {
        Self {
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn subject(mut self, sub: impl Into<String>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    pub fn issuer(mut self, iss: impl Into<String>) -> Self {
        self.iss = Some(iss.into());
        self
    }

    pub fn audience(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    pub fn expiration(mut self, exp: chrono::DateTime<chrono::Utc>) -> Self {
        self.exp = Some(exp.to_rfc3339());
        self
    }

    pub fn expires_in(self, duration: chrono::Duration) -> Self {
        self.expiration(chrono::Utc::now() + duration)
    }

    pub fn not_before(mut self, nbf: chrono::DateTime<chrono::Utc>) -> Self {
        self.nbf = Some(nbf.to_rfc3339());
        self
    }

    pub fn issued_at(mut self, iat: chrono::DateTime<chrono::Utc>) -> Self {
        self.iat = Some(iat.to_rfc3339());
        self
    }

    pub fn token_id(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    pub fn claim<T: Serialize>(mut self, key: impl Into<String>, value: T) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.extra.insert(key.into(), v);
        }
        self
    }

    pub fn is_expired(&self) -> bool {
        self.exp.as_ref().map_or(false, |exp| {
            chrono::DateTime::parse_from_rfc3339(exp)
                .map(|dt| chrono::Utc::now() > dt)
                .unwrap_or(false)
        })
    }
}

impl Default for PasetoClaims {
    fn default() -> Self {
        Self::new()
    }
}
