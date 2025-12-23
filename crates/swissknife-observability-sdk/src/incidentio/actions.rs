use crate::{Error, Result};
use crate::incidentio::IncidentIoClient;
use serde::{Deserialize, Serialize};

impl IncidentIoClient {
    pub async fn list_actions(&self, incident_id: &str) -> Result<ActionsResponse> {
        let response = self.client()
            .get(format!("{}/actions", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .query(&[("incident_id", incident_id)])
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

        let result: ActionsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn create_action(&self, request: CreateActionRequest) -> Result<ActionResponse> {
        let response = self.client()
            .post(format!("{}/actions", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn update_action(&self, action_id: &str, request: UpdateActionRequest) -> Result<ActionResponse> {
        let response = self.client()
            .patch(format!("{}/actions/{}", self.base_url(), action_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_incident_roles(&self) -> Result<IncidentRolesResponse> {
        let response = self.client()
            .get(format!("{}/incident_roles", self.base_url()))
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

        let result: IncidentRolesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_incident_types(&self) -> Result<IncidentTypesResponse> {
        let response = self.client()
            .get(format!("{}/incident_types", self.base_url()))
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

        let result: IncidentTypesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_custom_fields(&self) -> Result<CustomFieldsResponse> {
        let response = self.client()
            .get(format!("{}/custom_fields", self.base_url()))
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

        let result: CustomFieldsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_users(&self, page_size: Option<u32>) -> Result<UsersResponse> {
        let mut request = self.client()
            .get(format!("{}/users", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        if let Some(size) = page_size {
            request = request.query(&[("page_size", size.to_string())]);
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

        let result: UsersResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionsResponse {
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionResponse {
    pub action: Action,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub id: String,
    pub description: String,
    pub status: String,
    pub incident_id: String,
    pub assignee: Option<super::incidents::UserRef>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateActionRequest {
    pub incident_id: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateActionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentRolesResponse {
    pub incident_roles: Vec<IncidentRole>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentRole {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentTypesResponse {
    pub incident_types: Vec<IncidentType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomFieldsResponse {
    pub custom_fields: Vec<CustomField>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomField {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub field_type: String,
    pub required: Option<String>,
    pub options: Option<Vec<CustomFieldOption>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomFieldOption {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsersResponse {
    pub users: Vec<User>,
    pub pagination_meta: Option<super::incidents::PaginationMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub email: String,
    pub role: Option<String>,
}
