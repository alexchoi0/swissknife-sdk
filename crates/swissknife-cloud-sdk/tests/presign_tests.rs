use swissknife_cloud_sdk::{PresignedUrl, PresignOptions};
use chrono::Utc;

#[test]
fn test_presigned_url_creation() {
    let url = PresignedUrl {
        url: "https://bucket.s3.amazonaws.com/file?signature=abc".to_string(),
        method: "GET".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };

    assert!(url.url.contains("signature"));
    assert_eq!(url.method, "GET");
}

#[test]
fn test_presigned_url_put_method() {
    let url = PresignedUrl {
        url: "https://bucket.s3.amazonaws.com/upload?signature=xyz".to_string(),
        method: "PUT".to_string(),
        expires_at: Utc::now() + chrono::Duration::minutes(30),
    };

    assert_eq!(url.method, "PUT");
}

#[test]
fn test_presigned_url_clone() {
    let url = PresignedUrl {
        url: "https://example.com/signed".to_string(),
        method: "GET".to_string(),
        expires_at: Utc::now(),
    };

    let cloned = url.clone();
    assert_eq!(url.url, cloned.url);
    assert_eq!(url.method, cloned.method);
}

#[test]
fn test_presigned_url_debug() {
    let url = PresignedUrl {
        url: "https://debug.com/signed".to_string(),
        method: "GET".to_string(),
        expires_at: Utc::now(),
    };

    let debug_str = format!("{:?}", url);
    assert!(debug_str.contains("https://debug.com/signed"));
}

#[test]
fn test_presign_options_default() {
    let options = PresignOptions::default();

    assert_eq!(options.expires_in_seconds, 0);
    assert!(options.content_type.is_none());
}

#[test]
fn test_presign_options_with_expiry() {
    let options = PresignOptions {
        expires_in_seconds: 3600,
        content_type: None,
    };

    assert_eq!(options.expires_in_seconds, 3600);
}

#[test]
fn test_presign_options_with_content_type() {
    let options = PresignOptions {
        expires_in_seconds: 300,
        content_type: Some("application/pdf".to_string()),
    };

    assert_eq!(options.content_type, Some("application/pdf".to_string()));
}

#[test]
fn test_presign_options_various_expiry_times() {
    let expiry_times = vec![60, 300, 900, 3600, 7200, 86400];

    for expires in expiry_times {
        let options = PresignOptions {
            expires_in_seconds: expires,
            content_type: None,
        };
        assert_eq!(options.expires_in_seconds, expires);
    }
}

#[test]
fn test_presign_options_clone() {
    let options = PresignOptions {
        expires_in_seconds: 1800,
        content_type: Some("image/jpeg".to_string()),
    };

    let cloned = options.clone();
    assert_eq!(options.expires_in_seconds, cloned.expires_in_seconds);
    assert_eq!(options.content_type, cloned.content_type);
}

#[test]
fn test_presign_options_debug() {
    let options = PresignOptions::default();
    let debug_str = format!("{:?}", options);
    assert!(debug_str.contains("PresignOptions"));
}
