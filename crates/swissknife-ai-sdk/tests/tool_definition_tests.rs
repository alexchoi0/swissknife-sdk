use std::collections::HashMap;
use swissknife_ai_sdk::{OutputSchema, ParameterSchema, ParameterType, ParameterVisibility, ToolDefinition};

mod tool_definition_builder {
    use super::*;

    #[test]
    fn test_tool_definition_new_sets_default_version() {
        let def = ToolDefinition::new("test", "Test", "Description", "category");
        assert_eq!(def.version, "1.0.0");
    }

    #[test]
    fn test_tool_definition_with_version_overrides_default() {
        let def = ToolDefinition::new("test", "Test", "Description", "category")
            .with_version("2.0.0");
        assert_eq!(def.version, "2.0.0");
    }

    #[test]
    fn test_tool_definition_with_param_adds_parameter() {
        let def = ToolDefinition::new("test", "Test", "Description", "category")
            .with_param("name", ParameterSchema::string("A name"));
        assert!(def.params.contains_key("name"));
        assert_eq!(def.params["name"].param_type, ParameterType::String);
    }

    #[test]
    fn test_tool_definition_with_output_adds_output() {
        let def = ToolDefinition::new("test", "Test", "Description", "category")
            .with_output("result", OutputSchema::string("The result"));
        assert!(def.outputs.contains_key("result"));
    }

    #[test]
    fn test_tool_definition_chained_params() {
        let def = ToolDefinition::new("test", "Test", "Description", "category")
            .with_param("a", ParameterSchema::string("Param A"))
            .with_param("b", ParameterSchema::number("Param B"))
            .with_param("c", ParameterSchema::boolean("Param C"));
        assert_eq!(def.params.len(), 3);
    }
}

mod parameter_schema {
    use super::*;

    #[test]
    fn test_parameter_schema_string_type() {
        let schema = ParameterSchema::string("A string param");
        assert_eq!(schema.param_type, ParameterType::String);
        assert!(!schema.required);
        assert_eq!(schema.visibility, ParameterVisibility::UserOrLlm);
    }

    #[test]
    fn test_parameter_schema_number_type() {
        let schema = ParameterSchema::number("A number param");
        assert_eq!(schema.param_type, ParameterType::Number);
    }

    #[test]
    fn test_parameter_schema_integer_type() {
        let schema = ParameterSchema::integer("An integer param");
        assert_eq!(schema.param_type, ParameterType::Integer);
    }

    #[test]
    fn test_parameter_schema_boolean_type() {
        let schema = ParameterSchema::boolean("A boolean param");
        assert_eq!(schema.param_type, ParameterType::Boolean);
    }

    #[test]
    fn test_parameter_schema_json_type() {
        let schema = ParameterSchema::json("A JSON param");
        assert_eq!(schema.param_type, ParameterType::Json);
    }

    #[test]
    fn test_parameter_schema_array_type() {
        let items = ParameterSchema::string("Item");
        let schema = ParameterSchema::array("An array param", items);
        assert_eq!(schema.param_type, ParameterType::Array);
        assert!(schema.items.is_some());
    }

    #[test]
    fn test_parameter_schema_object_type() {
        let mut props = HashMap::new();
        props.insert("field".to_string(), ParameterSchema::string("A field"));
        let schema = ParameterSchema::object("An object param", props);
        assert_eq!(schema.param_type, ParameterType::Object);
        assert!(schema.properties.is_some());
    }

    #[test]
    fn test_parameter_schema_required_modifier() {
        let schema = ParameterSchema::string("Required param").required();
        assert!(schema.required);
    }

    #[test]
    fn test_parameter_schema_user_only_visibility() {
        let schema = ParameterSchema::string("User only").user_only();
        assert_eq!(schema.visibility, ParameterVisibility::UserOnly);
    }

    #[test]
    fn test_parameter_schema_llm_only_visibility() {
        let schema = ParameterSchema::string("LLM only").llm_only();
        assert_eq!(schema.visibility, ParameterVisibility::LlmOnly);
    }

    #[test]
    fn test_parameter_schema_hidden_visibility() {
        let schema = ParameterSchema::string("Hidden").hidden();
        assert_eq!(schema.visibility, ParameterVisibility::Hidden);
    }

    #[test]
    fn test_parameter_schema_with_default() {
        let schema = ParameterSchema::string("With default").with_default(serde_json::json!("default_value"));
        assert_eq!(schema.default, Some(serde_json::json!("default_value")));
    }

    #[test]
    fn test_parameter_schema_chained_modifiers() {
        let schema = ParameterSchema::string("Complex param")
            .required()
            .llm_only()
            .with_default(serde_json::json!("default"));
        assert!(schema.required);
        assert_eq!(schema.visibility, ParameterVisibility::LlmOnly);
        assert!(schema.default.is_some());
    }
}

mod output_schema {
    use super::*;

    #[test]
    fn test_output_schema_string() {
        let schema = OutputSchema::string("A string output");
        assert_eq!(schema.output_type, swissknife_ai_sdk::OutputType::String);
        assert!(!schema.optional);
    }

    #[test]
    fn test_output_schema_number() {
        let schema = OutputSchema::number("A number output");
        assert_eq!(schema.output_type, swissknife_ai_sdk::OutputType::Number);
    }

    #[test]
    fn test_output_schema_boolean() {
        let schema = OutputSchema::boolean("A boolean output");
        assert_eq!(schema.output_type, swissknife_ai_sdk::OutputType::Boolean);
    }

    #[test]
    fn test_output_schema_json() {
        let schema = OutputSchema::json("A JSON output");
        assert_eq!(schema.output_type, swissknife_ai_sdk::OutputType::Json);
    }

    #[test]
    fn test_output_schema_array() {
        let items = OutputSchema::string("Item");
        let schema = OutputSchema::array("An array output", items);
        assert_eq!(schema.output_type, swissknife_ai_sdk::OutputType::Array);
        assert!(schema.items.is_some());
    }

    #[test]
    fn test_output_schema_object() {
        let mut props = HashMap::new();
        props.insert("field".to_string(), OutputSchema::string("A field"));
        let schema = OutputSchema::object("An object output", props);
        assert_eq!(schema.output_type, swissknife_ai_sdk::OutputType::Object);
        assert!(schema.properties.is_some());
    }

    #[test]
    fn test_output_schema_optional() {
        let schema = OutputSchema::string("Optional output").optional();
        assert!(schema.optional);
    }
}
