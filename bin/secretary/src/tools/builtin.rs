use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use swissknife_ai_sdk::llm::{FunctionDefinition, ToolDefinition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirectoryArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilesArgs {
    pub pattern: String,
    #[serde(default)]
    pub path: Option<String>,
}

pub fn get_builtin_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "read_file".to_string(),
                description: Some("Read the contents of a file at the given path".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to read"
                        }
                    },
                    "required": ["path"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "list_directory".to_string(),
                description: Some("List the contents of a directory".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the directory to list"
                        }
                    },
                    "required": ["path"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "search_files".to_string(),
                description: Some("Search for files matching a glob pattern".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The glob pattern to search for (e.g., '*.rs', '**/*.txt')"
                        },
                        "path": {
                            "type": "string",
                            "description": "The directory to search in (defaults to current directory)"
                        }
                    },
                    "required": ["pattern"]
                }),
            },
        },
    ]
}

pub fn execute_builtin(name: &str, arguments: &str) -> Result<String, String> {
    match name {
        "read_file" => {
            let args: ReadFileArgs =
                serde_json::from_str(arguments).map_err(|e| format!("Invalid arguments: {}", e))?;
            read_file(&args.path)
        }
        "list_directory" => {
            let args: ListDirectoryArgs =
                serde_json::from_str(arguments).map_err(|e| format!("Invalid arguments: {}", e))?;
            list_directory(&args.path)
        }
        "search_files" => {
            let args: SearchFilesArgs =
                serde_json::from_str(arguments).map_err(|e| format!("Invalid arguments: {}", e))?;
            search_files(&args.pattern, args.path.as_deref())
        }
        _ => Err(format!("Unknown builtin tool: {}", name)),
    }
}

fn read_file(path: &str) -> Result<String, String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

fn list_directory(path: &str) -> Result<String, String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    let mut entries = Vec::new();
    let read_dir =
        std::fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_type = if entry.path().is_dir() {
            "dir"
        } else {
            "file"
        };
        entries.push(format!(
            "[{}] {}",
            file_type,
            entry.file_name().to_string_lossy()
        ));
    }

    entries.sort();
    Ok(entries.join("\n"))
}

fn search_files(pattern: &str, base_path: Option<&str>) -> Result<String, String> {
    let base = base_path.unwrap_or(".");
    let full_pattern = if pattern.starts_with('/') || pattern.starts_with('.') {
        pattern.to_string()
    } else {
        format!("{}/{}", base, pattern)
    };

    let paths: Vec<_> = glob::glob(&full_pattern)
        .map_err(|e| format!("Invalid pattern: {}", e))?
        .filter_map(|r| r.ok())
        .map(|p| p.display().to_string())
        .collect();

    if paths.is_empty() {
        Ok("No files found matching pattern".to_string())
    } else {
        Ok(paths.join("\n"))
    }
}
