mod cli;
mod commands;
mod config;
mod db;
mod format;
mod mcp_client;
mod sdk_tools_server;
mod tool_registry;
mod tools;

use clap::Parser;
use cli::{ChatCommands, Cli, Commands, ConfigCommands, McpCommands};
use config::Config;
use db::get_memory;
use format::{format_action_type, format_session, truncate, PREVIEW_LONG, PREVIEW_SHORT};
use std::io::{self, BufRead, Write};
use swissknife_ai_sdk::llm::anthropic::AnthropicClient;
use swissknife_ai_sdk::llm::voyage::VoyageClient;
use swissknife_ai_sdk::llm::{
    ChatMessage, ChatProvider, ChatRequest, ChatResponse, EmbeddingProvider, EmbeddingRequest,
};
use swissknife_ai_sdk::memory::{ActionType, DuckDBMemory};
use tool_registry::ToolRegistry;
use uuid::Uuid;

const EMBEDDING_MODEL: &str = "voyage-code-3";

struct Secretary {
    chat_client: AnthropicClient,
    embedding_client: Option<VoyageClient>,
    memory: DuckDBMemory,
    session_id: String,
    config: Config,
    tool_registry: ToolRegistry,
}

impl Secretary {
    fn new(
        session_id: Option<String>,
        config: Config,
        tool_registry: ToolRegistry,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let anthropic_key = config
            .get_anthropic_key()
            .ok_or("ANTHROPIC_API_KEY not found. Set it via config or environment variable.")?;

        let embedding_client = config.get_voyage_key().map(VoyageClient::from_api_key);
        let memory = get_memory()?;

        let session_id = match session_id {
            Some(id) => {
                let session = memory.get_or_create_session(&id)?;
                eprintln!(
                    "Session: {} ({})",
                    session.session_id,
                    session.title.as_deref().unwrap_or("Untitled")
                );
                session.session_id
            }
            None => {
                let sessions = memory.list_sessions(1)?;
                if let Some(session) = sessions.into_iter().next() {
                    eprintln!(
                        "Resuming session: {} ({})",
                        session.session_id,
                        session.title.as_deref().unwrap_or("Untitled")
                    );
                    session.session_id
                } else {
                    let new_id = Uuid::new_v4().to_string();
                    memory.create_session(&new_id, None)?;
                    eprintln!("New session: {}", new_id);
                    new_id
                }
            }
        };

        if config.thinking_enabled() {
            eprintln!(
                "Extended thinking enabled (budget: {} tokens)",
                config.model.thinking_budget
            );
        }

        if tool_registry.has_tools() {
            tool_registry.print_available_tools();
        }

        Ok(Self {
            chat_client: AnthropicClient::from_api_key(anthropic_key),
            embedding_client,
            memory,
            session_id,
            config,
            tool_registry,
        })
    }

    fn load_history(&self) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {
        let actions = self.memory.get_actions(&self.session_id)?;
        let system_prompt = if self.tool_registry.has_tools() {
            "You are Secretary, a helpful assistant with access to tools. Use tools when appropriate to help the user."
        } else {
            "You are Secretary, a helpful assistant."
        };
        let mut messages = vec![ChatMessage::system(system_prompt)];

        for action in actions {
            if action.action_type == ActionType::Message {
                if let Some(role) = &action.role {
                    let chat_msg = match role.as_str() {
                        "user" => ChatMessage::user(&action.content),
                        "assistant" => ChatMessage::assistant(&action.content),
                        "system" => ChatMessage::system(&action.content),
                        _ => continue,
                    };
                    messages.push(chat_msg);
                }
            }
        }
        Ok(messages)
    }

    async fn generate_embedding(&self, text: &str) -> Option<Vec<f32>> {
        let client = self.embedding_client.as_ref()?;
        let request = EmbeddingRequest::single(EMBEDDING_MODEL, text);
        match client.embed(&request).await {
            Ok(response) => response.first().map(|e| e.to_vec()),
            Err(e) => {
                eprintln!("Embedding error: {}", e);
                None
            }
        }
    }

    async fn store_message(
        &self,
        role: &str,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let action_id = self.memory.add_message(&self.session_id, role, content)?;
        if let Some(embedding) = self.generate_embedding(content).await {
            self.memory.add_embedding(&action_id, &embedding)?;
        }
        Ok(action_id)
    }

