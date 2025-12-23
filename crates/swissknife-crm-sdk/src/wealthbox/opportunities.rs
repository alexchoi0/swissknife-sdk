use crate::{Error, Result, Deal};
use crate::wealthbox::WealthboxClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl WealthboxClient {
    pub async fn list_opportunities(&self, page: u32, per_page: u32) -> Result<OpportunitiesResponse> {
        let response = self.client()
            .get(format!("{}/opportunities", self.base_url()))
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

        let result: OpportunitiesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_opportunity(&self, opportunity_id: &str) -> Result<WealthboxOpportunity> {
        let response = self.client()
            .get(format!("{}/opportunities/{}", self.base_url(), opportunity_id))
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

        let result: WealthboxOpportunity = response.json().await?;
        Ok(result)
    }

    pub async fn create_opportunity(&self, request: CreateOpportunityRequest) -> Result<WealthboxOpportunity> {
        let response = self.client()
            .post(format!("{}/opportunities", self.base_url()))
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

        let result: WealthboxOpportunity = response.json().await?;
        Ok(result)
    }

    pub async fn update_opportunity(&self, opportunity_id: &str, request: UpdateOpportunityRequest) -> Result<WealthboxOpportunity> {
        let response = self.client()
            .put(format!("{}/opportunities/{}", self.base_url(), opportunity_id))
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

        let result: WealthboxOpportunity = response.json().await?;
        Ok(result)
    }

    pub async fn delete_opportunity(&self, opportunity_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/opportunities/{}", self.base_url(), opportunity_id))
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpportunitiesResponse {
    pub opportunities: Vec<WealthboxOpportunity>,
    pub meta: Option<super::contacts::WealthboxMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WealthboxOpportunity {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub stage: Option<String>,
    pub probability: Option<f64>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub expected_close_date: Option<String>,
    pub actual_close_date: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub source: Option<String>,
    pub linked_to: Option<Vec<super::tasks::LinkedContact>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOpportunityRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_close_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_to: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateOpportunityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_close_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl From<WealthboxOpportunity> for Deal {
    fn from(o: WealthboxOpportunity) -> Self {
        Deal {
            id: Some(o.id.to_string()),
            name: o.name,
            amount: o.value,
            currency: o.currency,
            stage: o.stage,
            pipeline: None,
            probability: o.probability,
            expected_close_date: o.expected_close_date.and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            contact_id: o.linked_to.as_ref().and_then(|l| l.first().map(|c| c.id.to_string())),
            company_id: None,
            owner_id: o.assigned_to,
            custom_fields: HashMap::new(),
            created_at: o.created_at.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: o.updated_at.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}
