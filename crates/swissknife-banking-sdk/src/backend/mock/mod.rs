pub mod entity;

use super::{Backend, HttpRequest, HttpResponse};
use async_trait::async_trait;
use entity::{mock_request, mock_response, scenario};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Schema, Set,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MockBackend {
    db: DatabaseConnection,
    active_scenario: Arc<RwLock<Option<String>>>,
    request_counts: Arc<RwLock<HashMap<i32, i32>>>,
}

impl MockBackend {
    pub async fn new() -> crate::Result<Self> {
        let db = Database::connect("sqlite::memory:")
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to connect to SQLite: {}", e)))?;

        let backend = Self {
            db,
            active_scenario: Arc::new(RwLock::new(None)),
            request_counts: Arc::new(RwLock::new(HashMap::new())),
        };

        backend.create_tables().await?;

        Ok(backend)
    }

    pub async fn with_connection(db: DatabaseConnection) -> crate::Result<Self> {
        let backend = Self {
            db,
            active_scenario: Arc::new(RwLock::new(None)),
            request_counts: Arc::new(RwLock::new(HashMap::new())),
        };

        backend.create_tables().await?;

        Ok(backend)
    }

    async fn create_tables(&self) -> crate::Result<()> {
        let builder = self.db.get_database_backend();
        let schema = Schema::new(builder);

        let scenario_stmt = builder.build(&schema.create_table_from_entity(scenario::Entity));
        self.db
            .execute(scenario_stmt)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create scenarios table: {}", e)))?;

        let request_stmt = builder.build(&schema.create_table_from_entity(mock_request::Entity));
        self.db
            .execute(request_stmt)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create mock_requests table: {}", e)))?;

        let response_stmt = builder.build(&schema.create_table_from_entity(mock_response::Entity));
        self.db
            .execute(response_stmt)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create mock_responses table: {}", e)))?;

        Ok(())
    }

