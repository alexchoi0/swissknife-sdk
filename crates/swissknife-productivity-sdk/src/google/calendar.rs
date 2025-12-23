use crate::{Error, Result, Calendar, CalendarEvent, CalendarProvider};
use crate::google::GoogleClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const CALENDAR_URL: &str = "https://www.googleapis.com/calendar/v3";

impl GoogleClient {
    pub async fn list_calendars_api(&self) -> Result<CalendarListResponse> {
        let response = self.client()
            .get(format!("{}/users/me/calendarList", CALENDAR_URL))
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

        let list: CalendarListResponse = response.json().await?;
        Ok(list)
    }

    pub async fn list_events_api(&self, calendar_id: &str, time_min: &str, time_max: &str) -> Result<EventsListResponse> {
        let response = self.client()
            .get(format!("{}/calendars/{}/events", CALENDAR_URL, calendar_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[
                ("timeMin", time_min),
                ("timeMax", time_max),
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
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

        let events: EventsListResponse = response.json().await?;
        Ok(events)
    }

    pub async fn get_event_api(&self, calendar_id: &str, event_id: &str) -> Result<GoogleEvent> {
        let response = self.client()
            .get(format!("{}/calendars/{}/events/{}", CALENDAR_URL, calendar_id, event_id))
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

        let event: GoogleEvent = response.json().await?;
        Ok(event)
    }

    pub async fn create_event_api(&self, calendar_id: &str, event: CreateEventRequest) -> Result<GoogleEvent> {
        let response = self.client()
            .post(format!("{}/calendars/{}/events", CALENDAR_URL, calendar_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&event)
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

        let created: GoogleEvent = response.json().await?;
        Ok(created)
    }

    pub async fn update_event_api(&self, calendar_id: &str, event_id: &str, event: CreateEventRequest) -> Result<GoogleEvent> {
        let response = self.client()
            .put(format!("{}/calendars/{}/events/{}", CALENDAR_URL, calendar_id, event_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&event)
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

        let updated: GoogleEvent = response.json().await?;
        Ok(updated)
    }

    pub async fn delete_event_api(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/calendars/{}/events/{}", CALENDAR_URL, calendar_id, event_id))
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
pub struct CalendarListResponse {
    pub items: Vec<GoogleCalendar>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleCalendar {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsListResponse {
    pub items: Option<Vec<GoogleEvent>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleEvent {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: Option<EventDateTime>,
    pub end: Option<EventDateTime>,
    pub attendees: Option<Vec<EventAttendee>>,
    pub recurrence: Option<Vec<String>>,
    #[serde(rename = "htmlLink")]
    pub html_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDateTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventAttendee {
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateEventRequest {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub start: EventDateTime,
    pub end: EventDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendees: Option<Vec<CreateAttendee>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateAttendee {
    pub email: String,
}

fn parse_event_time(dt: &EventDateTime) -> Option<DateTime<Utc>> {
    if let Some(ref datetime) = dt.date_time {
        chrono::DateTime::parse_from_rfc3339(datetime).ok().map(|d| d.with_timezone(&Utc))
    } else if let Some(ref date) = dt.date {
        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
    } else {
        None
    }
}

#[async_trait]
impl CalendarProvider for GoogleClient {
    async fn list_calendars(&self) -> Result<Vec<Calendar>> {
        let response = self.list_calendars_api().await?;
        let calendars = response.items.into_iter().map(|c| Calendar {
            id: c.id,
            name: c.summary,
            description: c.description,
            timezone: c.time_zone,
            is_primary: c.primary.unwrap_or(false),
        }).collect();

        Ok(calendars)
    }

    async fn list_events(&self, calendar_id: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<CalendarEvent>> {
        let time_min = start.to_rfc3339();
        let time_max = end.to_rfc3339();

        let response = self.list_events_api(calendar_id, &time_min, &time_max).await?;
        let events = response.items.unwrap_or_default().into_iter().filter_map(|e| {
            let start_time = e.start.as_ref().and_then(parse_event_time)?;
            let end_time = e.end.as_ref().and_then(parse_event_time)?;
            let is_all_day = e.start.as_ref().and_then(|s| s.date.as_ref()).is_some();

            Some(CalendarEvent {
                id: e.id,
                title: e.summary.unwrap_or_default(),
                description: e.description,
                start_time,
                end_time,
                location: e.location,
                attendees: e.attendees.unwrap_or_default().into_iter().map(|a| a.email).collect(),
                is_all_day,
                recurrence: e.recurrence.and_then(|r| r.first().cloned()),
                calendar_id: Some(calendar_id.to_string()),
                url: e.html_link,
            })
        }).collect();

        Ok(events)
    }

    async fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<CalendarEvent> {
        let e = self.get_event_api(calendar_id, event_id).await?;
        let start_time = e.start.as_ref().and_then(parse_event_time)
            .ok_or_else(|| Error::InvalidRequest("Missing start time".to_string()))?;
        let end_time = e.end.as_ref().and_then(parse_event_time)
            .ok_or_else(|| Error::InvalidRequest("Missing end time".to_string()))?;
        let is_all_day = e.start.as_ref().and_then(|s| s.date.as_ref()).is_some();

        Ok(CalendarEvent {
            id: e.id,
            title: e.summary.unwrap_or_default(),
            description: e.description,
            start_time,
            end_time,
            location: e.location,
            attendees: e.attendees.unwrap_or_default().into_iter().map(|a| a.email).collect(),
            is_all_day,
            recurrence: e.recurrence.and_then(|r| r.first().cloned()),
            calendar_id: Some(calendar_id.to_string()),
            url: e.html_link,
        })
    }

    async fn create_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<CalendarEvent> {
        let request = CreateEventRequest {
            summary: event.title.clone(),
            description: event.description.clone(),
            location: event.location.clone(),
            start: if event.is_all_day {
                EventDateTime {
                    date: Some(event.start_time.format("%Y-%m-%d").to_string()),
                    date_time: None,
                    time_zone: None,
                }
            } else {
                EventDateTime {
                    date_time: Some(event.start_time.to_rfc3339()),
                    date: None,
                    time_zone: None,
                }
            },
            end: if event.is_all_day {
                EventDateTime {
                    date: Some(event.end_time.format("%Y-%m-%d").to_string()),
                    date_time: None,
                    time_zone: None,
                }
            } else {
                EventDateTime {
                    date_time: Some(event.end_time.to_rfc3339()),
                    date: None,
                    time_zone: None,
                }
            },
            attendees: if event.attendees.is_empty() {
                None
            } else {
                Some(event.attendees.iter().map(|e| CreateAttendee { email: e.clone() }).collect())
            },
        };

        let created = self.create_event_api(calendar_id, request).await?;
        self.get_event(calendar_id, &created.id).await
    }

    async fn update_event(&self, calendar_id: &str, event_id: &str, event: &CalendarEvent) -> Result<CalendarEvent> {
        let request = CreateEventRequest {
            summary: event.title.clone(),
            description: event.description.clone(),
            location: event.location.clone(),
            start: EventDateTime {
                date_time: Some(event.start_time.to_rfc3339()),
                date: None,
                time_zone: None,
            },
            end: EventDateTime {
                date_time: Some(event.end_time.to_rfc3339()),
                date: None,
                time_zone: None,
            },
            attendees: if event.attendees.is_empty() {
                None
            } else {
                Some(event.attendees.iter().map(|e| CreateAttendee { email: e.clone() }).collect())
            },
        };

        let updated = self.update_event_api(calendar_id, event_id, request).await?;
        self.get_event(calendar_id, &updated.id).await
    }

    async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        self.delete_event_api(calendar_id, event_id).await
    }
}
