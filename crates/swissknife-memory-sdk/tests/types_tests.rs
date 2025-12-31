use std::collections::HashMap;
use swissknife_memory_sdk::{AddMemoryOptions, Memory, Message, SearchOptions, SearchResult};

#[test]
fn memory_construction_with_all_fields() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), serde_json::json!("value"));
    metadata.insert("number".to_string(), serde_json::json!(42));

    let memory = Memory {
        id: "mem-001".to_string(),
        content: "User prefers dark mode".to_string(),
        user_id: Some("user-123".to_string()),
        agent_id: Some("agent-456".to_string()),
        session_id: Some("session-789".to_string()),
        metadata: metadata.clone(),
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
        updated_at: Some("2024-01-02T00:00:00Z".to_string()),
    };

    assert_eq!(memory.id, "mem-001");
    assert_eq!(memory.content, "User prefers dark mode");
    assert_eq!(memory.user_id.as_deref(), Some("user-123"));
    assert_eq!(memory.agent_id.as_deref(), Some("agent-456"));
    assert_eq!(memory.session_id.as_deref(), Some("session-789"));
    assert_eq!(memory.metadata.get("key"), Some(&serde_json::json!("value")));
    assert_eq!(memory.metadata.get("number"), Some(&serde_json::json!(42)));
    assert!(memory.created_at.is_some());
    assert!(memory.updated_at.is_some());
}

#[test]
fn memory_construction_with_minimal_fields() {
    let memory = Memory {
        id: "mem-minimal".to_string(),
        content: "Minimal memory".to_string(),
        user_id: None,
        agent_id: None,
        session_id: None,
        metadata: HashMap::new(),
        created_at: None,
        updated_at: None,
    };

    assert_eq!(memory.id, "mem-minimal");
    assert_eq!(memory.content, "Minimal memory");
    assert!(memory.user_id.is_none());
    assert!(memory.agent_id.is_none());
    assert!(memory.session_id.is_none());
    assert!(memory.metadata.is_empty());
    assert!(memory.created_at.is_none());
    assert!(memory.updated_at.is_none());
}

#[test]
fn memory_clone_creates_independent_copy() {
    let mut metadata = HashMap::new();
    metadata.insert("test".to_string(), serde_json::json!(true));

    let original = Memory {
        id: "original".to_string(),
        content: "Original content".to_string(),
        user_id: Some("user".to_string()),
        agent_id: None,
        session_id: None,
        metadata,
        created_at: None,
        updated_at: None,
    };

    let cloned = original.clone();
    assert_eq!(original.id, cloned.id);
    assert_eq!(original.content, cloned.content);
    assert_eq!(original.metadata, cloned.metadata);
}

#[test]
fn memory_debug_format_contains_fields() {
    let memory = Memory {
        id: "debug-test".to_string(),
        content: "Debug content".to_string(),
        user_id: None,
        agent_id: None,
        session_id: None,
        metadata: HashMap::new(),
        created_at: None,
        updated_at: None,
    };

    let debug_output = format!("{:?}", memory);
    assert!(debug_output.contains("debug-test"));
    assert!(debug_output.contains("Debug content"));
}

#[test]
fn memory_serialization_roundtrip() {
    let mut metadata = HashMap::new();
    metadata.insert("serialized".to_string(), serde_json::json!(true));

    let memory = Memory {
        id: "serialize-test".to_string(),
        content: "Serializable content".to_string(),
        user_id: Some("user".to_string()),
        agent_id: None,
        session_id: None,
        metadata,
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
        updated_at: None,
    };

    let serialized = serde_json::to_string(&memory).expect("serialization should succeed");
    let deserialized: Memory =
        serde_json::from_str(&serialized).expect("deserialization should succeed");

    assert_eq!(memory.id, deserialized.id);
    assert_eq!(memory.content, deserialized.content);
    assert_eq!(memory.user_id, deserialized.user_id);
    assert_eq!(memory.metadata, deserialized.metadata);
}

