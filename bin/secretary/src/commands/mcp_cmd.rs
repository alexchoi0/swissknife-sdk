use crate::cli::McpCommands;
use crate::config::{self, Config};

pub fn handle_mcp_command(command: &McpCommands) {
    match command {
        McpCommands::List => {
            let config = Config::load();
            if config.mcp.servers.is_empty() {
                println!("No MCP servers configured.");
            } else {
                for (i, server) in config.mcp.servers.iter().enumerate() {
                    println!("{}. {}", i + 1, server);
                }
            }
        }
        McpCommands::Add { command } => match config::add_mcp_server(command) {
            Ok(_) => println!("Added MCP server: {}", command),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        McpCommands::Remove { name } => match config::remove_mcp_server(name) {
            Ok(true) => println!("Removed MCP server matching: {}", name),
            Ok(false) => {
                eprintln!("No MCP server found matching: {}", name);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}
