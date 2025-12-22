use super::{AuthorizationRequest, DeviceAuthResponse, GrantType, OAuth2Config, PkceChallenge};
use crate::{Error, Result, TokenResponse, User};
use reqwest::Client;
use std::collections::HashMap;

pub struct OAuth2Client {
    config: OAuth2Config,
    http: Client,
}

impl OAuth2Client {
    pub fn new(config: OAuth2Config) -> Result<Self> {
        let http = Client::builder()
            .build()
            .map_err(Error::Http)?;
        Ok(Self { config, http })
    }

    pub fn authorization_url(&self, request: &AuthorizationRequest) -> String {
        let mut url = url::Url::parse(&self.config.auth_url).unwrap();
        {
            let mut params = url.query_pairs_mut();
            params.append_pair("client_id", &self.config.client_id);
            params.append_pair("redirect_uri", &self.config.redirect_uri);
            params.append_pair("response_type", request.response_type.as_str());
            params.append_pair("state", &request.state);

            if !self.config.scopes.is_empty() {
                params.append_pair("scope", &self.config.scopes.join(" "));
            }

            if let Some(nonce) = &request.nonce {
                params.append_pair("nonce", nonce);
            }

            if let Some(challenge) = &request.code_challenge {
                params.append_pair("code_challenge", challenge);
                if let Some(method) = &request.code_challenge_method {
                    params.append_pair("code_challenge_method", method);
                }
            }

            for (key, value) in &request.extra_params {
                params.append_pair(key, value);
            }
        }
        url.to_string()
    }

    pub fn authorization_url_with_pkce(&self) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = AuthorizationRequest::new().with_pkce(&pkce);
        let url = self.authorization_url(&request);
        (url, pkce)
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: Option<&str>) -> Result<TokenResponse> {
        let mut params = HashMap::new();
        params.insert("grant_type", GrantType::AuthorizationCode.as_str());
        params.insert("code", code);
        params.insert("redirect_uri", &self.config.redirect_uri);
        params.insert("client_id", &self.config.client_id);

        if let Some(secret) = &self.config.client_secret {
            params.insert("client_secret", secret.as_str());
        }

        let verifier_string;
        if let Some(verifier) = pkce_verifier {
            verifier_string = verifier.to_string();
            params.insert("code_verifier", &verifier_string);
        }

        let response = self.http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: super::OAuth2Error = response.json().await?;
            return Err(Error::OAuth(error.error_description.unwrap_or(error.error)));
        }

        let tokens: TokenResponse = response.json().await?;
        Ok(tokens)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let mut params = HashMap::new();
        params.insert("grant_type", GrantType::RefreshToken.as_str());
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", &self.config.client_id);

        if let Some(secret) = &self.config.client_secret {
            params.insert("client_secret", secret.as_str());
        }

        let response = self.http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: super::OAuth2Error = response.json().await?;
            return Err(Error::OAuth(error.error_description.unwrap_or(error.error)));
        }

        let tokens: TokenResponse = response.json().await?;
        Ok(tokens)
    }

    pub async fn client_credentials(&self, scopes: Option<&[&str]>) -> Result<TokenResponse> {
        let secret = self.config.client_secret.as_ref()
            .ok_or_else(|| Error::Config("client_secret required for client_credentials flow".into()))?;

        let mut params = HashMap::new();
        params.insert("grant_type", GrantType::ClientCredentials.as_str());
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", secret.as_str());

        let scope_string;
        if let Some(scopes) = scopes {
            scope_string = scopes.join(" ");
            params.insert("scope", &scope_string);
        }

        let response = self.http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: super::OAuth2Error = response.json().await?;
            return Err(Error::OAuth(error.error_description.unwrap_or(error.error)));
        }

        let tokens: TokenResponse = response.json().await?;
        Ok(tokens)
    }

    pub async fn password_grant(&self, username: &str, password: &str) -> Result<TokenResponse> {
        let secret = self.config.client_secret.as_ref()
            .ok_or_else(|| Error::Config("client_secret required for password grant".into()))?;

        let mut params = HashMap::new();
        params.insert("grant_type", GrantType::Password.as_str());
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", secret.as_str());
        params.insert("username", username);
        params.insert("password", password);

        if !self.config.scopes.is_empty() {
            let scope_string = self.config.scopes.join(" ");
            let mut params_with_scope = params.clone();
            params_with_scope.insert("scope", &scope_string);

            let response = self.http
                .post(&self.config.token_url)
                .form(&params_with_scope)
                .send()
                .await?;

            if !response.status().is_success() {
                let error: super::OAuth2Error = response.json().await?;
                return Err(Error::OAuth(error.error_description.unwrap_or(error.error)));
            }

            return Ok(response.json().await?);
        }

        let response = self.http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: super::OAuth2Error = response.json().await?;
            return Err(Error::OAuth(error.error_description.unwrap_or(error.error)));
        }

        let tokens: TokenResponse = response.json().await?;
        Ok(tokens)
    }

    pub async fn get_userinfo(&self, access_token: &str) -> Result<User> {
        let userinfo_url = self.config.userinfo_url.as_ref()
            .ok_or_else(|| Error::Config("userinfo_url not configured".into()))?;

        let response = self.http
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to fetch user info".into()));
        }

        let user: User = response.json().await?;
        Ok(user)
    }

    pub async fn device_authorization(&self, device_auth_url: &str) -> Result<DeviceAuthResponse> {
        let mut params = HashMap::new();
        params.insert("client_id", self.config.client_id.as_str());

        if !self.config.scopes.is_empty() {
            let scope_string = self.config.scopes.join(" ");
            let mut params_with_scope = params.clone();
            params_with_scope.insert("scope", &scope_string);

            let response = self.http
                .post(device_auth_url)
                .form(&params_with_scope)
                .send()
                .await?;

            if !response.status().is_success() {
                let error: super::OAuth2Error = response.json().await?;
                return Err(Error::OAuth(error.error_description.unwrap_or(error.error)));
            }

            return Ok(response.json().await?);
        }

        let response = self.http
            .post(device_auth_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: super::OAuth2Error = response.json().await?;
            return Err(Error::OAuth(error.error_description.unwrap_or(error.error)));
        }

        let auth: DeviceAuthResponse = response.json().await?;
        Ok(auth)
    }

    pub async fn poll_device_token(&self, device_code: &str) -> Result<Option<TokenResponse>> {
        let mut params = HashMap::new();
        params.insert("grant_type", GrantType::DeviceCode.as_str());
        params.insert("device_code", device_code);
        params.insert("client_id", &self.config.client_id);

        let response = self.http
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let tokens: TokenResponse = response.json().await?;
            return Ok(Some(tokens));
        }

        let error: super::OAuth2Error = response.json().await?;
        match error.error.as_str() {
            "authorization_pending" | "slow_down" => Ok(None),
            "expired_token" => Err(Error::TokenExpired),
            "access_denied" => Err(Error::AuthFailed("User denied access".into())),
            _ => Err(Error::OAuth(error.error_description.unwrap_or(error.error))),
        }
    }
}
