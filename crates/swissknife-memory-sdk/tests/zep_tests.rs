#![cfg(feature = "zep")]

use std::collections::HashMap;
use swissknife_memory_sdk::zep::{
    CreateSessionRequest, CreateUserRequest, GetMemoryParams, ListSessionsParams,
    ListUsersParams, SearchMemoryParams, UpdateUserRequest, ZepClient, ZepMemory,
    ZepMemoryResponse, ZepMessage, ZepMessageResponse, ZepSearchResult, ZepSession,
    ZepSummary, ZepUser,
};

#[test]
fn client_initialization_succeeds_with_valid_api_key() {
    let _client = ZepClient::new("zep-api-key-12345");
}

#[test]
fn client_initialization_with_empty_api_key() {
    let _client = ZepClient::new("");
}

#[test]
fn client_with_custom_base_url_succeeds() {
    let _client = ZepClient::with_base_url("key", "https://custom.zep.io/api/v3");
}

#[test]
fn client_with_trailing_slash_base_url() {
    let _client = ZepClient::with_base_url("key", "https://zep.example.com/");
}

#[test]
fn client_clone_creates_independent_instance() {
    let client1 = ZepClient::new("key1");
    let _client2 = ZepClient::new("key2");
    let _client3 = ZepClient::with_base_url("key3", "https://example.com");
    drop(client1);
}

#[test]
fn zep_message_construction_minimal() {
    let message = ZepMessage {
        role: "user".to_string(),
        content: "Hello Zep".to_string(),
        role_type: None,
        metadata: None,
    };

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello Zep");
    assert!(message.role_type.is_none());
    assert!(message.metadata.is_none());
}

#[test]
fn zep_message_construction_full() {
    let mut metadata = HashMap::new();
    metadata.insert("sentiment".to_string(), serde_json::json!("positive"));

    let message = ZepMessage {
        role: "assistant".to_string(),
        content: "Hello! How can I help?".to_string(),
        role_type: Some("ai".to_string()),
        metadata: Some(metadata),
    };

    assert_eq!(message.role, "assistant");
    assert_eq!(message.role_type.as_deref(), Some("ai"));
    assert!(message.metadata.is_some());
}

#[test]
fn zep_message_serialization_skips_none_fields() {
    let message = ZepMessage {
        role: "user".to_string(),
        content: "Test".to_string(),
        role_type: None,
        metadata: None,
    };

    let json = serde_json::to_value(&message).expect("serialization should succeed");
    assert_eq!(json["role"], "user");
    assert_eq!(json["content"], "Test");
    assert!(json.get("role_type").is_none());
    assert!(json.get("metadata").is_none());
}

#[test]
fn zep_message_serialization_includes_some_fields() {
    let message = ZepMessage {
        role: "assistant".to_string(),
        content: "Response".to_string(),
        role_type: Some("ai".to_string()),
        metadata: None,
    };

    let json = serde_json::to_value(&message).expect("serialization should succeed");
    assert_eq!(json["role_type"], "ai");
}

#[test]
fn create_session_request_minimal() {
    let request = CreateSessionRequest {
        session_id: "session-minimal".to_string(),
        user_id: None,
        metadata: None,
    };

    assert_eq!(request.session_id, "session-minimal");
    assert!(request.user_id.is_none());
    assert!(request.metadata.is_none());
}

#[test]
fn create_session_request_full() {
    let mut metadata = HashMap::new();
    metadata.insert("channel".to_string(), serde_json::json!("web"));

    let request = CreateSessionRequest {
        session_id: "session-full".to_string(),
        user_id: Some("user-for-session".to_string()),
        metadata: Some(metadata),
    };

    assert_eq!(request.session_id, "session-full");
    assert_eq!(request.user_id.as_deref(), Some("user-for-session"));
    assert!(request.metadata.is_some());
}

#[test]
fn create_session_request_serialization() {
    let request = CreateSessionRequest {
        session_id: "serialize-session".to_string(),
        user_id: Some("serialize-user".to_string()),
        metadata: None,
    };

    let json = serde_json::to_value(&request).expect("serialization should succeed");
    assert_eq!(json["session_id"], "serialize-session");
    assert_eq!(json["user_id"], "serialize-user");
}

