use crate::{Error, Result};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl Claims {
    pub fn new(subject: impl Into<String>, expires_in_secs: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: subject.into(),
            iat: now,
            exp: now + expires_in_secs,
            nbf: None,
            iss: None,
            aud: None,
            jti: None,
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.iss = Some(issuer.into());
        self
    }

    pub fn audience(mut self, audience: impl Into<String>) -> Self {
        self.aud = Some(audience.into());
        self
    }

    pub fn not_before(mut self, nbf: i64) -> Self {
        self.nbf = Some(nbf);
        self
    }

    pub fn jti(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    pub fn claim(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.extra.insert(key.into(), v);
        }
        self
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.exp
    }
}

pub struct JwtEncoder {
    key: EncodingKey,
    algorithm: Algorithm,
}

impl JwtEncoder {
    pub fn hs256(secret: &[u8]) -> Self {
        Self {
            key: EncodingKey::from_secret(secret),
            algorithm: Algorithm::HS256,
        }
    }

    pub fn hs384(secret: &[u8]) -> Self {
        Self {
            key: EncodingKey::from_secret(secret),
            algorithm: Algorithm::HS384,
        }
    }

    pub fn hs512(secret: &[u8]) -> Self {
        Self {
            key: EncodingKey::from_secret(secret),
            algorithm: Algorithm::HS512,
        }
    }

    pub fn rs256(private_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: EncodingKey::from_rsa_pem(private_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            algorithm: Algorithm::RS256,
        })
    }

    pub fn rs384(private_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: EncodingKey::from_rsa_pem(private_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            algorithm: Algorithm::RS384,
        })
    }

    pub fn rs512(private_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: EncodingKey::from_rsa_pem(private_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            algorithm: Algorithm::RS512,
        })
    }

    pub fn es256(private_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: EncodingKey::from_ec_pem(private_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            algorithm: Algorithm::ES256,
        })
    }

    pub fn es384(private_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: EncodingKey::from_ec_pem(private_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            algorithm: Algorithm::ES384,
        })
    }

    pub fn encode<T: Serialize>(&self, claims: &T) -> Result<String> {
        let header = Header::new(self.algorithm);
        encode(&header, claims, &self.key).map_err(|e| Error::Token(e.to_string()))
    }

    pub fn encode_with_kid<T: Serialize>(&self, claims: &T, kid: &str) -> Result<String> {
        let mut header = Header::new(self.algorithm);
        header.kid = Some(kid.to_string());
        encode(&header, claims, &self.key).map_err(|e| Error::Token(e.to_string()))
    }
}

pub struct JwtDecoder {
    key: DecodingKey,
    validation: Validation,
}

impl JwtDecoder {
    pub fn hs256(secret: &[u8]) -> Self {
        Self {
            key: DecodingKey::from_secret(secret),
            validation: Validation::new(Algorithm::HS256),
        }
    }

    pub fn hs384(secret: &[u8]) -> Self {
        Self {
            key: DecodingKey::from_secret(secret),
            validation: Validation::new(Algorithm::HS384),
        }
    }

    pub fn hs512(secret: &[u8]) -> Self {
        Self {
            key: DecodingKey::from_secret(secret),
            validation: Validation::new(Algorithm::HS512),
        }
    }

    pub fn rs256(public_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: DecodingKey::from_rsa_pem(public_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            validation: Validation::new(Algorithm::RS256),
        })
    }

    pub fn rs384(public_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: DecodingKey::from_rsa_pem(public_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            validation: Validation::new(Algorithm::RS384),
        })
    }

    pub fn rs512(public_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: DecodingKey::from_rsa_pem(public_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            validation: Validation::new(Algorithm::RS512),
        })
    }

    pub fn es256(public_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: DecodingKey::from_ec_pem(public_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            validation: Validation::new(Algorithm::ES256),
        })
    }

    pub fn es384(public_key_pem: &[u8]) -> Result<Self> {
        Ok(Self {
            key: DecodingKey::from_ec_pem(public_key_pem)
                .map_err(|e| Error::Token(e.to_string()))?,
            validation: Validation::new(Algorithm::ES384),
        })
    }

    pub fn require_issuer(mut self, issuer: &str) -> Self {
        self.validation.set_issuer(&[issuer]);
        self
    }

    pub fn require_audience(mut self, audience: &str) -> Self {
        let mut aud = HashSet::new();
        aud.insert(audience.to_string());
        self.validation.aud = Some(aud);
        self
    }

    pub fn leeway(mut self, leeway_secs: u64) -> Self {
        self.validation.leeway = leeway_secs;
        self
    }

    pub fn skip_expiration(mut self) -> Self {
        self.validation.validate_exp = false;
        self
    }

    pub fn decode<T: DeserializeOwned>(&self, token: &str) -> Result<TokenData<T>> {
        decode(token, &self.key, &self.validation).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Error::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => {
                Error::InvalidToken("Invalid token format".into())
            }
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                Error::InvalidToken("Invalid signature".into())
            }
            _ => Error::Token(e.to_string()),
        })
    }

    pub fn decode_claims(&self, token: &str) -> Result<TokenData<Claims>> {
        self.decode(token)
    }
}

pub fn decode_header(token: &str) -> Result<Header> {
    jsonwebtoken::decode_header(token).map_err(|e| Error::Token(e.to_string()))
}

pub fn dangerous_insecure_decode<T: DeserializeOwned>(token: &str) -> Result<TokenData<T>> {
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    decode(token, &DecodingKey::from_secret(&[]), &validation)
        .map_err(|e| Error::Token(e.to_string()))
}
