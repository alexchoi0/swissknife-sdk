use crate::Result;
use super::types::*;
use super::router::McpRouter;

pub struct McpHandler {
    router: McpRouter,
    initialized: bool,
}

impl McpHandler {
    pub fn new(router: McpRouter) -> Self {
        Self {
            router,
            initialized: false,
        }
    }

    pub async fn handle_message(&mut self, message: &str) -> Result<Option<String>> {
        let request: JsonRpcRequest = match serde_json::from_str(message) {
            Ok(req) => req,
            Err(e) => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: RequestId::Number(0),
                    result: None,
                    error: Some(JsonRpcError::parse_error(e.to_string())),
                };
                return Ok(Some(serde_json::to_string(&response)?));
            }
        };

        let result = self.handle_request(&request).await;

        let response = match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        };

        Ok(Some(serde_json::to_string(&response)?))
    }

    async fn handle_request(&mut self, request: &JsonRpcRequest) -> std::result::Result<serde_json::Value, JsonRpcError> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params.clone()).await,
            "initialized" => self.handle_initialized().await,
            "ping" => self.handle_ping().await,
            "tools/list" => self.handle_list_tools(request.params.clone()).await,
            "tools/call" => self.handle_call_tool(request.params.clone()).await,
            "resources/list" => self.handle_list_resources(request.params.clone()).await,
            "resources/read" => self.handle_read_resource(request.params.clone()).await,
            "prompts/list" => self.handle_list_prompts(request.params.clone()).await,
            "prompts/get" => self.handle_get_prompt(request.params.clone()).await,
            _ => Err(JsonRpcError::method_not_found(format!(
                "Unknown method: {}", request.method
            ))),
        }
    }

    async fn handle_initialize(
        &mut self,
        params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let _params: InitializeParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing initialize params"))
            .and_then(|p| serde_json::from_value(p)
                .map_err(|e| JsonRpcError::invalid_params(e.to_string())))?;

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: self.router.capabilities(),
            server_info: self.router.server_info(),
            instructions: self.router.instructions(),
        };

        self.initialized = true;
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    async fn handle_initialized(&mut self) -> std::result::Result<serde_json::Value, JsonRpcError> {
        Ok(serde_json::Value::Null)
    }

    async fn handle_ping(&self) -> std::result::Result<serde_json::Value, JsonRpcError> {
        Ok(serde_json::json!({}))
    }

    async fn handle_list_tools(
        &self,
        _params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let tools = self.router.list_tools();
        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    async fn handle_call_tool(
        &self,
        params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let params: CallToolParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing tool call params"))
            .and_then(|p| serde_json::from_value(p)
                .map_err(|e| JsonRpcError::invalid_params(e.to_string())))?;

        let result = self.router.call_tool(&params.name, params.arguments).await
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    async fn handle_list_resources(
        &self,
        _params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let resources = self.router.list_resources();
        let result = ListResourcesResult {
            resources,
            next_cursor: None,
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    async fn handle_read_resource(
        &self,
        params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let params: ReadResourceParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing resource params"))
            .and_then(|p| serde_json::from_value(p)
                .map_err(|e| JsonRpcError::invalid_params(e.to_string())))?;

        let content = self.router.read_resource(&params.uri).await
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

        let result = ReadResourceResult {
            contents: vec![content],
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    async fn handle_list_prompts(
        &self,
        _params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let prompts = self.router.list_prompts();
        let result = ListPromptsResult {
            prompts,
            next_cursor: None,
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    async fn handle_get_prompt(
        &self,
        params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let params: GetPromptParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing prompt params"))
            .and_then(|p| serde_json::from_value(p)
                .map_err(|e| JsonRpcError::invalid_params(e.to_string())))?;

        let arguments = serde_json::to_value(params.arguments)
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

        let content = self.router.get_prompt(&params.name, arguments).await
            .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

        serde_json::to_value(content).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }
}
