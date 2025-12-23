use crate::{Error, Result, Document, DocumentProvider};
use crate::notion::NotionClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl NotionClient {
    pub async fn get_page(&self, page_id: &str) -> Result<NotionPage> {
        let response = self.client()
            .get(format!("{}/pages/{}", self.base_url(), page_id))
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

        let page: NotionPage = response.json().await?;
        Ok(page)
    }

    pub async fn create_page(&self, parent_id: &str, properties: serde_json::Value) -> Result<NotionPage> {
        let body = serde_json::json!({
            "parent": { "page_id": parent_id },
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

        let page: NotionPage = response.json().await?;
        Ok(page)
    }

    pub async fn update_page(&self, page_id: &str, properties: serde_json::Value) -> Result<NotionPage> {
        let body = serde_json::json!({
            "properties": properties
        });

        let response = self.client()
            .patch(format!("{}/pages/{}", self.base_url(), page_id))
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

        let page: NotionPage = response.json().await?;
        Ok(page)
    }

    pub async fn archive_page(&self, page_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "archived": true
        });

        let response = self.client()
            .patch(format!("{}/pages/{}", self.base_url(), page_id))
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

        Ok(())
    }

    pub async fn get_block_children(&self, block_id: &str) -> Result<Vec<NotionBlock>> {
        let response = self.client()
            .get(format!("{}/blocks/{}/children", self.base_url(), block_id))
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

        let blocks_response: NotionBlocksResponse = response.json().await?;
        Ok(blocks_response.results)
    }

    pub async fn search(&self, query: &str, filter: Option<&str>) -> Result<NotionSearchResponse> {
        let mut body = serde_json::json!({
            "query": query
        });

        if let Some(f) = filter {
            body["filter"] = serde_json::json!({
                "property": "object",
                "value": f
            });
        }

        let response = self.client()
            .post(format!("{}/search", self.base_url()))
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

        let search_response: NotionSearchResponse = response.json().await?;
        Ok(search_response)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionPage {
    pub id: String,
    pub object: String,
    pub created_time: Option<String>,
    pub last_edited_time: Option<String>,
    pub archived: Option<bool>,
    pub url: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionBlock {
    pub id: String,
    pub object: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub created_time: Option<String>,
    pub last_edited_time: Option<String>,
    pub has_children: Option<bool>,
    #[serde(flatten)]
    pub content: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionBlocksResponse {
    pub object: String,
    pub results: Vec<NotionBlock>,
    pub has_more: Option<bool>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionSearchResponse {
    pub object: String,
    pub results: Vec<serde_json::Value>,
    pub has_more: Option<bool>,
    pub next_cursor: Option<String>,
}

fn extract_title(properties: &HashMap<String, serde_json::Value>) -> String {
    properties.iter()
        .find_map(|(_, v)| {
            v.get("title")
                .and_then(|arr| arr.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("plain_text"))
                .and_then(|s| s.as_str())
                .map(String::from)
        })
        .unwrap_or_default()
}

#[async_trait]
impl DocumentProvider for NotionClient {
    async fn get_document(&self, id: &str) -> Result<Document> {
        let page = self.get_page(id).await?;
        Ok(Document {
            id: page.id,
            title: extract_title(&page.properties),
            content: None,
            markdown: None,
            url: page.url,
            parent_id: None,
            created_at: page.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: page.last_edited_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            created_by: None,
            properties: page.properties,
        })
    }

    async fn create_document(&self, parent_id: Option<&str>, title: &str, _content: Option<&str>) -> Result<Document> {
        let properties = serde_json::json!({
            "title": {
                "title": [{ "text": { "content": title } }]
            }
        });

        let page = self.create_page(parent_id.unwrap_or(""), properties).await?;
        Ok(Document {
            id: page.id,
            title: title.to_string(),
            content: None,
            markdown: None,
            url: page.url,
            parent_id: parent_id.map(String::from),
            created_at: page.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: page.last_edited_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            created_by: None,
            properties: HashMap::new(),
        })
    }

    async fn update_document(&self, id: &str, title: Option<&str>, _content: Option<&str>) -> Result<Document> {
        let mut properties = serde_json::Map::new();
        if let Some(t) = title {
            properties.insert("title".to_string(), serde_json::json!({
                "title": [{ "text": { "content": t } }]
            }));
        }

        let page = self.update_page(id, serde_json::Value::Object(properties)).await?;
        Ok(Document {
            id: page.id,
            title: extract_title(&page.properties),
            content: None,
            markdown: None,
            url: page.url,
            parent_id: None,
            created_at: page.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: page.last_edited_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            created_by: None,
            properties: page.properties,
        })
    }

    async fn delete_document(&self, id: &str) -> Result<()> {
        self.archive_page(id).await
    }

    async fn search(&self, query: &str) -> Result<Vec<Document>> {
        let response = NotionClient::search(self, query, Some("page")).await?;
        let documents = response.results.into_iter().filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let url = v.get("url").and_then(|u| u.as_str()).map(String::from);
            let properties = v.get("properties")
                .and_then(|p| p.as_object())
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();

            Some(Document {
                id,
                title: extract_title(&properties),
                content: None,
                markdown: None,
                url,
                parent_id: None,
                created_at: None,
                updated_at: None,
                created_by: None,
                properties,
            })
        }).collect();

        Ok(documents)
    }
}
