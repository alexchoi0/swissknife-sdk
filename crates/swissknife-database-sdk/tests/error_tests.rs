use swissknife_database_sdk::Error;

#[test]
fn test_connection_error() {
    let error = Error::Connection("Failed to connect".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("connect") || error_string.contains("Connection"));
}

#[test]
fn test_query_error() {
    let error = Error::Query("Syntax error near SELECT".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Syntax") || error_string.contains("Query"));
}

#[test]
fn test_auth_error() {
    let error = Error::Auth("Invalid credentials".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("credentials") || error_string.contains("Auth"));
}

#[test]
fn test_not_found_error() {
    let error = Error::NotFound("Table not found".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("not found") || error_string.contains("Table"));
}

#[test]
fn test_constraint_error() {
    let error = Error::Constraint("Foreign key violation".to_string());
    let error_string = format!("{}", error);
    assert!(error_string.contains("Foreign") || error_string.contains("Constraint"));
}

#[test]
fn test_timeout_error() {
    let error = Error::Timeout;
    let error_string = format!("{}", error);
    assert!(error_string.to_lowercase().contains("timeout"));
}

#[test]
fn test_error_debug_impl() {
    let error = Error::Query("Debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Query"));
}
