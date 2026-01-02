use crate::cli::ImportCommands;
use crate::db::get_memory;
use chrono::Utc;
use swissknife_ai_sdk::claude_history::{discover_sessions, parse_session_jsonl, ClaudeHistoryImporter};
use swissknife_ai_sdk::memory::{ClaudeMessage as DbClaudeMessage, ClaudePrompt as DbClaudePrompt};
use uuid::Uuid;

pub fn handle_import_command(command: &ImportCommands) {
    let memory = match get_memory() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match command {
        ImportCommands::Claude { project, limit } => {
            let importer = ClaudeHistoryImporter::new(memory.clone());

            println!("Reading Claude Code history from ~/.claude/...");

            match importer.read_prompts() {
                Ok(prompts) => {
                    let filtered: Vec<_> = prompts
                        .into_iter()
                        .filter(|p| {
                            if let Some(proj) = project {
                                p.project.as_ref().is_some_and(|pp| pp.contains(proj))
                            } else {
                                true
                            }
                        })
                        .take(limit.unwrap_or(usize::MAX))
                        .collect();

                    println!("Found {} prompts to import", filtered.len());

                    let mut imported = 0;
                    let mut skipped = 0;

                    for prompt in filtered {
                        let db_prompt = DbClaudePrompt {
                            id: Uuid::new_v4().to_string(),
                            display: prompt.display.clone(),
                            timestamp: prompt.timestamp,
                            project: prompt.project.clone(),
                            session_id: prompt.session_id.clone(),
                            created_at: Utc::now(),
                        };

                        match memory.add_claude_prompt(&db_prompt) {
                            Ok(_) => imported += 1,
                            Err(e) => {
                                if std::env::var("SECRETARY_DEBUG").is_ok() {
                                    eprintln!("  Skipped prompt: {}", e);
                                }
                                skipped += 1;
                            }
                        }
                    }

                    println!("Imported {} prompts ({} skipped)", imported, skipped);
                }
                Err(e) => {
                    eprintln!("Error reading history: {}", e);
                    std::process::exit(1);
                }
            }

            println!("\nImporting session conversations...");
            let projects_dir = importer.projects_dir();
            match discover_sessions(&projects_dir) {
                Ok(sessions) => {
                    let filtered_sessions: Vec<_> = if let Some(proj) = project {
                        sessions
                            .into_iter()
                            .filter(|(_, path)| path.to_string_lossy().contains(proj))
                            .collect()
                    } else {
                        sessions
                    };

                    println!("Found {} session files to import", filtered_sessions.len());

                    let mut total_messages = 0;
                    let mut total_skipped = 0;

                    for (idx, (session_id, path)) in filtered_sessions.iter().enumerate() {
                        if idx % 100 == 0 {
                            print!(
                                "\rProcessing session {}/{}...",
                                idx + 1,
                                filtered_sessions.len()
                            );
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                        }

                        match parse_session_jsonl(path) {
                            Ok(messages) => {
                                for msg in messages {
                                    if msg.message_type == "summary"
                                        || msg.message_type == "file-history-snapshot"
                                    {
                                        continue;
                                    }

                                    let content_text = msg.content_text();
                                    let thinking_text = msg.thinking_text();
                                    let tool_uses = msg.tool_uses();
                                    let tool_use_json = if tool_uses.is_empty() {
                                        None
                                    } else {
                                        serde_json::to_string(&tool_uses).ok()
                                    };

                                    let db_msg = DbClaudeMessage {
                                        id: Uuid::new_v4().to_string(),
                                        uuid: msg.uuid.clone(),
                                        parent_uuid: msg.parent_uuid.clone(),
                                        session_id: msg
                                            .session_id
                                            .clone()
                                            .unwrap_or_else(|| session_id.clone()),
                                        message_type: msg.message_type.clone(),
                                        timestamp: msg.timestamp.clone().unwrap_or_default(),
                                        role: msg.role().map(String::from),
                                        content: content_text,
                                        thinking: thinking_text,
                                        tool_use: tool_use_json,
                                        cwd: msg.cwd.clone(),
                                        git_branch: msg.git_branch.clone(),
                                        created_at: Utc::now(),
                                    };

                                    match memory.add_claude_message(&db_msg) {
                                        Ok(_) => total_messages += 1,
                                        Err(e) => {
                                            if std::env::var("SECRETARY_DEBUG").is_ok() {
                                                eprintln!("  Skipped message: {}", e);
                                            }
                                            total_skipped += 1;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                if std::env::var("SECRETARY_DEBUG").is_ok() {
                                    eprintln!("  Skipped session {}: {}", session_id, e);
                                }
                                continue;
                            }
                        }
                    }

                    println!(
                        "\rImported {} messages ({} skipped)          ",
                        total_messages, total_skipped
                    );
                }
                Err(e) => eprintln!("Warning: Could not discover sessions: {}", e),
            }

            match importer.read_todos() {
                Ok(todos) => {
                    if !todos.is_empty() {
                        println!("Found {} todos", todos.len());
                        let mut todo_imported = 0;
                        for todo in todos {
                            let db_todo = swissknife_ai_sdk::memory::ClaudeTodo {
                                id: Uuid::new_v4().to_string(),
                                session_id: todo.session_id,
                                agent_id: todo.agent_id,
                                content: todo.content,
                                status: todo.status,
                                active_form: Some(todo.active_form),
                                created_at: Utc::now(),
                            };
                            if memory.add_claude_todo(&db_todo).is_ok() {
                                todo_imported += 1;
                            }
                        }
                        println!("Imported {} todos", todo_imported);
                    }
                }
                Err(e) => eprintln!("Warning: Could not read todos: {}", e),
            }

            println!("Import complete!");
        }
    }
}
