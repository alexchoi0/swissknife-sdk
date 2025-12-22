use crate::{Error, Result, TokenResponse, User};
use serde::{Deserialize, Serialize};

pub struct SupabaseAuth {
    url: String,
    anon_key: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct SignUpRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct SignInRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct MagicLinkRequest {
    email: String,
}

#[derive(Debug, Serialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Debug, Serialize)]
struct OAuthRequest {
    provider: String,
    redirect_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    expires_at: Option<u64>,
    refresh_token: String,
    user: SupabaseUser,
}

#[derive(Debug, Deserialize)]
struct SupabaseUser {
    id: String,
    email: Option<String>,
    email_confirmed_at: Option<String>,
    phone: Option<String>,
    created_at: String,
    updated_at: String,
    user_metadata: Option<serde_json::Value>,
    app_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OAuthUrlResponse {
    url: String,
}

impl SupabaseAuth {
    pub fn new(project_url: impl Into<String>, anon_key: impl Into<String>) -> Self {
        Self {
            url: project_url.into(),
            anon_key: anon_key.into(),
            http: reqwest::Client::new(),
        }
    }

    fn auth_url(&self) -> String {
        format!("{}/auth/v1", self.url)
    }

    pub async fn sign_up(&self, email: &str, password: &str) -> Result<(User, TokenResponse)> {
        let url = format!("{}/signup", self.auth_url());

        let body = SignUpRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let auth_response: AuthResponse = response.json().await?;
        let (user, tokens) = self.parse_auth_response(auth_response);

        Ok((user, tokens))
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<(User, TokenResponse)> {
        let url = format!("{}/token?grant_type=password", self.auth_url());

        let body = SignInRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let auth_response: AuthResponse = response.json().await?;
        let (user, tokens) = self.parse_auth_response(auth_response);

        Ok((user, tokens))
    }

    pub async fn sign_in_with_magic_link(&self, email: &str, redirect_to: Option<&str>) -> Result<()> {
        let mut url = format!("{}/magiclink", self.auth_url());
        if let Some(redirect) = redirect_to {
            url = format!("{}?redirect_to={}", url, urlencoding::encode(redirect));
        }

        let body = MagicLinkRequest {
            email: email.to_string(),
        };

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    pub async fn sign_in_with_otp(&self, email: &str) -> Result<()> {
        let url = format!("{}/otp", self.auth_url());

        let body = MagicLinkRequest {
            email: email.to_string(),
        };

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    pub async fn verify_otp(&self, email: &str, token: &str) -> Result<(User, TokenResponse)> {
        let url = format!("{}/verify", self.auth_url());

        #[derive(Serialize)]
        struct VerifyRequest {
            email: String,
            token: String,
            #[serde(rename = "type")]
            verify_type: String,
        }

        let body = VerifyRequest {
            email: email.to_string(),
            token: token.to_string(),
            verify_type: "email".to_string(),
        };

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let auth_response: AuthResponse = response.json().await?;
        let (user, tokens) = self.parse_auth_response(auth_response);

        Ok((user, tokens))
    }

    pub fn get_oauth_url(&self, provider: &str, redirect_to: Option<&str>) -> String {
        let mut url = format!("{}/authorize?provider={}", self.auth_url(), provider);
        if let Some(redirect) = redirect_to {
            url = format!("{}&redirect_to={}", url, urlencoding::encode(redirect));
        }
        url
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let url = format!("{}/token?grant_type=refresh_token", self.auth_url());

        let body = RefreshRequest {
            refresh_token: refresh_token.to_string(),
        };

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let auth_response: AuthResponse = response.json().await?;
        let (_, tokens) = self.parse_auth_response(auth_response);

        Ok(tokens)
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User> {
        let url = format!("{}/user", self.auth_url());

        let response = self.http
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let supabase_user: SupabaseUser = response.json().await?;

        Ok(User {
            id: supabase_user.id,
            email: supabase_user.email,
            email_verified: supabase_user.email_confirmed_at.map(|_| true),
            name: None,
            picture: None,
            provider: Some("supabase".into()),
            extra: std::collections::HashMap::new(),
        })
    }

    pub async fn sign_out(&self, access_token: &str) -> Result<()> {
        let url = format!("{}/logout", self.auth_url());

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    pub async fn reset_password(&self, email: &str, redirect_to: Option<&str>) -> Result<()> {
        let mut url = format!("{}/recover", self.auth_url());
        if let Some(redirect) = redirect_to {
            url = format!("{}?redirect_to={}", url, urlencoding::encode(redirect));
        }

        let body = MagicLinkRequest {
            email: email.to_string(),
        };

        let response = self.http
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    fn parse_auth_response(&self, response: AuthResponse) -> (User, TokenResponse) {
        let user = User {
            id: response.user.id,
            email: response.user.email,
            email_verified: response.user.email_confirmed_at.map(|_| true),
            name: None,
            picture: None,
            provider: Some("supabase".into()),
            extra: std::collections::HashMap::new(),
        };

        let tokens = TokenResponse {
            access_token: response.access_token,
            token_type: response.token_type,
            expires_in: Some(response.expires_in),
            refresh_token: Some(response.refresh_token),
            id_token: None,
            scope: None,
        };

        (user, tokens)
    }
}

pub struct SupabaseOAuthProvider;

impl SupabaseOAuthProvider {
    pub const GOOGLE: &'static str = "google";
    pub const FACEBOOK: &'static str = "facebook";
    pub const GITHUB: &'static str = "github";
    pub const GITLAB: &'static str = "gitlab";
    pub const BITBUCKET: &'static str = "bitbucket";
    pub const AZURE: &'static str = "azure";
    pub const APPLE: &'static str = "apple";
    pub const DISCORD: &'static str = "discord";
    pub const TWITTER: &'static str = "twitter";
    pub const TWITCH: &'static str = "twitch";
    pub const SPOTIFY: &'static str = "spotify";
    pub const SLACK: &'static str = "slack";
    pub const LINKEDIN: &'static str = "linkedin";
    pub const NOTION: &'static str = "notion";
    pub const ZOOM: &'static str = "zoom";
}
