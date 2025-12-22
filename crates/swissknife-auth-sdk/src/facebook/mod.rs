use crate::oauth2::{OAuth2Client, OAuth2Config};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

const AUTH_URL: &str = "https://www.facebook.com/v18.0/dialog/oauth";
const TOKEN_URL: &str = "https://graph.facebook.com/v18.0/oauth/access_token";
const GRAPH_URL: &str = "https://graph.facebook.com/v18.0";

pub struct FacebookAuth {
    oauth: OAuth2Client,
}

#[derive(Debug, Deserialize)]
struct FacebookUser {
    id: String,
    name: Option<String>,
    email: Option<String>,
    picture: Option<FacebookPicture>,
}

#[derive(Debug, Deserialize)]
struct FacebookPicture {
    data: FacebookPictureData,
}

#[derive(Debug, Deserialize)]
struct FacebookPictureData {
    url: String,
}

#[derive(Debug, Deserialize)]
struct DebugTokenResponse {
    data: DebugTokenData,
}

#[derive(Debug, Deserialize)]
struct DebugTokenData {
    is_valid: bool,
    user_id: Option<String>,
    expires_at: Option<i64>,
}

impl FacebookAuth {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>, redirect_uri: impl Into<String>) -> Result<Self> {
        let config = OAuth2Config::new(client_id, AUTH_URL, TOKEN_URL, redirect_uri)
            .client_secret(client_secret)
            .scope("email")
            .scope("public_profile");

        let oauth = OAuth2Client::new(config)?;
        Ok(Self { oauth })
    }

    pub fn with_scopes(client_id: impl Into<String>, client_secret: impl Into<String>, redirect_uri: impl Into<String>, scopes: Vec<String>) -> Result<Self> {
        let config = OAuth2Config::new(client_id, AUTH_URL, TOKEN_URL, redirect_uri)
            .client_secret(client_secret)
            .scopes(scopes);

        let oauth = OAuth2Client::new(config)?;
        Ok(Self { oauth })
    }

    pub fn authorization_url(&self) -> (String, String) {
        let request = crate::oauth2::AuthorizationRequest::new();
        let state = request.state.clone();
        let url = self.oauth.authorization_url(&request);
        (url, state)
    }

    pub fn authorization_url_reauthenticate(&self) -> (String, String) {
        let request = crate::oauth2::AuthorizationRequest::new()
            .extra_param("auth_type", "reauthenticate");
        let state = request.state.clone();
        let url = self.oauth.authorization_url(&request);
        (url, state)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, None).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let response = reqwest::Client::new()
            .get(format!("{}/me", GRAPH_URL))
            .query(&[("fields", "id,name,email,picture.type(large)"), ("access_token", access_token)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch Facebook user".into()));
        }

        let fb_user: FacebookUser = response.json().await?;

        Ok(User {
            id: fb_user.id,
            email: fb_user.email,
            email_verified: None,
            name: fb_user.name,
            picture: fb_user.picture.map(|p| p.data.url),
            provider: Some("facebook".into()),
            extra: std::collections::HashMap::new(),
        })
    }

    pub async fn debug_token(&self, access_token: &str, app_token: &str) -> Result<DebugTokenData> {
        let response = reqwest::Client::new()
            .get(format!("{}/debug_token", GRAPH_URL))
            .query(&[("input_token", access_token), ("access_token", app_token)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to debug token".into()));
        }

        let debug: DebugTokenResponse = response.json().await?;
        Ok(debug.data)
    }

    pub async fn authenticate(&self, code: &str) -> Result<AuthSession> {
        let tokens = self.exchange_code(code).await?;
        let user = self.get_user(&tokens.access_token).await?;
        Ok(AuthSession::new(user, tokens))
    }

    pub async fn get_long_lived_token(&self, short_lived_token: &str, app_id: &str, app_secret: &str) -> Result<TokenResponse> {
        let response = reqwest::Client::new()
            .get(format!("{}/oauth/access_token", GRAPH_URL))
            .query(&[
                ("grant_type", "fb_exchange_token"),
                ("client_id", app_id),
                ("client_secret", app_secret),
                ("fb_exchange_token", short_lived_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::OAuth("Failed to exchange for long-lived token".into()));
        }

        let tokens: TokenResponse = response.json().await?;
        Ok(tokens)
    }
}
