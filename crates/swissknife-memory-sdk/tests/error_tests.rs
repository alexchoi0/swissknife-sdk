use swissknife_memory_sdk::Error;

#[test]
fn api_error_display_contains_message() {
    let error = Error::Api {
        message: "Rate limit exceeded".to_string(),
        code: Some("429".to_string()),
    };

    let display = format!("{}", error);
    assert!(display.contains("Rate limit exceeded"));
}

#[test]
fn api_error_display_format() {
    let error = Error::Api {
        message: "Unauthorized".to_string(),
        code: Some("401".to_string()),
    };

    let display = format!("{}", error);
    assert!(display.contains("API error"));
    assert!(display.contains("Unauthorized"));
}

#[test]
fn api_error_without_code() {
    let error = Error::Api {
        message: "Unknown error".to_string(),
        code: None,
    };

    let display = format!("{}", error);
    assert!(display.contains("Unknown error"));
}

#[test]
fn memory_not_found_display() {
    let error = Error::MemoryNotFound("mem-not-found-123".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Memory not found"));
    assert!(display.contains("mem-not-found-123"));
}

#[test]
fn user_not_found_display() {
    let error = Error::UserNotFound("user-missing-456".to_string());

    let display = format!("{}", error);
    assert!(display.contains("User not found"));
    assert!(display.contains("user-missing-456"));
}

#[test]
fn session_not_found_display() {
    let error = Error::SessionNotFound("session-gone-789".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Session not found"));
    assert!(display.contains("session-gone-789"));
}

#[test]
fn invalid_input_display() {
    let error = Error::InvalidInput("Empty content not allowed".to_string());

    let display = format!("{}", error);
    assert!(display.contains("Invalid input"));
    assert!(display.contains("Empty content not allowed"));
}

#[test]
fn error_debug_format() {
    let error = Error::Api {
        message: "Debug test".to_string(),
        code: Some("500".to_string()),
    };

    let debug = format!("{:?}", error);
    assert!(debug.contains("Api"));
    assert!(debug.contains("Debug test"));
    assert!(debug.contains("500"));
}

#[test]
fn error_type_matching_api() {
    let error = Error::Api {
        message: "Test".to_string(),
        code: None,
    };

    let is_api_error = matches!(error, Error::Api { .. });
    assert!(is_api_error);
}

#[test]
fn error_type_matching_memory_not_found() {
    let error = Error::MemoryNotFound("test".to_string());

    let is_memory_not_found = matches!(error, Error::MemoryNotFound(_));
    assert!(is_memory_not_found);
}

#[test]
fn error_type_matching_user_not_found() {
    let error = Error::UserNotFound("test".to_string());

    let is_user_not_found = matches!(error, Error::UserNotFound(_));
    assert!(is_user_not_found);
}

#[test]
fn error_type_matching_session_not_found() {
    let error = Error::SessionNotFound("test".to_string());

    let is_session_not_found = matches!(error, Error::SessionNotFound(_));
    assert!(is_session_not_found);
}

#[test]
fn error_type_matching_invalid_input() {
    let error = Error::InvalidInput("test".to_string());

    let is_invalid_input = matches!(error, Error::InvalidInput(_));
    assert!(is_invalid_input);
}

#[test]
fn api_error_code_extraction() {
    let error = Error::Api {
        message: "Server error".to_string(),
        code: Some("503".to_string()),
    };

    if let Error::Api { message, code } = error {
        assert_eq!(message, "Server error");
        assert_eq!(code, Some("503".to_string()));
    } else {
        panic!("Expected Api error variant");
    }
}

#[test]
fn error_not_found_id_extraction() {
    let memory_error = Error::MemoryNotFound("mem-extract".to_string());
    if let Error::MemoryNotFound(id) = memory_error {
        assert_eq!(id, "mem-extract");
    }

    let user_error = Error::UserNotFound("user-extract".to_string());
    if let Error::UserNotFound(id) = user_error {
        assert_eq!(id, "user-extract");
    }

    let session_error = Error::SessionNotFound("session-extract".to_string());
    if let Error::SessionNotFound(id) = session_error {
        assert_eq!(id, "session-extract");
    }
}

#[test]
fn result_type_with_memory() {
    let result: swissknife_memory_sdk::Result<String> = Ok("success".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn result_type_with_error() {
    let result: swissknife_memory_sdk::Result<String> =
        Err(Error::InvalidInput("invalid".to_string()));
    assert!(result.is_err());
}
