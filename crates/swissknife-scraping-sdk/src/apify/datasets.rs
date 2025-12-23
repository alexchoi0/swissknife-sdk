use crate::{Error, Result};
use crate::apify::ApifyClient;
use serde::{Deserialize, Serialize};

impl ApifyClient {
    pub async fn list_datasets(&self, params: Option<ListDatasetsParams>) -> Result<DatasetsResponse> {
        let mut request = self.client()
            .get(format!("{}/datasets", self.base_url()))
            .query(&[("token", self.api_token())]);

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(unnamed) = p.unnamed {
                query.push(("unnamed", unnamed.to_string()));
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

        let result: DatasetsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_dataset(&self, dataset_id: &str) -> Result<Dataset> {
        let response = self.client()
            .get(format!("{}/datasets/{}", self.base_url(), dataset_id))
            .query(&[("token", self.api_token())])
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

        let result: DatasetResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn create_dataset(&self, name: Option<&str>) -> Result<Dataset> {
        let mut request = self.client()
            .post(format!("{}/datasets", self.base_url()))
            .query(&[("token", self.api_token())]);

        if let Some(n) = name {
            request = request.query(&[("name", n)]);
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

        let result: DatasetResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn delete_dataset(&self, dataset_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/datasets/{}", self.base_url(), dataset_id))
            .query(&[("token", self.api_token())])
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

    pub async fn get_dataset_items(&self, dataset_id: &str, params: Option<GetItemsParams>) -> Result<Vec<serde_json::Value>> {
        let mut request = self.client()
            .get(format!("{}/datasets/{}/items", self.base_url(), dataset_id))
            .query(&[("token", self.api_token())]);

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(clean) = p.clean {
                query.push(("clean", clean.to_string()));
            }
            if let Some(fields) = p.fields {
                query.push(("fields", fields.join(",")));
            }
            if let Some(format) = p.format {
                query.push(("format", format));
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

        let result: Vec<serde_json::Value> = response.json().await?;
        Ok(result)
    }

    pub async fn push_dataset_items(&self, dataset_id: &str, items: &[serde_json::Value]) -> Result<()> {
        let response = self.client()
            .post(format!("{}/datasets/{}/items", self.base_url(), dataset_id))
            .query(&[("token", self.api_token())])
            .json(items)
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
}

#[derive(Debug, Clone, Default)]
pub struct ListDatasetsParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub unnamed: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct GetItemsParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub clean: Option<bool>,
    pub fields: Option<Vec<String>>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetsResponse {
    pub data: DatasetsData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetsData {
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<Dataset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetResponse {
    pub data: Dataset,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dataset {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "modifiedAt")]
    pub modified_at: Option<String>,
    #[serde(rename = "accessedAt")]
    pub accessed_at: Option<String>,
    #[serde(rename = "itemCount")]
    pub item_count: Option<u64>,
    #[serde(rename = "cleanItemCount")]
    pub clean_item_count: Option<u64>,
}
