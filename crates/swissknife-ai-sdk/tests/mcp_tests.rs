use serde_json::json;

#[cfg(feature = "mcp")]
mod mcp_types_tests {
    use super::*;
    use swissknife_ai_sdk::mcp::*;

    #[test]
    fn test_request_id_string() {
        let id = RequestId::String("test-123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"test-123\"");

        let parsed: RequestId = serde_json::from_str(&json).unwrap();
        match parsed {
            RequestId::String(s) => assert_eq!(s, "test-123"),
            _ => panic!("Expected string ID"),
        }
    }

    #[test]
    fn test_request_id_number() {
        let id = RequestId::Number(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");

        let parsed: RequestId = serde_json::from_str(&json).unwrap();
        match parsed {
            RequestId::Number(n) => assert_eq!(n, 42),
            _ => panic!("Expected number ID"),
        }
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["method"], "tools/list");
        assert!(json.get("params").is_none());
    }

    #[test]
    fn test_json_rpc_request_with_params() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::String("abc".to_string()),
            method: "tools/call".to_string(),
            params: Some(json!({"name": "test_tool", "arguments": {}})),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["params"]["name"], "test_tool");
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            result: Some(json!({"status": "ok"})),
            error: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["result"]["status"], "ok");
        assert!(json.get("error").is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            result: None,
            error: Some(JsonRpcError::method_not_found("Unknown method")),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["error"]["code"], -32601);
        assert_eq!(json["error"]["message"], "Unknown method");
    }

    #[test]
    fn test_json_rpc_error_codes() {
        let parse_err = JsonRpcError::parse_error("Parse failed");
        assert_eq!(parse_err.code, -32700);

        let invalid_req = JsonRpcError::invalid_request("Bad request");
        assert_eq!(invalid_req.code, -32600);

        let not_found = JsonRpcError::method_not_found("Not found");
        assert_eq!(not_found.code, -32601);

        let invalid_params = JsonRpcError::invalid_params("Bad params");
        assert_eq!(invalid_params.code, -32602);

        let internal = JsonRpcError::internal_error("Internal error");
        assert_eq!(internal.code, -32603);
    }

    #[test]
    fn test_tool_definition_builder() {
        let tool = ToolDefinition::new("my_tool")
            .with_description("A test tool")
            .with_schema(json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                },
                "required": ["input"]
            }));

        assert_eq!(tool.name, "my_tool");
        assert_eq!(tool.description, Some("A test tool".to_string()));
        assert_eq!(tool.input_schema["properties"]["input"]["type"], "string");
    }

    #[test]
    fn test_tool_definition_minimal() {
        let tool = ToolDefinition::new("simple_tool");

        assert_eq!(tool.name, "simple_tool");
        assert!(tool.description.is_none());
        assert_eq!(tool.input_schema["type"], "object");
    }

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("Hello, world!");

        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_result_json() {
        let result = ToolResult::json(json!({"key": "value", "number": 42}));

        assert!(!result.is_error);
        match &result.content[0] {
            ToolContent::Text { text } => {
                let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
                assert_eq!(parsed["key"], "value");
                assert_eq!(parsed["number"], 42);
            },
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");

        assert!(result.is_error);
        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Something went wrong"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_result_image() {
        let result = ToolResult::image("base64data", "image/png");

        assert!(!result.is_error);
        match &result.content[0] {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "base64data");
                assert_eq!(mime_type, "image/png");
            },
            _ => panic!("Expected image content"),
        }
    }

    #[test]
    fn test_tool_content_serialization() {
        let text_content = ToolContent::Text { text: "Hello".to_string() };
        let json = serde_json::to_value(&text_content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello");

        let image_content = ToolContent::Image {
            data: "abc123".to_string(),
            mime_type: "image/jpeg".to_string(),
        };
        let json = serde_json::to_value(&image_content).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["data"], "abc123");
        assert_eq!(json["mimeType"], "image/jpeg");
    }

    #[test]
    fn test_server_capabilities() {
        let caps = ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: Some(true) }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            prompts: Some(PromptsCapability { list_changed: Some(true) }),
            logging: Some(LoggingCapability {}),
        };

        let json = serde_json::to_value(&caps).unwrap();
        assert!(json["tools"]["listChanged"].as_bool().unwrap());
        assert!(json["resources"]["subscribe"].as_bool().unwrap());
        assert!(json["prompts"]["listChanged"].as_bool().unwrap());
    }

    #[test]
    fn test_initialize_params() {
        let params = InitializeParams {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ClientCapabilities {
                roots: Some(RootsCapability { list_changed: Some(true) }),
                sampling: None,
            },
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["protocolVersion"], MCP_VERSION);
        assert_eq!(json["clientInfo"]["name"], "test-client");
    }

    #[test]
    fn test_initialize_result() {
        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability::default()),
                resources: None,
                prompts: None,
                logging: None,
            },
            server_info: ServerInfo {
                name: "test-server".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some("Use these tools wisely".to_string()),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["serverInfo"]["name"], "test-server");
        assert_eq!(json["instructions"], "Use these tools wisely");
    }

    #[test]
    fn test_resource_definition() {
        let resource = ResourceDefinition {
            uri: "file:///path/to/file.txt".to_string(),
            name: "My File".to_string(),
            description: Some("A text file".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["uri"], "file:///path/to/file.txt");
        assert_eq!(json["mimeType"], "text/plain");
    }

    #[test]
    fn test_prompt_definition() {
        let prompt = PromptDefinition {
            name: "code_review".to_string(),
            description: Some("Review code for issues".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "code".to_string(),
                    description: Some("The code to review".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language".to_string()),
                    required: false,
                },
            ]),
        };

        let json = serde_json::to_value(&prompt).unwrap();
        assert_eq!(json["name"], "code_review");
        assert_eq!(json["arguments"][0]["name"], "code");
        assert!(json["arguments"][0]["required"].as_bool().unwrap());
    }

    #[test]
    fn test_prompt_message_content() {
        let text_content = PromptMessageContent::Text {
            text: "Hello".to_string()
        };
        let json = serde_json::to_value(&text_content).unwrap();
        assert_eq!(json["type"], "text");

        let image_content = PromptMessageContent::Image {
            data: "base64".to_string(),
            mime_type: "image/png".to_string(),
        };
        let json = serde_json::to_value(&image_content).unwrap();
        assert_eq!(json["type"], "image");
    }

    #[test]
    fn test_log_level_serialization() {
        let levels = vec![
            (LogLevel::Debug, "debug"),
            (LogLevel::Info, "info"),
            (LogLevel::Notice, "notice"),
            (LogLevel::Warning, "warning"),
            (LogLevel::Error, "error"),
            (LogLevel::Critical, "critical"),
            (LogLevel::Alert, "alert"),
            (LogLevel::Emergency, "emergency"),
        ];

        for (level, expected) in levels {
            let json = serde_json::to_value(&level).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_call_tool_params() {
        let params = CallToolParams {
            name: "my_tool".to_string(),
            arguments: json!({"input": "test"}),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["name"], "my_tool");
        assert_eq!(json["arguments"]["input"], "test");
    }

    #[test]
    fn test_list_tools_result() {
        let result = ListToolsResult {
            tools: vec![
                ToolDefinition::new("tool1"),
                ToolDefinition::new("tool2"),
            ],
            next_cursor: Some("cursor123".to_string()),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["tools"].as_array().unwrap().len(), 2);
        assert_eq!(json["nextCursor"], "cursor123");
    }
}

#[cfg(feature = "mcp")]
mod mcp_router_tests {
    use super::*;
    use swissknife_ai_sdk::mcp::*;
    use swissknife_ai_sdk::Result;
    use async_trait::async_trait;

    struct MockToolProvider {
        name: String,
        tools: Vec<ToolDefinition>,
    }

    impl MockToolProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                tools: vec![
                    ToolDefinition::new(format!("{}_tool1", name))
                        .with_description("First tool"),
                    ToolDefinition::new(format!("{}_tool2", name))
                        .with_description("Second tool"),
                ],
            }
        }
    }

    #[async_trait]
    impl ToolProvider for MockToolProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock tool provider for testing"
        }

        fn tools(&self) -> Vec<ToolDefinition> {
            self.tools.clone()
        }

        async fn call(&self, tool_name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
            if tool_name.starts_with(&self.name) {
                Ok(ToolResult::json(json!({
                    "tool": tool_name,
                    "args": arguments,
                    "result": "success"
                })))
            } else {
                Ok(ToolResult::error(format!("Unknown tool: {}", tool_name)))
            }
        }
    }

    struct MockResourceProvider;

    #[async_trait]
    impl ResourceProvider for MockResourceProvider {
        fn name(&self) -> &str {
            "mock_resources"
        }

        fn resources(&self) -> Vec<ResourceDefinition> {
            vec![
                ResourceDefinition {
                    uri: "file:///test/file.txt".to_string(),
                    name: "Test File".to_string(),
                    description: Some("A test file".to_string()),
                    mime_type: Some("text/plain".to_string()),
                },
            ]
        }

        async fn read(&self, uri: &str) -> Result<ResourceContent> {
            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                text: Some("File contents".to_string()),
                blob: None,
            })
        }
    }

    struct MockPromptProvider;

    #[async_trait]
    impl PromptProvider for MockPromptProvider {
        fn name(&self) -> &str {
            "mock_prompts"
        }

        fn prompts(&self) -> Vec<PromptDefinition> {
            vec![
                PromptDefinition {
                    name: "test_prompt".to_string(),
                    description: Some("A test prompt".to_string()),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "topic".to_string(),
                            description: Some("The topic".to_string()),
                            required: true,
                        },
                    ]),
                },
            ]
        }

        async fn get(&self, _name: &str, arguments: serde_json::Value) -> Result<PromptContent> {
            let topic = arguments.get("topic")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            Ok(PromptContent {
                description: Some("Generated prompt".to_string()),
                messages: vec![
                    PromptMessage {
                        role: PromptRole::User,
                        content: PromptMessageContent::Text {
                            text: format!("Please discuss: {}", topic),
                        },
                    },
                ],
            })
        }
    }

    #[test]
    fn test_router_creation() {
        let router = McpRouter::new("test-server", "1.0.0");

        let info = router.server_info();
        assert_eq!(info.name, "test-server");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_router_with_instructions() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_instructions("Use these tools for testing");

        assert_eq!(router.instructions(), Some("Use these tools for testing".to_string()));
    }

    #[test]
    fn test_router_capabilities_empty() {
        let router = McpRouter::new("test-server", "1.0.0");
        let caps = router.capabilities();

        assert!(caps.tools.is_none());
        assert!(caps.resources.is_none());
        assert!(caps.prompts.is_none());
        assert!(caps.logging.is_some());
    }

    #[test]
    fn test_router_capabilities_with_providers() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_tool_provider(MockToolProvider::new("mock"))
            .with_resource_provider(MockResourceProvider)
            .with_prompt_provider(MockPromptProvider);

        let caps = router.capabilities();
        assert!(caps.tools.is_some());
        assert!(caps.resources.is_some());
        assert!(caps.prompts.is_some());
    }

    #[test]
    fn test_router_list_tools() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_tool_provider(MockToolProvider::new("provider1"))
            .with_tool_provider(MockToolProvider::new("provider2"));

        let tools = router.list_tools();
        assert_eq!(tools.len(), 4);

        let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"provider1_tool1"));
        assert!(names.contains(&"provider1_tool2"));
        assert!(names.contains(&"provider2_tool1"));
        assert!(names.contains(&"provider2_tool2"));
    }

    #[test]
    fn test_router_list_resources() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_resource_provider(MockResourceProvider);

        let resources = router.list_resources();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "file:///test/file.txt");
    }

    #[test]
    fn test_router_list_prompts() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_prompt_provider(MockPromptProvider);

        let prompts = router.list_prompts();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "test_prompt");
    }

    #[tokio::test]
    async fn test_router_call_tool() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_tool_provider(MockToolProvider::new("mock"));

        let result = router.call_tool("mock_tool1", json!({"input": "test"})).await.unwrap();

        assert!(!result.is_error);
        match &result.content[0] {
            ToolContent::Text { text } => {
                let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
                assert_eq!(parsed["tool"], "mock_tool1");
                assert_eq!(parsed["result"], "success");
            },
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_router_call_unknown_tool() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_tool_provider(MockToolProvider::new("mock"));

        let result = router.call_tool("nonexistent_tool", json!({})).await.unwrap();

        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_router_read_resource() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_resource_provider(MockResourceProvider);

        let content = router.read_resource("file:///test/file.txt").await.unwrap();

        assert_eq!(content.text, Some("File contents".to_string()));
    }

    #[tokio::test]
    async fn test_router_read_unknown_resource() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_resource_provider(MockResourceProvider);

        let result = router.read_resource("file:///unknown.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_router_get_prompt() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_prompt_provider(MockPromptProvider);

        let content = router.get_prompt("test_prompt", json!({"topic": "Rust"})).await.unwrap();

        assert_eq!(content.messages.len(), 1);
        match &content.messages[0].content {
            PromptMessageContent::Text { text } => {
                assert!(text.contains("Rust"));
            },
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_router_get_unknown_prompt() {
        let router = McpRouter::new("test-server", "1.0.0")
            .with_prompt_provider(MockPromptProvider);

        let result = router.get_prompt("unknown_prompt", json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_router_default() {
        let router = McpRouter::default();
        let info = router.server_info();

        assert_eq!(info.name, "swissknife-mcp");
    }

    #[test]
    fn test_router_add_providers_mutably() {
        let mut router = McpRouter::new("test-server", "1.0.0");

        router.add_tool_provider(MockToolProvider::new("added"));
        router.add_resource_provider(MockResourceProvider);
        router.add_prompt_provider(MockPromptProvider);

        assert_eq!(router.list_tools().len(), 2);
        assert_eq!(router.list_resources().len(), 1);
        assert_eq!(router.list_prompts().len(), 1);
    }
}

#[cfg(feature = "mcp")]
mod mcp_tools_helper_tests {
    use super::*;
    use swissknife_ai_sdk::mcp::tools::{
        schema_object, schema_string,
        get_string_param, get_i64_param, get_f64_param, get_bool_param, get_array_param,
    };

    #[test]
    fn test_schema_string() {
        let schema = schema_string("query", "The search query");

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["query"]["type"], "string");
        assert_eq!(schema["properties"]["query"]["description"], "The search query");
        assert_eq!(schema["required"][0], "query");
    }

    #[test]
    fn test_schema_object() {
        let schema = schema_object(
            json!({
                "name": {"type": "string", "description": "User name"},
                "age": {"type": "integer", "description": "User age"}
            }),
            vec!["name"]
        );

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["name"]["type"], "string");
        assert_eq!(schema["properties"]["age"]["type"], "integer");
        assert_eq!(schema["required"][0], "name");
    }

    #[test]
    fn test_get_string_param() {
        let args = json!({"name": "Alice", "count": 42});

        assert_eq!(get_string_param(&args, "name"), Some("Alice".to_string()));
        assert_eq!(get_string_param(&args, "count"), None);
        assert_eq!(get_string_param(&args, "missing"), None);
    }

    #[test]
    fn test_get_i64_param() {
        let args = json!({"count": 42, "name": "test", "float": 3.14});

        assert_eq!(get_i64_param(&args, "count"), Some(42));
        assert_eq!(get_i64_param(&args, "name"), None);
        assert_eq!(get_i64_param(&args, "missing"), None);
    }

    #[test]
    fn test_get_f64_param() {
        let args = json!({"value": 3.14, "int": 42, "name": "test"});

        assert_eq!(get_f64_param(&args, "value"), Some(3.14));
        assert_eq!(get_f64_param(&args, "int"), Some(42.0));
        assert_eq!(get_f64_param(&args, "name"), None);
    }

    #[test]
    fn test_get_bool_param() {
        let args = json!({"enabled": true, "disabled": false, "name": "test"});

        assert_eq!(get_bool_param(&args, "enabled"), Some(true));
        assert_eq!(get_bool_param(&args, "disabled"), Some(false));
        assert_eq!(get_bool_param(&args, "name"), None);
    }

    #[test]
    fn test_get_array_param() {
        let args = json!({
            "items": [1, 2, 3],
            "strings": ["a", "b"],
            "not_array": "test"
        });

        let items = get_array_param(&args, "items").unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], 1);

        let strings = get_array_param(&args, "strings").unwrap();
        assert_eq!(strings.len(), 2);

        assert!(get_array_param(&args, "not_array").is_none());
        assert!(get_array_param(&args, "missing").is_none());
    }
}

