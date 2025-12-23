use crate::{Error, Result};
use crate::clay::ClayClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl ClayClient {
    pub async fn list_tables(&self) -> Result<TablesResponse> {
        let response = self.client()
            .get(format!("{}/tables", self.base_url()))
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

        let result: TablesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_table(&self, table_id: &str) -> Result<ClayTable> {
        let response = self.client()
            .get(format!("{}/tables/{}", self.base_url(), table_id))
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

        let result: ClayTable = response.json().await?;
        Ok(result)
    }

    pub async fn list_rows(&self, table_id: &str, params: Option<ListRowsParams>) -> Result<RowsResponse> {
        let mut request = self.client()
            .get(format!("{}/tables/{}/rows", self.base_url(), table_id))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
            if let Some(cursor) = p.cursor {
                query.push(("cursor", cursor));
            }
            if !query.is_empty() {
                request = request.query(&query);
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: RowsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_row(&self, table_id: &str, row_id: &str) -> Result<ClayRow> {
        let response = self.client()
            .get(format!("{}/tables/{}/rows/{}", self.base_url(), table_id, row_id))
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

        let result: ClayRow = response.json().await?;
        Ok(result)
    }

    pub async fn create_row(&self, table_id: &str, data: HashMap<String, serde_json::Value>) -> Result<ClayRow> {
        let response = self.client()
            .post(format!("{}/tables/{}/rows", self.base_url(), table_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&serde_json::json!({ "data": data }))
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

        let result: ClayRow = response.json().await?;
        Ok(result)
    }

    pub async fn update_row(&self, table_id: &str, row_id: &str, data: HashMap<String, serde_json::Value>) -> Result<ClayRow> {
        let response = self.client()
            .patch(format!("{}/tables/{}/rows/{}", self.base_url(), table_id, row_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&serde_json::json!({ "data": data }))
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

        let result: ClayRow = response.json().await?;
        Ok(result)
    }

    pub async fn delete_row(&self, table_id: &str, row_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/tables/{}/rows/{}", self.base_url(), table_id, row_id))
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

        Ok(())
    }

    pub async fn bulk_add_rows(&self, table_id: &str, rows: Vec<HashMap<String, serde_json::Value>>) -> Result<BulkRowsResponse> {
        let response = self.client()
            .post(format!("{}/tables/{}/rows/bulk", self.base_url(), table_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&serde_json::json!({ "rows": rows }))
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

        let result: BulkRowsResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListRowsParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TablesResponse {
    pub tables: Vec<ClayTable>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClayTable {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub columns: Option<Vec<ClayColumn>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClayColumn {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RowsResponse {
    pub rows: Vec<ClayRow>,
    pub next_cursor: Option<String>,
    pub has_more: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClayRow {
    pub id: String,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkRowsResponse {
    pub rows: Vec<ClayRow>,
    pub errors: Option<Vec<BulkRowError>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkRowError {
    pub index: i32,
    pub error: String,
}