    async fn search_context(&self, query: &str, limit: usize) -> Vec<String> {
        if let Some(embedding) = self.generate_embedding(query).await {
            match self.memory.search_similar(&embedding, limit) {
                Ok(results) => results.into_iter().map(|r| r.action.content).collect(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }

    async fn chat(
        &self,
        messages: &[ChatMessage],
    ) -> Result<ChatResponse, Box<dyn std::error::Error>> {
        let mut request = ChatRequest::new(&self.config.model.name, messages.to_vec())
            .with_max_tokens(self.config.model.max_tokens);

        if self.config.thinking_enabled() {
            request = request.with_thinking(self.config.model.thinking_budget);
        }

        let tools = self.tool_registry.all_tool_definitions();
        if !tools.is_empty() {
            request = request.with_tools(tools);
        }

        Ok(self.chat_client.chat(&request).await?)
    }

    fn update_title_if_needed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let count = self.memory.action_count(&self.session_id)?;
        if count == 2 {
            if let Some(action) = self.memory.get_messages(&self.session_id)?.first() {
                let title = truncate(&action.content, 50);
                self.memory.update_session_title(&self.session_id, &title)?;
            }
        }
        Ok(())
    }

    fn handle_command(&self, input: &str) -> Option<bool> {
        match input {
            "exit" | "quit" => Some(true),
            "/sessions" => {
                match self.memory.list_sessions(10) {
                    Ok(sessions) => {
                        for session in sessions {
                            println!("{}", format_session(&session, Some(&self.session_id)));
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
                Some(false)
            }
            "/actions" => {
                match self.memory.get_actions(&self.session_id) {
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
                match self.memory.get_tool_calls(&self.session_id) {
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

    async fn handle_search(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        print!("Search query: ");
        stdout.flush()?;
        let mut query = String::new();
        stdin.lock().read_line(&mut query)?;
        let results = self.search_context(query.trim(), 5).await;
        if results.is_empty() {
            println!("No similar actions found");
        } else {
            for (i, content) in results.iter().enumerate() {
                println!("{}. {}", i + 1, truncate(content, 100));
            }
        }
        Ok(())
    }

    async fn process_tool_calls(
        &self,
        tool_calls: &[swissknife_ai_sdk::llm::ToolCall],
        messages: &mut Vec<ChatMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for tool_call in tool_calls {
            let source = self.tool_registry.tool_source(&tool_call.function.name);
            println!(
                " [{}] {}: {}",
                source, tool_call.function.name, tool_call.function.arguments
            );
            self.memory.add_tool_call(
                &self.session_id,
                &tool_call.id,
                &tool_call.function.name,
                &tool_call.function.arguments,
            )?;

            let result = self
                .tool_registry
                .execute_tool(&tool_call.function.name, &tool_call.function.arguments)
                .await;
            let result_str = match &result {
                Ok(output) => {
                    let truncated = if output.len() > 500 {
                        format!("{}... (truncated)", &output[..500])
                    } else {
                        output.clone()
                    };
                    println!("   OK {}", truncated.replace('\n', "\n     "));
                    output.clone()
                }
                Err(e) => {
                    println!("   Error: {}", e);
                    format!("Error: {}", e)
                }
            };

            self.memory
                .add_tool_result(&self.session_id, &tool_call.id, &result_str)?;
            messages.push(ChatMessage::tool_result(&tool_call.id, &result_str));
        }
        Ok(())
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut messages = self.load_history()?;

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

            if let Some(should_exit) = self.handle_command(input) {
                if should_exit {
                    break;
                }
                continue;
            }

            if input == "/search" {
                self.handle_search().await?;
                continue;
            }

            self.store_message("user", input).await?;
            messages.push(ChatMessage::user(input));

            loop {
                match self.chat(&messages).await {
                    Ok(response) => {
                        if let Some(thinking) = response.thinking() {
                            println!("\n Thinking:\n{}\n", thinking);
                            self.memory.add_thinking(&self.session_id, thinking)?;
                        }

                        if let Some(tool_calls) = response.tool_calls() {
                            let content = response.content().unwrap_or("");
                            if !content.is_empty() {
                                println!("Secretary: {}", content);
                            }

                            let msg_content = if content.is_empty() {
                                " ".to_string()
                            } else {
                                content.to_string()
                            };
                            let assistant_msg = ChatMessage {
                                role: swissknife_ai_sdk::llm::MessageRole::Assistant,
                                content: swissknife_ai_sdk::llm::MessageContent::Text(msg_content),
                                name: None,
                                tool_call_id: None,
                                tool_calls: Some(tool_calls.to_vec()),
                            };
                            messages.push(assistant_msg);

                            self.process_tool_calls(tool_calls, &mut messages).await?;
                            continue;
                        }

                        let content = response.content().unwrap_or("");
                        println!("Secretary: {}", content);
                        self.store_message("assistant", content).await?;
                        messages.push(ChatMessage::assistant(content));
                        self.update_title_if_needed()?;
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
}

async fn run_chat(
    cli: &Cli,
    config: Config,
    session_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let builtin_tools = config.tools.builtin && !cli.no_builtin;
    let sdk_tools = config.tools.sdk && !cli.no_sdk;

    let mut tool_registry = ToolRegistry::new(builtin_tools);

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

    let secretary = Secretary::new(session_id, config, tool_registry)?;
    secretary.run().await
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let config = match &cli.config {
        Some(path) => Config::load_from(path),
        None => Config::load(),
    };

    let config = apply_cli_overrides(config, &cli);

    match &cli.command {
        None => {
            if let Err(e) = run_chat(&cli, config, None).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Chat { command }) => {
            let session_id = match command {
                Some(ChatCommands::New) => Some(Uuid::new_v4().to_string()),
                Some(ChatCommands::Resume { id }) => Some(id.clone()),
                None => None,
            };
            if let Err(e) = run_chat(&cli, config, session_id).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Sessions { command }) => {
            commands::handle_sessions_command(command);
        }
        Some(Commands::Config { command }) => {
            handle_config_command(command, &config);
        }
        Some(Commands::Mcp { command }) => {
            handle_mcp_command(command);
        }
        Some(Commands::Import { command }) => {
            commands::handle_import_command(command);
        }
        Some(Commands::History { command }) => {
            commands::handle_history_command(command);
        }
    }
}

fn apply_cli_overrides(mut config: Config, cli: &Cli) -> Config {
    if cli.think && config.model.thinking_budget == 0 {
        config.model.thinking_budget = 10000;
    }
    if cli.no_think {
        config.model.thinking_budget = 0;
    }
    config
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
        ConfigCommands::Path => {
            println!("{}", Config::config_path().display());
        }
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
