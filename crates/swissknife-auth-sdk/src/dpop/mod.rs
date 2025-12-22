use crate::{Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpopProofHeader {
    pub typ: String,
    pub alg: String,
    pub jwk: DpopJwk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpopJwk {
    pub kty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpopProofClaims {
    pub jti: String,
    pub htm: String,
    pub htu: String,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

impl DpopProofClaims {
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            jti: uuid::Uuid::new_v4().to_string(),
            htm: method.into().to_uppercase(),
            htu: url.into(),
            iat: chrono::Utc::now().timestamp(),
            ath: None,
            nonce: None,
        }
    }

    pub fn with_access_token(mut self, access_token: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(access_token.as_bytes());
        let hash = hasher.finalize();
        self.ath = Some(URL_SAFE_NO_PAD.encode(hash));
        self
    }

    pub fn with_nonce(mut self, nonce: impl Into<String>) -> Self {
        self.nonce = Some(nonce.into());
        self
    }
}

#[cfg(feature = "jwt")]
pub struct DpopProofGenerator {
    private_key_pem: String,
    public_jwk: DpopJwk,
    algorithm: jsonwebtoken::Algorithm,
}

#[cfg(feature = "jwt")]
impl DpopProofGenerator {
    pub fn new_es256(private_key_pem: impl Into<String>, public_x: impl Into<String>, public_y: impl Into<String>) -> Self {
        Self {
            private_key_pem: private_key_pem.into(),
            public_jwk: DpopJwk {
                kty: "EC".into(),
                crv: Some("P-256".into()),
                x: Some(public_x.into()),
                y: Some(public_y.into()),
                n: None,
                e: None,
            },
            algorithm: jsonwebtoken::Algorithm::ES256,
        }
    }

    pub fn new_rs256(private_key_pem: impl Into<String>, public_n: impl Into<String>, public_e: impl Into<String>) -> Self {
        Self {
            private_key_pem: private_key_pem.into(),
            public_jwk: DpopJwk {
                kty: "RSA".into(),
                crv: None,
                x: None,
                y: None,
                n: Some(public_n.into()),
                e: Some(public_e.into()),
            },
            algorithm: jsonwebtoken::Algorithm::RS256,
        }
    }

    pub fn generate(&self, claims: &DpopProofClaims) -> Result<String> {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let mut header = Header::new(self.algorithm);
        header.typ = Some("dpop+jwt".into());

        #[derive(Serialize)]
        struct HeaderWithJwk {
            #[serde(flatten)]
            header: Header,
            jwk: DpopJwk,
        }

        let header_json = serde_json::json!({
            "typ": "dpop+jwt",
            "alg": match self.algorithm {
                jsonwebtoken::Algorithm::ES256 => "ES256",
                jsonwebtoken::Algorithm::RS256 => "RS256",
                _ => return Err(Error::Token("Unsupported algorithm".into())),
            },
            "jwk": self.public_jwk
        });

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.to_string());
        let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(claims)?);

        let message = format!("{}.{}", header_b64, claims_b64);

        let key = match self.algorithm {
            jsonwebtoken::Algorithm::ES256 => {
                EncodingKey::from_ec_pem(self.private_key_pem.as_bytes())
                    .map_err(|e| Error::Token(e.to_string()))?
            }
            jsonwebtoken::Algorithm::RS256 => {
                EncodingKey::from_rsa_pem(self.private_key_pem.as_bytes())
                    .map_err(|e| Error::Token(e.to_string()))?
            }
            _ => return Err(Error::Token("Unsupported algorithm".into())),
        };

        let token = encode(&header, claims, &key).map_err(|e| Error::Token(e.to_string()))?;
        Ok(token)
    }

    pub fn generate_for_token_request(&self, method: &str, url: &str, nonce: Option<&str>) -> Result<String> {
        let mut claims = DpopProofClaims::new(method, url);
        if let Some(n) = nonce {
            claims = claims.with_nonce(n);
        }
        self.generate(&claims)
    }

    pub fn generate_for_resource_request(
        &self,
        method: &str,
        url: &str,
        access_token: &str,
        nonce: Option<&str>,
    ) -> Result<String> {
        let mut claims = DpopProofClaims::new(method, url).with_access_token(access_token);
        if let Some(n) = nonce {
            claims = claims.with_nonce(n);
        }
        self.generate(&claims)
    }

    pub fn thumbprint(&self) -> Result<String> {
        let jwk_json = match self.public_jwk.kty.as_str() {
            "EC" => serde_json::json!({
                "crv": self.public_jwk.crv,
                "kty": "EC",
                "x": self.public_jwk.x,
                "y": self.public_jwk.y,
            }),
            "RSA" => serde_json::json!({
                "e": self.public_jwk.e,
                "kty": "RSA",
                "n": self.public_jwk.n,
            }),
            _ => return Err(Error::Token("Unsupported key type".into())),
        };

        let canonical = jwk_json.to_string();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(hasher.finalize()))
    }
}

