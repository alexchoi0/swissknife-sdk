#![cfg(feature = "oauth2")]

use swissknife_auth_sdk::oauth2::{
    AuthorizationRequest, OAuth2Client, OAuth2Config, PkceChallenge, ResponseType,
};

fn create_test_config() -> OAuth2Config {
    OAuth2Config::new(
        "test_client_id",
        "https://auth.example.com/authorize",
        "https://auth.example.com/token",
        "https://app.example.com/callback",
    )
}

#[test]
fn authorization_url_contains_client_id() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();
    let request = AuthorizationRequest::new();

    let url = client.authorization_url(&request);

    assert!(url.contains("client_id=test_client_id"));
}

#[test]
fn authorization_url_contains_redirect_uri() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();
    let request = AuthorizationRequest::new();

    let url = client.authorization_url(&request);

    assert!(url.contains("redirect_uri="));
    assert!(url.contains("app.example.com"));
}

#[test]
fn authorization_url_contains_response_type() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();
    let request = AuthorizationRequest::new();

    let url = client.authorization_url(&request);

    assert!(url.contains("response_type=code"));
}

#[test]
fn authorization_url_contains_state() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();
    let request = AuthorizationRequest::new().with_state("my-state");

    let url = client.authorization_url(&request);

    assert!(url.contains("state=my-state"));
}

#[test]
fn authorization_url_with_scopes() {
    let config = create_test_config()
        .scope("openid")
        .scope("profile")
        .scope("email");
    let client = OAuth2Client::new(config).unwrap();
    let request = AuthorizationRequest::new();

    let url = client.authorization_url(&request);

    assert!(url.contains("scope="));
}

#[test]
fn authorization_url_with_nonce() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();
    let request = AuthorizationRequest::new().with_nonce();

    let url = client.authorization_url(&request);

    assert!(url.contains("nonce="));
}

#[test]
fn authorization_url_with_pkce() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();
    let pkce = PkceChallenge::new();
    let request = AuthorizationRequest::new().with_pkce(&pkce);

    let url = client.authorization_url(&request);

    assert!(url.contains("code_challenge="));
    assert!(url.contains("code_challenge_method=S256"));
}

#[test]
fn authorization_url_with_extra_params() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();
    let request = AuthorizationRequest::new()
        .extra_param("prompt", "consent")
        .extra_param("login_hint", "user@example.com");

    let url = client.authorization_url(&request);

    assert!(url.contains("prompt=consent"));
    assert!(url.contains("login_hint="));
}

#[test]
fn authorization_url_with_pkce_shorthand() {
    let config = create_test_config();
    let client = OAuth2Client::new(config).unwrap();

    let (url, pkce) = client.authorization_url_with_pkce();

    assert!(url.contains("code_challenge="));
    assert!(!pkce.verifier().is_empty());
}

#[test]
fn pkce_challenge_generation() {
    let pkce = PkceChallenge::new();

    assert!(!pkce.challenge.is_empty());
    assert!(!pkce.verifier().is_empty());
}

#[test]
fn pkce_challenge_is_base64url_safe() {
    let pkce = PkceChallenge::new();

    assert!(!pkce.challenge.contains('+'));
    assert!(!pkce.challenge.contains('/'));
    assert!(!pkce.challenge.contains('='));
}

#[test]
fn pkce_verifier_is_base64url_safe() {
    let pkce = PkceChallenge::new();

    assert!(!pkce.verifier().contains('+'));
    assert!(!pkce.verifier().contains('/'));
    assert!(!pkce.verifier().contains('='));
}

#[test]
fn pkce_challenges_are_unique() {
    let pkce1 = PkceChallenge::new();
    let pkce2 = PkceChallenge::new();

    assert_ne!(pkce1.challenge, pkce2.challenge);
    assert_ne!(pkce1.verifier(), pkce2.verifier());
}

#[test]
fn pkce_challenge_derives_from_verifier() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use sha2::{Digest, Sha256};

    let pkce = PkceChallenge::new();
    let mut hasher = Sha256::new();
    hasher.update(pkce.verifier().as_bytes());
    let expected = URL_SAFE_NO_PAD.encode(hasher.finalize());

    assert_eq!(pkce.challenge, expected);
}

#[test]
fn oauth2_config_builder_pattern() {
    let config = OAuth2Config::new(
        "client_id",
        "https://auth.example.com/authorize",
        "https://auth.example.com/token",
        "https://app.example.com/callback",
    )
    .client_secret("secret")
    .scope("openid")
    .scope("profile")
    .userinfo_url("https://auth.example.com/userinfo");

    assert_eq!(config.client_id, "client_id");
    assert_eq!(config.client_secret, Some("secret".to_string()));
    assert_eq!(config.scopes, vec!["openid", "profile"]);
    assert_eq!(
        config.userinfo_url,
        Some("https://auth.example.com/userinfo".to_string())
    );
}

#[test]
fn oauth2_config_scopes_method() {
    let config = OAuth2Config::new(
        "client_id",
        "https://auth.example.com/authorize",
        "https://auth.example.com/token",
        "https://app.example.com/callback",
    )
    .scopes(vec!["openid".to_string(), "profile".to_string()]);

    assert_eq!(config.scopes, vec!["openid", "profile"]);
}

#[test]
fn response_type_code() {
    let request = AuthorizationRequest::new();

    assert_eq!(request.response_type, ResponseType::Code);
    assert_eq!(request.response_type.as_str(), "code");
}

#[test]
fn response_type_variants() {
    assert_eq!(ResponseType::Code.as_str(), "code");
    assert_eq!(ResponseType::Token.as_str(), "token");
    assert_eq!(ResponseType::IdToken.as_str(), "id_token");
}

#[test]
fn authorization_request_generates_uuid_state() {
    let request = AuthorizationRequest::new();

    assert!(!request.state.is_empty());
    assert!(uuid::Uuid::parse_str(&request.state).is_ok());
}

#[test]
fn pkce_default_implementation() {
    let pkce1 = PkceChallenge::default();
    let pkce2 = PkceChallenge::new();

    assert!(!pkce1.challenge.is_empty());
    assert!(!pkce2.challenge.is_empty());
}
