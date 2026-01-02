use std::io::{self, BufRead, Write};

use swissknife_ai_sdk::llm::ChatMessage;
use swissknife_ai_sdk::memory::DuckDBMemory;

use super::engine::ChatEngine;
use super::session::SessionManager;
use crate::format::{format_action_type, format_session, truncate, PREVIEW_LONG, PREVIEW_SHORT};

pub async fn run_repl(
    engine: &ChatEngine<'_>,
    session: &SessionManager<'_>,
    memory: &DuckDBMemory,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut messages = engine.load_history()?;

    if messages.len() > 1 {
        eprintln!("Loaded {} actions from history", messages.len() - 1);
    }

    loop {
        print!("You: ");
        stdout.flush()?;

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if let Some(should_exit) = handle_command(input, memory, &session.session_id) {
            if should_exit {
                break;
            }
            continue;
        }

        if input == "/search" {
            handle_search(engine).await?;
            continue;
        }

        engine.store_message("user", input).await?;
        messages.push(ChatMessage::user(input));

        loop {
            match engine.chat(&messages).await {
                Ok(response) => {
                    if let Some(thinking) = response.thinking() {
                        println!("\n Thinking:\n{}\n", thinking);
                        engine.store_thinking(thinking)?;
                    }

                    if let Some(tool_calls) = response.tool_calls() {
                        let content = response.content().unwrap_or("");
                        if !content.is_empty() {
                            println!("Secretary: {}", content);
                        }

                        let assistant_msg =
                            engine.build_assistant_message_with_tools(content, tool_calls);
                        messages.push(assistant_msg);

                        engine.process_tool_calls(tool_calls, &mut messages).await?;
                        continue;
                    }

                    let content = response.content().unwrap_or("");
                    println!("Secretary: {}", content);
                    engine.store_message("assistant", content).await?;
                    messages.push(ChatMessage::assistant(content));
                    session.update_title_if_needed()?;
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}

fn handle_command(input: &str, memory: &DuckDBMemory, session_id: &str) -> Option<bool> {
    match input {
        "exit" | "quit" => Some(true),
        "/sessions" => {
            match memory.list_sessions(10) {
                Ok(sessions) => {
                    for session in sessions {
                        println!("{}", format_session(&session, Some(session_id)));
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            Some(false)
        }
        "/actions" => {
            match memory.get_actions(session_id) {
                Ok(actions) => {
                    for action in actions {
                        println!(
                            "{:3}. {} {}",
                            action.sequence,
                            format_action_type(&action),
                            truncate(&action.content, PREVIEW_SHORT)
                        );
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            Some(false)
        }
        "/tools" => {
            match memory.get_tool_calls(session_id) {
                Ok(actions) => {
                    for action in actions {
                        println!(
                            "{}: {}",
                            action.tool_name.as_deref().unwrap_or("?"),
                            truncate(action.tool_input.as_deref().unwrap_or(""), PREVIEW_LONG)
                        );
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            Some(false)
        }
        _ => None,
    }
}

async fn handle_search(engine: &ChatEngine<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    print!("Search query: ");
    stdout.flush()?;
    let mut query = String::new();
    stdin.lock().read_line(&mut query)?;
    let results = engine.search_context(query.trim(), 5).await;
    if results.is_empty() {
        println!("No similar actions found");
    } else {
        for (i, content) in results.iter().enumerate() {
            println!("{}. {}", i + 1, truncate(content, 100));
        }
    }
    Ok(())
}
