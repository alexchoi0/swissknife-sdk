#![cfg(feature = "jwt")]

use swissknife_auth_sdk::jwt::{
    dangerous_insecure_decode, decode_header, Claims, JwtDecoder, JwtEncoder,
};

const SECRET: &[u8] = b"super_secret_key_for_testing_purposes_only";

#[test]
fn encode_and_decode_with_hs256() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET);

    let claims = Claims::new("user123", 3600);
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn encode_and_decode_with_hs384() {
    let encoder = JwtEncoder::hs384(SECRET);
    let decoder = JwtDecoder::hs384(SECRET);

    let claims = Claims::new("user123", 3600);
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn encode_and_decode_with_hs512() {
    let encoder = JwtEncoder::hs512(SECRET);
    let decoder = JwtDecoder::hs512(SECRET);

    let claims = Claims::new("user123", 3600);
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn decode_fails_with_wrong_secret() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(b"wrong_secret");

    let claims = Claims::new("user123", 3600);
    let token = encoder.encode(&claims).unwrap();
    let result = decoder.decode_claims(&token);

    assert!(result.is_err());
}

#[test]
fn claims_with_issuer() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET);

    let claims = Claims::new("user123", 3600).issuer("my-app");
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.iss, Some("my-app".to_string()));
}

#[test]
fn claims_with_audience() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET).require_audience("api.example.com");

    let claims = Claims::new("user123", 3600).audience("api.example.com");
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.aud, Some("api.example.com".to_string()));
}

#[test]
fn claims_with_jti() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET);

    let claims = Claims::new("user123", 3600).jti("unique-token-id");
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.jti, Some("unique-token-id".to_string()));
}

#[test]
fn claims_with_custom_claim() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET);

    let claims = Claims::new("user123", 3600)
        .claim("role", "admin")
        .claim("permissions", vec!["read", "write"]);
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(
        decoded.claims.extra.get("role"),
        Some(&serde_json::json!("admin"))
    );
}

#[test]
fn expired_token_fails_validation() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET);

    let claims = Claims::new("user123", -3600);
    let token = encoder.encode(&claims).unwrap();
    let result = decoder.decode_claims(&token);

    assert!(result.is_err());
}

#[test]
fn skip_expiration_allows_expired_token() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET).skip_expiration();

    let claims = Claims::new("user123", -3600);
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn require_issuer_validation() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET).require_issuer("expected-issuer");

    let claims = Claims::new("user123", 3600).issuer("wrong-issuer");
    let token = encoder.encode(&claims).unwrap();
    let result = decoder.decode_claims(&token);

    assert!(result.is_err());
}

#[test]
fn require_issuer_passes_when_correct() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET).require_issuer("my-app");

    let claims = Claims::new("user123", 3600).issuer("my-app");
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn require_audience_validation() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET).require_audience("expected-audience");

    let claims = Claims::new("user123", 3600).audience("wrong-audience");
    let token = encoder.encode(&claims).unwrap();
    let result = decoder.decode_claims(&token);

    assert!(result.is_err());
}

#[test]
fn require_audience_passes_when_correct() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET).require_audience("api.example.com");

    let claims = Claims::new("user123", 3600).audience("api.example.com");
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn leeway_allows_slight_expiration() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET).leeway(60);

    let claims = Claims::new("user123", -30);
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn encode_with_kid() {
    let encoder = JwtEncoder::hs256(SECRET);

    let claims = Claims::new("user123", 3600);
    let token = encoder.encode_with_kid(&claims, "key-id-123").unwrap();
    let header = decode_header(&token).unwrap();

    assert_eq!(header.kid, Some("key-id-123".to_string()));
}

#[test]
fn decode_header_extracts_algorithm() {
    let encoder = JwtEncoder::hs256(SECRET);

    let claims = Claims::new("user123", 3600);
    let token = encoder.encode(&claims).unwrap();
    let header = decode_header(&token).unwrap();

    assert_eq!(header.alg, jsonwebtoken::Algorithm::HS256);
}

#[test]
fn dangerous_insecure_decode_skips_validation() {
    let encoder = JwtEncoder::hs256(SECRET);

    let claims = Claims::new("user123", -3600);
    let token = encoder.encode(&claims).unwrap();
    let decoded: jsonwebtoken::TokenData<Claims> = dangerous_insecure_decode(&token).unwrap();

    assert_eq!(decoded.claims.sub, "user123");
}

#[test]
fn claims_is_expired_check() {
    let expired_claims = Claims::new("user", -100);
    let valid_claims = Claims::new("user", 3600);

    assert!(expired_claims.is_expired());
    assert!(!valid_claims.is_expired());
}

#[test]
fn claims_not_before() {
    let encoder = JwtEncoder::hs256(SECRET);
    let decoder = JwtDecoder::hs256(SECRET);

    let now = chrono::Utc::now().timestamp();
    let claims = Claims::new("user123", 3600).not_before(now - 100);
    let token = encoder.encode(&claims).unwrap();
    let decoded = decoder.decode_claims(&token).unwrap();

    assert_eq!(decoded.claims.nbf, Some(now - 100));
}

#[test]
fn token_structure_has_three_parts() {
    let encoder = JwtEncoder::hs256(SECRET);

    let claims = Claims::new("user123", 3600);
    let token = encoder.encode(&claims).unwrap();
    let parts: Vec<&str> = token.split('.').collect();

    assert_eq!(parts.len(), 3);
}

#[test]
fn different_subjects_produce_different_tokens() {
    let encoder = JwtEncoder::hs256(SECRET);

    let claims1 = Claims::new("user1", 3600);
    let claims2 = Claims::new("user2", 3600);
    let token1 = encoder.encode(&claims1).unwrap();
    let token2 = encoder.encode(&claims2).unwrap();

    assert_ne!(token1, token2);
}
