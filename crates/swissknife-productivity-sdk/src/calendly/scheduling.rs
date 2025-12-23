use crate::{Error, Result};
use crate::calendly::CalendlyClient;
use serde::Deserialize;

impl CalendlyClient {
    pub async fn list_event_types(&self, user_uri: &str) -> Result<EventTypesResponse> {
        let response = self.client()
            .get(format!("{}/event_types", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("user", user_uri)])
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

        let event_types: EventTypesResponse = response.json().await?;
        Ok(event_types)
    }

    pub async fn get_event_type(&self, event_type_uuid: &str) -> Result<EventTypeResponse> {
        let response = self.client()
            .get(format!("{}/event_types/{}", self.base_url(), event_type_uuid))
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

        let event_type: EventTypeResponse = response.json().await?;
        Ok(event_type)
    }

    pub async fn list_user_availability_schedules(&self, user_uri: &str) -> Result<AvailabilitySchedulesResponse> {
        let response = self.client()
            .get(format!("{}/user_availability_schedules", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("user", user_uri)])
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

        let schedules: AvailabilitySchedulesResponse = response.json().await?;
        Ok(schedules)
    }

    pub async fn get_user_busy_times(&self, user_uri: &str, start_time: &str, end_time: &str) -> Result<BusyTimesResponse> {
        let response = self.client()
            .get(format!("{}/user_busy_times", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[
                ("user", user_uri),
                ("start_time", start_time),
                ("end_time", end_time),
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

        let busy_times: BusyTimesResponse = response.json().await?;
        Ok(busy_times)
    }

    pub async fn create_single_use_scheduling_link(&self, event_type_uri: &str, max_event_count: u32) -> Result<SchedulingLinkResponse> {
        let body = serde_json::json!({
            "max_event_count": max_event_count,
            "owner": event_type_uri,
            "owner_type": "EventType"
        });

        let response = self.client()
            .post(format!("{}/scheduling_links", self.base_url()))
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

        let link: SchedulingLinkResponse = response.json().await?;
        Ok(link)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventTypesResponse {
    pub collection: Vec<EventType>,
    pub pagination: super::events::Pagination,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventTypeResponse {
    pub resource: EventType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventType {
    pub uri: String,
    pub name: String,
    pub slug: String,
    pub active: bool,
    pub duration: i32,
    pub kind: Option<String>,
    pub pooling_type: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub color: Option<String>,
    pub description_plain: Option<String>,
    pub description_html: Option<String>,
    pub scheduling_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AvailabilitySchedulesResponse {
    pub collection: Vec<AvailabilitySchedule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AvailabilitySchedule {
    pub uri: String,
    pub name: String,
    pub default: bool,
    pub timezone: String,
    pub rules: Vec<AvailabilityRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AvailabilityRule {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub intervals: Vec<TimeInterval>,
    pub wday: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeInterval {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusyTimesResponse {
    pub collection: Vec<BusyTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusyTime {
    #[serde(rename = "type")]
    pub busy_type: String,
    pub start_time: String,
    pub end_time: String,
    pub buffered_start_time: Option<String>,
    pub buffered_end_time: Option<String>,
    pub event: Option<BusyTimeEvent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusyTimeEvent {
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulingLinkResponse {
    pub resource: SchedulingLink,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulingLink {
    pub booking_url: String,
    pub owner: String,
    pub owner_type: String,
}
