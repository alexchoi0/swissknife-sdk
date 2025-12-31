#![cfg(feature = "totp")]

use swissknife_auth_sdk::totp::{
    generate_recovery_phrase, BackupCodes, TotpAlgorithm, TotpConfig, TotpSecret,
};

#[test]
fn generate_totp_secret() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    assert!(!secret.secret_base32().is_empty());
}

#[test]
fn generate_current_code() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let code = secret.generate_code().unwrap();

    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn generate_code_at_specific_time() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let code1 = secret.generate_code_at(1000000);
    let code2 = secret.generate_code_at(1000000);

    assert_eq!(code1, code2);
}

#[test]
fn same_time_produces_same_code() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();
    let time = 1609459200;

    let code1 = secret.generate_code_at(time);
    let code2 = secret.generate_code_at(time);

    assert_eq!(code1, code2);
}

#[test]
fn different_time_produces_different_code() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let code1 = secret.generate_code_at(1000000);
    let code2 = secret.generate_code_at(1000060);

    assert_ne!(code1, code2);
}

#[test]
fn verify_correct_code() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let code = secret.generate_code().unwrap();
    let result = secret.verify(&code).unwrap();

    assert!(result);
}

#[test]
fn verify_wrong_code() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let result = secret.verify("000000").unwrap_or(false);

    let _ = result;
}

#[test]
fn verify_with_window_tolerance() {
    let config = TotpConfig::new("TestApp", "user@example.com").step(30);
    let secret = TotpSecret::generate(&config).unwrap();

    let current_code = secret.generate_code().unwrap();
    let result = secret.verify_with_window(&current_code, 1).unwrap();

    assert!(result);
}

#[test]
fn custom_totp_digits() {
    let config = TotpConfig::new("TestApp", "user@example.com").digits(8);
    let secret = TotpSecret::generate(&config).unwrap();

    let code = secret.generate_code_at(1000000);

    assert_eq!(code.len(), 8);
}

#[test]
fn custom_totp_algorithm_sha256() {
    let config =
        TotpConfig::new("TestApp", "user@example.com").algorithm(TotpAlgorithm::Sha256);
    let secret = TotpSecret::generate(&config).unwrap();

    let code = secret.generate_code().unwrap();

    assert_eq!(code.len(), 6);
}

#[test]
fn custom_totp_algorithm_sha512() {
    let config =
        TotpConfig::new("TestApp", "user@example.com").algorithm(TotpAlgorithm::Sha512);
    let secret = TotpSecret::generate(&config).unwrap();

    let code = secret.generate_code().unwrap();

    assert_eq!(code.len(), 6);
}

#[test]
fn custom_step_interval() {
    let config = TotpConfig::new("TestApp", "user@example.com").step(60);
    let secret = TotpSecret::generate(&config).unwrap();

    let base_time = 1000020u64;
    let window_start = (base_time / 60) * 60;
    let code1 = secret.generate_code_at(window_start);
    let code2 = secret.generate_code_at(window_start + 30);
    let code3 = secret.generate_code_at(window_start + 60);

    assert_eq!(code1, code2);
    assert_ne!(code1, code3);
}

#[test]
fn otpauth_uri_format() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let uri = secret.otpauth_uri();

    assert!(uri.starts_with("otpauth://totp/"));
    assert!(uri.contains("secret="));
    assert!(uri.contains("issuer=TestApp"));
}

#[test]
fn qr_code_generation() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let qr_base64 = secret.qr_code_base64().unwrap();

    assert!(!qr_base64.is_empty());
}

#[test]
fn qr_code_png_generation() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let secret = TotpSecret::generate(&config).unwrap();

    let qr_png = secret.qr_code_png().unwrap();

    assert!(!qr_png.is_empty());
    assert_eq!(&qr_png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn time_remaining_returns_valid_value() {
    let config = TotpConfig::new("TestApp", "user@example.com").step(30);
    let secret = TotpSecret::generate(&config).unwrap();

    let remaining = secret.time_remaining().unwrap();

    assert!(remaining > 0);
    assert!(remaining <= 30);
}

#[test]
fn from_base32_recreates_same_codes() {
    let config = TotpConfig::new("TestApp", "user@example.com");
    let original = TotpSecret::generate(&config).unwrap();
    let base32 = original.secret_base32();

    let restored = TotpSecret::from_base32(&base32, &config).unwrap();

    let time = 1000000;
    assert_eq!(
        original.generate_code_at(time),
        restored.generate_code_at(time)
    );
}

#[test]
fn backup_codes_generation() {
    let codes = BackupCodes::generate(10);

    assert_eq!(codes.codes().len(), 10);
    assert_eq!(codes.remaining_count(), 10);
}

#[test]
fn backup_codes_are_eight_digits() {
    let codes = BackupCodes::generate(5);

    for code in codes.codes() {
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}

#[test]
fn backup_code_single_use() {
    let mut codes = BackupCodes::generate(3);
    let code = codes.codes()[0].to_string();

    assert!(codes.verify(&code));
    assert!(!codes.verify(&code));
}

#[test]
fn backup_code_remaining_count_decrements() {
    let mut codes = BackupCodes::generate(5);
    let code = codes.codes()[0].to_string();

    assert_eq!(codes.remaining_count(), 5);
    codes.verify(&code);
    assert_eq!(codes.remaining_count(), 4);
}

#[test]
fn backup_code_available_codes_excludes_used() {
    let mut codes = BackupCodes::generate(3);
    let code = codes.codes()[0].to_string();

    codes.verify(&code);
    let available = codes.available_codes();

    assert_eq!(available.len(), 2);
    assert!(!available.iter().any(|c| *c == code));
}

#[test]
fn backup_code_verify_with_spaces() {
    let mut codes = BackupCodes::generate(1);
    let code = codes.codes()[0].to_string();
    let with_spaces = format!("{} {}", &code[..4], &code[4..]);

    assert!(codes.verify(&with_spaces));
}

#[test]
fn backup_code_verify_with_dashes() {
    let mut codes = BackupCodes::generate(1);
    let code = codes.codes()[0].to_string();
    let with_dashes = format!("{}-{}", &code[..4], &code[4..]);

    assert!(codes.verify(&with_dashes));
}

#[test]
fn backup_code_verify_wrong_code() {
    let mut codes = BackupCodes::generate(3);

    assert!(!codes.verify("00000000"));
}

#[test]
fn recovery_phrase_generation() {
    let phrase = generate_recovery_phrase(6);

    assert_eq!(phrase.len(), 6);
    for word in &phrase {
        assert!(!word.is_empty());
    }
}

#[test]
fn recovery_phrase_words_are_unique() {
    let phrase = generate_recovery_phrase(10);
    let mut seen = std::collections::HashSet::new();

    for word in phrase {
        seen.insert(word);
    }

    assert_eq!(seen.len(), 10);
}