#[cfg(feature = "jwt")]
pub struct DpopProofVerifier {
    max_age: std::time::Duration,
    allowed_algorithms: Vec<String>,
}

#[cfg(feature = "jwt")]
impl Default for DpopProofVerifier {
    fn default() -> Self {
        Self {
            max_age: std::time::Duration::from_secs(60),
            allowed_algorithms: vec!["ES256".into(), "RS256".into()],
        }
    }
}

#[cfg(feature = "jwt")]
impl DpopProofVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_age(mut self, max_age: std::time::Duration) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn verify(
        &self,
        proof: &str,
        expected_method: &str,
        expected_url: &str,
        expected_access_token: Option<&str>,
        expected_nonce: Option<&str>,
    ) -> Result<DpopProofClaims> {
        use jsonwebtoken::{decode_header, Algorithm};

        let header = decode_header(proof).map_err(|e| Error::Token(e.to_string()))?;

        if header.typ.as_deref() != Some("dpop+jwt") {
            return Err(Error::Token("Invalid DPoP proof type".into()));
        }

        let alg_str = match header.alg {
            Algorithm::ES256 => "ES256",
            Algorithm::RS256 => "RS256",
            _ => return Err(Error::Token("Unsupported algorithm".into())),
        };

        if !self.allowed_algorithms.contains(&alg_str.to_string()) {
            return Err(Error::Token("Algorithm not allowed".into()));
        }

        let parts: Vec<&str> = proof.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::Token("Invalid JWT format".into()));
        }

        let claims_json = URL_SAFE_NO_PAD.decode(parts[1])?;
        let claims: DpopProofClaims = serde_json::from_slice(&claims_json)?;

        if claims.htm.to_uppercase() != expected_method.to_uppercase() {
            return Err(Error::Token("Method mismatch".into()));
        }

        if claims.htu != expected_url {
            return Err(Error::Token("URL mismatch".into()));
        }

        let now = chrono::Utc::now().timestamp();
        let age = now - claims.iat;
        if age < 0 || age > self.max_age.as_secs() as i64 {
            return Err(Error::Token("Proof expired or issued in future".into()));
        }

        if let Some(expected_token) = expected_access_token {
            let mut hasher = Sha256::new();
            hasher.update(expected_token.as_bytes());
            let expected_ath = URL_SAFE_NO_PAD.encode(hasher.finalize());

            if claims.ath.as_deref() != Some(&expected_ath) {
                return Err(Error::Token("Access token hash mismatch".into()));
            }
        }

        if let Some(expected) = expected_nonce {
            if claims.nonce.as_deref() != Some(expected) {
                return Err(Error::Token("Nonce mismatch".into()));
            }
        }

        Ok(claims)
    }
}

pub fn compute_dpop_thumbprint(jwk: &DpopJwk) -> Result<String> {
    let jwk_json = match jwk.kty.as_str() {
        "EC" => serde_json::json!({
            "crv": jwk.crv,
            "kty": "EC",
            "x": jwk.x,
            "y": jwk.y,
        }),
        "RSA" => serde_json::json!({
            "e": jwk.e,
            "kty": "RSA",
            "n": jwk.n,
        }),
        _ => return Err(Error::Token("Unsupported key type".into())),
    };

    let canonical = jwk_json.to_string();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(hasher.finalize()))
}