#[cfg(all(feature = "mcp", feature = "llm"))]
mod mcp_llm_tool_provider_tests {
    use super::*;
    use swissknife_ai_sdk::mcp::tools::LlmToolProvider;
    use swissknife_ai_sdk::mcp::ToolProvider;

    #[test]
    fn test_llm_tool_provider_new() {
        let provider = LlmToolProvider::new();

        assert_eq!(provider.name(), "llm");
        assert!(!provider.description().is_empty());
    }

    #[test]
    fn test_llm_tool_provider_no_tools_without_clients() {
        let provider = LlmToolProvider::new();
        let tools = provider.tools();

        assert!(tools.is_empty());
    }
}

#[cfg(all(feature = "mcp", feature = "search"))]
mod mcp_search_tool_provider_tests {
    use swissknife_ai_sdk::mcp::tools::SearchToolProvider;
    use swissknife_ai_sdk::mcp::ToolProvider;

    #[test]
    fn test_search_tool_provider_new() {
        let provider = SearchToolProvider::new();

        assert_eq!(provider.name(), "search");
        assert!(provider.description().contains("search"));
    }

    #[test]
    fn test_search_tool_provider_no_tools_without_clients() {
        let provider = SearchToolProvider::new();
        let tools = provider.tools();

        assert!(tools.is_empty());
    }
}

