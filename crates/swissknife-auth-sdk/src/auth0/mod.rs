use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::{Deserialize, Serialize};

pub struct Auth0Client {
    domain: String,
    oauth: OAuth2Client,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct Auth0User {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    nickname: Option<String>,
    picture: Option<String>,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct PasswordGrant {
    grant_type: String,
    client_id: String,
    client_secret: Option<String>,
    username: String,
    password: String,
    audience: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, Serialize)]
struct PasswordlessStart {
    client_id: String,
    client_secret: Option<String>,
    connection: String,
    email: Option<String>,
    phone_number: Option<String>,
    send: String,
}

#[derive(Debug, Serialize)]
struct PasswordlessVerify {
    grant_type: String,
    client_id: String,
    client_secret: Option<String>,
    username: String,
    otp: String,
    realm: String,
    audience: Option<String>,
    scope: Option<String>,
}

impl Auth0Client {
    pub fn new(
        domain: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let domain = domain.into();
        let auth_url = format!("https://{}/authorize", domain);
        let token_url = format!("https://{}/oauth/token", domain);
        let userinfo_url = format!("https://{}/userinfo", domain);

        let config = OAuth2Config::new(client_id, auth_url, token_url, redirect_uri)
            .client_secret(client_secret)
            .userinfo_url(userinfo_url)
            .scope("openid")
            .scope("email")
            .scope("profile");

        let oauth = OAuth2Client::new(config)?;
        let http = reqwest::Client::new();

        Ok(Self { domain, oauth, http })
    }

    pub fn authorization_url(&self) -> (String, PkceChallenge) {
        self.oauth.authorization_url_with_pkce()
    }

    pub fn authorization_url_with_connection(&self, connection: &str) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = crate::oauth2::AuthorizationRequest::new()
            .with_pkce(&pkce)
            .extra_param("connection", connection);
        let url = self.oauth.authorization_url(&request);
        (url, pkce)
    }

    pub fn authorization_url_with_audience(&self, audience: &str) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = crate::oauth2::AuthorizationRequest::new()
            .with_pkce(&pkce)
            .extra_param("audience", audience);
        let url = self.oauth.authorization_url(&request);
        (url, pkce)
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, Some(pkce_verifier)).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let url = format!("https://{}/userinfo", self.domain);
        let response = self.http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch Auth0 user".into()));
        }

        let auth0_user: Auth0User = response.json().await?;

        Ok(User {
            id: auth0_user.sub,
            email: auth0_user.email,
            email_verified: auth0_user.email_verified,
            name: auth0_user.name.or(auth0_user.nickname),
            picture: auth0_user.picture,
            provider: Some("auth0".into()),
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

    pub async fn start_passwordless_email(&self, email: &str, client_id: &str, client_secret: Option<&str>) -> Result<()> {
        let url = format!("https://{}/passwordless/start", self.domain);

        let body = PasswordlessStart {
            client_id: client_id.to_string(),
            client_secret: client_secret.map(|s| s.to_string()),
            connection: "email".to_string(),
            email: Some(email.to_string()),
            phone_number: None,
            send: "code".to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    pub async fn start_passwordless_sms(&self, phone: &str, client_id: &str, client_secret: Option<&str>) -> Result<()> {
        let url = format!("https://{}/passwordless/start", self.domain);

        let body = PasswordlessStart {
            client_id: client_id.to_string(),
            client_secret: client_secret.map(|s| s.to_string()),
            connection: "sms".to_string(),
            email: None,
            phone_number: Some(phone.to_string()),
            send: "code".to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    pub async fn verify_passwordless(
        &self,
        username: &str,
        code: &str,
        realm: &str,
        client_id: &str,
        client_secret: Option<&str>,
        audience: Option<&str>,
    ) -> Result<TokenResponse> {
        let url = format!("https://{}/oauth/token", self.domain);

        let body = PasswordlessVerify {
            grant_type: "http://auth0.com/oauth/grant-type/passwordless/otp".to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.map(|s| s.to_string()),
            username: username.to_string(),
            otp: code.to_string(),
            realm: realm.to_string(),
            audience: audience.map(|s| s.to_string()),
            scope: Some("openid email profile".to_string()),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let tokens: TokenResponse = response.json().await?;
        Ok(tokens)
    }

    pub async fn logout_url(&self, client_id: &str, return_to: &str) -> String {
        format!(
            "https://{}/v2/logout?client_id={}&returnTo={}",
            self.domain,
            client_id,
            urlencoding::encode(return_to)
        )
    }
}
