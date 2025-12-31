use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "auth")]
use swissknife_auth_sdk as auth;

#[derive(Clone)]
pub struct AuthTools {
    #[cfg(feature = "jwt")]
    pub jwt: Option<auth::jwt::JwtClient>,
    #[cfg(feature = "oauth2")]
    pub oauth2: Option<auth::oauth2::OAuth2Client>,
}

impl AuthTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "jwt")]
            jwt: None,
            #[cfg(feature = "oauth2")]
            oauth2: None,
        }
    }

    #[cfg(feature = "jwt")]
    pub fn with_jwt(mut self, client: auth::jwt::JwtClient) -> Self {
        self.jwt = Some(client);
        self
    }

    #[cfg(feature = "oauth2")]
    pub fn with_oauth2(mut self, client: auth::oauth2::OAuth2Client) -> Self {
        self.oauth2 = Some(client);
        self
    }
}

impl Default for AuthTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JwtCreateTokenRequest {
    pub subject: String,
    #[serde(default)]
    pub claims: Option<serde_json::Value>,
    #[serde(default)]
    pub expires_in_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JwtVerifyTokenRequest {
    pub token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JwtDecodeTokenRequest {
    pub token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OAuth2GetAuthorizationUrlRequest {
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OAuth2ExchangeCodeRequest {
    pub code: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OAuth2RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OAuth2ValidateTokenRequest {
    pub access_token: String,
}

#[tool_router]
impl AuthTools {
    #[cfg(feature = "jwt")]
    #[rmcp::tool(description = "Create a new JWT token")]
    pub async fn jwt_create_token(
        &self,
        #[rmcp::tool(aggr)] req: JwtCreateTokenRequest,
    ) -> Result<String, String> {
        let client = self.jwt.as_ref()
            .ok_or_else(|| "JWT client not configured".to_string())?;

        let token = client.create_token(
            &req.subject,
            req.claims,
            req.expires_in_seconds,
        ).map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "token": token
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "jwt")]
    #[rmcp::tool(description = "Verify a JWT token")]
    pub async fn jwt_verify_token(
        &self,
        #[rmcp::tool(aggr)] req: JwtVerifyTokenRequest,
    ) -> Result<String, String> {
        let client = self.jwt.as_ref()
            .ok_or_else(|| "JWT client not configured".to_string())?;

        let valid = client.verify_token(&req.token)
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "valid": valid
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "jwt")]
    #[rmcp::tool(description = "Decode a JWT token without verification")]
    pub async fn jwt_decode_token(
        &self,
        #[rmcp::tool(aggr)] req: JwtDecodeTokenRequest,
    ) -> Result<String, String> {
        let client = self.jwt.as_ref()
            .ok_or_else(|| "JWT client not configured".to_string())?;

        let claims = client.decode_token(&req.token)
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&claims).map_err(|e| e.to_string())
    }

    #[cfg(feature = "oauth2")]
    #[rmcp::tool(description = "Get OAuth2 authorization URL")]
    pub async fn oauth2_get_authorization_url(
        &self,
        #[rmcp::tool(aggr)] req: OAuth2GetAuthorizationUrlRequest,
    ) -> Result<String, String> {
        let client = self.oauth2.as_ref()
            .ok_or_else(|| "OAuth2 client not configured".to_string())?;

        let url = client.get_authorization_url(
            req.scopes.as_deref(),
            req.state.as_deref(),
        ).map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "authorization_url": url
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "oauth2")]
    #[rmcp::tool(description = "Exchange authorization code for tokens")]
    pub async fn oauth2_exchange_code(
        &self,
        #[rmcp::tool(aggr)] req: OAuth2ExchangeCodeRequest,
    ) -> Result<String, String> {
        let client = self.oauth2.as_ref()
            .ok_or_else(|| "OAuth2 client not configured".to_string())?;

        let tokens = client.exchange_code(&req.code).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&tokens).map_err(|e| e.to_string())
    }

    #[cfg(feature = "oauth2")]
    #[rmcp::tool(description = "Refresh an OAuth2 access token")]
    pub async fn oauth2_refresh_token(
        &self,
        #[rmcp::tool(aggr)] req: OAuth2RefreshTokenRequest,
    ) -> Result<String, String> {
        let client = self.oauth2.as_ref()
            .ok_or_else(|| "OAuth2 client not configured".to_string())?;

        let tokens = client.refresh_token(&req.refresh_token).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&tokens).map_err(|e| e.to_string())
    }

    #[cfg(feature = "oauth2")]
    #[rmcp::tool(description = "Validate an OAuth2 access token")]
    pub async fn oauth2_validate_token(
        &self,
        #[rmcp::tool(aggr)] req: OAuth2ValidateTokenRequest,
    ) -> Result<String, String> {
        let client = self.oauth2.as_ref()
            .ok_or_else(|| "OAuth2 client not configured".to_string())?;

        let info = client.validate_token(&req.access_token).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&info).map_err(|e| e.to_string())
    }
}