    pub async fn create_scenario(
        &self,
        data: scenario::CreateScenario,
    ) -> crate::Result<scenario::Model> {
        let now = chrono::Utc::now();
        let model = scenario::ActiveModel {
            name: Set(data.name),
            provider: Set(data.provider),
            description: Set(data.description),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create scenario: {}", e)))?;

        Ok(result)
    }

    pub async fn add_mock(
        &self,
        scenario_name: &str,
        request: mock_request::CreateMockRequest,
        response: mock_response::CreateMockResponse,
    ) -> crate::Result<(mock_request::Model, mock_response::Model)> {
        let scenario = scenario::Entity::find()
            .filter(scenario::Column::Name.eq(scenario_name))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to find scenario: {}", e)))?
            .ok_or_else(|| crate::Error::Provider(format!("Scenario not found: {}", scenario_name)))?;

        let max_order = mock_request::Entity::find()
            .filter(mock_request::Column::ScenarioId.eq(scenario.id))
            .order_by_desc(mock_request::Column::SequenceOrder)
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to query max order: {}", e)))?
            .map(|r| r.sequence_order)
            .unwrap_or(0);

        let request_model = mock_request::ActiveModel {
            scenario_id: Set(scenario.id),
            method: Set(request.method),
            path_pattern: Set(request.path_pattern),
            body_pattern: Set(request.body_pattern),
            headers_pattern: Set(request.headers_pattern),
            sequence_order: Set(request.sequence_order.unwrap_or(max_order + 1)),
            times_matched: Set(0),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let created_request = request_model
            .insert(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create mock request: {}", e)))?;

        let response_model = mock_response::ActiveModel {
            request_id: Set(created_request.id),
            status_code: Set(response.status_code),
            headers: Set(response.headers),
            body: Set(response.body),
            delay_ms: Set(response.delay_ms),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let created_response = response_model
            .insert(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create mock response: {}", e)))?;

        Ok((created_request, created_response))
    }

    pub async fn activate_scenario(&self, name: &str) -> crate::Result<()> {
        let scenario = scenario::Entity::find()
            .filter(scenario::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to find scenario: {}", e)))?
            .ok_or_else(|| crate::Error::Provider(format!("Scenario not found: {}", name)))?;

        let mut active = self.active_scenario.write().await;
        *active = Some(scenario.name);

        let mut counts = self.request_counts.write().await;
        counts.clear();

        Ok(())
    }

    pub async fn deactivate_scenario(&self) {
        let mut active = self.active_scenario.write().await;
        *active = None;

        let mut counts = self.request_counts.write().await;
        counts.clear();
    }

    pub async fn list_scenarios(&self) -> crate::Result<Vec<scenario::Model>> {
        scenario::Entity::find()
            .order_by_asc(scenario::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to list scenarios: {}", e)))
    }

    pub async fn get_scenario(&self, name: &str) -> crate::Result<Option<scenario::Model>> {
        scenario::Entity::find()
            .filter(scenario::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get scenario: {}", e)))
    }

    pub async fn delete_scenario(&self, name: &str) -> crate::Result<()> {
        let scenario = scenario::Entity::find()
            .filter(scenario::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to find scenario: {}", e)))?
            .ok_or_else(|| crate::Error::Provider(format!("Scenario not found: {}", name)))?;

        let requests = mock_request::Entity::find()
            .filter(mock_request::Column::ScenarioId.eq(scenario.id))
            .all(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to find requests: {}", e)))?;

        for request in requests {
            mock_response::Entity::delete_many()
                .filter(mock_response::Column::RequestId.eq(request.id))
                .exec(&self.db)
                .await
                .map_err(|e| crate::Error::Provider(format!("Failed to delete responses: {}", e)))?;
        }

        mock_request::Entity::delete_many()
            .filter(mock_request::Column::ScenarioId.eq(scenario.id))
            .exec(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to delete requests: {}", e)))?;

        scenario::Entity::delete_by_id(scenario.id)
            .exec(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to delete scenario: {}", e)))?;

        Ok(())
    }

    async fn find_matching_request(
        &self,
        http_request: &HttpRequest,
    ) -> crate::Result<Option<(mock_request::Model, mock_response::Model)>> {
        let active_scenario = self.active_scenario.read().await;
        let scenario_name = match active_scenario.as_ref() {
            Some(name) => name.clone(),
            None => return Ok(None),
        };
        drop(active_scenario);

        let scenario = match self.get_scenario(&scenario_name).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let requests = mock_request::Entity::find()
            .filter(mock_request::Column::ScenarioId.eq(scenario.id))
            .filter(mock_request::Column::Method.eq(http_request.method.to_string()))
            .order_by_asc(mock_request::Column::SequenceOrder)
            .all(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to query requests: {}", e)))?;

        for request in requests {
            if self.matches_path(&request.path_pattern, &http_request.url)?
                && self.matches_body(&request.body_pattern, &http_request.body)?
                && self.matches_headers(&request.headers_pattern, &http_request.headers)?
            {
                let response = mock_response::Entity::find()
                    .filter(mock_response::Column::RequestId.eq(request.id))
                    .one(&self.db)
                    .await
                    .map_err(|e| crate::Error::Provider(format!("Failed to query response: {}", e)))?;

                if let Some(response) = response {
                    return Ok(Some((request, response)));
                }
            }
        }

        Ok(None)
    }

    fn matches_path(&self, pattern: &str, url: &str) -> crate::Result<bool> {
        let path = url.split('?').next().unwrap_or(url);
        let path = path
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let path = path.find('/').map(|i| &path[i..]).unwrap_or("/");

        let regex_pattern = pattern
            .replace("{id}", r"[^/]+")
            .replace("{user_id}", r"[^/]+")
            .replace("{account_id}", r"[^/]+")
            .replace("{transaction_id}", r"[^/]+")
            .replace("{institution_id}", r"[^/]+")
            .replace("{member_guid}", r"[^/]+")
            .replace("{*}", r".*");

        let regex_pattern = format!("^{}$", regex_pattern);

        let regex = Regex::new(&regex_pattern)
            .map_err(|e| crate::Error::Provider(format!("Invalid path pattern: {}", e)))?;

        Ok(regex.is_match(path))
    }

    fn matches_body(
        &self,
        pattern: &Option<String>,
        body: &Option<String>,
    ) -> crate::Result<bool> {
        match (pattern, body) {
            (None, _) => Ok(true),
            (Some(_), None) => Ok(false),
            (Some(pattern), Some(body)) => {
                if pattern == "*" {
                    return Ok(true);
                }

                if let (Ok(pattern_json), Ok(body_json)) = (
                    serde_json::from_str::<serde_json::Value>(pattern),
                    serde_json::from_str::<serde_json::Value>(body),
                ) {
                    return Ok(self.json_matches(&pattern_json, &body_json));
                }

                let regex = Regex::new(pattern)
                    .map_err(|e| crate::Error::Provider(format!("Invalid body pattern: {}", e)))?;
                Ok(regex.is_match(body))
            }
        }
    }

    fn json_matches(&self, pattern: &serde_json::Value, value: &serde_json::Value) -> bool {
        match (pattern, value) {
            (serde_json::Value::Object(p), serde_json::Value::Object(v)) => {
                for (key, pattern_value) in p {
                    match v.get(key) {
                        Some(actual_value) => {
                            if !self.json_matches(pattern_value, actual_value) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            }
            (serde_json::Value::Array(p), serde_json::Value::Array(v)) => {
                if p.len() != v.len() {
                    return false;
                }
                p.iter().zip(v.iter()).all(|(p, v)| self.json_matches(p, v))
            }
            (serde_json::Value::String(p), _) if p == "*" => true,
            (p, v) => p == v,
        }
    }

    fn matches_headers(
        &self,
        pattern: &Option<String>,
        headers: &HashMap<String, String>,
    ) -> crate::Result<bool> {
        match pattern {
            None => Ok(true),
            Some(pattern) => {
                let pattern_headers: HashMap<String, String> = serde_json::from_str(pattern)
                    .map_err(|e| crate::Error::Provider(format!("Invalid headers pattern: {}", e)))?;

                for (key, value) in pattern_headers {
                    match headers.get(&key) {
                        Some(actual_value) if actual_value == &value || value == "*" => continue,
                        _ => return Ok(false),
                    }
                }

                Ok(true)
            }
        }
    }

    pub async fn reset(&self) -> crate::Result<()> {
        mock_response::Entity::delete_many()
            .exec(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to delete responses: {}", e)))?;

        mock_request::Entity::delete_many()
            .exec(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to delete requests: {}", e)))?;

        scenario::Entity::delete_many()
            .exec(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to delete scenarios: {}", e)))?;

        self.deactivate_scenario().await;

        Ok(())
    }
}

#[async_trait]
impl Backend for MockBackend {
    async fn execute(&self, request: HttpRequest) -> crate::Result<HttpResponse> {
        let matched = self.find_matching_request(&request).await?;

        match matched {
            Some((mock_req, mock_resp)) => {
                if let Some(delay) = mock_resp.delay_ms {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
                }

                let mut counts = self.request_counts.write().await;
                *counts.entry(mock_req.id).or_insert(0) += 1;

                let headers: HashMap<String, String> = mock_resp
                    .headers
                    .and_then(|h| serde_json::from_str(&h).ok())
                    .unwrap_or_default();

                Ok(HttpResponse {
                    status: mock_resp.status_code as u16,
                    headers,
                    body: mock_resp.body,
                })
            }
            None => {
                Err(crate::Error::Provider(format!(
                    "No mock found for {} {}",
                    request.method, request.url
                )))
            }
        }
    }
}

pub struct MockBuilder {
    backend: MockBackend,
    current_scenario: Option<String>,
}

impl MockBuilder {
    pub async fn new() -> crate::Result<Self> {
        Ok(Self {
            backend: MockBackend::new().await?,
            current_scenario: None,
        })
    }

    pub async fn scenario(
        mut self,
        name: impl Into<String>,
        provider: impl Into<String>,
    ) -> crate::Result<Self> {
        let name = name.into();
        self.backend
            .create_scenario(scenario::CreateScenario::new(name.clone(), provider))
            .await?;
        self.current_scenario = Some(name);
        Ok(self)
    }

    pub async fn on_get(self, path: impl Into<String>) -> MockRequestBuilder {
        MockRequestBuilder {
            builder: self,
            request: mock_request::CreateMockRequest::get(path),
        }
    }

    pub async fn on_post(self, path: impl Into<String>) -> MockRequestBuilder {
        MockRequestBuilder {
            builder: self,
            request: mock_request::CreateMockRequest::post(path),
        }
    }

    pub async fn on_delete(self, path: impl Into<String>) -> MockRequestBuilder {
        MockRequestBuilder {
            builder: self,
            request: mock_request::CreateMockRequest::delete(path),
        }
    }

    pub async fn activate(self, scenario: &str) -> crate::Result<MockBackend> {
        self.backend.activate_scenario(scenario).await?;
        Ok(self.backend)
    }

    pub fn build(self) -> MockBackend {
        self.backend
    }
}

pub struct MockRequestBuilder {
    builder: MockBuilder,
    request: mock_request::CreateMockRequest,
}

impl MockRequestBuilder {
    pub fn with_body_containing(mut self, pattern: impl Into<String>) -> Self {
        self.request = self.request.with_body_pattern(pattern);
        self
    }

    pub async fn respond(
        self,
        response: mock_response::CreateMockResponse,
    ) -> crate::Result<MockBuilder> {
        let scenario_name = self.builder.current_scenario.as_ref().ok_or_else(|| {
            crate::Error::Provider("No active scenario".to_string())
        })?;

        self.builder
            .backend
            .add_mock(scenario_name, self.request, response)
            .await?;

        Ok(self.builder)
    }

    pub async fn respond_ok(self, body: impl Into<String>) -> crate::Result<MockBuilder> {
        self.respond(mock_response::CreateMockResponse::ok(body)).await
    }

    pub async fn respond_json<T: serde::Serialize>(self, data: &T) -> crate::Result<MockBuilder> {
        self.respond(mock_response::CreateMockResponse::json(data)?).await
    }

    pub async fn respond_error(
        self,
        status: i32,
        body: impl Into<String>,
    ) -> crate::Result<MockBuilder> {
        self.respond(mock_response::CreateMockResponse::ok(body).with_status(status))
            .await
    }
}
