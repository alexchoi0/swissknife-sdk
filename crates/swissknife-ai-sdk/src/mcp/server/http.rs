use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Router,
};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct McpHttpServerConfig {
    pub name: String,
    pub version: String,
    pub cors_origins: Vec<String>,
}

impl Default for McpHttpServerConfig {
    fn default() -> Self {
        Self {
            name: "swissknife-mcp".to_string(),
            version: "0.1.0".to_string(),
            cors_origins: vec!["*".to_string()],
        }
    }
}

#[derive(Clone, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub tools: Vec<ToolInfo>,
}

#[derive(Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Serialize)]
pub struct ToolCallResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct SseMessage {
    pub event_type: String,
    pub data: serde_json::Value,
}

pub type ToolHandler = Arc<
    dyn Fn(serde_json::Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send>>
        + Send
        + Sync,
>;

pub struct McpHttpServer {
    config: McpHttpServerConfig,
    tools: Arc<RwLock<HashMap<String, (ToolInfo, ToolHandler)>>>,
    event_tx: broadcast::Sender<SseMessage>,
}

impl McpHttpServer {
    pub fn new(config: McpHttpServerConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            config,
            tools: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    pub async fn register_tool<F, Fut>(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
        handler: F,
    ) where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static,
    {
        let name = name.into();
        let info = ToolInfo {
            name: name.clone(),
            description: description.into(),
            parameters,
        };

        let handler: ToolHandler = Arc::new(move |args| Box::pin(handler(args)));

        let mut tools = self.tools.write().await;
        tools.insert(name, (info, handler));
    }

    pub fn broadcast(&self, event_type: impl Into<String>, data: serde_json::Value) {
        let _ = self.event_tx.send(SseMessage {
            event_type: event_type.into(),
            data,
        });
    }

    pub async fn serve(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state = Arc::new(AppState {
            config: self.config,
            tools: self.tools,
            event_tx: self.event_tx,
        });

        let app = Router::new()
            .route("/", get(health_handler))
            .route("/health", get(health_handler))
            .route("/info", get(info_handler))
            .route("/tools", get(list_tools_handler))
            .route("/tools/:name", post(call_tool_handler))
            .route("/call", post(call_tool_by_body_handler))
            .route("/sse", get(sse_handler))
            .route("/events", get(sse_handler))
            .layer(tower_http::cors::CorsLayer::permissive())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        println!("MCP HTTP server listening on http://{}", addr);

        axum::serve(listener, app).await?;
        Ok(())
    }
}

struct AppState {
    config: McpHttpServerConfig,
    tools: Arc<RwLock<HashMap<String, (ToolInfo, ToolHandler)>>>,
    event_tx: broadcast::Sender<SseMessage>,
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn info_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tools = state.tools.read().await;
    let tool_list: Vec<ToolInfo> = tools.values().map(|(info, _)| info.clone()).collect();

    Json(ServerInfo {
        name: state.config.name.clone(),
        version: state.config.version.clone(),
        tools: tool_list,
    })
}

async fn list_tools_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tools = state.tools.read().await;
    let tool_list: Vec<ToolInfo> = tools.values().map(|(info, _)| info.clone()).collect();
    Json(tool_list)
}

async fn call_tool_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(arguments): Json<serde_json::Value>,
) -> impl IntoResponse {
    let tools = state.tools.read().await;

    match tools.get(&name) {
        Some((_, handler)) => {
            let handler = handler.clone();
            drop(tools);

            match handler(arguments).await {
                Ok(result) => (
                    StatusCode::OK,
                    Json(ToolCallResponse {
                        success: true,
                        result: Some(result),
                        error: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ToolCallResponse {
                        success: false,
                        result: None,
                        error: Some(e),
                    }),
                ),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ToolCallResponse {
                success: false,
                result: None,
                error: Some(format!("Tool '{}' not found", name)),
            }),
        ),
    }
}

async fn call_tool_by_body_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ToolCallRequest>,
) -> impl IntoResponse {
    let tools = state.tools.read().await;

    match tools.get(&request.name) {
        Some((_, handler)) => {
            let handler = handler.clone();
            drop(tools);

            match handler(request.arguments).await {
                Ok(result) => {
                    let _ = state.event_tx.send(SseMessage {
                        event_type: "tool_result".to_string(),
                        data: serde_json::json!({
                            "tool": request.name,
                            "result": result
                        }),
                    });

                    (
                        StatusCode::OK,
                        Json(ToolCallResponse {
                            success: true,
                            result: Some(result),
                            error: None,
                        }),
                    )
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ToolCallResponse {
                        success: false,
                        result: None,
                        error: Some(e),
                    }),
                ),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ToolCallResponse {
                success: false,
                result: None,
                error: Some(format!("Tool '{}' not found", request.name)),
            }),
        ),
    }
}

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_tx.subscribe();

    let stream = async_stream::stream! {
        yield Ok(Event::default().event("connected").data("Connected to MCP SSE server"));

        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let data = serde_json::to_string(&msg.data).unwrap_or_default();
                    yield Ok(Event::default().event(msg.event_type).data(data));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)))
}
