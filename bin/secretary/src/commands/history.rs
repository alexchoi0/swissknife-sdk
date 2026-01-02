use crate::cli::HistoryCommands;
use crate::db::get_memory;
use crate::format::{format_timestamp, truncate, PREVIEW_SHORT};

pub fn handle_history_command(command: &HistoryCommands) {
    let memory = match get_memory() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match command {
        HistoryCommands::Search { query, limit } => {
            match memory.search_claude_prompts(query, *limit) {
                Ok(prompts) => {
                    if prompts.is_empty() {
                        println!("No prompts found matching '{}'", query);
                    } else {
                        for prompt in prompts {
                            let ts = format_timestamp(prompt.timestamp);
                            let proj = prompt.project.as_deref().unwrap_or("-");
                            let display = truncate(&prompt.display, PREVIEW_SHORT);
                            println!("[{}] {} | {}", ts, proj, display);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        HistoryCommands::Prompts { project, limit } => {
            match memory.get_claude_prompts(project.as_deref(), *limit) {
                Ok(prompts) => {
                    if prompts.is_empty() {
                        println!("No prompts found.");
                    } else {
                        for prompt in prompts {
                            let ts = format_timestamp(prompt.timestamp);
                            let proj = prompt.project.as_deref().unwrap_or("-");
                            let display = truncate(&prompt.display, PREVIEW_SHORT);
                            println!("[{}] {} | {}", ts, proj, display);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        HistoryCommands::Messages { session_id, limit } => {
            match memory.get_claude_messages(session_id, *limit) {
                Ok(messages) => {
                    if messages.is_empty() {
                        println!("No messages found for session '{}'", session_id);
                    } else {
                        for msg in messages {
                            let role = msg.role.as_deref().unwrap_or(&msg.message_type);
                            let content = msg.content.as_deref().unwrap_or("");
                            let preview: String = content.chars().take(80).collect();
                            println!("[{}] {}", role, preview.replace('\n', " "));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        HistoryCommands::Stats => {
            match memory.get_history_stats() {
                Ok((prompts, messages, todos)) => {
                    println!("Claude Code History Statistics:");
                    println!("  Prompts:  {}", prompts);
                    println!("  Messages: {}", messages);
                    println!("  Todos:    {}", todos);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        HistoryCommands::Sql { query } => {
            match memory.execute_sql(query) {
                Ok(rows) => {
                    if rows.is_empty() {
                        println!("No results.");
                    } else {
                        for (i, row) in rows.iter().enumerate() {
                            if i == 0 {
                                println!("{}", row.join(" | "));
                                println!("{}", "-".repeat(row.iter().map(|s| s.len() + 3).sum::<usize>()));
                            } else {
                                println!("{}", row.join(" | "));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("SQL Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
