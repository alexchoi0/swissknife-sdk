use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

const AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";
const TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";
const API_URL: &str = "https://api.twitter.com/2";

pub struct TwitterAuth {
    oauth: OAuth2Client,
}

#[derive(Debug, Deserialize)]
struct TwitterResponse {
    data: TwitterUser,
}

#[derive(Debug, Deserialize)]
struct TwitterUser {
    id: String,
    name: String,
    username: String,
    profile_image_url: Option<String>,
    description: Option<String>,
    verified: Option<bool>,
}

impl TwitterAuth {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>, redirect_uri: impl Into<String>) -> Result<Self> {
        let config = OAuth2Config::new(client_id, AUTH_URL, TOKEN_URL, redirect_uri)
            .client_secret(client_secret)
            .scope("tweet.read")
            .scope("users.read")
            .scope("offline.access");

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

    pub fn authorization_url(&self) -> (String, PkceChallenge) {
        self.oauth.authorization_url_with_pkce()
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, Some(pkce_verifier)).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let response = reqwest::Client::new()
            .get(format!("{}/users/me", API_URL))
            .query(&[("user.fields", "id,name,username,profile_image_url,description,verified")])
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch Twitter user".into()));
        }

        let twitter_response: TwitterResponse = response.json().await?;
        let twitter_user = twitter_response.data;

        Ok(User {
            id: twitter_user.id,
            email: None,
            email_verified: None,
            name: Some(twitter_user.name),
            picture: twitter_user.profile_image_url,
            provider: Some("twitter".into()),
            extra: {
                let mut extra = std::collections::HashMap::new();
                extra.insert("username".into(), serde_json::Value::String(twitter_user.username));
                if let Some(verified) = twitter_user.verified {
                    extra.insert("verified".into(), serde_json::Value::Bool(verified));
                }
                extra
            },
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

    pub async fn revoke_token(&self, token: &str, client_id: &str, client_secret: &str) -> Result<()> {
        let response = reqwest::Client::new()
            .post("https://api.twitter.com/2/oauth2/revoke")
            .basic_auth(client_id, Some(client_secret))
            .form(&[("token", token)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::OAuth("Failed to revoke token".into()));
        }

        Ok(())
    }
}
