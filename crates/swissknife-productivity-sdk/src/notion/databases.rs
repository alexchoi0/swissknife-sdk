use crate::{Error, Result, Database, DatabaseEntry, DatabaseProvider, QueryFilter, PropertySchema, PropertyType};
use crate::notion::NotionClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl NotionClient {
    pub async fn get_database(&self, database_id: &str) -> Result<NotionDatabase> {
        let response = self.client()
            .get(format!("{}/databases/{}", self.base_url(), database_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let database: NotionDatabase = response.json().await?;
        Ok(database)
    }

    pub async fn query_database(&self, database_id: &str, filter: Option<serde_json::Value>, sorts: Option<serde_json::Value>) -> Result<NotionQueryResponse> {
        let mut body = serde_json::Map::new();
        if let Some(f) = filter {
            body.insert("filter".to_string(), f);
        }
        if let Some(s) = sorts {
            body.insert("sorts".to_string(), s);
        }

        let response = self.client()
            .post(format!("{}/databases/{}/query", self.base_url(), database_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("Notion-Version", "2022-06-28")
            .json(&serde_json::Value::Object(body))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let query_response: NotionQueryResponse = response.json().await?;
        Ok(query_response)
    }

    pub async fn create_database_entry(&self, database_id: &str, properties: serde_json::Value) -> Result<NotionDatabaseEntry> {
        let body = serde_json::json!({
            "parent": { "database_id": database_id },
            "properties": properties
        });

        let response = self.client()
            .post(format!("{}/pages", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("Notion-Version", "2022-06-28")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let entry: NotionDatabaseEntry = response.json().await?;
        Ok(entry)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionDatabase {
    pub id: String,
    pub object: String,
    pub title: Vec<NotionRichText>,
    pub description: Option<Vec<NotionRichText>>,
    pub url: Option<String>,
    pub properties: HashMap<String, NotionPropertySchema>,
    pub created_time: Option<String>,
    pub last_edited_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionRichText {
    pub plain_text: Option<String>,
    #[serde(rename = "type")]
    pub text_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionPropertySchema {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub property_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionQueryResponse {
    pub object: String,
    pub results: Vec<NotionDatabaseEntry>,
    pub has_more: Option<bool>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionDatabaseEntry {
    pub id: String,
    pub object: String,
    pub created_time: Option<String>,
    pub last_edited_time: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

fn map_property_type(notion_type: &str) -> PropertyType {
    match notion_type {
        "title" => PropertyType::Title,
        "rich_text" => PropertyType::RichText,
        "number" => PropertyType::Number,
        "select" => PropertyType::Select,
        "multi_select" => PropertyType::MultiSelect,
        "date" => PropertyType::Date,
        "checkbox" => PropertyType::Checkbox,
        "url" => PropertyType::Url,
        "email" => PropertyType::Email,
        "phone_number" => PropertyType::Phone,
        "formula" => PropertyType::Formula,
        "relation" => PropertyType::Relation,
        "rollup" => PropertyType::Rollup,
        "created_time" => PropertyType::CreatedTime,
        "created_by" => PropertyType::CreatedBy,
        "last_edited_time" => PropertyType::LastEditedTime,
        "last_edited_by" => PropertyType::LastEditedBy,
        "files" => PropertyType::Files,
        "people" => PropertyType::People,
        "status" => PropertyType::Status,
        _ => PropertyType::RichText,
    }
}

#[async_trait]
impl DatabaseProvider for NotionClient {
    async fn get_database(&self, id: &str) -> Result<Database> {
        let db = NotionClient::get_database(self, id).await?;
        let title = db.title.first()
            .and_then(|t| t.plain_text.clone())
            .unwrap_or_default();

        let description = db.description
            .and_then(|d| d.first().and_then(|t| t.plain_text.clone()));

        let properties: HashMap<String, PropertySchema> = db.properties.into_iter()
            .map(|(name, schema)| {
                (name.clone(), PropertySchema {
                    name: schema.name,
                    property_type: map_property_type(&schema.property_type),
                    options: None,
                })
            })
            .collect();

        Ok(Database {
            id: db.id,
            title,
            description,
            url: db.url,
            properties,
            created_at: db.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: db.last_edited_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }

    async fn query_database(&self, id: &str, _filter: &QueryFilter) -> Result<Vec<DatabaseEntry>> {
        let response = NotionClient::query_database(self, id, None, None).await?;
        let entries = response.results.into_iter().map(|entry| {
            DatabaseEntry {
                id: entry.id,
                properties: entry.properties,
                created_at: entry.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
                updated_at: entry.last_edited_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            }
        }).collect();

        Ok(entries)
    }

    async fn create_entry(&self, database_id: &str, properties: HashMap<String, serde_json::Value>) -> Result<DatabaseEntry> {
        let entry = self.create_database_entry(database_id, serde_json::Value::Object(properties.into_iter().collect())).await?;
        Ok(DatabaseEntry {
            id: entry.id,
            properties: entry.properties,
            created_at: entry.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: entry.last_edited_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }

    async fn update_entry(&self, entry_id: &str, properties: HashMap<String, serde_json::Value>) -> Result<DatabaseEntry> {
        let response = self.client()
            .patch(format!("{}/pages/{}", self.base_url(), entry_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("Notion-Version", "2022-06-28")
            .json(&serde_json::json!({ "properties": properties }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let entry: NotionDatabaseEntry = response.json().await?;
        Ok(DatabaseEntry {
            id: entry.id,
            properties: entry.properties,
            created_at: entry.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: entry.last_edited_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }

    async fn delete_entry(&self, entry_id: &str) -> Result<()> {
        self.archive_page(entry_id).await
    }
}
