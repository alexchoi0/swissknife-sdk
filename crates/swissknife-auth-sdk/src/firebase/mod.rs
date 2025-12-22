use crate::{Error, Result, TokenResponse, User};
use serde::{Deserialize, Serialize};

const IDENTITY_TOOLKIT_URL: &str = "https://identitytoolkit.googleapis.com/v1";
const SECURE_TOKEN_URL: &str = "https://securetoken.googleapis.com/v1";

pub struct FirebaseAuth {
    api_key: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignInRequest {
    email: String,
    password: String,
    return_secure_token: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignUpRequest {
    email: String,
    password: String,
    return_secure_token: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SendOobCodeRequest {
    request_type: String,
    email: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VerifyOobCodeRequest {
    oob_code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResetPasswordRequest {
    oob_code: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthResponse {
    id_token: String,
    refresh_token: String,
    expires_in: String,
    local_id: String,
    email: Option<String>,
    display_name: Option<String>,
    photo_url: Option<String>,
    registered: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshTokenResponse {
    access_token: String,
    expires_in: String,
    token_type: String,
    refresh_token: String,
    id_token: String,
    user_id: String,
    project_id: String,
}

#[derive(Debug, Deserialize)]
struct FirebaseError {
    error: FirebaseErrorDetail,
}

#[derive(Debug, Deserialize)]
struct FirebaseErrorDetail {
    code: i32,
    message: String,
}

impl FirebaseAuth {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn sign_in_with_email(&self, email: &str, password: &str) -> Result<(User, TokenResponse)> {
        let url = format!(
            "{}/accounts:signInWithPassword?key={}",
            IDENTITY_TOOLKIT_URL, self.api_key
        );

        let body = SignInRequest {
            email: email.to_string(),
            password: password.to_string(),
            return_secure_token: true,
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        let auth_response: AuthResponse = response.json().await?;
        let (user, tokens) = self.parse_auth_response(auth_response);

        Ok((user, tokens))
    }

    pub async fn sign_up_with_email(&self, email: &str, password: &str) -> Result<(User, TokenResponse)> {
        let url = format!(
            "{}/accounts:signUp?key={}",
            IDENTITY_TOOLKIT_URL, self.api_key
        );

        let body = SignUpRequest {
            email: email.to_string(),
            password: password.to_string(),
            return_secure_token: true,
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        let auth_response: AuthResponse = response.json().await?;
        let (user, tokens) = self.parse_auth_response(auth_response);

        Ok((user, tokens))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let url = format!("{}/token?key={}", SECURE_TOKEN_URL, self.api_key);

        let body = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        let token_response: RefreshTokenResponse = response.json().await?;

        Ok(TokenResponse {
            access_token: token_response.access_token,
            token_type: token_response.token_type,
            expires_in: token_response.expires_in.parse().ok(),
            refresh_token: Some(token_response.refresh_token),
            id_token: Some(token_response.id_token),
            scope: None,
        })
    }

    pub async fn send_password_reset_email(&self, email: &str) -> Result<()> {
        let url = format!(
            "{}/accounts:sendOobCode?key={}",
            IDENTITY_TOOLKIT_URL, self.api_key
        );

        let body = SendOobCodeRequest {
            request_type: "PASSWORD_RESET".to_string(),
            email: email.to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        Ok(())
    }

    pub async fn send_email_verification(&self, id_token: &str) -> Result<()> {
        let url = format!(
            "{}/accounts:sendOobCode?key={}",
            IDENTITY_TOOLKIT_URL, self.api_key
        );

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct VerifyEmailRequest {
            request_type: String,
            id_token: String,
        }

        let body = VerifyEmailRequest {
            request_type: "VERIFY_EMAIL".to_string(),
            id_token: id_token.to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        Ok(())
    }

    pub async fn confirm_password_reset(&self, oob_code: &str, new_password: &str) -> Result<()> {
        let url = format!(
            "{}/accounts:resetPassword?key={}",
            IDENTITY_TOOLKIT_URL, self.api_key
        );

        let body = ResetPasswordRequest {
            oob_code: oob_code.to_string(),
            new_password: new_password.to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        Ok(())
    }

    pub async fn verify_email(&self, oob_code: &str) -> Result<()> {
        let url = format!(
            "{}/accounts:update?key={}",
            IDENTITY_TOOLKIT_URL, self.api_key
        );

        let body = VerifyOobCodeRequest {
            oob_code: oob_code.to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        Ok(())
    }

    pub async fn get_user(&self, id_token: &str) -> Result<User> {
        let url = format!(
            "{}/accounts:lookup?key={}",
            IDENTITY_TOOLKIT_URL, self.api_key
        );

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct LookupRequest {
            id_token: String,
        }

        #[derive(Deserialize)]
        struct LookupResponse {
            users: Vec<FirebaseUser>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct FirebaseUser {
            local_id: String,
            email: Option<String>,
            email_verified: Option<bool>,
            display_name: Option<String>,
            photo_url: Option<String>,
        }

        let body = LookupRequest {
            id_token: id_token.to_string(),
        };

        let response = self.http.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let error: FirebaseError = response.json().await?;
            return Err(Error::AuthFailed(error.error.message));
        }

        let lookup: LookupResponse = response.json().await?;
        let firebase_user = lookup.users.into_iter().next()
            .ok_or_else(|| Error::UserNotFound)?;

        Ok(User {
            id: firebase_user.local_id,
            email: firebase_user.email,
            email_verified: firebase_user.email_verified,
            name: firebase_user.display_name,
            picture: firebase_user.photo_url,
            provider: Some("firebase".into()),
            extra: std::collections::HashMap::new(),
        })
    }

    fn parse_auth_response(&self, response: AuthResponse) -> (User, TokenResponse) {
        let user = User {
            id: response.local_id,
            email: response.email,
            email_verified: None,
            name: response.display_name,
            picture: response.photo_url,
            provider: Some("firebase".into()),
            extra: std::collections::HashMap::new(),
        };

        let tokens = TokenResponse {
            access_token: response.id_token.clone(),
            token_type: "Bearer".to_string(),
            expires_in: response.expires_in.parse().ok(),
            refresh_token: Some(response.refresh_token),
            id_token: Some(response.id_token),
            scope: None,
        };

        (user, tokens)
    }
}
