#[cfg(feature = "mem0")]
mod mem0_tests {
    use swissknife_memory_sdk::mem0::{
        Mem0Client, Mem0Message, AddOptions, SearchParams, HistoryParams,
    };

    #[test]
    fn test_mem0_client_creation() {
        let client = Mem0Client::new("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_mem0_client_with_base_url() {
        let client = Mem0Client::with_base_url("test-api-key", "https://custom.mem0.ai");
        assert!(true);
    }

    #[test]
    fn test_mem0_message_construction() {
        let message = Mem0Message {
            role: "user".to_string(),
            content: "Remember that I prefer dark mode.".to_string(),
        };

        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Remember that I prefer dark mode.");
    }

    #[test]
    fn test_add_options_builder() {
        let options = AddOptions {
            user_id: Some("user-123".to_string()),
            agent_id: Some("agent-456".to_string()),
            run_id: Some("run-789".to_string()),
            metadata: None,
            filters: None,
            prompt: None,
        };

        assert_eq!(options.user_id, Some("user-123".to_string()));
        assert_eq!(options.agent_id, Some("agent-456".to_string()));
        assert_eq!(options.run_id, Some("run-789".to_string()));
    }

    #[test]
    fn test_search_params_defaults() {
        let params = SearchParams::default();
        assert!(params.user_id.is_none());
        assert!(params.agent_id.is_none());
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_history_params() {
        let params = HistoryParams {
            memory_id: "mem-123".to_string(),
        };

        assert_eq!(params.memory_id, "mem-123");
    }
}

#[cfg(feature = "zep")]
mod zep_tests {
    use swissknife_memory_sdk::zep::{
        ZepClient, ZepMessage, CreateSessionRequest, CreateUserRequest, UpdateUserRequest,
        GetMemoryParams, SearchMemoryParams, ListSessionsParams, ListUsersParams,
    };
    use std::collections::HashMap;

    #[test]
    fn test_zep_client_creation() {
        let client = ZepClient::new("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_zep_client_with_base_url() {
        let client = ZepClient::with_base_url("test-api-key", "https://custom.zep.ai");
        assert!(true);
    }

    #[test]
    fn test_zep_message_construction() {
        let message = ZepMessage {
            role: "user".to_string(),
            content: "Hello, I'm testing the Zep integration.".to_string(),
            role_type: Some("human".to_string()),
            metadata: None,
        };

        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello, I'm testing the Zep integration.");
        assert_eq!(message.role_type, Some("human".to_string()));
    }

    #[test]
    fn test_create_session_request() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), serde_json::json!("value"));

        let request = CreateSessionRequest {
            session_id: "session-123".to_string(),
            user_id: Some("user-456".to_string()),
            metadata: Some(metadata),
        };

        assert_eq!(request.session_id, "session-123");
        assert_eq!(request.user_id, Some("user-456".to_string()));
        assert!(request.metadata.is_some());
    }

    #[test]
    fn test_create_user_request() {
        let request = CreateUserRequest {
            user_id: "user-123".to_string(),
            email: Some("test@example.com".to_string()),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            metadata: None,
        };

        assert_eq!(request.user_id, "user-123");
        assert_eq!(request.email, Some("test@example.com".to_string()));
        assert_eq!(request.first_name, Some("Test".to_string()));
    }

    #[test]
    fn test_update_user_request() {
        let request = UpdateUserRequest {
            email: Some("newemail@example.com".to_string()),
            first_name: None,
            last_name: None,
            metadata: None,
        };

        assert_eq!(request.email, Some("newemail@example.com".to_string()));
        assert!(request.first_name.is_none());
    }

    #[test]
    fn test_get_memory_params() {
        let params = GetMemoryParams {
            lastn: Some(10),
            min_rating: Some(0.5),
        };

        assert_eq!(params.lastn, Some(10));
        assert_eq!(params.min_rating, Some(0.5));
    }

    #[test]
    fn test_search_memory_params() {
        let params = SearchMemoryParams {
            limit: Some(5),
            min_score: Some(0.8),
            search_type: Some("similarity".to_string()),
            search_scope: Some("messages".to_string()),
        };

        assert_eq!(params.limit, Some(5));
        assert_eq!(params.min_score, Some(0.8));
        assert_eq!(params.search_type, Some("similarity".to_string()));
    }

    #[test]
    fn test_list_sessions_params() {
        let params = ListSessionsParams {
            limit: Some(20),
            cursor: Some(100),
        };

        assert_eq!(params.limit, Some(20));
        assert_eq!(params.cursor, Some(100));
    }

    #[test]
    fn test_list_users_params() {
        let params = ListUsersParams {
            limit: Some(50),
            cursor: Some(200),
        };

        assert_eq!(params.limit, Some(50));
        assert_eq!(params.cursor, Some(200));
    }

    #[test]
    fn test_params_defaults() {
        let memory_params = GetMemoryParams::default();
        let search_params = SearchMemoryParams::default();
        let session_params = ListSessionsParams::default();

        assert!(memory_params.lastn.is_none());
        assert!(search_params.limit.is_none());
        assert!(session_params.cursor.is_none());
    }
}

mod error_tests {
    use swissknife_memory_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Rate limit exceeded".to_string(),
            code: Some("429".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Rate limit exceeded"));
    }

    #[test]
    fn test_session_not_found_error() {
        let error = Error::SessionNotFound("session-123".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("session-123"));
    }

    #[test]
    fn test_user_not_found_error() {
        let error = Error::UserNotFound("user-456".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("user-456"));
    }

    #[test]
    fn test_memory_not_found_error() {
        let error = Error::MemoryNotFound("mem-789".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("mem-789"));
    }
}
