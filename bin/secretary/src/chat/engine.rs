use swissknife_ai_sdk::llm::anthropic::AnthropicClient;
use swissknife_ai_sdk::llm::voyage::VoyageClient;
use swissknife_ai_sdk::llm::{
    ChatMessage, ChatProvider, ChatRequest, ChatResponse, EmbeddingProvider, EmbeddingRequest,
    MessageContent, MessageRole, ToolCall,
};
use swissknife_ai_sdk::memory::{ActionType, DuckDBMemory};

use crate::config::Config;
use crate::tools::ToolRegistry;

const EMBEDDING_MODEL: &str = "voyage-code-3";

pub struct ChatEngine<'a> {
    chat_client: AnthropicClient,
    embedding_client: Option<VoyageClient>,
    memory: &'a DuckDBMemory,
    session_id: &'a str,
    config: &'a Config,
    tool_registry: &'a ToolRegistry,
}

impl<'a> ChatEngine<'a> {
    pub fn new(
        memory: &'a DuckDBMemory,
        session_id: &'a str,
        config: &'a Config,
        tool_registry: &'a ToolRegistry,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let anthropic_key = config
            .get_anthropic_key()
            .ok_or("ANTHROPIC_API_KEY not found. Set it via config or environment variable.")?;

        let embedding_client = config.get_voyage_key().map(VoyageClient::from_api_key);

        Ok(Self {
            chat_client: AnthropicClient::from_api_key(anthropic_key),
            embedding_client,
            memory,
            session_id,
            config,
            tool_registry,
        })
    }

    pub fn load_history(&self) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {
        let actions = self.memory.get_actions(self.session_id)?;
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

    pub async fn generate_embedding(&self, text: &str) -> Option<Vec<f32>> {
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

    pub async fn store_message(
        &self,
        role: &str,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let action_id = self.memory.add_message(self.session_id, role, content)?;
        if let Some(embedding) = self.generate_embedding(content).await {
            self.memory.add_embedding(&action_id, &embedding)?;
        }
        Ok(action_id)
    }

    pub async fn search_context(&self, query: &str, limit: usize) -> Vec<String> {
        if let Some(embedding) = self.generate_embedding(query).await {
            match self.memory.search_similar(&embedding, limit) {
                Ok(results) => results.into_iter().map(|r| r.action.content).collect(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }

    pub async fn chat(
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

    pub async fn process_tool_calls(
        &self,
        tool_calls: &[ToolCall],
        messages: &mut Vec<ChatMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for tool_call in tool_calls {
            let source = self.tool_registry.tool_source(&tool_call.function.name);
            println!(
                " [{}] {}: {}",
                source, tool_call.function.name, tool_call.function.arguments
            );
            self.memory.add_tool_call(
                self.session_id,
                &tool_call.id,
                &tool_call.function.name,
                &tool_call.function.arguments,
            )?;

            let result = self
                .tool_registry
                .execute_tool(&tool_call.function.name, &tool_call.function.arguments, self.memory)
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
                .add_tool_result(self.session_id, &tool_call.id, &result_str)?;
            messages.push(ChatMessage::tool_result(&tool_call.id, &result_str));
        }
        Ok(())
    }

    pub fn store_thinking(&self, thinking: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.memory.add_thinking(self.session_id, thinking)?;
        Ok(())
    }

    pub fn build_assistant_message_with_tools(
        &self,
        content: &str,
        tool_calls: &[ToolCall],
    ) -> ChatMessage {
        let msg_content = if content.is_empty() {
            " ".to_string()
        } else {
            content.to_string()
        };
        ChatMessage {
            role: MessageRole::Assistant,
            content: MessageContent::Text(msg_content),
            name: None,
            tool_call_id: None,
            tool_calls: Some(tool_calls.to_vec()),
        }
    }
}
