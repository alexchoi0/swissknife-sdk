use swissknife_communication_sdk::Error;

#[test]
fn test_api_error() {
    let error = Error::Api {
        code: 400,
        message: "Bad request".to_string(),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("400") || error_string.contains("Bad request"));
}

#[test]
fn test_auth_error() {
    let error = Error::Auth("Invalid API key".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid API key") || error_string.contains("Auth"));
}

#[test]
fn test_invalid_phone_number_error() {
    let error = Error::InvalidPhoneNumber("+1invalid".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("invalid") || error_string.contains("phone"));
}

#[test]
fn test_rate_limited_error() {
    let error = Error::RateLimited { retry_after: 60 };
    let error_string = format!("{}", error);
    assert!(error_string.contains("60") || error_string.to_lowercase().contains("rate"));
}

#[test]
fn test_config_error() {
    let error = Error::Config("Missing required field".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Missing") || error_string.contains("Config"));
}

#[test]
fn test_error_debug_impl() {
    let error = Error::Auth("Debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Auth"));
}
