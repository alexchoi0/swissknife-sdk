use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "scenarios")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::mock_request::Entity")]
    MockRequests,
}

impl Related<super::mock_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MockRequests.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScenario {
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
}

impl CreateScenario {
    pub fn new(name: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
