use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mock_responses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub request_id: i32,
    pub status_code: i32,
    pub headers: Option<String>,
    pub body: String,
    pub delay_ms: Option<i32>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mock_request::Entity",
        from = "Column::RequestId",
        to = "super::mock_request::Column::Id"
    )]
    MockRequest,
}

impl Related<super::mock_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MockRequest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMockResponse {
    pub request_id: i32,
    pub status_code: i32,
    pub headers: Option<String>,
    pub body: String,
    pub delay_ms: Option<i32>,
}

impl CreateMockResponse {
    pub fn ok(body: impl Into<String>) -> Self {
        Self {
            request_id: 0,
            status_code: 200,
            headers: None,
            body: body.into(),
            delay_ms: None,
        }
    }

    pub fn created(body: impl Into<String>) -> Self {
        Self {
            request_id: 0,
            status_code: 201,
            headers: None,
            body: body.into(),
            delay_ms: None,
        }
    }

    pub fn no_content() -> Self {
        Self {
            request_id: 0,
            status_code: 204,
            headers: None,
            body: String::new(),
            delay_ms: None,
        }
    }

    pub fn bad_request(body: impl Into<String>) -> Self {
        Self {
            request_id: 0,
            status_code: 400,
            headers: None,
            body: body.into(),
            delay_ms: None,
        }
    }

    pub fn unauthorized(body: impl Into<String>) -> Self {
        Self {
            request_id: 0,
            status_code: 401,
            headers: None,
            body: body.into(),
            delay_ms: None,
        }
    }

    pub fn not_found(body: impl Into<String>) -> Self {
        Self {
            request_id: 0,
            status_code: 404,
            headers: None,
            body: body.into(),
            delay_ms: None,
        }
    }

    pub fn internal_error(body: impl Into<String>) -> Self {
        Self {
            request_id: 0,
            status_code: 500,
            headers: None,
            body: body.into(),
            delay_ms: None,
        }
    }

    pub fn rate_limited() -> Self {
        Self {
            request_id: 0,
            status_code: 429,
            headers: None,
            body: r#"{"error": "rate_limited", "message": "Too many requests"}"#.to_string(),
            delay_ms: None,
        }
    }

    pub fn with_status(mut self, status_code: i32) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn with_headers(mut self, headers: impl Into<String>) -> Self {
        self.headers = Some(headers.into());
        self
    }

    pub fn with_delay(mut self, delay_ms: i32) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    pub fn for_request(mut self, request_id: i32) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn json<T: Serialize>(data: &T) -> crate::Result<Self> {
        Ok(Self {
            request_id: 0,
            status_code: 200,
            headers: Some(r#"{"Content-Type": "application/json"}"#.to_string()),
            body: serde_json::to_string(data)?,
            delay_ms: None,
        })
    }

    pub fn json_pretty<T: Serialize>(data: &T) -> crate::Result<Self> {
        Ok(Self {
            request_id: 0,
            status_code: 200,
            headers: Some(r#"{"Content-Type": "application/json"}"#.to_string()),
            body: serde_json::to_string_pretty(data)?,
            delay_ms: None,
        })
    }
}
