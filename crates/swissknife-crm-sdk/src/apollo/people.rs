use crate::{Error, Result, Contact, Lead, ListOptions, ListResult};
use crate::apollo::ApolloClient;
use serde::{Deserialize, Serialize};

impl ApolloClient {
    pub async fn search_people(&self, request: SearchPeopleRequest) -> Result<SearchPeopleResponse> {
        let response = self.client()
            .post(format!("{}/mixed_people/search", self.base_url()))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "api_key": self.api_key(),
                "q_organization_domains": request.domains,
                "person_titles": request.titles,
                "person_locations": request.locations,
                "q_keywords": request.keywords,
                "page": request.page.unwrap_or(1),
                "per_page": request.per_page.unwrap_or(25)
            }))
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

        let result: SearchPeopleResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_person(&self, person_id: &str) -> Result<ApolloPerson> {
        let response = self.client()
            .get(format!("{}/people/{}", self.base_url(), person_id))
            .header("Content-Type", "application/json")
            .query(&[("api_key", self.api_key())])
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

        let result: PersonResponse = response.json().await?;
        Ok(result.person)
    }

    pub async fn match_person(&self, email: Option<&str>, name: Option<&str>, domain: Option<&str>) -> Result<ApolloPerson> {
        let mut body = serde_json::json!({
            "api_key": self.api_key()
        });

        if let Some(e) = email {
            body["email"] = serde_json::Value::String(e.to_string());
        }
        if let Some(n) = name {
            body["name"] = serde_json::Value::String(n.to_string());
        }
        if let Some(d) = domain {
            body["domain"] = serde_json::Value::String(d.to_string());
        }

        let response = self.client()
            .post(format!("{}/people/match", self.base_url()))
            .header("Content-Type", "application/json")
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

        let result: PersonResponse = response.json().await?;
        Ok(result.person)
    }

    pub async fn list_contacts_apollo(&self, options: &ListOptions) -> Result<ContactsResponse> {
        let mut body = serde_json::json!({
            "api_key": self.api_key(),
            "page": options.offset.map(|o| o / options.limit.unwrap_or(25) + 1).unwrap_or(1),
            "per_page": options.limit.unwrap_or(25)
        });

        if let Some(q) = &options.query {
            body["q_keywords"] = serde_json::Value::String(q.clone());
        }

        let response = self.client()
            .post(format!("{}/contacts/search", self.base_url()))
            .header("Content-Type", "application/json")
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

        let result: ContactsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn create_contact_apollo(&self, contact: &Contact) -> Result<ApolloContact> {
        let body = serde_json::json!({
            "api_key": self.api_key(),
            "first_name": contact.first_name,
            "last_name": contact.last_name,
            "email": contact.email,
            "organization_name": contact.company,
            "title": contact.title,
            "phone_numbers": contact.phone.as_ref().map(|p| vec![serde_json::json!({"raw_number": p})])
        });

        let response = self.client()
            .post(format!("{}/contacts", self.base_url()))
            .header("Content-Type", "application/json")
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

        let result: ContactResponse = response.json().await?;
        Ok(result.contact)
    }

    pub async fn update_contact_apollo(&self, contact_id: &str, contact: &Contact) -> Result<ApolloContact> {
        let body = serde_json::json!({
            "api_key": self.api_key(),
            "first_name": contact.first_name,
            "last_name": contact.last_name,
            "email": contact.email,
            "organization_name": contact.company,
            "title": contact.title
        });

        let response = self.client()
            .put(format!("{}/contacts/{}", self.base_url(), contact_id))
            .header("Content-Type", "application/json")
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

        let result: ContactResponse = response.json().await?;
        Ok(result.contact)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchPeopleRequest {
    pub domains: Option<Vec<String>>,
    pub titles: Option<Vec<String>>,
    pub locations: Option<Vec<String>>,
    pub keywords: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchPeopleResponse {
    pub people: Vec<ApolloPerson>,
    pub pagination: Option<ApolloPagination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonResponse {
    pub person: ApolloPerson,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloPerson {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub title: Option<String>,
    pub linkedin_url: Option<String>,
    pub twitter_url: Option<String>,
    pub facebook_url: Option<String>,
    pub github_url: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub organization: Option<ApolloOrganizationRef>,
    pub phone_numbers: Option<Vec<ApolloPhoneNumber>>,
    pub departments: Option<Vec<String>>,
    pub seniority: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloOrganizationRef {
    pub id: Option<String>,
    pub name: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloPhoneNumber {
    pub raw_number: Option<String>,
    pub sanitized_number: Option<String>,
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloPagination {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub total_entries: Option<i32>,
    pub total_pages: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactsResponse {
    pub contacts: Vec<ApolloContact>,
    pub pagination: Option<ApolloPagination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactResponse {
    pub contact: ApolloContact,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloContact {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub title: Option<String>,
    pub organization_name: Option<String>,
    pub account_id: Option<String>,
    pub owner_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<ApolloPerson> for Lead {
    fn from(p: ApolloPerson) -> Self {
        Lead {
            id: Some(p.id),
            email: p.email,
            first_name: p.first_name,
            last_name: p.last_name,
            company: p.organization.and_then(|o| o.name),
            title: p.title,
            phone: p.phone_numbers.and_then(|phones| phones.first().and_then(|p| p.raw_number.clone())),
            source: Some("apollo".to_string()),
            status: None,
            owner_id: None,
            custom_fields: std::collections::HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

impl From<ApolloContact> for Contact {
    fn from(c: ApolloContact) -> Self {
        Contact {
            id: Some(c.id),
            email: c.email,
            first_name: c.first_name,
            last_name: c.last_name,
            phone: None,
            company: c.organization_name,
            title: c.title,
            address: None,
            custom_fields: std::collections::HashMap::new(),
            created_at: c.created_at.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: c.updated_at.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}
