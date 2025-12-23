use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mock_requests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub scenario_id: i32,
    pub method: String,
    pub path_pattern: String,
    pub body_pattern: Option<String>,
    pub headers_pattern: Option<String>,
    pub sequence_order: i32,
    pub times_matched: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scenario::Entity",
        from = "Column::ScenarioId",
        to = "super::scenario::Column::Id"
    )]
    Scenario,
    #[sea_orm(has_one = "super::mock_response::Entity")]
    MockResponse,
}

impl Related<super::scenario::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Scenario.def()
    }
}

impl Related<super::mock_response::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MockResponse.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMockRequest {
    pub scenario_id: i32,
    pub method: String,
    pub path_pattern: String,
    pub body_pattern: Option<String>,
    pub headers_pattern: Option<String>,
    pub sequence_order: Option<i32>,
}

impl CreateMockRequest {
    pub fn get(path_pattern: impl Into<String>) -> Self {
        Self {
            scenario_id: 0,
            method: "GET".to_string(),
            path_pattern: path_pattern.into(),
            body_pattern: None,
            headers_pattern: None,
            sequence_order: None,
        }
    }

    pub fn post(path_pattern: impl Into<String>) -> Self {
        Self {
            scenario_id: 0,
            method: "POST".to_string(),
            path_pattern: path_pattern.into(),
            body_pattern: None,
            headers_pattern: None,
            sequence_order: None,
        }
    }

    pub fn delete(path_pattern: impl Into<String>) -> Self {
        Self {
            scenario_id: 0,
            method: "DELETE".to_string(),
            path_pattern: path_pattern.into(),
            body_pattern: None,
            headers_pattern: None,
            sequence_order: None,
        }
    }

    pub fn with_body_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.body_pattern = Some(pattern.into());
        self
    }

    pub fn with_headers_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.headers_pattern = Some(pattern.into());
        self
    }

    pub fn with_sequence(mut self, order: i32) -> Self {
        self.sequence_order = Some(order);
        self
    }

    pub fn for_scenario(mut self, scenario_id: i32) -> Self {
        self.scenario_id = scenario_id;
        self
    }
}
