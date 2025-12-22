use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub userinfo_url: Option<String>,
}

impl OAuth2Config {
    pub fn new(
        client_id: impl Into<String>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: None,
            auth_url: auth_url.into(),
            token_url: token_url.into(),
            redirect_uri: redirect_uri.into(),
            scopes: Vec::new(),
            userinfo_url: None,
        }
    }

    pub fn client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    pub fn scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn userinfo_url(mut self, url: impl Into<String>) -> Self {
        self.userinfo_url = Some(url.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantType {
    AuthorizationCode,
    ClientCredentials,
    RefreshToken,
    Password,
    DeviceCode,
}

impl GrantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GrantType::AuthorizationCode => "authorization_code",
            GrantType::ClientCredentials => "client_credentials",
            GrantType::RefreshToken => "refresh_token",
            GrantType::Password => "password",
            GrantType::DeviceCode => "urn:ietf:params:oauth:grant-type:device_code",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    Code,
    Token,
    IdToken,
}

impl ResponseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseType::Code => "code",
            ResponseType::Token => "token",
            ResponseType::IdToken => "id_token",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    pub response_type: ResponseType,
    pub state: String,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub extra_params: Vec<(String, String)>,
}

impl Default for AuthorizationRequest {
    fn default() -> Self {
        Self {
            response_type: ResponseType::Code,
            state: uuid::Uuid::new_v4().to_string(),
            nonce: None,
            code_challenge: None,
            code_challenge_method: None,
            extra_params: Vec::new(),
        }
    }
}

impl AuthorizationRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }

    pub fn with_nonce(mut self) -> Self {
        self.nonce = Some(uuid::Uuid::new_v4().to_string());
        self
    }

    pub fn with_pkce(mut self, challenge: &super::PkceChallenge) -> Self {
        self.code_challenge = Some(challenge.challenge.clone());
        self.code_challenge_method = Some("S256".to_string());
        self
    }

    pub fn extra_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_params.push((key.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub refresh_token: Option<String>,
    pub code_verifier: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Error {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: Option<u64>,
}
