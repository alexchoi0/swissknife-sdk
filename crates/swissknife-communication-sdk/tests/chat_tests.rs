#![cfg(feature = "chat")]

use swissknife_communication_sdk::chat::{ChatMessage, ChatResponse};

#[test]
fn test_chat_message_new() {
    let message = ChatMessage::new("Hello, World!");

    assert_eq!(message.text, "Hello, World!");
    assert!(message.username.is_none());
    assert!(message.icon_url.is_none());
    assert!(message.icon_emoji.is_none());
}

#[test]
fn test_chat_message_with_username() {
    let message = ChatMessage::new("Hello from bot")
        .username("MyBot");

    assert_eq!(message.username, Some("MyBot".to_string()));
}

#[test]
fn test_chat_message_with_icon_url() {
    let message = ChatMessage::new("Message with icon")
        .icon_url("https://example.com/icon.png");

    assert_eq!(message.icon_url, Some("https://example.com/icon.png".to_string()));
}

#[test]
fn test_chat_message_with_icon_emoji() {
    let message = ChatMessage::new("Emoji message")
        .icon_emoji(":rocket:");

    assert_eq!(message.icon_emoji, Some(":rocket:".to_string()));
}

#[test]
fn test_chat_message_chaining() {
    let message = ChatMessage::new("Full featured message")
        .username("Bot")
        .icon_url("https://img.com/bot.png")
        .icon_emoji(":robot:");

    assert!(message.username.is_some());
    assert!(message.icon_url.is_some());
    assert!(message.icon_emoji.is_some());
}

#[test]
fn test_chat_message_clone() {
    let message = ChatMessage::new("Clone me")
        .username("CloneBot");

    let cloned = message.clone();
    assert_eq!(message.text, cloned.text);
    assert_eq!(message.username, cloned.username);
}

#[test]
fn test_chat_message_debug() {
    let message = ChatMessage::new("Debug message");
    let debug_str = format!("{:?}", message);
    assert!(debug_str.contains("Debug message") || debug_str.contains("ChatMessage"));
}

#[test]
fn test_chat_response_creation() {
    let response = ChatResponse {
        message_id: Some("chat123".to_string()),
        channel: Some("#general".to_string()),
        timestamp: Some("1234567890.123456".to_string()),
    };

    assert_eq!(response.message_id, Some("chat123".to_string()));
    assert_eq!(response.channel, Some("#general".to_string()));
    assert!(response.timestamp.is_some());
}

#[test]
fn test_chat_response_minimal() {
    let response = ChatResponse {
        message_id: None,
        channel: None,
        timestamp: None,
    };

    assert!(response.message_id.is_none());
    assert!(response.channel.is_none());
    assert!(response.timestamp.is_none());
}

#[test]
fn test_chat_response_clone() {
    let response = ChatResponse {
        message_id: Some("clone".to_string()),
        channel: Some("#test".to_string()),
        timestamp: None,
    };

    let cloned = response.clone();
    assert_eq!(response.message_id, cloned.message_id);
    assert_eq!(response.channel, cloned.channel);
}

#[test]
fn test_chat_response_debug() {
    let response = ChatResponse {
        message_id: Some("debug".to_string()),
        channel: None,
        timestamp: None,
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("debug") || debug_str.contains("ChatResponse"));
}

mod serialization_tests {
    use super::*;

    #[test]
    fn test_chat_message_serialize() {
        let message = ChatMessage::new("Test message");
        let json = serde_json::to_string(&message).unwrap();

        assert!(json.contains("Test message"));
    }

    #[test]
    fn test_chat_message_serialize_with_username() {
        let message = ChatMessage::new("Hello")
            .username("Bot");

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("Bot"));
    }

    #[test]
    fn test_chat_message_deserialize() {
        let json = r#"{"text":"Deserialized message"}"#;
        let message: ChatMessage = serde_json::from_str(json).unwrap();

        assert_eq!(message.text, "Deserialized message");
    }
}