#[test]
fn create_user_request_minimal() {
    let request = CreateUserRequest {
        user_id: "minimal-user".to_string(),
        email: None,
        first_name: None,
        last_name: None,
        metadata: None,
    };

    assert_eq!(request.user_id, "minimal-user");
}

#[test]
fn create_user_request_full() {
    let mut metadata = HashMap::new();
    metadata.insert("tier".to_string(), serde_json::json!("premium"));

    let request = CreateUserRequest {
        user_id: "full-user".to_string(),
        email: Some("user@example.com".to_string()),
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        metadata: Some(metadata),
    };

    assert_eq!(request.user_id, "full-user");
    assert_eq!(request.email.as_deref(), Some("user@example.com"));
    assert_eq!(request.first_name.as_deref(), Some("John"));
    assert_eq!(request.last_name.as_deref(), Some("Doe"));
    assert!(request.metadata.is_some());
}

#[test]
fn update_user_request_default() {
    let request = UpdateUserRequest::default();

    assert!(request.email.is_none());
    assert!(request.first_name.is_none());
    assert!(request.last_name.is_none());
    assert!(request.metadata.is_none());
}

#[test]
fn update_user_request_partial() {
    let request = UpdateUserRequest {
        email: Some("new@example.com".to_string()),
        first_name: None,
        last_name: None,
        metadata: None,
    };

    assert_eq!(request.email.as_deref(), Some("new@example.com"));
    assert!(request.first_name.is_none());
}

#[test]
fn get_memory_params_default() {
    let params = GetMemoryParams::default();

    assert!(params.lastn.is_none());
    assert!(params.min_rating.is_none());
}

#[test]
fn get_memory_params_with_values() {
    let params = GetMemoryParams {
        lastn: Some(50),
        min_rating: Some(0.6),
    };

    assert_eq!(params.lastn, Some(50));
    assert!((params.min_rating.unwrap() - 0.6).abs() < f32::EPSILON);
}

#[test]
fn search_memory_params_default() {
    let params = SearchMemoryParams::default();

    assert!(params.limit.is_none());
    assert!(params.min_score.is_none());
    assert!(params.search_type.is_none());
    assert!(params.search_scope.is_none());
}

#[test]
fn search_memory_params_full() {
    let params = SearchMemoryParams {
        limit: Some(20),
        min_score: Some(0.7),
        search_type: Some("mmr".to_string()),
        search_scope: Some("messages".to_string()),
    };

    assert_eq!(params.limit, Some(20));
    assert!((params.min_score.unwrap() - 0.7).abs() < f32::EPSILON);
    assert_eq!(params.search_type.as_deref(), Some("mmr"));
    assert_eq!(params.search_scope.as_deref(), Some("messages"));
}

#[test]
fn list_sessions_params_default() {
    let params = ListSessionsParams::default();

    assert!(params.limit.is_none());
    assert!(params.cursor.is_none());
}

#[test]
fn list_sessions_params_with_pagination() {
    let params = ListSessionsParams {
        limit: Some(100),
        cursor: Some(12345),
    };

    assert_eq!(params.limit, Some(100));
    assert_eq!(params.cursor, Some(12345));
}

#[test]
fn list_users_params_default() {
    let params = ListUsersParams::default();

    assert!(params.limit.is_none());
    assert!(params.cursor.is_none());
}

#[test]
fn list_users_params_with_values() {
    let params = ListUsersParams {
        limit: Some(25),
        cursor: Some(999),
    };

    assert_eq!(params.limit, Some(25));
    assert_eq!(params.cursor, Some(999));
}

#[test]
fn zep_memory_response_deserialization() {
    let json = r#"{"messages": [{"role": "user", "content": "Hello"}]}"#;
    let response: ZepMemoryResponse =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert!(response.messages.is_some());
    assert_eq!(response.messages.as_ref().unwrap().len(), 1);
}

#[test]
fn zep_memory_response_empty_messages() {
    let json = r#"{"messages": null}"#;
    let response: ZepMemoryResponse =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert!(response.messages.is_none());
}

