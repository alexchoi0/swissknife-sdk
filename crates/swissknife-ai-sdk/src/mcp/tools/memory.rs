use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "memory")]
use swissknife_memory_sdk as mem;

#[derive(Clone)]
pub struct MemoryTools {
    #[cfg(feature = "mem0")]
    pub mem0: Option<mem::mem0::Mem0Client>,
    #[cfg(feature = "zep")]
    pub zep: Option<mem::zep::ZepClient>,
}

impl MemoryTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "mem0")]
            mem0: None,
            #[cfg(feature = "zep")]
            zep: None,
        }
    }

    #[cfg(feature = "mem0")]
    pub fn with_mem0(mut self, client: mem::mem0::Mem0Client) -> Self {
        self.mem0 = Some(client);
        self
    }

    #[cfg(feature = "zep")]
    pub fn with_zep(mut self, client: mem::zep::ZepClient) -> Self {
        self.zep = Some(client);
        self
    }
}

impl Default for MemoryTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Mem0Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Mem0AddMemoryRequest {
    pub messages: Vec<Mem0Message>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Mem0GetMemoriesRequest {
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Mem0SearchMemoriesRequest {
    pub query: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Mem0DeleteMemoryRequest {
    pub memory_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZepMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZepAddMemoryRequest {
    pub session_id: String,
    pub messages: Vec<ZepMessage>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZepGetMemoryRequest {
    pub session_id: String,
    #[serde(default)]
    pub lastn: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZepSearchMemoryRequest {
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZepCreateUserRequest {
    pub user_id: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZepGetUserRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZepCreateSessionRequest {
    pub session_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[tool_box]
impl MemoryTools {
    #[cfg(feature = "mem0")]
    #[rmcp::tool(description = "Add a memory to Mem0")]
    pub async fn mem0_add_memory(
        &self,
        #[rmcp::tool(aggr)] req: Mem0AddMemoryRequest,
    ) -> Result<String, String> {
        let client = self.mem0.as_ref()
            .ok_or_else(|| "Mem0 client not configured".to_string())?;

        let mem_messages: Vec<mem::mem0::Message> = req.messages.iter()
            .map(|m| mem::mem0::Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let mut request = mem::mem0::AddMemoryRequest::new(mem_messages);

        if let Some(user_id) = req.user_id {
            request = request.with_user_id(user_id);
        }
        if let Some(agent_id) = req.agent_id {
            request = request.with_agent_id(agent_id);
        }
        if let Some(run_id) = req.run_id {
            request = request.with_run_id(run_id);
        }
        if let Some(metadata) = req.metadata {
            request = request.with_metadata(metadata);
        }

        let response = client.add(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "memories": response.results.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "memory": r.memory,
                    "event": r.event
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "mem0")]
    #[rmcp::tool(description = "Get memories from Mem0")]
    pub async fn mem0_get_memories(
        &self,
        #[rmcp::tool(aggr)] req: Mem0GetMemoriesRequest,
    ) -> Result<String, String> {
        let client = self.mem0.as_ref()
            .ok_or_else(|| "Mem0 client not configured".to_string())?;

        let mut request = mem::mem0::GetMemoriesRequest::new();

        if let Some(user_id) = req.user_id {
            request = request.with_user_id(user_id);
        }
        if let Some(agent_id) = req.agent_id {
            request = request.with_agent_id(agent_id);
        }
        if let Some(run_id) = req.run_id {
            request = request.with_run_id(run_id);
        }

        let memories = client.get_all(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "memories": memories.iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "memory": m.memory,
                    "user_id": m.user_id,
                    "created_at": m.created_at,
                    "updated_at": m.updated_at
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "mem0")]
    #[rmcp::tool(description = "Search memories in Mem0")]
    pub async fn mem0_search_memories(
        &self,
        #[rmcp::tool(aggr)] req: Mem0SearchMemoriesRequest,
    ) -> Result<String, String> {
        let client = self.mem0.as_ref()
            .ok_or_else(|| "Mem0 client not configured".to_string())?;

        let mut request = mem::mem0::SearchMemoriesRequest::new(req.query);

        if let Some(user_id) = req.user_id {
            request = request.with_user_id(user_id);
        }
        if let Some(agent_id) = req.agent_id {
            request = request.with_agent_id(agent_id);
        }
        if let Some(limit) = req.limit {
            request = request.with_limit(limit);
        }

        let results = client.search(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "memory": r.memory,
                    "score": r.score,
                    "user_id": r.user_id
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "mem0")]
    #[rmcp::tool(description = "Delete a memory from Mem0")]
    pub async fn mem0_delete_memory(
        &self,
        #[rmcp::tool(aggr)] req: Mem0DeleteMemoryRequest,
    ) -> Result<String, String> {
        let client = self.mem0.as_ref()
            .ok_or_else(|| "Mem0 client not configured".to_string())?;

        client.delete(&req.memory_id).await
            .map_err(|e| e.to_string())?;

        Ok("Memory deleted successfully".to_string())
    }

    #[cfg(feature = "zep")]
    #[rmcp::tool(description = "Add messages to Zep session memory")]
    pub async fn zep_add_memory(
        &self,
        #[rmcp::tool(aggr)] req: ZepAddMemoryRequest,
    ) -> Result<String, String> {
        let client = self.zep.as_ref()
            .ok_or_else(|| "Zep client not configured".to_string())?;

        let zep_messages: Vec<mem::zep::ZepMessage> = req.messages.iter()
            .map(|m| {
                let role_type = match m.role.as_str() {
                    "assistant" | "ai" => mem::zep::RoleType::Assistant,
                    "system" => mem::zep::RoleType::System,
                    "function" => mem::zep::RoleType::Function,
                    "tool" => mem::zep::RoleType::Tool,
                    _ => mem::zep::RoleType::Human,
                };
                mem::zep::ZepMessage {
                    role: m.role.clone(),
                    role_type,
                    content: m.content.clone(),
                    uuid: None,
                    created_at: None,
                    token_count: None,
                    metadata: None,
                }
            })
            .collect();

        let request = mem::zep::AddMemoryRequest {
            messages: zep_messages,
        };

        client.memory().add(&req.session_id, &request).await
            .map_err(|e| e.to_string())?;

        Ok("Memory added successfully".to_string())
    }

    #[cfg(feature = "zep")]
    #[rmcp::tool(description = "Get memory from a Zep session")]
    pub async fn zep_get_memory(
        &self,
        #[rmcp::tool(aggr)] req: ZepGetMemoryRequest,
    ) -> Result<String, String> {
        let client = self.zep.as_ref()
            .ok_or_else(|| "Zep client not configured".to_string())?;

        let memory = client.memory().get(&req.session_id, req.lastn).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "messages": memory.messages.iter().map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                    "uuid": m.uuid,
                    "created_at": m.created_at
                })
            }).collect::<Vec<_>>(),
            "summary": memory.summary.map(|s| {
                serde_json::json!({
                    "content": s.content,
                    "uuid": s.uuid
                })
            }),
            "facts": memory.facts
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "zep")]
    #[rmcp::tool(description = "Search Zep session memory")]
    pub async fn zep_search_memory(
        &self,
        #[rmcp::tool(aggr)] req: ZepSearchMemoryRequest,
    ) -> Result<String, String> {
        let client = self.zep.as_ref()
            .ok_or_else(|| "Zep client not configured".to_string())?;

        let request = mem::zep::SearchMemoryRequest {
            text: req.text,
            metadata: None,
            search_type: None,
            search_scope: None,
            mmr_lambda: None,
        };

        let results = client.memory().search(&req.session_id, &request, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "message": r.message.as_ref().map(|m| {
                        serde_json::json!({
                            "role": m.role,
                            "content": m.content
                        })
                    }),
                    "summary": r.summary.as_ref().map(|s| &s.content),
                    "score": r.score,
                    "dist": r.dist
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "zep")]
    #[rmcp::tool(description = "Create a user in Zep")]
    pub async fn zep_create_user(
        &self,
        #[rmcp::tool(aggr)] req: ZepCreateUserRequest,
    ) -> Result<String, String> {
        let client = self.zep.as_ref()
            .ok_or_else(|| "Zep client not configured".to_string())?;

        let request = mem::zep::CreateUserRequest {
            user_id: req.user_id,
            email: req.email,
            first_name: req.first_name,
            last_name: req.last_name,
            metadata: req.metadata,
        };

        let user = client.users().create(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "user_id": user.user_id,
            "email": user.email,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "created_at": user.created_at
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "zep")]
    #[rmcp::tool(description = "Get a user from Zep")]
    pub async fn zep_get_user(
        &self,
        #[rmcp::tool(aggr)] req: ZepGetUserRequest,
    ) -> Result<String, String> {
        let client = self.zep.as_ref()
            .ok_or_else(|| "Zep client not configured".to_string())?;

        let user = client.users().get(&req.user_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "user_id": user.user_id,
            "email": user.email,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "created_at": user.created_at,
            "updated_at": user.updated_at
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "zep")]
    #[rmcp::tool(description = "Create a session in Zep")]
    pub async fn zep_create_session(
        &self,
        #[rmcp::tool(aggr)] req: ZepCreateSessionRequest,
    ) -> Result<String, String> {
        let client = self.zep.as_ref()
            .ok_or_else(|| "Zep client not configured".to_string())?;

        let request = mem::zep::CreateSessionRequest {
            session_id: req.session_id,
            user_id: req.user_id,
            metadata: req.metadata,
        };

        let session = client.memory().create_session(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "session_id": session.session_id,
            "user_id": session.user_id,
            "created_at": session.created_at
        })).map_err(|e| e.to_string())
    }
}
