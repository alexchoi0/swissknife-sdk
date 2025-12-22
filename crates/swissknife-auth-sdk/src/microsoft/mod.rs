use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const GRAPH_URL: &str = "https://graph.microsoft.com/v1.0";

pub struct MicrosoftAuth {
    oauth: OAuth2Client,
    tenant: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftUser {
    id: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "givenName")]
    given_name: Option<String>,
    surname: Option<String>,
    #[serde(rename = "userPrincipalName")]
    user_principal_name: Option<String>,
    mail: Option<String>,
}

impl MicrosoftAuth {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        Self::with_tenant(client_id, client_secret, redirect_uri, "common")
    }

    pub fn with_tenant(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
        tenant: impl Into<String>,
    ) -> Result<Self> {
        let tenant = tenant.into();
        let auth_url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize", tenant);
        let token_url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant);

        let config = OAuth2Config::new(client_id, auth_url, token_url, redirect_uri)
            .client_secret(client_secret)
            .scope("openid")
            .scope("email")
            .scope("profile")
            .scope("User.Read");

        let oauth = OAuth2Client::new(config)?;
        Ok(Self { oauth, tenant })
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
            .get(format!("{}/me", GRAPH_URL))
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch Microsoft user".into()));
        }

        let ms_user: MicrosoftUser = response.json().await?;
        let email = ms_user.mail.or(ms_user.user_principal_name);

        Ok(User {
            id: ms_user.id,
            email,
            email_verified: Some(true),
            name: ms_user.display_name,
            picture: None,
            provider: Some("microsoft".into()),
            extra: std::collections::HashMap::new(),
        })
    }

    pub async fn get_photo(&self, access_token: &str) -> Result<Vec<u8>> {
        let response = reqwest::Client::new()
            .get(format!("{}/me/photo/$value", GRAPH_URL))
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch photo".into()));
        }

        Ok(response.bytes().await?.to_vec())
    }

    pub async fn authenticate(&self, code: &str, pkce_verifier: &str) -> Result<AuthSession> {
        let tokens = self.exchange_code(code, pkce_verifier).await?;
        let user = self.get_user(&tokens.access_token).await?;
        Ok(AuthSession::new(user, tokens))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        self.oauth.refresh_token(refresh_token).await
    }

    pub async fn client_credentials(&self, scope: &str) -> Result<TokenResponse> {
        self.oauth.client_credentials(Some(&[scope])).await
    }
}
