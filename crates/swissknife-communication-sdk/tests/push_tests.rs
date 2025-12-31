#![cfg(feature = "push")]

use swissknife_communication_sdk::push::{PushNotification, PushResponse};

#[test]
fn test_push_notification_new() {
    let notification = PushNotification::new("Test Title", "Test Body");

    assert_eq!(notification.title, "Test Title");
    assert_eq!(notification.body, "Test Body");
    assert!(notification.image.is_none());
    assert!(notification.data.is_none());
}

#[test]
fn test_push_notification_with_image() {
    let notification = PushNotification::new("Title", "Body")
        .image("https://example.com/image.jpg");

    assert_eq!(notification.image, Some("https://example.com/image.jpg".to_string()));
}

#[test]
fn test_push_notification_with_data() {
    let notification = PushNotification::new("Title", "Body")
        .data("key1", "value1")
        .data("key2", "value2");

    assert!(notification.data.is_some());
    let data = notification.data.unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data.get("key1"), Some(&"value1".to_string()));
}

#[test]
fn test_push_notification_chaining() {
    let notification = PushNotification::new("Alert", "You have a message")
        .image("https://img.com/alert.png")
        .data("type", "message")
        .data("sender_id", "user123");

    assert_eq!(notification.title, "Alert");
    assert!(notification.image.is_some());
    assert!(notification.data.is_some());
}

#[test]
fn test_push_notification_clone() {
    let notification = PushNotification::new("Clone Title", "Clone Body")
        .data("key", "value");

    let cloned = notification.clone();
    assert_eq!(notification.title, cloned.title);
    assert_eq!(notification.body, cloned.body);
}

#[test]
fn test_push_notification_debug() {
    let notification = PushNotification::new("Debug", "Debug body");
    let debug_str = format!("{:?}", notification);
    assert!(debug_str.contains("Debug"));
}

#[test]
fn test_push_response_creation() {
    let response = PushResponse {
        message_id: Some("push123".to_string()),
        success_count: 10,
        failure_count: 2,
    };

    assert_eq!(response.message_id, Some("push123".to_string()));
    assert_eq!(response.success_count, 10);
    assert_eq!(response.failure_count, 2);
}

#[test]
fn test_push_response_all_success() {
    let response = PushResponse {
        message_id: Some("success".to_string()),
        success_count: 100,
        failure_count: 0,
    };

    assert_eq!(response.failure_count, 0);
}

#[test]
fn test_push_response_all_failure() {
    let response = PushResponse {
        message_id: None,
        success_count: 0,
        failure_count: 50,
    };

    assert_eq!(response.success_count, 0);
    assert!(response.message_id.is_none());
}

#[test]
fn test_push_response_clone() {
    let response = PushResponse {
        message_id: Some("clone".to_string()),
        success_count: 5,
        failure_count: 1,
    };

    let cloned = response.clone();
    assert_eq!(response.message_id, cloned.message_id);
    assert_eq!(response.success_count, cloned.success_count);
}

#[test]
fn test_push_response_debug() {
    let response = PushResponse {
        message_id: Some("debug".to_string()),
        success_count: 1,
        failure_count: 0,
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("debug") || debug_str.contains("PushResponse"));
}

mod serialization_tests {
    use super::*;

    #[test]
    fn test_push_notification_serialize() {
        let notification = PushNotification::new("Title", "Body");
        let json = serde_json::to_string(&notification).unwrap();

        assert!(json.contains("Title"));
        assert!(json.contains("Body"));
    }

    #[test]
    fn test_push_notification_serialize_with_data() {
        let notification = PushNotification::new("Title", "Body")
            .data("key", "value");

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }

    #[test]
    fn test_push_notification_serialize_with_image() {
        let notification = PushNotification::new("Title", "Body")
            .image("https://example.com/img.jpg");

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("https://example.com/img.jpg"));
    }

    #[test]
    fn test_push_notification_deserialize() {
        let json = r#"{"title":"Deserialized","body":"Content"}"#;
        let notification: PushNotification = serde_json::from_str(json).unwrap();

        assert_eq!(notification.title, "Deserialized");
        assert_eq!(notification.body, "Content");
    }
}
