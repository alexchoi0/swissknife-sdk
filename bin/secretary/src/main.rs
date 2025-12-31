use std::io::{self, BufRead, Write};
use swissknife_ai_sdk::llm::anthropic::AnthropicClient;
use swissknife_ai_sdk::llm::voyage::VoyageClient;
use swissknife_ai_sdk::llm::{ChatMessage, ChatProvider, ChatRequest, ChatResponse, EmbeddingProvider, EmbeddingRequest};
use swissknife_ai_sdk::memory::{ActionType, DuckDBMemory, MemoryConfig};
use uuid::Uuid;

const MODEL: &str = "claude-haiku-4-5";
const EMBEDDING_MODEL: &str = "voyage-code-3";
const MAX_TOKENS: u32 = 16000;
const THINKING_BUDGET: u32 = 10000;

struct Secretary {
    chat_client: AnthropicClient,
    embedding_client: Option<VoyageClient>,
    memory: DuckDBMemory,
    session_id: String,
    thinking_enabled: bool,
}

impl Secretary {
    fn new(session_id: Option<String>, thinking_enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY not found")?;

        let embedding_client = std::env::var("VOYAGE_API_KEY")
            .ok()
            .map(|key| VoyageClient::from_api_key(key));

        let db_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("secretary")
            .join("secretary.duckdb");

        let config = MemoryConfig::new()
            .with_db_path(db_path.to_string_lossy());

        let memory = DuckDBMemory::new(config)?;

        let session_id = match session_id {
            Some(id) => {
                let session = memory.get_or_create_session(&id)?;
                eprintln!("Session: {} ({})", session.session_id, session.title.as_deref().unwrap_or("Untitled"));
                session.session_id
            }
            None => {
                let sessions = memory.list_sessions(1)?;
                if let Some(session) = sessions.into_iter().next() {
                    eprintln!("Resuming session: {} ({})", session.session_id, session.title.as_deref().unwrap_or("Untitled"));
                    session.session_id
                } else {
                    let new_id = Uuid::new_v4().to_string();
                    memory.create_session(&new_id, None)?;
                    eprintln!("New session: {}", new_id);
                    new_id
                }
            }
        };

        if thinking_enabled {
            eprintln!("Extended thinking enabled (budget: {} tokens)", THINKING_BUDGET);
        }

        Ok(Self {
            chat_client: AnthropicClient::from_api_key(anthropic_key),
            embedding_client,
            memory,
            session_id,
            thinking_enabled,
        })
    }

    fn load_history(&self) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {
        let actions = self.memory.get_actions(&self.session_id)?;
        let mut messages = vec![ChatMessage::system("You are Secretary, a helpful assistant.")];

        for action in actions {
            match action.action_type {
                ActionType::Message => {
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
                _ => {}
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

    async fn store_message(&self, role: &str, content: &str) -> Result<String, Box<dyn std::error::Error>> {
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

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, Box<dyn std::error::Error>> {
        let mut request = ChatRequest::new(MODEL, messages.to_vec()).with_max_tokens(MAX_TOKENS);

        if self.thinking_enabled {
            request = request.with_thinking(THINKING_BUDGET);
        }

        let response = self.chat_client.chat(&request).await?;
        Ok(response)
    }

    fn update_title_if_needed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let count = self.memory.action_count(&self.session_id)?;
        if count == 2 {
            if let Some(action) = self.memory.get_messages(&self.session_id)?.first() {
                let title: String = action.content.chars().take(50).collect();
                self.memory.update_session_title(&self.session_id, &title)?;
            }
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

            match input {
                "exit" | "quit" => break,
                "/new" => {
                    eprintln!("Use --new flag or --session <id> to manage sessions");
                    continue;
                }
                "/sessions" => {
                    match self.memory.list_sessions(10) {
                        Ok(sessions) => {
                            for session in sessions {
                                let marker = if session.session_id == self.session_id { "* " } else { "  " };
                                println!(
                                    "{}{}: {} ({})",
                                    marker,
                                    &session.session_id[..8],
                                    session.title.as_deref().unwrap_or("Untitled"),
                                    session.updated_at.format("%Y-%m-%d %H:%M")
                                );
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    continue;
                }
                "/actions" => {
                    match self.memory.get_actions(&self.session_id) {
                        Ok(actions) => {
                            for action in actions {
                                let type_str = match action.action_type {
                                    ActionType::Message => format!("[{}]", action.role.as_deref().unwrap_or("?")),
                                    ActionType::ToolCall => format!("[tool:{}]", action.tool_name.as_deref().unwrap_or("?")),
                                    ActionType::ToolResult => "[result]".to_string(),
                                    ActionType::Thinking => "[thinking]".to_string(),
                                };
                                println!(
                                    "{:3}. {} {}",
                                    action.sequence,
                                    type_str,
                                    action.content.chars().take(60).collect::<String>()
                                );
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    continue;
                }
                "/tools" => {
                    match self.memory.get_tool_calls(&self.session_id) {
                        Ok(actions) => {
                            for action in actions {
                                println!(
                                    "{}: {}",
                                    action.tool_name.as_deref().unwrap_or("?"),
                                    action.tool_input.as_deref().unwrap_or("").chars().take(80).collect::<String>()
                                );
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    continue;
                }
                "/search" => {
                    print!("Search query: ");
                    stdout.flush()?;
                    let mut query = String::new();
                    stdin.lock().read_line(&mut query)?;
                    let results = self.search_context(query.trim(), 5).await;
                    if results.is_empty() {
                        println!("No similar actions found");
                    } else {
                        for (i, content) in results.iter().enumerate() {
                            println!("{}. {}", i + 1, content.chars().take(100).collect::<String>());
                        }
                    }
                    continue;
                }
                _ => {}
            }

            self.store_message("user", input).await?;
            messages.push(ChatMessage::user(input));

            match self.chat(&messages).await {
                Ok(response) => {
                    if let Some(thinking) = response.thinking() {
                        println!("\n💭 Thinking:\n{}\n", thinking);
                        self.memory.add_thinking(&self.session_id, thinking)?;
                    }
                    let content = response.content().unwrap_or("");
                    println!("Secretary: {}", content);
                    self.store_message("assistant", content).await?;
                    messages.push(ChatMessage::assistant(content));
                    self.update_title_if_needed()?;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();

    let session_id = if args.contains(&"--new".to_string()) {
        Some(Uuid::new_v4().to_string())
    } else if let Some(pos) = args.iter().position(|a| a == "--session") {
        args.get(pos + 1).cloned()
    } else {
        None
    };

    let thinking_enabled = args.contains(&"--think".to_string());

    match Secretary::new(session_id, thinking_enabled) {
        Ok(secretary) => {
            if let Err(e) = secretary.run().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
