use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use swissknife_ai_sdk::{
    Error, OutputSchema, ParameterSchema, Result, Tool, ToolSpec, ToolRegistry, ToolOutput,
};

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn definition(&self) -> ToolSpec {
        ToolSpec::new("echo", "Echo Tool", "Returns the input message", "utility")
            .with_param("message", ParameterSchema::string("The message to echo").required())
            .with_output("result", OutputSchema::string("The echoed message"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolOutput> {
        let message = params
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::MissingParameter("message".to_string()))?;
        Ok(ToolOutput::success(serde_json::json!({ "result": message })))
    }
}

struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn definition(&self) -> ToolSpec {
        ToolSpec::new("calculator", "Calculator", "Performs arithmetic", "math")
            .with_param("a", ParameterSchema::number("First operand").required())
            .with_param("b", ParameterSchema::number("Second operand").required())
            .with_param("operation", ParameterSchema::string("add, sub, mul, div").required())
            .with_output("result", OutputSchema::number("The calculation result"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolOutput> {
        let a = params.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = params.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let op = params.get("operation").and_then(|v| v.as_str()).unwrap_or("add");

        let result = match op {
            "add" => a + b,
            "sub" => a - b,
            "mul" => a * b,
            "div" => {
                if b == 0.0 {
                    return Ok(ToolOutput::error("Division by zero"));
                }
                a / b
            }
            _ => return Ok(ToolOutput::error("Unknown operation")),
        };
        Ok(ToolOutput::success(serde_json::json!({ "result": result })))
    }
}

struct HiddenParamTool;

#[async_trait]
impl Tool for HiddenParamTool {
    fn definition(&self) -> ToolSpec {
        ToolSpec::new("hidden_param", "Hidden Param Tool", "Has hidden params", "utility")
            .with_param("visible", ParameterSchema::string("Visible param"))
            .with_param("hidden", ParameterSchema::string("Hidden param").hidden())
            .with_param("llm_only", ParameterSchema::string("LLM only param").llm_only())
            .with_param("user_only", ParameterSchema::string("User only param").user_only())
    }

    async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> Result<ToolOutput> {
        Ok(ToolOutput::success(serde_json::json!({})))
    }
}

mod tool_registry_behavior {
    use super::*;

    #[test]
    fn test_registry_starts_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_register_increments_count() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_returns_none_for_unknown_tool() {
        let registry = ToolRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_contains_returns_false_for_unknown() {
        let registry = ToolRegistry::new();
        assert!(!registry.contains("nonexistent"));
    }

    #[test]
    fn test_registry_retrieves_registered_tool_by_id() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let tool = registry.get("echo");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().id(), "echo");
    }

    #[test]
    fn test_registry_contains_returns_true_for_registered() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        assert!(registry.contains("echo"));
    }

    #[test]
    fn test_registry_register_multiple_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        registry.register(CalculatorTool);
        assert_eq!(registry.len(), 2);
        assert!(registry.contains("echo"));
        assert!(registry.contains("calculator"));
    }

    #[test]
    fn test_registry_list_returns_all_tool_ids() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        registry.register(CalculatorTool);
        let ids = registry.list();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"echo"));
        assert!(ids.contains(&"calculator"));
    }

    #[test]
    fn test_registry_remove_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        assert!(registry.contains("echo"));
        let removed = registry.remove("echo");
        assert!(removed.is_some());
        assert!(!registry.contains("echo"));
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_remove_returns_none_for_nonexistent() {
        let mut registry = ToolRegistry::new();
        let removed = registry.remove("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_registry_register_arc_adds_tool() {
        let mut registry = ToolRegistry::new();
        let tool: Arc<dyn Tool> = Arc::new(EchoTool);
        registry.register_arc(tool);
        assert!(registry.contains("echo"));
    }

    #[test]
    fn test_registry_method_chaining() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool).register(CalculatorTool);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_default_is_empty() {
        let registry = ToolRegistry::default();
        assert!(registry.is_empty());
    }
}

mod tool_category_filtering {
    use super::*;

    #[test]
    fn test_list_by_category_returns_matching_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        registry.register(CalculatorTool);
        let utility_tools = registry.list_by_category("utility");
        assert_eq!(utility_tools.len(), 1);
        assert!(utility_tools.contains(&"echo"));
    }

    #[test]
    fn test_list_by_category_returns_empty_for_unknown_category() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let unknown = registry.list_by_category("unknown_category");
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_categories_returns_unique_sorted_list() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        registry.register(HiddenParamTool);
        registry.register(CalculatorTool);
        let categories = registry.categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"utility".to_string()));
        assert!(categories.contains(&"math".to_string()));
    }

    #[test]
    fn test_definitions_by_category_returns_tool_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        registry.register(CalculatorTool);
        let math_defs = registry.definitions_by_category("math");
        assert_eq!(math_defs.len(), 1);
        assert_eq!(math_defs[0].id, "calculator");
    }
}

mod openai_anthropic_format_conversion {
    use super::*;

