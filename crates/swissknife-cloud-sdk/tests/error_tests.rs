use swissknife_cloud_sdk::Error;

#[test]
fn test_api_error() {
    let error = Error::Api {
        message: "Access denied".to_string(),
        code: Some("403".to_string()),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("Access denied"));
}

#[test]
fn test_api_error_without_code() {
    let error = Error::Api {
        message: "Unknown error".to_string(),
        code: None,
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("Unknown"));
}

#[test]
fn test_auth_error() {
    let error = Error::Auth("Invalid credentials".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("credentials") || error_string.contains("Auth"));
}

#[test]
fn test_bucket_not_found_error() {
    let error = Error::BucketNotFound("non-existent-bucket".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("bucket") || error_string.contains("non-existent"));
}

#[test]
fn test_object_not_found_error() {
    let error = Error::ObjectNotFound("missing-file.txt".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("missing-file.txt") || error_string.contains("not found"));
}

#[test]
fn test_permission_denied_error() {
    let error = Error::PermissionDenied("Write access required".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Permission") || error_string.contains("Write"));
}

#[test]
fn test_upload_failed_error() {
    let error = Error::UploadFailed("Connection timeout".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("timeout") || error_string.contains("Upload"));
}

#[test]
fn test_error_debug_impl() {
    let error = Error::Auth("Debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Auth"));
}
