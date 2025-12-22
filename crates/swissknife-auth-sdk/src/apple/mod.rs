use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::Deserialize;

const AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";
const TOKEN_URL: &str = "https://appleid.apple.com/auth/token";

pub struct AppleAuth {
    oauth: OAuth2Client,
    team_id: String,
    key_id: String,
    private_key: String,
}

#[derive(Debug, Deserialize)]
struct AppleIdToken {
    sub: String,
    email: Option<String>,
    email_verified: Option<String>,
    is_private_email: Option<String>,
}

impl AppleAuth {
    pub fn new(
        client_id: impl Into<String>,
        team_id: impl Into<String>,
        key_id: impl Into<String>,
        private_key: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let config = OAuth2Config::new(client_id, AUTH_URL, TOKEN_URL, redirect_uri)
            .scope("name")
            .scope("email");

        let oauth = OAuth2Client::new(config)?;
        Ok(Self {
            oauth,
            team_id: team_id.into(),
            key_id: key_id.into(),
            private_key: private_key.into(),
        })
    }

    pub fn authorization_url(&self) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let request = crate::oauth2::AuthorizationRequest::new()
            .with_pkce(&pkce)
            .extra_param("response_mode", "form_post");
        let url = self.oauth.authorization_url(&request);
        (url, pkce)
    }

    fn generate_client_secret(&self) -> Result<String> {
        #[cfg(feature = "jwt")]
        {
            use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
            use serde::Serialize;

            #[derive(Serialize)]
            struct Claims {
                iss: String,
                iat: i64,
                exp: i64,
                aud: String,
                sub: String,
            }

            let now = chrono::Utc::now().timestamp();
            let claims = Claims {
                iss: self.team_id.clone(),
                iat: now,
                exp: now + 86400 * 180,
                aud: "https://appleid.apple.com".into(),
                sub: "".into(),
            };

            let mut header = Header::new(Algorithm::ES256);
            header.kid = Some(self.key_id.clone());

            let key = EncodingKey::from_ec_pem(self.private_key.as_bytes())
                .map_err(|e| Error::Token(e.to_string()))?;

            encode(&header, &claims, &key)
                .map_err(|e| Error::Token(e.to_string()))
        }

        #[cfg(not(feature = "jwt"))]
        {
            Err(Error::Config("jwt feature required for Apple auth".into()))
        }
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        let client_secret = self.generate_client_secret()?;

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", ""),
            ("client_id", ""),
            ("client_secret", &client_secret),
            ("code_verifier", pkce_verifier),
        ];

        let response = reqwest::Client::new()
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            return Err(Error::OAuth(text));
        }

        let tokens: TokenResponse = response.json().await?;
        Ok(tokens)
    }

    pub fn parse_id_token(&self, id_token: &str) -> Result<User> {
        #[cfg(feature = "jwt")]
        {
            use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

            let mut validation = Validation::new(Algorithm::RS256);
            validation.validate_exp = false;
            validation.insecure_disable_signature_validation();

            let token_data = decode::<AppleIdToken>(id_token, &DecodingKey::from_secret(&[]), &validation)
                .map_err(|e| Error::Token(e.to_string()))?;

            let claims = token_data.claims;
            Ok(User {
                id: claims.sub,
                email: claims.email,
                email_verified: claims.email_verified.map(|v| v == "true"),
                name: None,
                picture: None,
                provider: Some("apple".into()),
                extra: std::collections::HashMap::new(),
            })
        }

        #[cfg(not(feature = "jwt"))]
        {
            let _ = id_token;
            Err(Error::Config("jwt feature required".into()))
        }
    }

    pub async fn authenticate(&self, code: &str, pkce_verifier: &str) -> Result<AuthSession> {
        let tokens = self.exchange_code(code, pkce_verifier).await?;
        let user = if let Some(id_token) = &tokens.id_token {
            self.parse_id_token(id_token)?
        } else {
            return Err(Error::Token("No ID token in response".into()));
        };
        Ok(AuthSession::new(user, tokens))
    }
}
