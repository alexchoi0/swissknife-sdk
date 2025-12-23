use crate::{Error, Result};
use crate::confluence::ConfluenceClient;
use serde::{Deserialize, Serialize};

impl ConfluenceClient {
    pub async fn get_page(&self, page_id: &str) -> Result<Page> {
        let response = self.client()
            .get(format!("{}/wiki/api/v2/pages/{}", self.base_url(), page_id))
            .header("Authorization", self.auth_header())
            .query(&[("body-format", "storage")])
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

        let result: Page = response.json().await?;
        Ok(result)
    }

    pub async fn get_page_by_title(&self, space_id: &str, title: &str) -> Result<Option<Page>> {
        let response = self.client()
            .get(format!("{}/wiki/api/v2/spaces/{}/pages", self.base_url(), space_id))
            .header("Authorization", self.auth_header())
            .query(&[("title", title), ("body-format", "storage")])
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

        let result: PagesResponse = response.json().await?;
        Ok(result.results.into_iter().next())
    }

    pub async fn list_pages(&self, space_id: Option<&str>, limit: Option<u32>) -> Result<PagesResponse> {
        let mut request = self.client()
            .get(format!("{}/wiki/api/v2/pages", self.base_url()))
            .header("Authorization", self.auth_header());

        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(id) = space_id {
            query.push(("space-id", id.to_string()));
        }
        if let Some(l) = limit {
            query.push(("limit", l.to_string()));
        }
        query.push(("body-format", "storage".to_string()));

        if !query.is_empty() {
            request = request.query(&query);
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

        let result: PagesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn create_page(&self, request: CreatePageRequest) -> Result<Page> {
        let response = self.client()
            .post(format!("{}/wiki/api/v2/pages", self.base_url()))
            .header("Authorization", self.auth_header())
            .json(&request)
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

        let result: Page = response.json().await?;
        Ok(result)
    }

    pub async fn update_page(&self, page_id: &str, request: UpdatePageRequest) -> Result<Page> {
        let response = self.client()
            .put(format!("{}/wiki/api/v2/pages/{}", self.base_url(), page_id))
            .header("Authorization", self.auth_header())
            .json(&request)
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

        let result: Page = response.json().await?;
        Ok(result)
    }

    pub async fn delete_page(&self, page_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/wiki/api/v2/pages/{}", self.base_url(), page_id))
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

        Ok(())
    }

    pub async fn get_page_children(&self, page_id: &str, limit: Option<u32>) -> Result<PagesResponse> {
        let mut request = self.client()
            .get(format!("{}/wiki/api/v2/pages/{}/children", self.base_url(), page_id))
            .header("Authorization", self.auth_header());

        if let Some(l) = limit {
            request = request.query(&[("limit", l.to_string())]);
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

        let result: PagesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_page_ancestors(&self, page_id: &str) -> Result<Vec<PageAncestor>> {
        let response = self.client()
            .get(format!("{}/wiki/api/v2/pages/{}/ancestors", self.base_url(), page_id))
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

        let result: AncestorsResponse = response.json().await?;
        Ok(result.results)
    }

    pub async fn get_page_labels(&self, page_id: &str) -> Result<Vec<Label>> {
        let response = self.client()
            .get(format!("{}/wiki/api/v2/pages/{}/labels", self.base_url(), page_id))
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

        let result: LabelsResponse = response.json().await?;
        Ok(result.results)
    }

    pub async fn add_page_label(&self, page_id: &str, label: &str) -> Result<Label> {
        let body = serde_json::json!({
            "name": label
        });

        let response = self.client()
            .post(format!("{}/wiki/api/v2/pages/{}/labels", self.base_url(), page_id))
            .header("Authorization", self.auth_header())
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

        let result: Label = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagesResponse {
    pub results: Vec<Page>,
    #[serde(rename = "_links")]
    pub links: Option<PaginationLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationLinks {
    pub next: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    pub id: String,
    pub status: String,
    pub title: String,
    #[serde(rename = "spaceId")]
    pub space_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "parentType")]
    pub parent_type: Option<String>,
    pub position: Option<i32>,
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    pub version: Option<PageVersion>,
    pub body: Option<PageBody>,
    #[serde(rename = "_links")]
    pub links: Option<PageLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageVersion {
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    pub message: Option<String>,
    pub number: i32,
    #[serde(rename = "minorEdit")]
    pub minor_edit: bool,
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageBody {
    pub storage: Option<BodyContent>,
    pub atlas_doc_format: Option<BodyContent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BodyContent {
    pub representation: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageLinks {
    #[serde(rename = "webui")]
    pub web_ui: Option<String>,
    #[serde(rename = "editui")]
    pub edit_ui: Option<String>,
    pub tinyui: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePageRequest {
    #[serde(rename = "spaceId")]
    pub space_id: String,
    pub status: String,
    pub title: String,
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<CreatePageBody>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePageBody {
    pub representation: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatePageRequest {
    pub id: String,
    pub status: String,
    pub title: String,
    pub version: UpdateVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<CreatePageBody>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateVersion {
    pub number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageAncestor {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AncestorsResponse {
    pub results: Vec<PageAncestor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub id: Option<String>,
    pub name: String,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LabelsResponse {
    pub results: Vec<Label>,
}

impl From<Page> for crate::Document {
    fn from(page: Page) -> Self {
        let content = page.body.as_ref()
            .and_then(|b| b.storage.as_ref())
            .map(|s| s.value.clone());

        let url = page.links.as_ref()
            .and_then(|l| l.web_ui.clone());

        Self {
            id: page.id,
            title: page.title,
            content,
            markdown: None,
            url,
            parent_id: page.parent_id,
            created_at: None,
            updated_at: None,
            created_by: page.author_id,
            properties: std::collections::HashMap::new(),
        }
    }
}
