use crate::{Error, Result, Lead};
use crate::hunter::HunterClient;
use serde::Deserialize;

impl HunterClient {
    pub async fn domain_search(&self, domain: &str, params: Option<DomainSearchParams>) -> Result<DomainSearchResponse> {
        let mut query = vec![
            ("domain", domain.to_string()),
            ("api_key", self.api_key().to_string()),
        ];

        if let Some(p) = params {
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
            if let Some(t) = p.email_type {
                query.push(("type", t));
            }
            if let Some(seniority) = p.seniority {
                query.push(("seniority", seniority.join(",")));
            }
            if let Some(department) = p.department {
                query.push(("department", department.join(",")));
            }
        }

        let response = self.client()
            .get(format!("{}/domain-search", self.base_url()))
            .query(&query)
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

        let result: HunterResponse<DomainSearchData> = response.json().await?;
        Ok(DomainSearchResponse {
            data: result.data,
            meta: result.meta,
        })
    }

    pub async fn email_finder(&self, domain: &str, first_name: Option<&str>, last_name: Option<&str>, full_name: Option<&str>) -> Result<EmailFinderResponse> {
        let mut query = vec![
            ("domain", domain.to_string()),
            ("api_key", self.api_key().to_string()),
        ];

        if let Some(fn_) = first_name {
            query.push(("first_name", fn_.to_string()));
        }
        if let Some(ln) = last_name {
            query.push(("last_name", ln.to_string()));
        }
        if let Some(name) = full_name {
            query.push(("full_name", name.to_string()));
        }

        let response = self.client()
            .get(format!("{}/email-finder", self.base_url()))
            .query(&query)
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

        let result: HunterResponse<EmailFinderData> = response.json().await?;
        Ok(EmailFinderResponse {
            data: result.data,
            meta: result.meta,
        })
    }

    pub async fn email_count(&self, domain: &str) -> Result<EmailCountResponse> {
        let response = self.client()
            .get(format!("{}/email-count", self.base_url()))
            .query(&[
                ("domain", domain),
                ("api_key", self.api_key()),
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

        let result: HunterResponse<EmailCountData> = response.json().await?;
        Ok(EmailCountResponse { data: result.data })
    }

    pub async fn author_finder(&self, url: &str) -> Result<AuthorFinderResponse> {
        let response = self.client()
            .get(format!("{}/author-finder", self.base_url()))
            .query(&[
                ("url", url),
                ("api_key", self.api_key()),
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

        let result: HunterResponse<AuthorFinderData> = response.json().await?;
        Ok(AuthorFinderResponse { data: result.data })
    }
}

#[derive(Debug, Clone, Default)]
pub struct DomainSearchParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub email_type: Option<String>,
    pub seniority: Option<Vec<String>>,
    pub department: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HunterResponse<T> {
    pub data: T,
    pub meta: Option<HunterMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HunterMeta {
    pub results: Option<i32>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainSearchResponse {
    pub data: DomainSearchData,
    pub meta: Option<HunterMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainSearchData {
    pub domain: Option<String>,
    pub disposable: Option<bool>,
    pub webmail: Option<bool>,
    pub accept_all: Option<bool>,
    pub pattern: Option<String>,
    pub organization: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub linkedin: Option<String>,
    pub instagram: Option<String>,
    pub youtube: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub street: Option<String>,
    pub headcount: Option<String>,
    pub company_type: Option<String>,
    pub emails: Option<Vec<HunterEmail>>,
    pub linked_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HunterEmail {
    pub value: String,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
    pub confidence: Option<i32>,
    pub sources: Option<Vec<HunterSource>>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub position: Option<String>,
    pub seniority: Option<String>,
    pub department: Option<String>,
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub phone_number: Option<String>,
    pub verification: Option<HunterVerification>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HunterSource {
    pub domain: Option<String>,
    pub uri: Option<String>,
    pub extracted_on: Option<String>,
    pub last_seen_on: Option<String>,
    pub still_on_page: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HunterVerification {
    pub date: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailFinderResponse {
    pub data: EmailFinderData,
    pub meta: Option<HunterMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailFinderData {
    pub email: Option<String>,
    pub score: Option<i32>,
    pub domain: Option<String>,
    pub accept_all: Option<bool>,
    pub position: Option<String>,
    pub twitter: Option<String>,
    pub linkedin_url: Option<String>,
    pub phone_number: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub sources: Option<Vec<HunterSource>>,
    pub verification: Option<HunterVerification>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailCountResponse {
    pub data: EmailCountData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailCountData {
    pub total: Option<i32>,
    pub personal_emails: Option<i32>,
    pub generic_emails: Option<i32>,
    pub department: Option<serde_json::Value>,
    pub seniority: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorFinderResponse {
    pub data: AuthorFinderData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorFinderData {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub score: Option<i32>,
    pub domain: Option<String>,
    pub position: Option<String>,
    pub twitter: Option<String>,
    pub linkedin_url: Option<String>,
    pub sources: Option<Vec<HunterSource>>,
}

impl From<HunterEmail> for Lead {
    fn from(e: HunterEmail) -> Self {
        Lead {
            id: None,
            email: Some(e.value),
            first_name: e.first_name,
            last_name: e.last_name,
            company: None,
            title: e.position,
            phone: e.phone_number,
            source: Some("hunter".to_string()),
            status: None,
            owner_id: None,
            custom_fields: std::collections::HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }
}