#[cfg(all(feature = "mcp", feature = "communication"))]
mod mcp_communication_tool_provider_tests {
    use swissknife_ai_sdk::mcp::tools::CommunicationToolProvider;
    use swissknife_ai_sdk::mcp::ToolProvider;

    #[test]
    fn test_communication_tool_provider_new() {
        let provider = CommunicationToolProvider::new();

        assert_eq!(provider.name(), "communication");
        assert!(provider.description().contains("Communication"));
    }

    #[test]
    fn test_communication_tool_provider_default() {
        let provider = CommunicationToolProvider::default();
        let tools = provider.tools();

        assert!(tools.is_empty());
    }
}

#[cfg(all(feature = "mcp", feature = "productivity"))]
mod mcp_productivity_tool_provider_tests {
    use swissknife_ai_sdk::mcp::tools::ProductivityToolProvider;
    use swissknife_ai_sdk::mcp::ToolProvider;

    #[test]
    fn test_productivity_tool_provider_new() {
        let provider = ProductivityToolProvider::new();

        assert_eq!(provider.name(), "productivity");
        assert!(provider.description().contains("Productivity"));
    }
}

#[cfg(all(feature = "mcp", feature = "database"))]
mod mcp_database_tool_provider_tests {
    use swissknife_ai_sdk::mcp::tools::DatabaseToolProvider;
    use swissknife_ai_sdk::mcp::ToolProvider;

