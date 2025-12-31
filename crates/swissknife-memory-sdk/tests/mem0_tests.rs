#![cfg(feature = "mem0")]

use std::collections::HashMap;
use swissknife_memory_sdk::mem0::{
    AddOptions, AddedMemory, DeleteAllParams, GetAllParams, Mem0Client, Mem0Memory,
    Mem0Message, Mem0SearchResult, MemoryHistory, SearchParams,
};
use swissknife_memory_sdk::{Memory, SearchResult};

#[test]
fn client_initialization_succeeds_with_valid_api_key() {
    let _client = Mem0Client::new("test-api-key-12345");
}

#[test]
fn client_initialization_with_empty_api_key() {
    let _client = Mem0Client::new("");
}

#[test]
fn client_with_custom_base_url_succeeds() {
    let _client = Mem0Client::with_base_url("key", "https://custom.api.com/v2");
}

#[test]
fn client_with_trailing_slash_base_url() {
    let _client = Mem0Client::with_base_url("key", "https://api.example.com/");
}

#[test]
fn client_clone_creates_independent_instance() {
    let client1 = Mem0Client::new("key1");
    let _client2 = Mem0Client::new("key2");
    let _client3 = Mem0Client::with_base_url("key3", "https://example.com");
    drop(client1);
}

#[test]
fn mem0_message_construction() {
    let message = Mem0Message {
        role: "user".to_string(),
        content: "Remember my preference for vim".to_string(),
    };

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Remember my preference for vim");
}

#[test]
fn mem0_message_serialization() {
    let message = Mem0Message {
        role: "assistant".to_string(),
        content: "I'll remember that".to_string(),
    };

    let json = serde_json::to_value(&message).expect("serialization should succeed");
    assert_eq!(json["role"], "assistant");
    assert_eq!(json["content"], "I'll remember that");
}

#[test]
fn add_options_default_all_none() {
    let options = AddOptions::default();

    assert!(options.user_id.is_none());
    assert!(options.agent_id.is_none());
    assert!(options.run_id.is_none());
    assert!(options.metadata.is_none());
    assert!(options.output_format.is_none());
}

#[test]
fn add_options_with_all_fields() {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!("test"));

    let options = AddOptions {
        user_id: Some("user-add".to_string()),
        agent_id: Some("agent-add".to_string()),
        run_id: Some("run-add".to_string()),
        metadata: Some(metadata),
        output_format: Some("v1.1".to_string()),
    };

    assert_eq!(options.user_id.as_deref(), Some("user-add"));
    assert_eq!(options.agent_id.as_deref(), Some("agent-add"));
    assert_eq!(options.run_id.as_deref(), Some("run-add"));
    assert!(options.metadata.is_some());
    assert_eq!(options.output_format.as_deref(), Some("v1.1"));
}

#[test]
fn search_params_default_all_none() {
    let params = SearchParams::default();

    assert!(params.user_id.is_none());
    assert!(params.agent_id.is_none());
    assert!(params.run_id.is_none());
    assert!(params.limit.is_none());
}

#[test]
fn search_params_with_limit() {
    let params = SearchParams {
        user_id: Some("user-search".to_string()),
        agent_id: None,
        run_id: None,
        limit: Some(25),
    };

    assert_eq!(params.user_id.as_deref(), Some("user-search"));
    assert_eq!(params.limit, Some(25));
}

#[test]
fn get_all_params_default() {
    let params = GetAllParams::default();

    assert!(params.user_id.is_none());
    assert!(params.agent_id.is_none());
    assert!(params.run_id.is_none());
}

#[test]
fn get_all_params_with_filters() {
    let params = GetAllParams {
        user_id: Some("filter-user".to_string()),
        agent_id: Some("filter-agent".to_string()),
        run_id: Some("filter-run".to_string()),
    };

    assert_eq!(params.user_id.as_deref(), Some("filter-user"));
    assert_eq!(params.agent_id.as_deref(), Some("filter-agent"));
    assert_eq!(params.run_id.as_deref(), Some("filter-run"));
}

#[test]
fn delete_all_params_default() {
    let params = DeleteAllParams::default();

    assert!(params.user_id.is_none());
    assert!(params.agent_id.is_none());
    assert!(params.run_id.is_none());
}