#[test]
fn message_construction_with_all_fields() {
    let mut metadata = HashMap::new();
    metadata.insert("intent".to_string(), serde_json::json!("greeting"));

    let message = Message {
        role: "user".to_string(),
        content: "Hello, how are you?".to_string(),
        metadata: Some(metadata.clone()),
        timestamp: Some("2024-01-01T12:00:00Z".to_string()),
    };

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello, how are you?");
    assert!(message.metadata.is_some());
    assert_eq!(
        message.metadata.as_ref().unwrap().get("intent"),
        Some(&serde_json::json!("greeting"))
    );
    assert_eq!(
        message.timestamp.as_deref(),
        Some("2024-01-01T12:00:00Z")
    );
}

#[test]
fn message_role_accepts_standard_values() {
    let user_message = Message {
        role: "user".to_string(),
        content: "User message".to_string(),
        metadata: None,
        timestamp: None,
    };
    assert_eq!(user_message.role, "user");

    let assistant_message = Message {
        role: "assistant".to_string(),
        content: "Assistant message".to_string(),
        metadata: None,
        timestamp: None,
    };
    assert_eq!(assistant_message.role, "assistant");

    let system_message = Message {
        role: "system".to_string(),
        content: "System message".to_string(),
        metadata: None,
        timestamp: None,
    };
    assert_eq!(system_message.role, "system");
}

#[test]
fn message_role_accepts_custom_values() {
    let custom_message = Message {
        role: "custom_role".to_string(),
        content: "Custom role message".to_string(),
        metadata: None,
        timestamp: None,
    };
    assert_eq!(custom_message.role, "custom_role");
}

#[test]
fn message_serialization_roundtrip() {
    let message = Message {
        role: "user".to_string(),
        content: "Test message".to_string(),
        metadata: None,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
    };

    let serialized = serde_json::to_string(&message).expect("serialization should succeed");
    let deserialized: Message =
        serde_json::from_str(&serialized).expect("deserialization should succeed");

    assert_eq!(message.role, deserialized.role);
    assert_eq!(message.content, deserialized.content);
    assert_eq!(message.timestamp, deserialized.timestamp);
}

#[test]
fn add_memory_options_default_values() {
    let options = AddMemoryOptions::default();

    assert!(options.user_id.is_none());
    assert!(options.agent_id.is_none());
    assert!(options.session_id.is_none());
    assert!(options.metadata.is_empty());
}

#[test]
fn add_memory_options_builder_pattern() {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!("api"));

    let options = AddMemoryOptions {
        user_id: Some("user-builder".to_string()),
        agent_id: Some("agent-builder".to_string()),
        session_id: Some("session-builder".to_string()),
        metadata,
    };

    assert_eq!(options.user_id.as_deref(), Some("user-builder"));
    assert_eq!(options.agent_id.as_deref(), Some("agent-builder"));
    assert_eq!(options.session_id.as_deref(), Some("session-builder"));
    assert_eq!(
        options.metadata.get("source"),
        Some(&serde_json::json!("api"))
    );
}

#[test]
fn add_memory_options_clone_independence() {
    let mut metadata = HashMap::new();
    metadata.insert("cloned".to_string(), serde_json::json!(true));

    let original = AddMemoryOptions {
        user_id: Some("original-user".to_string()),
        agent_id: None,
        session_id: None,
        metadata,
    };

    let cloned = original.clone();
    assert_eq!(original.user_id, cloned.user_id);
    assert_eq!(original.metadata, cloned.metadata);
}

#[test]
fn search_options_default_values() {
    let options = SearchOptions::default();

    assert!(options.user_id.is_none());
    assert!(options.agent_id.is_none());
    assert!(options.session_id.is_none());
    assert!(options.limit.is_none());
    assert!(options.threshold.is_none());
}

