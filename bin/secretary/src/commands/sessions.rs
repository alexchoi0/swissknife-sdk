use crate::cli::SessionsCommands;
use crate::format::{format_action_type, format_session, truncate, PREVIEW_LONG};
use swissknife_ai_sdk::memory::DuckDBMemory;

pub fn handle_sessions_command(command: &SessionsCommands, memory: &DuckDBMemory) {
    match command {
        SessionsCommands::List { limit } => {
            match memory.list_sessions(*limit) {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        println!("No sessions found.");
                    } else {
                        for session in sessions {
                            println!("{}", format_session(&session, None));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionsCommands::Delete { id: _ } => {
            eprintln!("Session deletion not yet implemented.");
            std::process::exit(1);
        }
        SessionsCommands::Show { id } => {
            match memory.get_actions(id) {
                Ok(actions) => {
                    for action in actions {
                        let type_str = format_action_type(&action);
                        println!(
                            "{:3}. {} {}",
                            action.sequence,
                            type_str,
                            truncate(&action.content, PREVIEW_LONG)
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
