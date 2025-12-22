use crate::{Error, Result, User};
use serde::{Deserialize, Serialize};

const CLERK_API_URL: &str = "https://api.clerk.com/v1";

pub struct ClerkClient {
    secret_key: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct ClerkUser {
    id: String,
    primary_email_address_id: Option<String>,
    primary_phone_number_id: Option<String>,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    image_url: Option<String>,
    email_addresses: Vec<ClerkEmailAddress>,
    phone_numbers: Vec<ClerkPhoneNumber>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
struct ClerkEmailAddress {
    id: String,
    email_address: String,
    verification: Option<ClerkVerification>,
}

#[derive(Debug, Deserialize)]
struct ClerkPhoneNumber {
    id: String,
    phone_number: String,
    verification: Option<ClerkVerification>,
}

#[derive(Debug, Deserialize)]
struct ClerkVerification {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ClerkSession {
    id: String,
    user_id: String,
    status: String,
    expire_at: i64,
    last_active_at: i64,
}

#[derive(Debug, Deserialize)]
struct ClerkSessionToken {
    jwt: String,
}

impl ClerkClient {
    pub fn new(secret_key: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn get_user(&self, user_id: &str) -> Result<User> {
        let url = format!("{}/users/{}", CLERK_API_URL, user_id);

        let response = self.http
            .get(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let clerk_user: ClerkUser = response.json().await?;
        Ok(self.parse_user(clerk_user))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let url = format!("{}/users?email_address={}", CLERK_API_URL, urlencoding::encode(email));

        let response = self.http
            .get(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let users: Vec<ClerkUser> = response.json().await?;
        Ok(users.into_iter().next().map(|u| self.parse_user(u)))
    }

    pub async fn list_users(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<User>> {
        let mut url = format!("{}/users", CLERK_API_URL);
        let mut params = Vec::new();

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self.http
            .get(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let users: Vec<ClerkUser> = response.json().await?;
        Ok(users.into_iter().map(|u| self.parse_user(u)).collect())
    }

    pub async fn create_user(
        &self,
        email: &str,
        password: Option<&str>,
        first_name: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<User> {
        let url = format!("{}/users", CLERK_API_URL);

        #[derive(Serialize)]
        struct CreateUserRequest {
            email_address: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            password: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            first_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            last_name: Option<String>,
        }

        let body = CreateUserRequest {
            email_address: vec![email.to_string()],
            password: password.map(|p| p.to_string()),
            first_name: first_name.map(|n| n.to_string()),
            last_name: last_name.map(|n| n.to_string()),
        };

        let response = self.http
            .post(&url)
            .bearer_auth(&self.secret_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let clerk_user: ClerkUser = response.json().await?;
        Ok(self.parse_user(clerk_user))
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let url = format!("{}/users/{}", CLERK_API_URL, user_id);

        let response = self.http
            .delete(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    pub async fn verify_session(&self, session_id: &str) -> Result<ClerkSession> {
        let url = format!("{}/sessions/{}", CLERK_API_URL, session_id);

        let response = self.http
            .get(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let session: ClerkSession = response.json().await?;

        if session.status != "active" {
            return Err(Error::InvalidToken("Session not active".into()));
        }

        Ok(session)
    }

    pub async fn revoke_session(&self, session_id: &str) -> Result<()> {
        let url = format!("{}/sessions/{}/revoke", CLERK_API_URL, session_id);

        let response = self.http
            .post(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        Ok(())
    }

    pub async fn get_session_token(&self, session_id: &str, template: Option<&str>) -> Result<String> {
        let mut url = format!("{}/sessions/{}/tokens", CLERK_API_URL, session_id);
        if let Some(t) = template {
            url = format!("{}/{}", url, t);
        }

        let response = self.http
            .post(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::AuthFailed(text));
        }

        let token: ClerkSessionToken = response.json().await?;
        Ok(token.jwt)
    }

    pub fn verify_token(&self, token: &str) -> Result<crate::jwt::Claims> {
        #[cfg(feature = "jwt")]
        {
            let claims = crate::jwt::dangerous_insecure_decode::<crate::jwt::Claims>(token)?;
            Ok(claims.claims)
        }

        #[cfg(not(feature = "jwt"))]
        {
            let _ = token;
            Err(Error::Config("jwt feature required".into()))
        }
    }

    fn parse_user(&self, clerk_user: ClerkUser) -> User {
        let primary_email = clerk_user.primary_email_address_id
            .and_then(|id| {
                clerk_user.email_addresses.iter()
                    .find(|e| e.id == id)
                    .map(|e| e.email_address.clone())
            })
            .or_else(|| {
                clerk_user.email_addresses.first()
                    .map(|e| e.email_address.clone())
            });

        let email_verified = clerk_user.email_addresses.iter()
            .any(|e| e.verification.as_ref().map(|v| v.status == "verified").unwrap_or(false));

        let name = match (clerk_user.first_name, clerk_user.last_name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first),
            (None, Some(last)) => Some(last),
            (None, None) => clerk_user.username,
        };

        User {
            id: clerk_user.id,
            email: primary_email,
            email_verified: Some(email_verified),
            name,
            picture: clerk_user.image_url,
            provider: Some("clerk".into()),
            extra: std::collections::HashMap::new(),
        }
    }
}

pub struct ClerkFrontend {
    publishable_key: String,
}

impl ClerkFrontend {
    pub fn new(publishable_key: impl Into<String>) -> Self {
        Self {
            publishable_key: publishable_key.into(),
        }
    }

    pub fn sign_in_url(&self) -> String {
        format!("https://accounts.clerk.dev/sign-in?publishable_key={}", self.publishable_key)
    }

    pub fn sign_up_url(&self) -> String {
        format!("https://accounts.clerk.dev/sign-up?publishable_key={}", self.publishable_key)
    }

    pub fn user_profile_url(&self) -> String {
        format!("https://accounts.clerk.dev/user?publishable_key={}", self.publishable_key)
    }
}
