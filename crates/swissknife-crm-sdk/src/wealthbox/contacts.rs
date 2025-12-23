use crate::{Error, Result, Contact, Address, CrmProvider, ListOptions, ListResult};
use crate::wealthbox::WealthboxClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl WealthboxClient {
    pub async fn list_contacts(&self, page: u32, per_page: u32) -> Result<WealthboxContactsResponse> {
        let response = self.client()
            .get(format!("{}/contacts", self.base_url()))
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

        let result: WealthboxContactsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_contact(&self, contact_id: &str) -> Result<WealthboxContact> {
        let response = self.client()
            .get(format!("{}/contacts/{}", self.base_url(), contact_id))
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

        let result: WealthboxContact = response.json().await?;
        Ok(result)
    }

    pub async fn create_contact(&self, request: CreateWealthboxContactRequest) -> Result<WealthboxContact> {
        let response = self.client()
            .post(format!("{}/contacts", self.base_url()))
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

        let result: WealthboxContact = response.json().await?;
        Ok(result)
    }

    pub async fn update_contact(&self, contact_id: &str, request: UpdateWealthboxContactRequest) -> Result<WealthboxContact> {
        let response = self.client()
            .put(format!("{}/contacts/{}", self.base_url(), contact_id))
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

        let result: WealthboxContact = response.json().await?;
        Ok(result)
    }

    pub async fn delete_contact(&self, contact_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/contacts/{}", self.base_url(), contact_id))
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

    pub async fn search_contacts(&self, query: &str, page: u32, per_page: u32) -> Result<WealthboxContactsResponse> {
        let response = self.client()
            .get(format!("{}/contacts/search", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[
                ("q", query.to_string()),
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

        let result: WealthboxContactsResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WealthboxContactsResponse {
    pub contacts: Vec<WealthboxContact>,
    pub meta: Option<WealthboxMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WealthboxMeta {
    pub total_count: Option<i32>,
    pub total_pages: Option<i32>,
    pub current_page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WealthboxContact {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub nickname: Option<String>,
    pub job_title: Option<String>,
    pub company: Option<String>,
    pub email_addresses: Option<Vec<WealthboxEmail>>,
    pub phone_numbers: Option<Vec<WealthboxPhone>>,
    pub street_addresses: Option<Vec<WealthboxAddress>>,
    pub contact_type: Option<String>,
    pub assigned_to: Option<String>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub birthday: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthboxEmail {
    pub address: String,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthboxPhone {
    pub number: String,
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthboxAddress {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "type")]
    pub address_type: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateWealthboxContactRequest {
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_addresses: Option<Vec<WealthboxEmail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_numbers: Option<Vec<WealthboxPhone>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_addresses: Option<Vec<WealthboxAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateWealthboxContactRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_addresses: Option<Vec<WealthboxEmail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_numbers: Option<Vec<WealthboxPhone>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_addresses: Option<Vec<WealthboxAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl From<WealthboxContact> for Contact {
    fn from(c: WealthboxContact) -> Self {
        let email = c.email_addresses
            .as_ref()
            .and_then(|e| e.iter().find(|em| em.primary.unwrap_or(false)).or(e.first()))
            .map(|e| e.address.clone());

        let phone = c.phone_numbers
            .as_ref()
            .and_then(|p| p.iter().find(|ph| ph.primary.unwrap_or(false)).or(p.first()))
            .map(|p| p.number.clone());

        let address = c.street_addresses
            .as_ref()
            .and_then(|a| a.iter().find(|ad| ad.primary.unwrap_or(false)).or(a.first()))
            .map(|a| Address {
                street: a.street.clone(),
                city: a.city.clone(),
                state: a.state.clone(),
                postal_code: a.zip_code.clone(),
                country: a.country.clone(),
            });

        Contact {
            id: Some(c.id.to_string()),
            email,
            first_name: c.first_name,
            last_name: c.last_name,
            phone,
            company: c.company,
            title: c.job_title,
            address,
            custom_fields: HashMap::new(),
            created_at: c.created_at.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: c.updated_at.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}
