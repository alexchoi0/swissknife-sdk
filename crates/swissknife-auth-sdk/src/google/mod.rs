use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

pub struct GoogleAuth {
    oauth: OAuth2Client,
}

#[derive(Debug, Deserialize)]
struct GoogleUser {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    locale: Option<String>,
}

impl GoogleAuth {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>, redirect_uri: impl Into<String>) -> Result<Self> {
        let config = OAuth2Config::new(client_id, AUTH_URL, TOKEN_URL, redirect_uri)
            .client_secret(client_secret)
            .userinfo_url(USERINFO_URL)
            .scope("openid")
            .scope("email")
            .scope("profile");

        let oauth = OAuth2Client::new(config)?;
        Ok(Self { oauth })
    }

    pub fn authorization_url(&self) -> (String, PkceChallenge) {
        self.oauth.authorization_url_with_pkce()
    }

    pub fn authorization_url_with_prompt(&self, prompt: &str) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = crate::oauth2::AuthorizationRequest::new()
            .with_pkce(&pkce)
            .extra_param("prompt", prompt);
        let url = self.oauth.authorization_url(&request);
        (url, pkce)
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, Some(pkce_verifier)).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let response = reqwest::Client::new()
            .get(USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch Google user".into()));
        }

        let google_user: GoogleUser = response.json().await?;
        Ok(User {
            id: google_user.sub,
            email: google_user.email,
            email_verified: google_user.email_verified,
            name: google_user.name,
            picture: google_user.picture,
            provider: Some("google".into()),
            extra: std::collections::HashMap::new(),
        })
    }

    pub async fn authenticate(&self, code: &str, pkce_verifier: &str) -> Result<AuthSession> {
        let tokens = self.exchange_code(code, pkce_verifier).await?;
        let user = self.get_user(&tokens.access_token).await?;
        Ok(AuthSession::new(user, tokens))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        self.oauth.refresh_token(refresh_token).await
    }
}