    #[test]
    fn test_to_openai_functions_returns_valid_format() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let functions = registry.to_openai_functions();
        assert_eq!(functions.len(), 1);
        let func = &functions[0];
        assert_eq!(func["name"], "echo");
        assert!(func["description"].as_str().is_some());
        assert!(func["parameters"]["type"].as_str() == Some("object"));
        assert!(func["parameters"]["properties"]["message"].is_object());
    }

    #[test]
    fn test_to_anthropic_tools_returns_valid_format() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let tools = registry.to_anthropic_tools();
        assert_eq!(tools.len(), 1);
        let tool = &tools[0];
        assert_eq!(tool["name"], "echo");
        assert!(tool["input_schema"]["type"].as_str() == Some("object"));
        assert!(tool["input_schema"]["properties"]["message"].is_object());
    }

    #[test]
    fn test_openai_function_includes_required_params() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let functions = registry.to_openai_functions();
        let required = functions[0]["parameters"]["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("message")));
    }

    #[test]
    fn test_anthropic_tool_includes_required_params() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let tools = registry.to_anthropic_tools();
        let required = tools[0]["input_schema"]["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("message")));
    }

    #[test]
    fn test_openai_function_excludes_hidden_params() {
        let mut registry = ToolRegistry::new();
        registry.register(HiddenParamTool);
        let functions = registry.to_openai_functions();
        let properties = functions[0]["parameters"]["properties"].as_object().unwrap();
        assert!(!properties.contains_key("hidden"));
        assert!(properties.contains_key("visible"));
        assert!(properties.contains_key("llm_only"));
        assert!(properties.contains_key("user_only"));
    }

    #[test]
    fn test_anthropic_tool_excludes_hidden_params() {
        let mut registry = ToolRegistry::new();
        registry.register(HiddenParamTool);
        let tools = registry.to_anthropic_tools();
        let properties = tools[0]["input_schema"]["properties"].as_object().unwrap();
        assert!(!properties.contains_key("hidden"));
    }
}

mod tool_execution {
    use super::*;

    #[tokio::test]
    async fn test_execute_returns_success_for_valid_params() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let mut params = HashMap::new();
        params.insert("message".to_string(), serde_json::json!("hello"));
        let result = registry.execute("echo", params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.output["result"], "hello");
    }

    #[tokio::test]
    async fn test_execute_returns_error_for_unknown_tool() {
        let registry = ToolRegistry::new();
        let params = HashMap::new();
        let result = registry.execute("nonexistent", params).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ToolNotFound(id) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected ToolNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_execute_calculator_add() {
        let mut registry = ToolRegistry::new();
        registry.register(CalculatorTool);
        let mut params = HashMap::new();
        params.insert("a".to_string(), serde_json::json!(5));
        params.insert("b".to_string(), serde_json::json!(3));
        params.insert("operation".to_string(), serde_json::json!("add"));
        let result = registry.execute("calculator", params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output["result"], 8.0);
    }

    #[tokio::test]
    async fn test_execute_calculator_division_by_zero() {
        let mut registry = ToolRegistry::new();
        registry.register(CalculatorTool);
        let mut params = HashMap::new();
        params.insert("a".to_string(), serde_json::json!(10));
        params.insert("b".to_string(), serde_json::json!(0));
        params.insert("operation".to_string(), serde_json::json!("div"));
        let result = registry.execute("calculator", params).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.error, Some("Division by zero".to_string()));
    }

    #[tokio::test]
    async fn test_tool_response_success_has_no_error() {
        let response = ToolOutput::success(serde_json::json!({"data": "test"}));
        assert!(response.success);
        assert!(response.error.is_none());
        assert!(response.timing.is_none());
    }

    #[tokio::test]
    async fn test_tool_response_error_has_null_output() {
        let response = ToolOutput::error("Something failed");
        assert!(!response.success);
        assert_eq!(response.error, Some("Something failed".to_string()));
        assert!(response.output.is_null());
    }

    #[tokio::test]
    async fn test_tool_response_with_timing() {
        let start = chrono::Utc::now();
        let end = start + chrono::Duration::milliseconds(100);
        let response = ToolOutput::success(serde_json::json!({})).with_timing(start, end);
        assert!(response.timing.is_some());
        let timing = response.timing.unwrap();
        assert_eq!(timing.duration_ms, 100);
    }
}

mod tool_trait_default_implementations {
    use super::*;

    #[test]
    fn test_tool_id_from_definition() {
        let tool = EchoTool;
        assert_eq!(tool.id(), "echo");
    }

    #[test]
    fn test_tool_name_from_definition() {
        let tool = EchoTool;
        assert_eq!(tool.name(), "Echo Tool");
    }

    #[test]
    fn test_tool_description_from_definition() {
        let tool = EchoTool;
        assert_eq!(tool.description(), "Returns the input message");
    }

    #[test]
    fn test_tool_category_from_definition() {
        let tool = EchoTool;
        assert_eq!(tool.category(), "utility");
    }
}

mod definitions_list {
    use super::*;

    #[test]
    fn test_definitions_returns_all_tool_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        registry.register(CalculatorTool);
        let definitions = registry.definitions();
        assert_eq!(definitions.len(), 2);
    }

    #[test]
    fn test_definitions_contains_correct_data() {
        let mut registry = ToolRegistry::new();
        registry.register(EchoTool);
        let definitions = registry.definitions();
        let echo_def = definitions.iter().find(|d| d.id == "echo").unwrap();
        assert_eq!(echo_def.name, "Echo Tool");
        assert_eq!(echo_def.category, "utility");
    }
}
