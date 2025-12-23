use crate::{Error, Result};
use crate::wealthbox::WealthboxClient;
use serde::{Deserialize, Serialize};

impl WealthboxClient {
    pub async fn list_tasks(&self, page: u32, per_page: u32) -> Result<WealthboxTasksResponse> {
        let response = self.client()
            .get(format!("{}/tasks", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
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

        let result: WealthboxTasksResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<WealthboxTask> {
        let response = self.client()
            .get(format!("{}/tasks/{}", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let result: WealthboxTask = response.json().await?;
        Ok(result)
    }

    pub async fn create_task(&self, request: CreateWealthboxTaskRequest) -> Result<WealthboxTask> {
        let response = self.client()
            .post(format!("{}/tasks", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let result: WealthboxTask = response.json().await?;
        Ok(result)
    }

    pub async fn update_task(&self, task_id: &str, request: UpdateWealthboxTaskRequest) -> Result<WealthboxTask> {
        let response = self.client()
            .put(format!("{}/tasks/{}", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let result: WealthboxTask = response.json().await?;
        Ok(result)
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/tasks/{}", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

    pub async fn complete_task(&self, task_id: &str) -> Result<WealthboxTask> {
        self.update_task(task_id, UpdateWealthboxTaskRequest {
            complete: Some(true),
            ..Default::default()
        }).await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WealthboxTasksResponse {
    pub tasks: Vec<WealthboxTask>,
    pub meta: Option<super::contacts::WealthboxMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WealthboxTask {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub complete: Option<bool>,
    pub completed_at: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub assigned_to: Option<String>,
    pub linked_to: Option<Vec<LinkedContact>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkedContact {
    pub id: i64,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateWealthboxTaskRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_to: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateWealthboxTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}