#[test]
fn zep_memory_full_deserialization() {
    let json = r#"{
        "messages": [{"role": "user", "content": "Hi"}],
        "summary": {"content": "A greeting", "uuid": "sum-1"},
        "facts": ["User said hi"],
        "relevant_facts": ["Greeting detected"]
    }"#;

    let memory: ZepMemory = serde_json::from_str(json).expect("deserialization should succeed");

    assert!(memory.messages.is_some());
    assert!(memory.summary.is_some());
    assert_eq!(memory.summary.as_ref().unwrap().content, "A greeting");
    assert!(memory.facts.is_some());
    assert!(memory.relevant_facts.is_some());
}

#[test]
fn zep_summary_deserialization() {
    let json = r#"{
        "uuid": "summary-uuid",
        "content": "Summary of conversation",
        "token_count": 150,
        "created_at": "2024-01-01T00:00:00Z"
    }"#;

    let summary: ZepSummary =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(summary.uuid.as_deref(), Some("summary-uuid"));
    assert_eq!(summary.content, "Summary of conversation");
    assert_eq!(summary.token_count, Some(150));
}

#[test]
fn zep_search_result_deserialization() {
    let json = r#"{
        "message": {"role": "user", "content": "Search match"},
        "score": 0.85,
        "dist": 0.15
    }"#;

    let result: ZepSearchResult =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert!(result.message.is_some());
    assert!((result.score.unwrap() - 0.85).abs() < f32::EPSILON);
    assert!((result.dist.unwrap() - 0.15).abs() < f32::EPSILON);
}

#[test]
fn zep_search_result_with_summary() {
    let json = r#"{
        "summary": {"content": "Search summary", "uuid": "sum-search"},
        "score": 0.75
    }"#;

    let result: ZepSearchResult =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert!(result.message.is_none());
    assert!(result.summary.is_some());
    assert_eq!(result.summary.as_ref().unwrap().content, "Search summary");
}

#[test]
fn zep_session_deserialization() {
    let json = r#"{
        "uuid": "session-uuid",
        "session_id": "my-session",
        "user_id": "my-user",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-02T00:00:00Z"
    }"#;

    let session: ZepSession =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(session.uuid.as_deref(), Some("session-uuid"));
    assert_eq!(session.session_id, "my-session");
    assert_eq!(session.user_id.as_deref(), Some("my-user"));
}

#[test]
fn zep_session_minimal() {
    let json = r#"{"session_id": "minimal-session"}"#;

    let session: ZepSession =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(session.session_id, "minimal-session");
    assert!(session.uuid.is_none());
    assert!(session.user_id.is_none());
}

#[test]
fn zep_user_deserialization() {
    let json = r#"{
        "uuid": "user-uuid",
        "user_id": "user-123",
        "email": "test@example.com",
        "first_name": "Test",
        "last_name": "User",
        "created_at": "2024-01-01T00:00:00Z"
    }"#;

    let user: ZepUser = serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(user.uuid.as_deref(), Some("user-uuid"));
    assert_eq!(user.user_id, "user-123");
    assert_eq!(user.email.as_deref(), Some("test@example.com"));
    assert_eq!(user.first_name.as_deref(), Some("Test"));
    assert_eq!(user.last_name.as_deref(), Some("User"));
}

#[test]
fn zep_user_minimal() {
    let json = r#"{"user_id": "minimal-user-id"}"#;

    let user: ZepUser = serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(user.user_id, "minimal-user-id");
    assert!(user.uuid.is_none());
    assert!(user.email.is_none());
}

#[test]
fn zep_message_response_deserialization() {
    let json = r#"{
        "uuid": "msg-uuid",
        "role": "user",
        "content": "Message content",
        "role_type": "human",
        "token_count": 10,
        "created_at": "2024-01-01T00:00:00Z"
    }"#;

    let response: ZepMessageResponse =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(response.uuid.as_deref(), Some("msg-uuid"));
    assert_eq!(response.role, "user");
    assert_eq!(response.content, "Message content");
    assert_eq!(response.role_type.as_deref(), Some("human"));
    assert_eq!(response.token_count, Some(10));
}
