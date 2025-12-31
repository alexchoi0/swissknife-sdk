use swissknife_search_sdk::Error;

#[test]
fn test_api_error_display() {
    let error = Error::Api {
        message: "Rate limit exceeded".to_string(),
        code: Some("429".to_string()),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("Rate limit"));
}

#[test]
fn test_api_error_without_code() {
    let error = Error::Api {
        message: "Unknown error".to_string(),
        code: None,
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("Unknown error"));
}

#[test]
fn test_auth_error() {
    let error = Error::Auth("Invalid API key".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid API key"));
}

#[test]
fn test_rate_limited_error() {
    let error = Error::RateLimited;
    let error_string = format!("{}", error);
    assert!(error_string.to_lowercase().contains("rate"));
}

#[test]
fn test_invalid_request_error() {
    let error = Error::InvalidRequest("Query too long".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Query too long"));
}

#[test]
fn test_not_found_error() {
    let error = Error::NotFound("Resource not found".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("not found") || error_string.contains("Resource"));
}

#[test]
fn test_parse_error() {
    let error = Error::Parse("Failed to parse response".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("parse") || error_string.contains("Parse"));
}

#[test]
fn test_error_debug_impl() {
    let error = Error::Auth("Test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Auth"));
}
