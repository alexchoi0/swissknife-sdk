use crate::{Error, Result};
use crate::typeform::TypeformClient;
use serde::Deserialize;

impl TypeformClient {
    pub async fn list_responses(&self, form_id: &str, params: Option<ListResponsesParams>) -> Result<ResponsesResponse> {
        let mut request = self.client()
            .get(format!("{}/forms/{}/responses", self.base_url(), form_id))
            .header("Authorization", format!("Bearer {}", self.access_token()));

        if let Some(p) = params {
            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(page_size) = p.page_size {
                query_params.push(("page_size", page_size.to_string()));
            }
            if let Some(since) = p.since {
                query_params.push(("since", since));
            }
            if let Some(until) = p.until {
                query_params.push(("until", until));
            }
            if let Some(after) = p.after {
                query_params.push(("after", after));
            }
            if let Some(before) = p.before {
                query_params.push(("before", before));
            }
            if let Some(completed) = p.completed {
                query_params.push(("completed", completed.to_string()));
            }
            if !query_params.is_empty() {
                request = request.query(&query_params);
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

        let responses: ResponsesResponse = response.json().await?;
        Ok(responses)
    }

    pub async fn delete_responses(&self, form_id: &str, response_ids: &[&str]) -> Result<()> {
        let ids = response_ids.join(",");

        let response = self.client()
            .delete(format!("{}/forms/{}/responses", self.base_url(), form_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("included_response_ids", ids)])
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

    pub async fn get_insights(&self, form_id: &str) -> Result<InsightsResponse> {
        let response = self.client()
            .get(format!("{}/insights/{}/summary", self.base_url(), form_id))
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

        let insights: InsightsResponse = response.json().await?;
        Ok(insights)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListResponsesParams {
    pub page_size: Option<u32>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub completed: Option<bool>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsesResponse {
    pub total_items: i32,
    pub page_count: i32,
    pub items: Vec<FormResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormResponse {
    pub response_id: String,
    pub landed_at: String,
    pub submitted_at: Option<String>,
    pub token: Option<String>,
    pub metadata: Option<ResponseMetadata>,
    pub hidden: Option<serde_json::Value>,
    pub calculated: Option<CalculatedFields>,
    pub answers: Option<Vec<Answer>>,
    pub variables: Option<Vec<Variable>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMetadata {
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub referer: Option<String>,
    pub network_id: Option<String>,
    pub browser: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalculatedFields {
    pub score: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Answer {
    pub field: AnswerField,
    #[serde(rename = "type")]
    pub answer_type: String,
    pub text: Option<String>,
    pub email: Option<String>,
    pub number: Option<f64>,
    pub boolean: Option<bool>,
    pub date: Option<String>,
    pub url: Option<String>,
    pub file_url: Option<String>,
    pub choice: Option<AnswerChoice>,
    pub choices: Option<AnswerChoices>,
    pub phone_number: Option<String>,
    pub payment: Option<PaymentAnswer>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnswerField {
    pub id: String,
    #[serde(rename = "ref")]
    pub field_ref: Option<String>,
    #[serde(rename = "type")]
    pub field_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnswerChoice {
    pub id: Option<String>,
    pub label: Option<String>,
    #[serde(rename = "ref")]
    pub choice_ref: Option<String>,
    pub other: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnswerChoices {
    pub ids: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    #[serde(rename = "refs")]
    pub choice_refs: Option<Vec<String>>,
    pub other: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentAnswer {
    pub amount: Option<String>,
    pub last4: Option<String>,
    pub name: Option<String>,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Variable {
    pub key: String,
    #[serde(rename = "type")]
    pub variable_type: String,
    pub number: Option<f64>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InsightsResponse {
    pub form_id: String,
    pub total_responses: i32,
    pub total_visits: Option<i32>,
    pub unique_visits: Option<i32>,
    pub completion_rate: Option<f64>,
    pub average_time: Option<i32>,
    pub fields: Option<Vec<FieldInsight>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldInsight {
    pub id: String,
    #[serde(rename = "ref")]
    pub field_ref: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub field_type: String,
    pub total_responses: Option<i32>,
    pub response_counts: Option<serde_json::Value>,
}
