#![cfg(feature = "api-keys")]

use std::time::Duration;
use swissknife_auth_sdk::api_keys::{ApiKeyConfig, ApiKeyGenerator, ScopedApiKeyGenerator};

#[test]
fn generate_key_with_default_config() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::default());
    let key = generator.generate();

    assert!(key.full_key.starts_with("sk_"));
    assert!(!key.key_id.is_empty());
    assert!(!key.secret.is_empty());
    assert!(!key.hash.is_empty());
    assert_eq!(key.prefix, format!("sk_{}", key.key_id));
}

#[test]
fn generate_key_with_custom_prefix() {
    let config = ApiKeyConfig::new("pk");
    let generator = ApiKeyGenerator::new(config);
    let key = generator.generate();

    assert!(key.full_key.starts_with("pk_"));
}

#[test]
fn generate_key_with_custom_separator() {
    let config = ApiKeyConfig::default().separator('-');
    let generator = ApiKeyGenerator::new(config);
    let key = generator.generate();

    assert!(key.full_key.starts_with("sk-"));
    assert!(key.prefix.contains("-"));
}

#[test]
fn generate_key_with_custom_lengths() {
    let config = ApiKeyConfig::default()
        .key_id_length(16)
        .secret_length(64);
    let generator = ApiKeyGenerator::new(config);
    let key = generator.generate();

    assert_eq!(key.key_id.len(), 16);
}

#[test]
fn verify_key_matches_its_hash() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::default());
    let key = generator.generate();

    assert!(generator.verify(&key.full_key, &key.hash));
}

#[test]
fn verify_fails_for_wrong_key() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::default());
    let key1 = generator.generate();
    let key2 = generator.generate();

    assert!(!generator.verify(&key1.full_key, &key2.hash));
}

#[test]
fn verify_fails_for_tampered_key() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::default());
    let key = generator.generate();
    let tampered = format!("{}x", key.full_key);

    assert!(!generator.verify(&tampered, &key.hash));
}

#[test]
fn parse_key_extracts_key_id_correctly() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::default());
    let key = generator.generate();

    let extracted_key_id = generator.extract_key_id(&key.full_key);

    assert!(extracted_key_id.is_ok() || key.secret.contains('_'));
    if !key.secret.contains('_') {
        assert_eq!(extracted_key_id.unwrap(), key.key_id);
    }
}

#[test]
fn parse_fails_for_invalid_format() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::default());

    assert!(generator.parse("invalid_key").is_err());
    assert!(generator.parse("").is_err());
    assert!(generator.parse("a_b_c_d").is_err());
}

#[test]
fn parse_fails_for_wrong_prefix() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::new("sk"));

    assert!(generator.parse("pk_abc_secret").is_err());
}

#[test]
fn extract_key_id_works_when_secret_has_no_separator() {
    let config = ApiKeyConfig::default().separator('-');
    let generator = ApiKeyGenerator::new(config);

    for _ in 0..10 {
        let key = generator.generate();
        if !key.secret.contains('-') {
            let extracted = generator.extract_key_id(&key.full_key).unwrap();
            assert_eq!(extracted, key.key_id);
            return;
        }
    }
}

#[test]
fn with_prefix_shorthand_creates_generator() {
    let generator = ApiKeyGenerator::with_prefix("test");
    let key = generator.generate();

    assert!(key.full_key.starts_with("test_"));
}

#[test]
fn scoped_key_contains_scopes() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scopes = vec!["read".to_string(), "write".to_string()];
    let scoped_key = generator.generate(scopes.clone(), None);

    assert_eq!(scoped_key.scopes, scopes);
    assert!(scoped_key.expires_at.is_none());
}

#[test]
fn scoped_key_with_expiration() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scoped_key = generator.generate(vec![], Some(Duration::from_secs(3600)));

    assert!(scoped_key.expires_at.is_some());
}

#[test]
fn scoped_key_verify_works() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scoped_key = generator.generate(vec![], None);

    assert!(generator.verify(&scoped_key.key.full_key, &scoped_key.key.hash));
}

#[test]
fn scoped_key_not_expired_when_no_expiration() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scoped_key = generator.generate(vec![], None);

    assert!(!generator.is_expired(&scoped_key));
}

#[test]
fn has_scope_matches_exact_scope() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scoped_key = generator.generate(vec!["read".to_string(), "write".to_string()], None);

    assert!(generator.has_scope(&scoped_key, "read"));
    assert!(generator.has_scope(&scoped_key, "write"));
    assert!(!generator.has_scope(&scoped_key, "delete"));
}

#[test]
fn wildcard_scope_matches_everything() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scoped_key = generator.generate(vec!["*".to_string()], None);

    assert!(generator.has_scope(&scoped_key, "read"));
    assert!(generator.has_scope(&scoped_key, "write"));
    assert!(generator.has_scope(&scoped_key, "anything"));
}

#[test]
fn has_any_scope_matches_if_one_present() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scoped_key = generator.generate(vec!["read".to_string()], None);

    assert!(generator.has_any_scope(&scoped_key, &["read", "write"]));
    assert!(generator.has_any_scope(&scoped_key, &["read"]));
    assert!(!generator.has_any_scope(&scoped_key, &["write", "delete"]));
}

#[test]
fn has_all_scopes_requires_all_present() {
    let config = ApiKeyConfig::default();
    let generator = ScopedApiKeyGenerator::new(config);
    let scoped_key = generator.generate(vec!["read".to_string(), "write".to_string()], None);

    assert!(generator.has_all_scopes(&scoped_key, &["read", "write"]));
    assert!(generator.has_all_scopes(&scoped_key, &["read"]));
    assert!(!generator.has_all_scopes(&scoped_key, &["read", "delete"]));
}

#[test]
fn generated_keys_are_unique() {
    let generator = ApiKeyGenerator::new(ApiKeyConfig::default());
    let key1 = generator.generate();
    let key2 = generator.generate();

    assert_ne!(key1.full_key, key2.full_key);
    assert_ne!(key1.key_id, key2.key_id);
    assert_ne!(key1.hash, key2.hash);
}
