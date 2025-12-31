use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    pub anthropic_key: Option<String>,
    pub voyage_key: Option<String>,
    pub tavily_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(default = "default_model")]
    pub name: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub thinking_budget: u32,
}

fn default_model() -> String {
    "claude-haiku-4-5".to_string()
}

fn default_max_tokens() -> u32 {
    16000
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: default_model(),
            max_tokens: default_max_tokens(),
            thinking_budget: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_true")]
    pub builtin: bool,
    #[serde(default = "default_true")]
    pub sdk: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            builtin: true,
            sdk: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: Vec<String>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("secretary")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        Self::load_from(Self::config_path())
    }

    pub fn load_from(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Warning: Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Warning: Failed to read config: {}", e),
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_to(Self::config_path())
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_anthropic_key(&self) -> Option<String> {
        self.api.anthropic_key.clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
    }

    pub fn get_voyage_key(&self) -> Option<String> {
        self.api.voyage_key.clone()
            .or_else(|| std::env::var("VOYAGE_API_KEY").ok())
    }

    pub fn get_tavily_key(&self) -> Option<String> {
        self.api.tavily_key.clone()
            .or_else(|| std::env::var("TAVILY_API_KEY").ok())
    }

    pub fn thinking_enabled(&self) -> bool {
        self.model.thinking_budget > 0
    }
}

pub fn set_config_value(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Config::config_path();

    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };

    let mut doc = content.parse::<toml_edit::DocumentMut>()?;

    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err("Invalid key".into());
    }

    let value = parse_toml_value(value);

    match parts.len() {
        1 => {
            doc[parts[0]] = value;
        }
        2 => {
            if !doc.contains_key(parts[0]) {
                doc[parts[0]] = toml_edit::Item::Table(toml_edit::Table::new());
            }
            doc[parts[0]][parts[1]] = value;
        }
        _ => return Err("Key depth > 2 not supported".into()),
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, doc.to_string())?;
    Ok(())
}

pub fn get_config_value(key: &str) -> Option<String> {
    let config = Config::load();
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["api", "anthropic_key"] => config.api.anthropic_key,
        ["api", "voyage_key"] => config.api.voyage_key,
        ["api", "tavily_key"] => config.api.tavily_key,
        ["model", "name"] => Some(config.model.name),
        ["model", "max_tokens"] => Some(config.model.max_tokens.to_string()),
        ["model", "thinking_budget"] => Some(config.model.thinking_budget.to_string()),
        ["tools", "builtin"] => Some(config.tools.builtin.to_string()),
        ["tools", "sdk"] => Some(config.tools.sdk.to_string()),
        ["mcp", "servers"] => Some(format!("{:?}", config.mcp.servers)),
        _ => None,
    }
}

pub fn unset_config_value(key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Config::config_path();

    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)?;
    let mut doc = content.parse::<toml_edit::DocumentMut>()?;

    let parts: Vec<&str> = key.split('.').collect();

    match parts.len() {
        1 => {
            doc.remove(parts[0]);
        }
        2 => {
            if let Some(table) = doc[parts[0]].as_table_mut() {
                table.remove(parts[1]);
            }
        }
        _ => return Err("Key depth > 2 not supported".into()),
    }

    std::fs::write(&path, doc.to_string())?;
    Ok(())
}

pub fn add_mcp_server(cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load();
    if !config.mcp.servers.contains(&cmd.to_string()) {
        config.mcp.servers.push(cmd.to_string());
        config.save()?;
    }
    Ok(())
}

pub fn remove_mcp_server(name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let mut config = Config::load();
    let original_len = config.mcp.servers.len();
    config.mcp.servers.retain(|s| !s.contains(name));
    if config.mcp.servers.len() < original_len {
        config.save()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn parse_toml_value(s: &str) -> toml_edit::Item {
    if s == "true" {
        toml_edit::value(true)
    } else if s == "false" {
        toml_edit::value(false)
    } else if let Ok(n) = s.parse::<i64>() {
        toml_edit::value(n)
    } else {
        toml_edit::value(s)
    }
}
