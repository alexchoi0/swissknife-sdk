mod error;

pub use error::{Error, Result};

#[cfg(feature = "oauth2")]
pub mod oauth2;

#[cfg(feature = "jwt")]
pub mod jwt;

#[cfg(feature = "paseto")]
pub mod paseto;

#[cfg(feature = "password")]
pub mod password;

#[cfg(feature = "totp")]
pub mod totp;

#[cfg(feature = "webauthn")]
pub mod webauthn;

#[cfg(feature = "magic-link")]
pub mod magic_link;

#[cfg(feature = "api-keys")]
pub mod api_keys;

#[cfg(feature = "auth0")]
pub mod auth0;

#[cfg(feature = "firebase-auth")]
pub mod firebase;

#[cfg(feature = "okta")]
pub mod okta;

#[cfg(feature = "cognito")]
pub mod cognito;

#[cfg(feature = "keycloak")]
pub mod keycloak;

#[cfg(feature = "supabase")]
pub mod supabase;

#[cfg(feature = "clerk")]
pub mod clerk;

#[cfg(feature = "google")]
pub mod google;

#[cfg(feature = "apple")]
pub mod apple;

#[cfg(feature = "github")]
pub mod github;

#[cfg(feature = "microsoft")]
pub mod microsoft;

#[cfg(feature = "facebook")]
pub mod facebook;

#[cfg(feature = "twitter-auth")]
pub mod twitter;

#[cfg(feature = "oidc")]
pub mod oidc;

#[cfg(feature = "jwks")]
pub mod jwks;

#[cfg(feature = "introspection")]
pub mod introspection;

#[cfg(feature = "dpop")]
pub mod dpop;

#[cfg(feature = "saml")]
pub mod saml;

#[cfg(feature = "ldap")]
pub mod ldap;

#[cfg(feature = "scim")]
pub mod scim;

#[cfg(feature = "rippling")]
pub mod rippling;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub provider: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user: User,
    pub tokens: TokenResponse,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AuthSession {
    pub fn new(user: User, tokens: TokenResponse) -> Self {
        let created_at = chrono::Utc::now();
        let expires_at = tokens.expires_in.map(|secs| {
            created_at + chrono::Duration::seconds(secs as i64)
        });
        Self {
            user,
            tokens,
            created_at,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| chrono::Utc::now() > exp)
            .unwrap_or(false)
    }
}