#[test]
fn search_options_builder_pattern_with_all_fields() {
    let options = SearchOptions {
        user_id: Some("search-user".to_string()),
        agent_id: Some("search-agent".to_string()),
        session_id: Some("search-session".to_string()),
        limit: Some(10),
        threshold: Some(0.75),
    };

    assert_eq!(options.user_id.as_deref(), Some("search-user"));
    assert_eq!(options.agent_id.as_deref(), Some("search-agent"));
    assert_eq!(options.session_id.as_deref(), Some("search-session"));
    assert_eq!(options.limit, Some(10));
    assert_eq!(options.threshold, Some(0.75));
}

#[test]
fn search_options_limit_boundary_values() {
    let zero_limit = SearchOptions {
        user_id: None,
        agent_id: None,
        session_id: None,
        limit: Some(0),
        threshold: None,
    };
    assert_eq!(zero_limit.limit, Some(0));

    let max_limit = SearchOptions {
        user_id: None,
        agent_id: None,
        session_id: None,
        limit: Some(u32::MAX),
        threshold: None,
    };
    assert_eq!(max_limit.limit, Some(u32::MAX));
}

#[test]
fn search_options_threshold_boundary_values() {
    let zero_threshold = SearchOptions {
        user_id: None,
        agent_id: None,
        session_id: None,
        limit: None,
        threshold: Some(0.0),
    };
    assert_eq!(zero_threshold.threshold, Some(0.0));

    let max_threshold = SearchOptions {
        user_id: None,
        agent_id: None,
        session_id: None,
        limit: None,
        threshold: Some(1.0),
    };
    assert_eq!(max_threshold.threshold, Some(1.0));
}

#[test]
fn search_result_with_high_score() {
    let memory = Memory {
        id: "high-score".to_string(),
        content: "High relevance content".to_string(),
        user_id: None,
        agent_id: None,
        session_id: None,
        metadata: HashMap::new(),
        created_at: None,
        updated_at: None,
    };

    let result = SearchResult {
        memory,
        score: 0.98,
    };

    assert_eq!(result.memory.id, "high-score");
    assert!(result.score > 0.9);
}

#[test]
fn search_result_with_low_score() {
    let memory = Memory {
        id: "low-score".to_string(),
        content: "Low relevance content".to_string(),
        user_id: None,
        agent_id: None,
        session_id: None,
        metadata: HashMap::new(),
        created_at: None,
        updated_at: None,
    };

    let result = SearchResult {
        memory,
        score: 0.15,
    };

    assert_eq!(result.memory.id, "low-score");
    assert!(result.score < 0.2);
}

#[test]
fn search_result_score_comparison() {
    let memory1 = Memory {
        id: "mem1".to_string(),
        content: "First".to_string(),
        user_id: None,
        agent_id: None,
        session_id: None,
        metadata: HashMap::new(),
        created_at: None,
        updated_at: None,
    };

    let memory2 = Memory {
        id: "mem2".to_string(),
        content: "Second".to_string(),
        user_id: None,
        agent_id: None,
        session_id: None,
        metadata: HashMap::new(),
        created_at: None,
        updated_at: None,
    };

    let result1 = SearchResult {
        memory: memory1,
        score: 0.85,
    };

    let result2 = SearchResult {
        memory: memory2,
        score: 0.65,
    };

    assert!(result1.score > result2.score);
}

#[test]
fn search_result_serialization_roundtrip() {
    let memory = Memory {
        id: "serialize-search".to_string(),
        content: "Serializable search result".to_string(),
        user_id: Some("user".to_string()),
        agent_id: None,
        session_id: None,
        metadata: HashMap::new(),
        created_at: None,
        updated_at: None,
    };

    let result = SearchResult {
        memory,
        score: 0.77,
    };

    let serialized = serde_json::to_string(&result).expect("serialization should succeed");
    let deserialized: SearchResult =
        serde_json::from_str(&serialized).expect("deserialization should succeed");

    assert_eq!(result.memory.id, deserialized.memory.id);
    assert!((result.score - deserialized.score).abs() < f32::EPSILON);
}
