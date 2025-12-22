use crate::oauth2::{OAuth2Client, OAuth2Config};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const API_URL: &str = "https://api.github.com";

pub struct GitHubAuth {
    oauth: OAuth2Client,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    html_url: String,
    bio: Option<String>,
    company: Option<String>,
    location: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    verified: bool,
    primary: bool,
}

impl GitHubAuth {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>, redirect_uri: impl Into<String>) -> Result<Self> {
        let config = OAuth2Config::new(client_id, AUTH_URL, TOKEN_URL, redirect_uri)
            .client_secret(client_secret)
            .scope("user:email")
            .scope("read:user");

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

    pub fn authorization_url_with_login(&self, login: &str) -> (String, String) {
        let request = crate::oauth2::AuthorizationRequest::new()
            .extra_param("login", login);
        let state = request.state.clone();
        let url = self.oauth.authorization_url(&request);
        (url, state)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let response = reqwest::Client::new()
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", ""),
                ("client_secret", ""),
                ("code", code),
            ])
            .send()
            .await?;

        self.oauth.exchange_code(code, None).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/user", API_URL))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "swissknife-auth")
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch GitHub user".into()));
        }

        let github_user: GitHubUser = response.json().await?;

        let email = if github_user.email.is_some() {
            github_user.email
        } else {
            self.get_primary_email(access_token).await.ok()
        };

        Ok(User {
            id: github_user.id.to_string(),
            email,
            email_verified: Some(true),
            name: github_user.name.or(Some(github_user.login)),
            picture: github_user.avatar_url,
            provider: Some("github".into()),
            extra: std::collections::HashMap::new(),
        })
    }

    async fn get_primary_email(&self, access_token: &str) -> Result<String> {
        let response = reqwest::Client::new()
            .get(format!("{}/user/emails", API_URL))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "swissknife-auth")
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch GitHub emails".into()));
        }

        let emails: Vec<GitHubEmail> = response.json().await?;
        emails.into_iter()
            .find(|e| e.primary && e.verified)
            .map(|e| e.email)
            .ok_or_else(|| Error::AuthFailed("No primary verified email".into()))
    }

    pub async fn authenticate(&self, code: &str) -> Result<AuthSession> {
        let tokens = self.exchange_code(code).await?;
        let user = self.get_user(&tokens.access_token).await?;
        Ok(AuthSession::new(user, tokens))
    }
}
