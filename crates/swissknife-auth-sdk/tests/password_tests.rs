#![cfg(feature = "password")]

use swissknife_auth_sdk::password::{
    check_strength, needs_rehash, verify, Algorithm, PasswordHasher,
};

#[test]
fn hash_and_verify_with_argon2id() {
    let hasher = PasswordHasher::argon2id();
    let password = "my_secure_password_123";

    let hash = hasher.hash(password).unwrap();

    assert!(hash.starts_with("$argon2id$"));
    assert!(verify(password, &hash).unwrap());
}

#[test]
fn hash_and_verify_with_argon2i() {
    let hasher = PasswordHasher::new(Algorithm::Argon2i);
    let password = "my_secure_password_123";

    let hash = hasher.hash(password).unwrap();

    assert!(hash.starts_with("$argon2i$"));
    assert!(verify(password, &hash).unwrap());
}

#[test]
fn hash_and_verify_with_argon2d() {
    let hasher = PasswordHasher::new(Algorithm::Argon2d);
    let password = "my_secure_password_123";

    let hash = hasher.hash(password).unwrap();

    assert!(hash.starts_with("$argon2d$"));
    assert!(verify(password, &hash).unwrap());
}

#[test]
fn hash_and_verify_with_bcrypt() {
    let hasher = PasswordHasher::bcrypt();
    let password = "my_secure_password";

    let hash = hasher.hash(password).unwrap();

    assert!(hash.starts_with("$2"));
    assert!(verify(password, &hash).unwrap());
}

#[test]
fn hash_and_verify_with_scrypt() {
    let hasher = PasswordHasher::scrypt();
    let password = "my_secure_password";

    let hash = hasher.hash(password).unwrap();

    assert!(hash.starts_with("$scrypt$"));
    assert!(verify(password, &hash).unwrap());
}

#[test]
fn verify_fails_for_wrong_password() {
    let hasher = PasswordHasher::argon2id();
    let hash = hasher.hash("correct_password").unwrap();

    assert!(!verify("wrong_password", &hash).unwrap());
}

#[test]
fn verify_fails_for_unknown_hash_format() {
    let result = verify("password", "$unknown$hash$format");

    assert!(result.is_err());
}

#[test]
fn custom_argon2_params() {
    let hasher = PasswordHasher::argon2id()
        .argon2_memory_cost(32768)
        .argon2_time_cost(2)
        .argon2_parallelism(2);
    let password = "test_password";

    let hash = hasher.hash(password).unwrap();

    assert!(verify(password, &hash).unwrap());
}

#[test]
fn custom_bcrypt_cost() {
    let hasher = PasswordHasher::bcrypt().bcrypt_cost(8);
    let password = "test_password";

    let hash = hasher.hash(password).unwrap();

    assert!(verify(password, &hash).unwrap());
}

#[test]
fn custom_scrypt_params() {
    let hasher = PasswordHasher::scrypt().scrypt_params(14, 8, 1);
    let password = "test_password";

    let hash = hasher.hash(password).unwrap();

    assert!(verify(password, &hash).unwrap());
}

#[test]
fn needs_rehash_when_algorithm_differs() {
    let hasher = PasswordHasher::argon2id();
    let hash = hasher.hash("password").unwrap();
    let desired = PasswordHasher::bcrypt();

    assert!(needs_rehash(&hash, &desired));
}

#[test]
fn no_rehash_needed_when_algorithm_matches() {
    let hasher = PasswordHasher::argon2id();
    let hash = hasher.hash("password").unwrap();
    let desired = PasswordHasher::argon2id();

    assert!(!needs_rehash(&hash, &desired));
}

#[test]
fn needs_rehash_for_bcrypt_to_argon2() {
    let hasher = PasswordHasher::bcrypt();
    let hash = hasher.hash("password").unwrap();
    let desired = PasswordHasher::argon2id();

    assert!(needs_rehash(&hash, &desired));
}

#[test]
fn needs_rehash_for_unknown_format() {
    let desired = PasswordHasher::argon2id();

    assert!(needs_rehash("$unknown$format", &desired));
}

#[test]
fn strength_check_weak_password() {
    let result = check_strength("abc");

    assert!(result.score < 3);
    assert!(!result.feedback.is_empty());
}

#[test]
fn strength_check_strong_password() {
    let result = check_strength("MyStr0ng!Passw0rd#2024");

    assert!(result.score >= 4);
}

#[test]
fn strength_check_common_password_penalty() {
    let result = check_strength("password123!");

    assert!(result.feedback.iter().any(|f| f.contains("common")));
}

#[test]
fn strength_check_missing_uppercase() {
    let result = check_strength("allowercase123!");

    assert!(result.feedback.iter().any(|f| f.contains("uppercase")));
}

#[test]
fn strength_check_missing_digits() {
    let result = check_strength("NoDigitsHere!");

    assert!(result.feedback.iter().any(|f| f.contains("numbers")));
}

#[test]
fn strength_check_missing_special_chars() {
    let result = check_strength("NoSpecialChars123");

    assert!(result.feedback.iter().any(|f| f.contains("special")));
}

#[test]
fn strength_check_short_password() {
    let result = check_strength("Ab1!");

    assert!(result.feedback.iter().any(|f| f.contains("8 characters")));
}

#[test]
fn strength_score_capped_at_five() {
    let result = check_strength("VeryLongAndSecurePassword123!@#$%^&*()");

    assert!(result.score <= 5);
}

#[test]
fn same_password_produces_different_hashes() {
    let hasher = PasswordHasher::argon2id();
    let password = "same_password";

    let hash1 = hasher.hash(password).unwrap();
    let hash2 = hasher.hash(password).unwrap();

    assert_ne!(hash1, hash2);
    assert!(verify(password, &hash1).unwrap());
    assert!(verify(password, &hash2).unwrap());
}

#[test]
fn algorithm_as_str() {
    assert_eq!(Algorithm::Argon2id.as_str(), "argon2id");
    assert_eq!(Algorithm::Argon2i.as_str(), "argon2i");
    assert_eq!(Algorithm::Argon2d.as_str(), "argon2d");
    assert_eq!(Algorithm::Bcrypt.as_str(), "bcrypt");
    assert_eq!(Algorithm::Scrypt.as_str(), "scrypt");
}