    #[test]
    fn test_database_tool_provider_new() {
        let provider = DatabaseToolProvider::new();

        assert_eq!(provider.name(), "database");
        assert!(provider.description().contains("Database"));
    }

    #[test]
    fn test_database_tool_provider_default() {
        let provider = DatabaseToolProvider::default();
        let tools = provider.tools();

        assert!(tools.is_empty());
    }
}

#[cfg(all(feature = "mcp", feature = "memory"))]
mod mcp_memory_tool_provider_tests {
    use swissknife_ai_sdk::mcp::tools::MemoryToolProvider;
    use swissknife_ai_sdk::mcp::ToolProvider;

    #[test]
    fn test_memory_tool_provider_new() {
        let provider = MemoryToolProvider::new();

        assert_eq!(provider.name(), "memory");
        assert!(provider.description().contains("memory"));
    }

    #[test]
    fn test_memory_tool_provider_default() {
        let provider = MemoryToolProvider::default();
        let tools = provider.tools();

        assert!(tools.is_empty());
    }
}

#[cfg(all(feature = "mcp", feature = "scraping"))]
mod mcp_scraping_tool_provider_tests {
    use swissknife_ai_sdk::mcp::tools::ScrapingToolProvider;
    use swissknife_ai_sdk::mcp::ToolProvider;

