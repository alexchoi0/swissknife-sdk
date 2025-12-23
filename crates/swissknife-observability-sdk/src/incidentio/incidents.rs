use crate::{Error, Result};
use crate::incidentio::IncidentIoClient;
use serde::{Deserialize, Serialize};

impl IncidentIoClient {
    pub async fn list_incidents(&self, params: Option<ListIncidentsParams>) -> Result<IncidentsResponse> {
        let mut request = self.client()
            .get(format!("{}/incidents", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(status) = p.status {
                for s in status {
                    query.push(("status", s));
                }
            }
            if let Some(severity) = p.severity {
                for s in severity {
                    query.push(("severity", s));
                }
            }
            if let Some(page_size) = p.page_size {
                query.push(("page_size", page_size.to_string()));
            }
            if let Some(after) = p.after {
                query.push(("after", after));
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

        let result: IncidentsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_incident(&self, incident_id: &str) -> Result<IncidentResponse> {
        let response = self.client()
            .get(format!("{}/incidents/{}", self.base_url(), incident_id))
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

        let result: IncidentResponse = response.json().await?;
        Ok(result)
    }

    pub async fn create_incident(&self, request: CreateIncidentRequest) -> Result<IncidentResponse> {
        let response = self.client()
            .post(format!("{}/incidents", self.base_url()))
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

        let result: IncidentResponse = response.json().await?;
        Ok(result)
    }

    pub async fn update_incident(&self, incident_id: &str, request: UpdateIncidentRequest) -> Result<IncidentResponse> {
        let response = self.client()
            .patch(format!("{}/incidents/{}", self.base_url(), incident_id))
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

        let result: IncidentResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_incident_updates(&self, incident_id: &str) -> Result<IncidentUpdatesResponse> {
        let response = self.client()
            .get(format!("{}/incidents/{}/updates", self.base_url(), incident_id))
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

        let result: IncidentUpdatesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn create_incident_update(&self, incident_id: &str, message: &str) -> Result<IncidentUpdateResponse> {
        let body = serde_json::json!({
            "message": message
        });

        let response = self.client()
            .post(format!("{}/incidents/{}/updates", self.base_url(), incident_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let result: IncidentUpdateResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_severities(&self) -> Result<SeveritiesResponse> {
        let response = self.client()
            .get(format!("{}/severities", self.base_url()))
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

        let result: SeveritiesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_incident_statuses(&self) -> Result<IncidentStatusesResponse> {
        let response = self.client()
            .get(format!("{}/incident_statuses", self.base_url()))
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

        let result: IncidentStatusesResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListIncidentsParams {
    pub status: Option<Vec<String>>,
    pub severity: Option<Vec<String>>,
    pub page_size: Option<u32>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentsResponse {
    pub incidents: Vec<Incident>,
    pub pagination_meta: Option<PaginationMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentResponse {
    pub incident: Incident,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Incident {
    pub id: String,
    pub name: String,
    pub reference: Option<String>,
    pub status: Option<IncidentStatusRef>,
    pub severity: Option<SeverityRef>,
    pub summary: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub resolved_at: Option<String>,
    pub permalink: Option<String>,
    pub slack_channel_id: Option<String>,
    pub slack_channel_name: Option<String>,
    pub incident_role_assignments: Option<Vec<RoleAssignment>>,
    pub custom_field_entries: Option<Vec<CustomFieldEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentStatusRef {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeverityRef {
    pub id: String,
    pub name: String,
    pub rank: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleAssignment {
    pub role: RoleRef,
    pub assignee: Option<UserRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserRef {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomFieldEntry {
    pub custom_field: CustomFieldRef,
    pub values: Vec<CustomFieldValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomFieldRef {
    pub id: String,
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomFieldValue {
    pub value_link: Option<String>,
    pub value_numeric: Option<String>,
    pub value_option: Option<OptionRef>,
    pub value_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionRef {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationMeta {
    pub after: Option<String>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateIncidentRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateIncidentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentUpdatesResponse {
    pub incident_updates: Vec<IncidentUpdate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentUpdateResponse {
    pub incident_update: IncidentUpdate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentUpdate {
    pub id: String,
    pub message: Option<String>,
    pub created_at: String,
    pub updater: Option<UserRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeveritiesResponse {
    pub severities: Vec<Severity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Severity {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rank: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentStatusesResponse {
    pub incident_statuses: Vec<IncidentStatus>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentStatus {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub rank: i32,
}
