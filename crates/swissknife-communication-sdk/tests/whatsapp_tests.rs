#![cfg(feature = "whatsapp")]

use swissknife_communication_sdk::whatsapp::WhatsAppResponse;

#[test]
fn test_whatsapp_response_creation() {
    let response = WhatsAppResponse {
        message_id: "wa123".to_string(),
        status: "sent".to_string(),
    };

    assert_eq!(response.message_id, "wa123");
    assert_eq!(response.status, "sent");
}

#[test]
fn test_whatsapp_response_clone() {
    let response = WhatsAppResponse {
        message_id: "clone-wa".to_string(),
        status: "delivered".to_string(),
    };

    let cloned = response.clone();
    assert_eq!(response.message_id, cloned.message_id);
    assert_eq!(response.status, cloned.status);
}

#[test]
fn test_whatsapp_response_debug() {
    let response = WhatsAppResponse {
        message_id: "debug-wa".to_string(),
        status: "read".to_string(),
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("debug-wa") || debug_str.contains("WhatsAppResponse"));
}
