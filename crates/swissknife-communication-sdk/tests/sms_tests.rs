#![cfg(feature = "sms")]

use swissknife_communication_sdk::sms::SmsResponse;

#[test]
fn test_sms_response_creation() {
    let response = SmsResponse {
        message_id: "sms123".to_string(),
        status: "sent".to_string(),
    };

    assert_eq!(response.message_id, "sms123");
    assert_eq!(response.status, "sent");
}

#[test]
fn test_sms_response_various_statuses() {
    let statuses = vec!["queued", "sent", "delivered", "failed", "undelivered"];

    for status in statuses {
        let response = SmsResponse {
            message_id: "msg".to_string(),
            status: status.to_string(),
        };
        assert_eq!(response.status, status);
    }
}

#[test]
fn test_sms_response_clone() {
    let response = SmsResponse {
        message_id: "clone-sms".to_string(),
        status: "sent".to_string(),
    };

    let cloned = response.clone();
    assert_eq!(response.message_id, cloned.message_id);
    assert_eq!(response.status, cloned.status);
}

#[test]
fn test_sms_response_debug() {
    let response = SmsResponse {
        message_id: "debug-sms".to_string(),
        status: "pending".to_string(),
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("debug-sms") || debug_str.contains("SmsResponse"));
}
