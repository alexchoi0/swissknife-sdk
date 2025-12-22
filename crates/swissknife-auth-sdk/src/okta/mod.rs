use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

pub struct OktaClient {
    domain: String,
    oauth: OAuth2Client,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct OktaUser {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    preferred_username: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    locale: Option<String>,
    zoneinfo: Option<String>,
}

impl OktaClient {
    pub fn new(
        domain: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let domain = domain.into();
        let auth_url = format!("https://{}/oauth2/default/v1/authorize", domain);
        let token_url = format!("https://{}/oauth2/default/v1/token", domain);
        let userinfo_url = format!("https://{}/oauth2/default/v1/userinfo", domain);

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

    pub fn with_authorization_server(
        domain: impl Into<String>,
        auth_server_id: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let domain = domain.into();
        let auth_server = auth_server_id.into();
        let auth_url = format!("https://{}/oauth2/{}/v1/authorize", domain, auth_server);
        let token_url = format!("https://{}/oauth2/{}/v1/token", domain, auth_server);
        let userinfo_url = format!("https://{}/oauth2/{}/v1/userinfo", domain, auth_server);

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

    pub fn authorization_url_with_idp(&self, idp: &str) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = crate::oauth2::AuthorizationRequest::new()
            .with_pkce(&pkce)
            .extra_param("idp", idp);
        let url = self.oauth.authorization_url(&request);
        (url, pkce)
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, Some(pkce_verifier)).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let url = format!("https://{}/oauth2/default/v1/userinfo", self.domain);
        let response = self.http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch Okta user".into()));
        }

        let okta_user: OktaUser = response.json().await?;

        Ok(User {
            id: okta_user.sub,
            email: okta_user.email,
            email_verified: okta_user.email_verified,
            name: okta_user.name,
            picture: None,
            provider: Some("okta".into()),
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

    pub async fn introspect_token(&self, token: &str, client_id: &str, client_secret: &str) -> Result<bool> {
        let url = format!("https://{}/oauth2/default/v1/introspect", self.domain);

        let response = self.http
            .post(&url)
            .basic_auth(client_id, Some(client_secret))
            .form(&[("token", token), ("token_type_hint", "access_token")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to introspect token".into()));
        }

        #[derive(Deserialize)]
        struct IntrospectResponse {
            active: bool,
        }

        let result: IntrospectResponse = response.json().await?;
        Ok(result.active)
    }

    pub async fn revoke_token(&self, token: &str, client_id: &str, client_secret: &str) -> Result<()> {
        let url = format!("https://{}/oauth2/default/v1/revoke", self.domain);

        let response = self.http
            .post(&url)
            .basic_auth(client_id, Some(client_secret))
            .form(&[("token", token), ("token_type_hint", "access_token")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to revoke token".into()));
        }

        Ok(())
    }
}
