use crate::{Error, Result};
use crate::confluence::ConfluenceClient;
use serde::{Deserialize, Serialize};

impl ConfluenceClient {
    pub async fn list_spaces(&self, params: Option<ListSpacesParams>) -> Result<SpacesResponse> {
        let mut request = self.client()
            .get(format!("{}/wiki/api/v2/spaces", self.base_url()))
            .header("Authorization", self.auth_header());

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(ids) = p.ids {
                for id in ids {
                    query.push(("id", id));
                }
            }
            if let Some(keys) = p.keys {
                for key in keys {
                    query.push(("key", key));
                }
            }
            if let Some(t) = p.space_type {
                query.push(("type", t));
            }
            if let Some(status) = p.status {
                query.push(("status", status));
            }
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
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

        let result: SpacesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_space(&self, space_id: &str) -> Result<Space> {
        let response = self.client()
            .get(format!("{}/wiki/api/v2/spaces/{}", self.base_url(), space_id))
            .header("Authorization", self.auth_header())
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

        let result: Space = response.json().await?;
        Ok(result)
    }

    pub async fn get_space_by_key(&self, space_key: &str) -> Result<Option<Space>> {
        let params = ListSpacesParams {
            keys: Some(vec![space_key.to_string()]),
            ..Default::default()
        };

        let response = self.list_spaces(Some(params)).await?;
        Ok(response.results.into_iter().next())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListSpacesParams {
    pub ids: Option<Vec<String>>,
    pub keys: Option<Vec<String>>,
    pub space_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpacesResponse {
    pub results: Vec<Space>,
    #[serde(rename = "_links")]
    pub links: Option<super::pages::PaginationLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Space {
    pub id: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub space_type: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "homepageId")]
    pub homepage_id: Option<String>,
    pub description: Option<SpaceDescription>,
    pub icon: Option<SpaceIcon>,
    #[serde(rename = "_links")]
    pub links: Option<SpaceLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpaceDescription {
    pub plain: Option<DescriptionContent>,
    pub view: Option<DescriptionContent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DescriptionContent {
    pub value: String,
    pub representation: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpaceIcon {
    pub path: String,
    #[serde(rename = "apiDownloadLink")]
    pub api_download_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpaceLinks {
    #[serde(rename = "webui")]
    pub web_ui: Option<String>,
}
