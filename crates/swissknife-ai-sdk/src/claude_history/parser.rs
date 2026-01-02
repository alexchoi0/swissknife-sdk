use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::Result;

use super::{ClaudeMessage, ClaudePrompt, ClaudeTodo, RawTodoItem};

pub fn parse_history_jsonl(path: &Path) -> Result<Vec<ClaudePrompt>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|e| crate::Error::Internal(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut prompts = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<ClaudePrompt>(&line) {
            Ok(prompt) => prompts.push(prompt),
            Err(_) => continue,
        }
    }

    Ok(prompts)
}

pub fn parse_session_jsonl(path: &Path) -> Result<Vec<ClaudeMessage>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|e| crate::Error::Internal(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let message_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match message_type {
            "summary" | "file-history-snapshot" => continue,
            "user" | "assistant" => {
                match serde_json::from_value::<ClaudeMessage>(parsed) {
                    Ok(msg) => messages.push(msg),
                    Err(_) => continue,
                }
            }
            _ => {
                match serde_json::from_value::<ClaudeMessage>(parsed) {
                    Ok(msg) => messages.push(msg),
                    Err(_) => continue,
                }
            }
        }
    }

    Ok(messages)
}

pub fn parse_todos_dir(path: &Path) -> Result<Vec<ClaudeTodo>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(path).map_err(|e| crate::Error::Internal(e.to_string()))?;

    let mut all_todos = Vec::new();

    for entry in entries.flatten() {
        let file_path = entry.path();

        if file_path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let filename = match file_path.file_stem().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let parts: Vec<&str> = filename.splitn(2, "-agent-").collect();
        if parts.len() != 2 {
            continue;
        }

        let session_id = parts[0].to_string();
        let agent_id = parts[1].to_string();

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let items: Vec<RawTodoItem> = match serde_json::from_str(&content) {
            Ok(items) => items,
            Err(_) => continue,
        };

        for item in items {
            all_todos.push(ClaudeTodo {
                session_id: session_id.clone(),
                agent_id: agent_id.clone(),
                content: item.content,
                status: item.status,
                active_form: item.active_form,
            });
        }
    }

    Ok(all_todos)
}

pub fn discover_sessions(projects_dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();

    let project_entries =
        fs::read_dir(projects_dir).map_err(|e| crate::Error::Internal(e.to_string()))?;

    for project_entry in project_entries.flatten() {
        let project_path = project_entry.path();

        if !project_path.is_dir() {
            continue;
        }

        let session_entries = match fs::read_dir(&project_path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for session_entry in session_entries.flatten() {
            let session_path = session_entry.path();

            if session_path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let session_id = match session_path.file_stem().and_then(|s| s.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            sessions.push((session_id, session_path));
        }
    }

    Ok(sessions)
}