#[test]
fn delete_all_params_with_user_id() {
    let params = DeleteAllParams {
        user_id: Some("delete-user".to_string()),
        agent_id: None,
        run_id: None,
    };

    assert_eq!(params.user_id.as_deref(), Some("delete-user"));
}

#[test]
fn mem0_memory_to_memory_conversion() {
    let mut metadata = HashMap::new();
    metadata.insert("converted".to_string(), serde_json::json!(true));

    let mem0_memory = Mem0Memory {
        id: "mem0-id".to_string(),
        memory: "Converted memory content".to_string(),
        user_id: Some("converted-user".to_string()),
        agent_id: Some("converted-agent".to_string()),
        run_id: Some("converted-run".to_string()),
        hash: Some("abc123".to_string()),
        metadata: Some(metadata),
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
        updated_at: Some("2024-01-02T00:00:00Z".to_string()),
    };

    let memory: Memory = mem0_memory.into();

    assert_eq!(memory.id, "mem0-id");
    assert_eq!(memory.content, "Converted memory content");
    assert_eq!(memory.user_id.as_deref(), Some("converted-user"));
    assert_eq!(memory.agent_id.as_deref(), Some("converted-agent"));
    assert_eq!(memory.session_id.as_deref(), Some("converted-run"));
    assert!(memory.metadata.contains_key("converted"));
}

#[test]
fn mem0_memory_to_memory_with_none_metadata() {
    let mem0_memory = Mem0Memory {
        id: "no-meta".to_string(),
        memory: "No metadata".to_string(),
        user_id: None,
        agent_id: None,
        run_id: None,
        hash: None,
        metadata: None,
        created_at: None,
        updated_at: None,
    };

    let memory: Memory = mem0_memory.into();

    assert!(memory.metadata.is_empty());
}

#[test]
fn mem0_search_result_to_search_result_conversion() {
    let mem0_result = Mem0SearchResult {
        id: "search-id".to_string(),
        memory: "Search result content".to_string(),
        score: 0.92,
        user_id: Some("search-user".to_string()),
        agent_id: None,
        run_id: None,
        metadata: None,
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
    };

    let result: SearchResult = mem0_result.into();

    assert_eq!(result.memory.id, "search-id");
    assert_eq!(result.memory.content, "Search result content");
    assert!((result.score - 0.92).abs() < f32::EPSILON);
    assert_eq!(result.memory.user_id.as_deref(), Some("search-user"));
}

#[test]
fn mem0_search_result_updated_at_is_none_after_conversion() {
    let mem0_result = Mem0SearchResult {
        id: "no-update".to_string(),
        memory: "No update time".to_string(),
        score: 0.5,
        user_id: None,
        agent_id: None,
        run_id: None,
        metadata: None,
        created_at: None,
    };

    let result: SearchResult = mem0_result.into();

    assert!(result.memory.updated_at.is_none());
}

#[test]
fn added_memory_deserialization() {
    let json = r#"{"id": "added-123", "memory": "Added content", "event": "ADD"}"#;
    let added: AddedMemory = serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(added.id, "added-123");
    assert_eq!(added.memory, "Added content");
    assert_eq!(added.event, "ADD");
}

#[test]
fn memory_history_deserialization() {
    let json = r#"{
        "id": "history-1",
        "memory_id": "mem-1",
        "prev_value": "old value",
        "new_value": "new value",
        "event": "UPDATE",
        "timestamp": "2024-01-01T00:00:00Z"
    }"#;

    let history: MemoryHistory =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(history.id, "history-1");
    assert_eq!(history.memory_id, "mem-1");
    assert_eq!(history.prev_value.as_deref(), Some("old value"));
    assert_eq!(history.new_value.as_deref(), Some("new value"));
    assert_eq!(history.event, "UPDATE");
}

#[test]
fn memory_history_with_null_values() {
    let json = r#"{
        "id": "history-null",
        "memory_id": "mem-null",
        "prev_value": null,
        "new_value": "created value",
        "event": "ADD",
        "timestamp": "2024-01-01T00:00:00Z"
    }"#;

    let history: MemoryHistory =
        serde_json::from_str(json).expect("deserialization should succeed");

    assert!(history.prev_value.is_none());
    assert!(history.new_value.is_some());
}
