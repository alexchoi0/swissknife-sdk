use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Result, TokenResponse, User};
use serde::Deserialize;

pub struct CognitoClient {
    region: String,
    user_pool_id: String,
    oauth: OAuth2Client,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct CognitoUser {
    sub: String,
    email: Option<String>,
    email_verified: Option<String>,
    name: Option<String>,
    preferred_username: Option<String>,
    cognito_username: Option<String>,
}

impl CognitoClient {
    pub fn new(
        region: impl Into<String>,
        user_pool_id: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: Option<String>,
        domain: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let region = region.into();
        let user_pool_id = user_pool_id.into();
        let domain = domain.into();

        let auth_url = format!("https://{}.auth.{}.amazoncognito.com/oauth2/authorize", domain, region);
        let token_url = format!("https://{}.auth.{}.amazoncognito.com/oauth2/token", domain, region);
        let userinfo_url = format!("https://{}.auth.{}.amazoncognito.com/oauth2/userInfo", domain, region);

        let mut config = OAuth2Config::new(client_id, auth_url, token_url, redirect_uri)
            .userinfo_url(userinfo_url)
            .scope("openid")
            .scope("email")
            .scope("profile");

        if let Some(secret) = client_secret {
            config = config.client_secret(secret);
        }

        let oauth = OAuth2Client::new(config)?;
        let http = reqwest::Client::new();

        Ok(Self {
            region,
            user_pool_id,
            oauth,
            http,
        })
    }

    pub fn authorization_url(&self) -> (String, PkceChallenge) {
        self.oauth.authorization_url_with_pkce()
    }

    pub fn authorization_url_with_identity_provider(&self, provider: &str) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = crate::oauth2::AuthorizationRequest::new()
            .with_pkce(&pkce)
            .extra_param("identity_provider", provider);
        let url = self.oauth.authorization_url(&request);
        (url, pkce)
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, Some(pkce_verifier)).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let response = self.oauth.get_userinfo(access_token).await?;
        Ok(User {
            provider: Some("cognito".into()),
            ..response
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

    pub fn hosted_ui_url(&self, domain: &str) -> String {
        format!(
            "https://{}.auth.{}.amazoncognito.com/login",
            domain, self.region
        )
    }

    pub fn logout_url(&self, domain: &str, client_id: &str, logout_uri: &str) -> String {
        format!(
            "https://{}.auth.{}.amazoncognito.com/logout?client_id={}&logout_uri={}",
            domain,
            self.region,
            client_id,
            urlencoding::encode(logout_uri)
        )
    }
}

pub struct CognitoIdentityProvider {
    pub name: &'static str,
}

impl CognitoIdentityProvider {
    pub const FACEBOOK: Self = Self { name: "Facebook" };
    pub const GOOGLE: Self = Self { name: "Google" };
    pub const AMAZON: Self = Self { name: "LoginWithAmazon" };
    pub const APPLE: Self = Self { name: "SignInWithApple" };
}
