use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParameterVisibility {
    UserOrLlm,
    UserOnly,
    LlmOnly,
    Hidden,
}

impl Default for ParameterVisibility {
    fn default() -> Self {
        Self::UserOrLlm
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Json,
}

impl Default for ParameterType {
    fn default() -> Self {
        Self::String
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub visibility: ParameterVisibility,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,
    pub items: Option<Box<ParameterSchema>>,
    pub properties: Option<HashMap<String, ParameterSchema>>,
}

impl ParameterSchema {
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            param_type: ParameterType::String,
            required: false,
            visibility: ParameterVisibility::UserOrLlm,
            description: Some(description.into()),
            default: None,
            items: None,
            properties: None,
        }
    }

    pub fn number(description: impl Into<String>) -> Self {
        Self {
            param_type: ParameterType::Number,
            required: false,
            visibility: ParameterVisibility::UserOrLlm,
            description: Some(description.into()),
            default: None,
            items: None,
            properties: None,
        }
    }

    pub fn integer(description: impl Into<String>) -> Self {
        Self {
            param_type: ParameterType::Integer,
            required: false,
            visibility: ParameterVisibility::UserOrLlm,
            description: Some(description.into()),
            default: None,
            items: None,
            properties: None,
        }
    }

    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            param_type: ParameterType::Boolean,
            required: false,
            visibility: ParameterVisibility::UserOrLlm,
            description: Some(description.into()),
            default: None,
            items: None,
            properties: None,
        }
    }

    pub fn json(description: impl Into<String>) -> Self {
        Self {
            param_type: ParameterType::Json,
            required: false,
            visibility: ParameterVisibility::UserOrLlm,
            description: Some(description.into()),
            default: None,
            items: None,
            properties: None,
        }
    }

    pub fn array(description: impl Into<String>, items: ParameterSchema) -> Self {
        Self {
            param_type: ParameterType::Array,
            required: false,
            visibility: ParameterVisibility::UserOrLlm,
            description: Some(description.into()),
            default: None,
            items: Some(Box::new(items)),
            properties: None,
        }
    }

    pub fn object(description: impl Into<String>, properties: HashMap<String, ParameterSchema>) -> Self {
        Self {
            param_type: ParameterType::Object,
            required: false,
            visibility: ParameterVisibility::UserOrLlm,
            description: Some(description.into()),
            default: None,
            items: None,
            properties: Some(properties),
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn user_only(mut self) -> Self {
        self.visibility = ParameterVisibility::UserOnly;
        self
    }

    pub fn llm_only(mut self) -> Self {
        self.visibility = ParameterVisibility::LlmOnly;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visibility = ParameterVisibility::Hidden;
        self
    }

    pub fn with_default(mut self, value: serde_json::Value) -> Self {
        self.default = Some(value);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Json,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchema {
    #[serde(rename = "type")]
    pub output_type: OutputType,
    pub description: Option<String>,
    #[serde(default)]
    pub optional: bool,
    pub items: Option<Box<OutputSchema>>,
    pub properties: Option<HashMap<String, OutputSchema>>,
}

impl OutputSchema {
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            output_type: OutputType::String,
            description: Some(description.into()),
            optional: false,
            items: None,
            properties: None,
        }
    }

    pub fn number(description: impl Into<String>) -> Self {
        Self {
            output_type: OutputType::Number,
            description: Some(description.into()),
            optional: false,
            items: None,
            properties: None,
        }
    }

    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            output_type: OutputType::Boolean,
            description: Some(description.into()),
            optional: false,
            items: None,
            properties: None,
        }
    }

    pub fn json(description: impl Into<String>) -> Self {
        Self {
            output_type: OutputType::Json,
            description: Some(description.into()),
            optional: false,
            items: None,
            properties: None,
        }
    }

    pub fn array(description: impl Into<String>, items: OutputSchema) -> Self {
        Self {
            output_type: OutputType::Array,
            description: Some(description.into()),
            optional: false,
            items: Some(Box::new(items)),
            properties: None,
        }
    }

    pub fn object(description: impl Into<String>, properties: HashMap<String, OutputSchema>) -> Self {
        Self {
            output_type: OutputType::Object,
            description: Some(description.into()),
            optional: false,
            items: None,
            properties: Some(properties),
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub success: bool,
    pub output: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<ToolTiming>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTiming {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
}

impl ToolOutput {
    pub fn success(output: serde_json::Value) -> Self {
        Self {
            success: true,
            output,
            error: None,
            timing: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            error: Some(message.into()),
            timing: None,
        }
    }

    pub fn with_timing(mut self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Self {
        let duration_ms = (end - start).num_milliseconds() as u64;
        self.timing = Some(ToolTiming {
            start_time: start,
            end_time: end,
            duration_ms,
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub params: HashMap<String, ParameterSchema>,
    pub outputs: HashMap<String, OutputSchema>,
}

impl ToolSpec {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            version: "1.0.0".into(),
            category: category.into(),
            params: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_param(mut self, name: impl Into<String>, schema: ParameterSchema) -> Self {
        self.params.insert(name.into(), schema);
        self
    }

    pub fn with_output(mut self, name: impl Into<String>, schema: OutputSchema) -> Self {
        self.outputs.insert(name.into(), schema);
        self
    }

    pub fn to_openai_function(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for (name, schema) in &self.params {
            if schema.visibility == ParameterVisibility::Hidden {
                continue;
            }

            let mut prop = serde_json::Map::new();
            prop.insert("type".into(), serde_json::json!(format!("{:?}", schema.param_type).to_lowercase()));
            if let Some(desc) = &schema.description {
                prop.insert("description".into(), serde_json::json!(desc));
            }

            properties.insert(name.clone(), serde_json::Value::Object(prop));

            if schema.required {
                required.push(name.clone());
            }
        }

        serde_json::json!({
            "name": self.id,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": required
            }
        })
    }

    pub fn to_anthropic_tool(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for (name, schema) in &self.params {
            if schema.visibility == ParameterVisibility::Hidden {
                continue;
            }

            let mut prop = serde_json::Map::new();
            prop.insert("type".into(), serde_json::json!(format!("{:?}", schema.param_type).to_lowercase()));
            if let Some(desc) = &schema.description {
                prop.insert("description".into(), serde_json::json!(desc));
            }

            properties.insert(name.clone(), serde_json::Value::Object(prop));

            if schema.required {
                required.push(name.clone());
            }
        }

        serde_json::json!({
            "name": self.id,
            "description": self.description,
            "input_schema": {
                "type": "object",
                "properties": properties,
                "required": required
            }
        })
    }
}
