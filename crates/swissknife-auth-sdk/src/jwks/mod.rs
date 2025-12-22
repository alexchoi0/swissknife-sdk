use crate::{Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,
    #[serde(rename = "use", skip_serializing_if = "Option::is_none")]
    pub use_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_ops: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub k: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x5c: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x5t: Option<String>,
    #[serde(rename = "x5t#S256", skip_serializing_if = "Option::is_none")]
    pub x5t_s256: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Jwk {
    pub fn is_rsa(&self) -> bool {
        self.kty == "RSA"
    }

    pub fn is_ec(&self) -> bool {
        self.kty == "EC"
    }

    pub fn is_symmetric(&self) -> bool {
        self.kty == "oct"
    }

    pub fn is_signing_key(&self) -> bool {
        self.use_.as_deref() == Some("sig") || self.use_.is_none()
    }

    pub fn is_encryption_key(&self) -> bool {
        self.use_.as_deref() == Some("enc")
    }

    pub fn to_pem(&self) -> Result<String> {
        if self.is_rsa() {
            self.rsa_to_pem()
        } else if self.is_ec() {
            self.ec_to_pem()
        } else {
            Err(Error::Token(format!("Unsupported key type: {}", self.kty)))
        }
    }

    fn rsa_to_pem(&self) -> Result<String> {
        let n = self.n.as_ref().ok_or_else(|| Error::Token("Missing n parameter".into()))?;
        let e = self.e.as_ref().ok_or_else(|| Error::Token("Missing e parameter".into()))?;

        let n_bytes = URL_SAFE_NO_PAD.decode(n)?;
        let e_bytes = URL_SAFE_NO_PAD.decode(e)?;

        let der = encode_rsa_public_key_der(&n_bytes, &e_bytes);
        let pem = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
            base64::engine::general_purpose::STANDARD.encode(&der)
                .chars()
                .collect::<Vec<_>>()
                .chunks(64)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(pem)
    }

    fn ec_to_pem(&self) -> Result<String> {
        let crv = self.crv.as_ref().ok_or_else(|| Error::Token("Missing crv parameter".into()))?;
        let x = self.x.as_ref().ok_or_else(|| Error::Token("Missing x parameter".into()))?;
        let y = self.y.as_ref().ok_or_else(|| Error::Token("Missing y parameter".into()))?;

        let x_bytes = URL_SAFE_NO_PAD.decode(x)?;
        let y_bytes = URL_SAFE_NO_PAD.decode(y)?;

        let der = encode_ec_public_key_der(crv, &x_bytes, &y_bytes)?;
        let pem = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
            base64::engine::general_purpose::STANDARD.encode(&der)
                .chars()
                .collect::<Vec<_>>()
                .chunks(64)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(pem)
    }
}

fn encode_rsa_public_key_der(n: &[u8], e: &[u8]) -> Vec<u8> {
    fn encode_integer(bytes: &[u8]) -> Vec<u8> {
        let mut result = vec![0x02];
        let needs_padding = bytes.first().map(|b| b & 0x80 != 0).unwrap_or(false);
        let len = bytes.len() + if needs_padding { 1 } else { 0 };
        result.extend(encode_length(len));
        if needs_padding {
            result.push(0x00);
        }
        result.extend(bytes);
        result
    }

    fn encode_length(len: usize) -> Vec<u8> {
        if len < 128 {
            vec![len as u8]
        } else if len < 256 {
            vec![0x81, len as u8]
        } else {
            vec![0x82, (len >> 8) as u8, (len & 0xff) as u8]
        }
    }

    let n_encoded = encode_integer(n);
    let e_encoded = encode_integer(e);

    let rsa_key_len = n_encoded.len() + e_encoded.len();
    let mut rsa_key = vec![0x30];
    rsa_key.extend(encode_length(rsa_key_len));
    rsa_key.extend(n_encoded);
    rsa_key.extend(e_encoded);

    let rsa_oid: Vec<u8> = vec![
        0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86,
        0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00
    ];

    let bit_string_len = rsa_key.len() + 1;
    let mut bit_string = vec![0x03];
    bit_string.extend(encode_length(bit_string_len));
    bit_string.push(0x00);
    bit_string.extend(rsa_key);

    let total_len = rsa_oid.len() + bit_string.len();
    let mut result = vec![0x30];
    result.extend(encode_length(total_len));
    result.extend(rsa_oid);
    result.extend(bit_string);

    result
}

fn encode_ec_public_key_der(crv: &str, x: &[u8], y: &[u8]) -> Result<Vec<u8>> {
    fn encode_length(len: usize) -> Vec<u8> {
        if len < 128 {
            vec![len as u8]
        } else if len < 256 {
            vec![0x81, len as u8]
        } else {
            vec![0x82, (len >> 8) as u8, (len & 0xff) as u8]
        }
    }

    let oid = match crv {
        "P-256" => vec![0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07],
        "P-384" => vec![0x06, 0x05, 0x2b, 0x81, 0x04, 0x00, 0x22],
        "P-521" => vec![0x06, 0x05, 0x2b, 0x81, 0x04, 0x00, 0x23],
        _ => return Err(Error::Token(format!("Unsupported curve: {}", crv))),
    };

    let ec_oid = vec![0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];

    let mut algorithm = vec![0x30];
    let algo_content_len = ec_oid.len() + oid.len();
    algorithm.extend(encode_length(algo_content_len));
    algorithm.extend(ec_oid);
    algorithm.extend(oid);

    let mut point = vec![0x04];
    point.extend(x);
    point.extend(y);

    let bit_string_len = point.len() + 1;
    let mut bit_string = vec![0x03];
    bit_string.extend(encode_length(bit_string_len));
    bit_string.push(0x00);
    bit_string.extend(point);

    let total_len = algorithm.len() + bit_string.len();
    let mut result = vec![0x30];
    result.extend(encode_length(total_len));
    result.extend(algorithm);
    result.extend(bit_string);

    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

impl JwkSet {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    pub fn add_key(&mut self, key: Jwk) {
        self.keys.push(key);
    }

    pub fn find_by_kid(&self, kid: &str) -> Option<&Jwk> {
        self.keys.iter().find(|k| k.kid.as_deref() == Some(kid))
    }

    pub fn find_signing_keys(&self) -> Vec<&Jwk> {
        self.keys.iter().filter(|k| k.is_signing_key()).collect()
    }

    pub fn find_by_alg(&self, alg: &str) -> Option<&Jwk> {
        self.keys.iter().find(|k| k.alg.as_deref() == Some(alg))
    }
}

impl Default for JwkSet {
    fn default() -> Self {
        Self::new()
    }
}

struct CachedJwkSet {
    jwks: JwkSet,
    fetched_at: Instant,
}

pub struct JwksClient {
    http: reqwest::Client,
    jwks_uri: String,
    cache: Arc<RwLock<Option<CachedJwkSet>>>,
    cache_duration: Duration,
}

impl JwksClient {
    pub fn new(jwks_uri: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            jwks_uri: jwks_uri.into(),
            cache: Arc::new(RwLock::new(None)),
            cache_duration: Duration::from_secs(3600),
        }
    }

    pub fn with_cache_duration(mut self, duration: Duration) -> Self {
        self.cache_duration = duration;
        self
    }

    pub async fn fetch(&self) -> Result<JwkSet> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.cache_duration {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        let response = self.http.get(&self.jwks_uri).send().await?;

        if !response.status().is_success() {
            return Err(Error::OAuth(format!(
                "Failed to fetch JWKS: {}",
                response.status()
            )));
        }

        let jwks: JwkSet = response.json().await?;

        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedJwkSet {
                jwks: jwks.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(jwks)
    }

    pub async fn get_key(&self, kid: &str) -> Result<Jwk> {
        let jwks = self.fetch().await?;
        jwks.find_by_kid(kid)
            .cloned()
            .ok_or_else(|| Error::Token(format!("Key not found: {}", kid)))
    }

    pub async fn get_signing_key(&self) -> Result<Jwk> {
        let jwks = self.fetch().await?;
        jwks.find_signing_keys()
            .first()
            .cloned()
            .cloned()
            .ok_or_else(|| Error::Token("No signing key found".into()))
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.try_write() {
            *cache = None;
        }
    }
}

#[cfg(feature = "jwt")]
pub async fn verify_with_jwks<T: serde::de::DeserializeOwned>(
    token: &str,
    jwks_client: &JwksClient,
) -> Result<jsonwebtoken::TokenData<T>> {
    use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

    let header = decode_header(token).map_err(|e| Error::Token(e.to_string()))?;

    let kid = header.kid.ok_or_else(|| Error::Token("Token missing kid".into()))?;
    let jwk = jwks_client.get_key(&kid).await?;

    let algorithm = match header.alg {
        Algorithm::RS256 => Algorithm::RS256,
        Algorithm::RS384 => Algorithm::RS384,
        Algorithm::RS512 => Algorithm::RS512,
        Algorithm::ES256 => Algorithm::ES256,
        Algorithm::ES384 => Algorithm::ES384,
        alg => return Err(Error::Token(format!("Unsupported algorithm: {:?}", alg))),
    };

    let pem = jwk.to_pem()?;
    let decoding_key = if jwk.is_rsa() {
        DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(|e| Error::Token(e.to_string()))?
    } else if jwk.is_ec() {
        DecodingKey::from_ec_pem(pem.as_bytes()).map_err(|e| Error::Token(e.to_string()))?
    } else {
        return Err(Error::Token(format!("Unsupported key type: {}", jwk.kty)));
    };

    let validation = Validation::new(algorithm);
    decode(token, &decoding_key, &validation).map_err(|e| Error::Token(e.to_string()))
}
