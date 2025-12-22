use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

pub struct KeycloakClient {
    base_url: String,
    realm: String,
    oauth: OAuth2Client,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct KeycloakUser {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    preferred_username: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
}

impl KeycloakClient {
    pub fn new(
        base_url: impl Into<String>,
        realm: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: Option<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let base_url = base_url.into();
        let realm = realm.into();

        let auth_url = format!("{}/realms/{}/protocol/openid-connect/auth", base_url, realm);
        let token_url = format!("{}/realms/{}/protocol/openid-connect/token", base_url, realm);
        let userinfo_url = format!("{}/realms/{}/protocol/openid-connect/userinfo", base_url, realm);

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
            base_url,
            realm,
            oauth,
            http,
        })
    }

    pub fn authorization_url(&self) -> (String, PkceChallenge) {
        self.oauth.authorization_url_with_pkce()
    }

    pub fn authorization_url_with_idp(&self, idp_hint: &str) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = crate::oauth2::AuthorizationRequest::new()
            .with_pkce(&pkce)
            .extra_param("kc_idp_hint", idp_hint);
        let url = self.oauth.authorization_url(&request);
        (url, pkce)
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, Some(pkce_verifier)).await
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let url = format!(
            "{}/realms/{}/protocol/openid-connect/userinfo",
            self.base_url, self.realm
        );

        let response = self.http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch Keycloak user".into()));
        }

        let keycloak_user: KeycloakUser = response.json().await?;

        Ok(User {
            id: keycloak_user.sub,
            email: keycloak_user.email,
            email_verified: keycloak_user.email_verified,
            name: keycloak_user.name,
            picture: None,
            provider: Some("keycloak".into()),
            extra: {
                let mut extra = std::collections::HashMap::new();
                if let Some(username) = keycloak_user.preferred_username {
                    extra.insert("username".into(), serde_json::Value::String(username));
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

    pub async fn introspect_token(&self, token: &str, client_id: &str, client_secret: &str) -> Result<bool> {
        let url = format!(
            "{}/realms/{}/protocol/openid-connect/token/introspect",
            self.base_url, self.realm
        );

        let response = self.http
            .post(&url)
            .basic_auth(client_id, Some(client_secret))
            .form(&[("token", token)])
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

    pub fn logout_url(&self, id_token_hint: Option<&str>, post_logout_redirect_uri: Option<&str>) -> String {
        let mut url = format!(
            "{}/realms/{}/protocol/openid-connect/logout",
            self.base_url, self.realm
        );

        let mut params = Vec::new();
        if let Some(token) = id_token_hint {
            params.push(format!("id_token_hint={}", urlencoding::encode(token)));
        }
        if let Some(uri) = post_logout_redirect_uri {
            params.push(format!("post_logout_redirect_uri={}", urlencoding::encode(uri)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        url
    }
}
