mod app;
mod chat;
mod cli;
mod commands;
mod config;
mod db;
mod error;
mod format;
mod security;
mod tools;

pub use error::{Result, ResultExt, SecretaryError};

use app::App;
use chat::{run_repl, ChatEngine, SessionManager};
use clap::Parser;
use cli::{ChatCommands, Cli, Commands, ConfigCommands, McpCommands};
use config::Config;
use tools::ToolRegistry;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    security::init_sensitive_inodes();

    let cli = Cli::parse();

    let config = match &cli.config {
        Some(path) => Config::load_from(path),
        None => Config::load(),
    };

    let config = apply_cli_overrides(config, &cli);

    match &cli.command {
        None => run_chat(&cli, config, None).await,
        Some(Commands::Chat { command }) => {
            let session_id = match command {
                Some(ChatCommands::New) => Some(Uuid::new_v4().to_string()),
                Some(ChatCommands::Resume { id }) => Some(id.clone()),
                None => None,
            };
            run_chat(&cli, config, session_id).await;
        }
        Some(Commands::Sessions { command }) => commands::handle_sessions_command(command),
        Some(Commands::Config { command }) => handle_config_command(command, &config),
        Some(Commands::Mcp { command }) => handle_mcp_command(command),
        Some(Commands::Import { command }) => commands::handle_import_command(command),
        Some(Commands::History { command }) => commands::handle_history_command(command),
    }
}

async fn run_chat(cli: &Cli, config: Config, session_id: Option<String>) {
    let app = match App::new(config.clone()) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to initialize: {}", e);
            std::process::exit(1);
        }
    };

    let builtin_tools = config.tools.builtin && !cli.no_builtin;
    let sdk_tools = config.tools.sdk && !cli.no_sdk;

    let mut tool_registry = ToolRegistry::new(builtin_tools, builtin_tools);

    if sdk_tools {
        if let Err(e) = tool_registry.enable_sdk_mcp().await {
            eprintln!("Failed to start in-process MCP: {}", e);
            std::process::exit(1);
        }
    }

    for (idx, cmd) in config.mcp.servers.iter().enumerate() {
        let name = format!("mcp-{}", idx);
        if let Err(e) = tool_registry.add_external_mcp(&name, cmd).await {
            eprintln!("Failed to connect to MCP server '{}': {}", cmd, e);
            std::process::exit(1);
        }
    }

    if config.thinking_enabled() {
        eprintln!("Extended thinking enabled (budget: {} tokens)", config.model.thinking_budget);
    }

    if tool_registry.has_tools() {
        tool_registry.print_available_tools();
    }

    let session = match SessionManager::new(&app.memory, session_id) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to initialize session: {}", e);
            std::process::exit(1);
        }
    };

    let engine = match ChatEngine::new(&app.memory, &session.session_id, &app.config, &tool_registry) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to initialize chat engine: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = run_repl(&engine, &session, &app.memory).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn apply_cli_overrides(mut config: Config, cli: &Cli) -> Config {
    if cli.think && config.model.thinking_budget == 0 {
        config.model.thinking_budget = 10000;
    }
    if cli.no_think {
        config.model.thinking_budget = 0;
    }
    if let Some(model) = &cli.model {
        config.model.name = resolve_model_name(model);
    }
    config
}

fn resolve_model_name(model: &str) -> String {
    match model.to_lowercase().as_str() {
        "haiku" => "claude-haiku-4-5-20250514".to_string(),
        "sonnet" => "claude-sonnet-4-20250514".to_string(),
        "opus" => "claude-opus-4-20250514".to_string(),
        _ => model.to_string(),
    }
}

fn handle_config_command(command: &ConfigCommands, config: &Config) {
    match command {
        ConfigCommands::Show => match toml::to_string_pretty(config) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        ConfigCommands::Path => println!("{}", Config::config_path().display()),
        ConfigCommands::Init => {
            let path = Config::config_path();
            if path.exists() {
                eprintln!("Config file already exists at: {}", path.display());
                std::process::exit(1);
            }
            match Config::default().save() {
                Ok(_) => println!("Created config file at: {}", path.display()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ConfigCommands::Set { key, value } => match config::set_config_value(key, value) {
            Ok(_) => println!("Set {} = {}", key, value),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        ConfigCommands::Get { key } => match config::get_config_value(key) {
            Some(value) => println!("{}", value),
            None => {
                eprintln!("Key not found: {}", key);
                std::process::exit(1);
            }
        },
        ConfigCommands::Unset { key } => match config::unset_config_value(key) {
            Ok(_) => println!("Unset {}", key),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}

fn handle_mcp_command(command: &McpCommands) {
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
