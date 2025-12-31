#![cfg(feature = "duckdb")]

use swissknife_ai_sdk::memory::{ActionType, DuckDBMemory, MemoryConfig};

fn create_test_memory() -> DuckDBMemory {
    let config = MemoryConfig::new().with_embedding_dim(128);
    DuckDBMemory::in_memory(config).expect("Failed to create in-memory database")
}

#[test]
fn test_create_session_returns_id() {
    let memory = create_test_memory();
    let id = memory.create_session("session-1", Some("Test Session"));
    assert!(id.is_ok());
    assert!(!id.unwrap().is_empty());
}

#[test]
fn test_get_session_returns_none_for_unknown() {
    let memory = create_test_memory();
    let session = memory.get_session("nonexistent").unwrap();
    assert!(session.is_none());
}

#[test]
fn test_add_message_returns_id() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let action_id = memory.add_message("session-1", "user", "Hello world").unwrap();
    assert!(!action_id.is_empty());
}

#[test]
fn test_add_tool_call_returns_id() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let id = memory.add_tool_call("session-1", "search", r#"{"query":"test"}"#, "call_123").unwrap();
    assert!(!id.is_empty());
}

#[test]
fn test_add_tool_result_returns_id() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let id = memory.add_tool_result("session-1", "call_123", "Result data").unwrap();
    assert!(!id.is_empty());
}

#[test]
fn test_add_thinking_returns_id() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let id = memory.add_thinking("session-1", "Let me think about this...").unwrap();
    assert!(!id.is_empty());
}

#[test]
fn test_action_count_starts_at_zero() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    assert_eq!(memory.action_count("session-1").unwrap(), 0);
}

#[test]
fn test_action_count_increments_with_messages() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    memory.add_message("session-1", "user", "Hello").unwrap();
    assert_eq!(memory.action_count("session-1").unwrap(), 1);
    memory.add_message("session-1", "assistant", "Hi").unwrap();
    assert_eq!(memory.action_count("session-1").unwrap(), 2);
}

#[test]
fn test_action_count_increments_with_tool_calls() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    memory.add_tool_call("session-1", "search", "{}", "call_1").unwrap();
    assert_eq!(memory.action_count("session-1").unwrap(), 1);
}

#[test]
fn test_add_embedding_validates_dimension_too_small() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let action_id = memory.add_message("session-1", "user", "Hello").unwrap();
    let wrong_dim_embedding: Vec<f32> = vec![0.1; 64];
    let result = memory.add_embedding(&action_id, &wrong_dim_embedding);
    assert!(result.is_err());
}

#[test]
fn test_add_embedding_validates_dimension_too_large() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let action_id = memory.add_message("session-1", "user", "Hello").unwrap();
    let wrong_dim_embedding: Vec<f32> = vec![0.1; 256];
    let result = memory.add_embedding(&action_id, &wrong_dim_embedding);
    assert!(result.is_err());
}

#[test]
fn test_add_embedding_correct_dimension() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let action_id = memory.add_message("session-1", "user", "Hello").unwrap();
    let embedding: Vec<f32> = vec![0.1; 128];
    let result = memory.add_embedding(&action_id, &embedding);
    assert!(result.is_ok());
}

#[test]
fn test_memory_config_defaults() {
    let config = MemoryConfig::new();
    assert!(config.db_path.is_none());
    assert_eq!(config.embedding_dim, 1024);
}

#[test]
fn test_memory_config_with_db_path() {
    let config = MemoryConfig::new().with_db_path("/tmp/test.duckdb");
    assert_eq!(config.db_path, Some("/tmp/test.duckdb".to_string()));
}

#[test]
fn test_memory_config_with_embedding_dim() {
    let config = MemoryConfig::new().with_embedding_dim(512);
    assert_eq!(config.embedding_dim, 512);
}

#[test]
fn test_memory_config_chained() {
    let config = MemoryConfig::new()
        .with_db_path("/tmp/memory.db")
        .with_embedding_dim(768);
    assert_eq!(config.db_path, Some("/tmp/memory.db".to_string()));
    assert_eq!(config.embedding_dim, 768);
}

#[test]
fn test_action_type_as_str() {
    assert_eq!(ActionType::Message.as_str(), "message");
    assert_eq!(ActionType::ToolCall.as_str(), "tool_call");
    assert_eq!(ActionType::ToolResult.as_str(), "tool_result");
    assert_eq!(ActionType::Thinking.as_str(), "thinking");
}

#[test]
fn test_action_type_from_str_valid() {
    assert_eq!(ActionType::from_str("message"), Some(ActionType::Message));
    assert_eq!(ActionType::from_str("tool_call"), Some(ActionType::ToolCall));
    assert_eq!(ActionType::from_str("tool_result"), Some(ActionType::ToolResult));
    assert_eq!(ActionType::from_str("thinking"), Some(ActionType::Thinking));
}

#[test]
fn test_action_type_from_str_invalid() {
    assert_eq!(ActionType::from_str("unknown"), None);
    assert_eq!(ActionType::from_str(""), None);
    assert_eq!(ActionType::from_str("Message"), None);
}

#[test]
fn test_action_type_display() {
    assert_eq!(format!("{}", ActionType::Message), "message");
    assert_eq!(format!("{}", ActionType::ToolCall), "tool_call");
    assert_eq!(format!("{}", ActionType::ToolResult), "tool_result");
    assert_eq!(format!("{}", ActionType::Thinking), "thinking");
}

#[test]
fn test_duckdb_memory_clone_shares_connection() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    let cloned = memory.clone();
    cloned.add_message("session-1", "user", "From clone").unwrap();
    assert_eq!(memory.action_count("session-1").unwrap(), 1);
}

#[test]
fn test_update_session_title_succeeds() {
    let memory = create_test_memory();
    memory.create_session("session-1", Some("Old Title")).unwrap();
    let result = memory.update_session_title("session-1", "New Title");
    assert!(result.is_ok());
}

#[test]
fn test_multiple_sessions_isolated_action_counts() {
    let memory = create_test_memory();
    memory.create_session("session-1", None).unwrap();
    memory.create_session("session-2", None).unwrap();
    memory.add_message("session-1", "user", "Message 1").unwrap();
    memory.add_message("session-1", "user", "Message 2").unwrap();
    memory.add_message("session-2", "user", "Message A").unwrap();
    assert_eq!(memory.action_count("session-1").unwrap(), 2);
    assert_eq!(memory.action_count("session-2").unwrap(), 1);
}

#[test]
fn test_in_memory_database_creation() {
    let config = MemoryConfig::new().with_embedding_dim(256);
    let result = DuckDBMemory::in_memory(config);
    assert!(result.is_ok());
}

#[test]
fn test_action_count_for_nonexistent_session() {
    let memory = create_test_memory();
    let count = memory.action_count("nonexistent").unwrap();
    assert_eq!(count, 0);
}