    #[test]
    fn test_scraping_tool_provider_new() {
        let provider = ScrapingToolProvider::new();

        assert_eq!(provider.name(), "scraping");
        assert!(provider.description().contains("scraping"));
    }

    #[test]
    fn test_scraping_tool_provider_default() {
        let provider = ScrapingToolProvider::default();
        let tools = provider.tools();

        assert!(tools.is_empty());
    }
}

#[cfg(feature = "mcp")]
mod mcp_integration_tests {
    use super::*;
    use swissknife_ai_sdk::mcp::*;
    use swissknife_ai_sdk::Result;
    use async_trait::async_trait;

    struct EchoToolProvider;

    #[async_trait]
    impl ToolProvider for EchoToolProvider {
        fn name(&self) -> &str {
            "echo"
        }

        fn description(&self) -> &str {
            "Echo tool for testing"
        }

        fn tools(&self) -> Vec<ToolDefinition> {
            vec![
                ToolDefinition::new("echo")
                    .with_description("Echoes back the input")
                    .with_schema(json!({
                        "type": "object",
                        "properties": {
                            "message": {"type": "string", "description": "Message to echo"}
                        },
                        "required": ["message"]
                    })),
                ToolDefinition::new("echo_json")
                    .with_description("Echoes back JSON input")
                    .with_schema(json!({
                        "type": "object",
                        "properties": {
                            "data": {"type": "object", "description": "Data to echo"}
                        },
                        "required": ["data"]
                    })),
            ]
        }

