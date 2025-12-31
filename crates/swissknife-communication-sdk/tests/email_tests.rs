#![cfg(feature = "email")]

use swissknife_communication_sdk::email::{Email, EmailAddress, EmailResponse};

#[test]
fn test_email_address_new() {
    let addr = EmailAddress::new("test@example.com");

    assert_eq!(addr.email, "test@example.com");
    assert!(addr.name.is_none());
}

#[test]
fn test_email_address_with_name() {
    let addr = EmailAddress::with_name("test@example.com", "Test User");

    assert_eq!(addr.email, "test@example.com");
    assert_eq!(addr.name, Some("Test User".to_string()));
}

#[test]
fn test_email_address_clone() {
    let addr = EmailAddress::with_name("clone@test.com", "Clone User");
    let cloned = addr.clone();

    assert_eq!(addr.email, cloned.email);
    assert_eq!(addr.name, cloned.name);
}

#[test]
fn test_email_address_debug() {
    let addr = EmailAddress::new("debug@test.com");
    let debug_str = format!("{:?}", addr);
    assert!(debug_str.contains("debug@test.com"));
}

#[test]
fn test_email_creation() {
    let from = EmailAddress::new("sender@example.com");
    let to = EmailAddress::new("recipient@example.com");
    let email = Email::new(from, to, "Test Subject");

    assert_eq!(email.from.email, "sender@example.com");
    assert_eq!(email.to.len(), 1);
    assert_eq!(email.subject, "Test Subject");
    assert!(email.text.is_none());
    assert!(email.html.is_none());
}

#[test]
fn test_email_with_text() {
    let from = EmailAddress::new("sender@test.com");
    let to = EmailAddress::new("recipient@test.com");
    let email = Email::new(from, to, "Subject")
        .text("Plain text content");

    assert_eq!(email.text, Some("Plain text content".to_string()));
}

#[test]
fn test_email_with_html() {
    let from = EmailAddress::new("sender@test.com");
    let to = EmailAddress::new("recipient@test.com");
    let email = Email::new(from, to, "Subject")
        .html("<h1>Hello</h1>");

    assert_eq!(email.html, Some("<h1>Hello</h1>".to_string()));
}

#[test]
fn test_email_with_text_and_html() {
    let from = EmailAddress::new("sender@test.com");
    let to = EmailAddress::new("recipient@test.com");
    let email = Email::new(from, to, "Subject")
        .text("Plain text")
        .html("<p>HTML content</p>");

    assert!(email.text.is_some());
    assert!(email.html.is_some());
}

#[test]
fn test_email_add_to() {
    let from = EmailAddress::new("sender@test.com");
    let to = EmailAddress::new("first@test.com");
    let email = Email::new(from, to, "Subject")
        .add_to(EmailAddress::new("second@test.com"))
        .add_to(EmailAddress::new("third@test.com"));

    assert_eq!(email.to.len(), 3);
}

#[test]
fn test_email_chaining() {
    let from = EmailAddress::with_name("sender@test.com", "Sender");
    let to = EmailAddress::with_name("recipient@test.com", "Recipient");
    let email = Email::new(from, to, "Chained Email")
        .text("Text body")
        .html("<p>HTML body</p>")
        .add_to(EmailAddress::new("cc@test.com"));

    assert_eq!(email.subject, "Chained Email");
    assert_eq!(email.to.len(), 2);
    assert!(email.text.is_some());
    assert!(email.html.is_some());
}

#[test]
fn test_email_clone() {
    let from = EmailAddress::new("sender@test.com");
    let to = EmailAddress::new("recipient@test.com");
    let email = Email::new(from, to, "Clone Test")
        .text("Clone content");

    let cloned = email.clone();
    assert_eq!(email.subject, cloned.subject);
    assert_eq!(email.to.len(), cloned.to.len());
}

#[test]
fn test_email_debug() {
    let from = EmailAddress::new("sender@test.com");
    let to = EmailAddress::new("recipient@test.com");
    let email = Email::new(from, to, "Debug Test");

    let debug_str = format!("{:?}", email);
    assert!(debug_str.contains("Debug Test") || debug_str.contains("Email"));
}

#[test]
fn test_email_response_creation() {
    let response = EmailResponse {
        message_id: Some("msg123".to_string()),
        status: "sent".to_string(),
    };

    assert_eq!(response.message_id, Some("msg123".to_string()));
    assert_eq!(response.status, "sent");
}

#[test]
fn test_email_response_without_id() {
    let response = EmailResponse {
        message_id: None,
        status: "queued".to_string(),
    };

    assert!(response.message_id.is_none());
    assert_eq!(response.status, "queued");
}

#[test]
fn test_email_response_clone() {
    let response = EmailResponse {
        message_id: Some("clone123".to_string()),
        status: "delivered".to_string(),
    };

    let cloned = response.clone();
    assert_eq!(response.message_id, cloned.message_id);
    assert_eq!(response.status, cloned.status);
}
