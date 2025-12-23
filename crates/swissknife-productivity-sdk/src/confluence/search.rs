use crate::{Error, Result};
use crate::confluence::ConfluenceClient;
use serde::Deserialize;

impl ConfluenceClient {
    pub async fn search(&self, cql: &str, params: Option<SearchParams>) -> Result<SearchResponse> {
        let mut request = self.client()
            .get(format!("{}/wiki/rest/api/search", self.base_url()))
            .header("Authorization", self.auth_header())
            .query(&[("cql", cql)]);

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(start) = p.start {
                query.push(("start", start.to_string()));
            }
            if let Some(excerpt) = p.include_excerpt {
                if excerpt {
                    query.push(("excerpt", "highlight".to_string()));
                }
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

        let result: SearchResponse = response.json().await?;
        Ok(result)
    }

    pub async fn search_content(&self, query: &str, space_key: Option<&str>, content_type: Option<&str>, limit: Option<u32>) -> Result<SearchResponse> {
        let mut cql = format!("text ~ \"{}\"", query.replace('"', "\\\""));

        if let Some(key) = space_key {
            cql.push_str(&format!(" AND space.key = \"{}\"", key));
        }

        if let Some(ct) = content_type {
            cql.push_str(&format!(" AND type = \"{}\"", ct));
        }

        let params = SearchParams {
            limit,
            include_excerpt: Some(true),
            ..Default::default()
        };

        self.search(&cql, Some(params)).await
    }

    pub async fn search_by_label(&self, label: &str, space_key: Option<&str>, limit: Option<u32>) -> Result<SearchResponse> {
        let mut cql = format!("label = \"{}\"", label);

        if let Some(key) = space_key {
            cql.push_str(&format!(" AND space.key = \"{}\"", key));
        }

        let params = SearchParams {
            limit,
            include_excerpt: Some(true),
            ..Default::default()
        };

        self.search(&cql, Some(params)).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub limit: Option<u32>,
    pub start: Option<u32>,
    pub include_excerpt: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub start: Option<i32>,
    pub limit: Option<i32>,
    pub size: Option<i32>,
    #[serde(rename = "totalSize")]
    pub total_size: Option<i32>,
    #[serde(rename = "cqlQuery")]
    pub cql_query: Option<String>,
    #[serde(rename = "searchDuration")]
    pub search_duration: Option<i32>,
    #[serde(rename = "_links")]
    pub links: Option<SearchLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub content: Option<SearchContent>,
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "resultGlobalContainer")]
    pub result_global_container: Option<SearchContainer>,
    #[serde(rename = "breadcrumbs")]
    pub breadcrumbs: Option<Vec<Breadcrumb>>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    #[serde(rename = "iconCssClass")]
    pub icon_css_class: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(rename = "friendlyLastModified")]
    pub friendly_last_modified: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchContent {
    pub id: String,
    #[serde(rename = "type")]
    pub content_type: String,
    pub status: Option<String>,
    pub title: Option<String>,
    pub space: Option<SearchSpace>,
    #[serde(rename = "_links")]
    pub links: Option<ContentLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchSpace {
    pub id: Option<i64>,
    pub key: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub space_type: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "_links")]
    pub links: Option<SpaceSearchLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpaceSearchLinks {
    #[serde(rename = "webui")]
    pub web_ui: Option<String>,
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentLinks {
    #[serde(rename = "webui")]
    pub web_ui: Option<String>,
    #[serde(rename = "self")]
    pub self_link: Option<String>,
    pub tinyui: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchContainer {
    pub title: Option<String>,
    #[serde(rename = "displayUrl")]
    pub display_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Breadcrumb {
    pub label: Option<String>,
    pub url: Option<String>,
    pub separator: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchLinks {
    pub base: Option<String>,
    pub context: Option<String>,
    pub next: Option<String>,
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}