        async fn call(&self, tool_name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
            match tool_name {
                "echo" => {
                    let message = arguments.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("no message");
                    Ok(ToolResult::text(format!("Echo: {}", message)))
                },
                "echo_json" => {
                    let data = arguments.get("data").cloned().unwrap_or(json!({}));
                    Ok(ToolResult::json(json!({
                        "echo": data
                    })))
                },
                _ => Ok(ToolResult::error(format!("Unknown tool: {}", tool_name))),
            }
        }
    }

    #[tokio::test]
    async fn test_full_mcp_workflow() {
        let router = McpRouter::new("test-mcp-server", "1.0.0")
            .with_instructions("This is a test MCP server")
            .with_tool_provider(EchoToolProvider);

        let info = router.server_info();
        assert_eq!(info.name, "test-mcp-server");
        assert_eq!(info.version, "1.0.0");

        let caps = router.capabilities();
        assert!(caps.tools.is_some());

        let tools = router.list_tools();
        assert_eq!(tools.len(), 2);

        let echo_tool = tools.iter().find(|t| t.name == "echo").unwrap();
        assert!(echo_tool.description.as_ref().unwrap().contains("Echoes"));

        let result = router.call_tool("echo", json!({"message": "Hello, MCP!"})).await.unwrap();
        assert!(!result.is_error);
        match &result.content[0] {
            ToolContent::Text { text } => {
                assert_eq!(text, "Echo: Hello, MCP!");
            },
            _ => panic!("Expected text content"),
        }

        let json_result = router.call_tool("echo_json", json!({
            "data": {"key": "value", "num": 42}
        })).await.unwrap();
        assert!(!json_result.is_error);
    }

    #[tokio::test]
    async fn test_multiple_tool_providers() {
        struct AddToolProvider;

        #[async_trait]
        impl ToolProvider for AddToolProvider {
            fn name(&self) -> &str { "math" }
            fn description(&self) -> &str { "Math operations" }
            fn tools(&self) -> Vec<ToolDefinition> {
                vec![ToolDefinition::new("add")
                    .with_description("Add two numbers")
                    .with_schema(json!({
                        "type": "object",
                        "properties": {
                            "a": {"type": "number"},
                            "b": {"type": "number"}
                        },
                        "required": ["a", "b"]
                    }))]
            }
            async fn call(&self, _: &str, args: serde_json::Value) -> Result<ToolResult> {
                let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Ok(ToolResult::json(json!({"result": a + b})))
            }
        }

        let router = McpRouter::new("multi-provider", "1.0.0")
            .with_tool_provider(EchoToolProvider)
            .with_tool_provider(AddToolProvider);

        let tools = router.list_tools();
        assert_eq!(tools.len(), 3);

        let echo_result = router.call_tool("echo", json!({"message": "test"})).await.unwrap();
        assert!(!echo_result.is_error);

        let add_result = router.call_tool("add", json!({"a": 5, "b": 3})).await.unwrap();
        assert!(!add_result.is_error);
        match &add_result.content[0] {
            ToolContent::Text { text } => {
                let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
                assert_eq!(parsed["result"], 8.0);
            },
            _ => panic!("Expected text content"),
        }
    }
}
