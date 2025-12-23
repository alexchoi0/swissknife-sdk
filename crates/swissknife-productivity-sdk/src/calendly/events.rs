use crate::{Error, Result};
use crate::calendly::CalendlyClient;
use serde::Deserialize;

impl CalendlyClient {
    pub async fn get_current_user(&self) -> Result<UserResponse> {
        let response = self.client()
            .get(format!("{}/users/me", self.base_url()))
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

        let user: UserResponse = response.json().await?;
        Ok(user)
    }

    pub async fn list_scheduled_events(&self, user_uri: &str, params: Option<ListEventsParams>) -> Result<ScheduledEventsResponse> {
        let mut request = self.client()
            .get(format!("{}/scheduled_events", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("user", user_uri)]);

        if let Some(p) = params {
            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(status) = p.status {
                query_params.push(("status", status));
            }
            if let Some(min_start) = p.min_start_time {
                query_params.push(("min_start_time", min_start));
            }
            if let Some(max_start) = p.max_start_time {
                query_params.push(("max_start_time", max_start));
            }
            if let Some(count) = p.count {
                query_params.push(("count", count.to_string()));
            }
            if let Some(token) = p.page_token {
                query_params.push(("page_token", token));
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

        let events: ScheduledEventsResponse = response.json().await?;
        Ok(events)
    }

    pub async fn get_scheduled_event(&self, event_uuid: &str) -> Result<ScheduledEventResponse> {
        let response = self.client()
            .get(format!("{}/scheduled_events/{}", self.base_url(), event_uuid))
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

        let event: ScheduledEventResponse = response.json().await?;
        Ok(event)
    }

    pub async fn cancel_event(&self, event_uuid: &str, reason: Option<&str>) -> Result<CancellationResponse> {
        let body = serde_json::json!({
            "reason": reason.unwrap_or("")
        });

        let response = self.client()
            .post(format!("{}/scheduled_events/{}/cancellation", self.base_url(), event_uuid))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let cancellation: CancellationResponse = response.json().await?;
        Ok(cancellation)
    }

    pub async fn list_event_invitees(&self, event_uuid: &str) -> Result<InviteesResponse> {
        let response = self.client()
            .get(format!("{}/scheduled_events/{}/invitees", self.base_url(), event_uuid))
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

        let invitees: InviteesResponse = response.json().await?;
        Ok(invitees)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListEventsParams {
    pub status: Option<String>,
    pub min_start_time: Option<String>,
    pub max_start_time: Option<String>,
    pub count: Option<u32>,
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub resource: CalendlyUser,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalendlyUser {
    pub uri: String,
    pub name: String,
    pub slug: String,
    pub email: String,
    pub timezone: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub current_organization: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledEventsResponse {
    pub collection: Vec<ScheduledEvent>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledEventResponse {
    pub resource: ScheduledEvent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledEvent {
    pub uri: String,
    pub name: String,
    pub status: String,
    pub start_time: String,
    pub end_time: String,
    pub event_type: Option<String>,
    pub location: Option<EventLocation>,
    pub invitees_counter: Option<InviteesCounter>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub event_memberships: Option<Vec<EventMembership>>,
    pub event_guests: Option<Vec<EventGuest>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventLocation {
    #[serde(rename = "type")]
    pub location_type: Option<String>,
    pub location: Option<String>,
    pub join_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InviteesCounter {
    pub total: i32,
    pub active: i32,
    pub limit: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventMembership {
    pub user: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventGuest {
    pub email: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub count: i32,
    pub next_page: Option<String>,
    pub previous_page: Option<String>,
    pub next_page_token: Option<String>,
    pub previous_page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancellationResponse {
    pub resource: Cancellation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cancellation {
    pub canceled_by: String,
    pub reason: Option<String>,
    pub canceler_type: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InviteesResponse {
    pub collection: Vec<Invitee>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Invitee {
    pub uri: String,
    pub email: String,
    pub name: Option<String>,
    pub status: String,
    pub timezone: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub questions_and_answers: Option<Vec<QuestionAndAnswer>>,
    pub reschedule_url: Option<String>,
    pub cancel_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestionAndAnswer {
    pub question: String,
    pub answer: Option<String>,
}
