#![cfg(feature = "voice")]

use swissknife_communication_sdk::voice::CallResponse;

#[test]
fn test_call_response_creation() {
    let response = CallResponse {
        call_id: "call123".to_string(),
        status: "initiated".to_string(),
    };

    assert_eq!(response.call_id, "call123");
    assert_eq!(response.status, "initiated");
}

#[test]
fn test_call_response_various_statuses() {
    let statuses = vec![
        "queued",
        "ringing",
        "in-progress",
        "completed",
        "busy",
        "failed",
        "no-answer",
        "canceled",
    ];

    for status in statuses {
        let response = CallResponse {
            call_id: "call".to_string(),
            status: status.to_string(),
        };
        assert_eq!(response.status, status);
    }
}

#[test]
fn test_call_response_clone() {
    let response = CallResponse {
        call_id: "clone-call".to_string(),
        status: "completed".to_string(),
    };

    let cloned = response.clone();
    assert_eq!(response.call_id, cloned.call_id);
    assert_eq!(response.status, cloned.status);
}

#[test]
fn test_call_response_debug() {
    let response = CallResponse {
        call_id: "debug-call".to_string(),
        status: "ringing".to_string(),
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("debug-call") || debug_str.contains("CallResponse"));
}
