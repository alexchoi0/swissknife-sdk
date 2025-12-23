use crate::{Error, Result};
use crate::airtable::AirtableClient;
use serde::Deserialize;

const META_URL: &str = "https://api.airtable.com/v0/meta";

impl AirtableClient {
    pub async fn list_bases(&self) -> Result<BasesResponse> {
        let response = self.client()
            .get(format!("{}/bases", META_URL))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let bases: BasesResponse = response.json().await?;
        Ok(bases)
    }

    pub async fn get_base_schema(&self, base_id: &str) -> Result<BaseSchema> {
        let response = self.client()
            .get(format!("{}/bases/{}/tables", META_URL, base_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let schema: BaseSchema = response.json().await?;
        Ok(schema)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BasesResponse {
    pub bases: Vec<AirtableBase>,
    pub offset: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AirtableBase {
    pub id: String,
    pub name: String,
    #[serde(rename = "permissionLevel")]
    pub permission_level: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BaseSchema {
    pub tables: Vec<TableSchema>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableSchema {
    pub id: String,
    pub name: String,
    #[serde(rename = "primaryFieldId")]
    pub primary_field_id: Option<String>,
    pub fields: Vec<FieldSchema>,
    pub views: Option<Vec<ViewSchema>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldSchema {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub description: Option<String>,
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewSchema {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub view_type: String,
}
