use crate::{Error, Result, Lead, Company};
use crate::clay::ClayClient;
use serde::Deserialize;

impl ClayClient {
    pub async fn enrich_person(&self, request: PersonEnrichmentRequest) -> Result<PersonEnrichmentResponse> {
        let response = self.client()
            .post(format!("{}/enrich/person", self.base_url()))
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

        let result: PersonEnrichmentResponse = response.json().await?;
        Ok(result)
    }

    pub async fn enrich_company(&self, request: CompanyEnrichmentRequest) -> Result<CompanyEnrichmentResponse> {
        let response = self.client()
            .post(format!("{}/enrich/company", self.base_url()))
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

        let result: CompanyEnrichmentResponse = response.json().await?;
        Ok(result)
    }

    pub async fn find_email(&self, first_name: &str, last_name: &str, domain: &str) -> Result<EmailFinderResponse> {
        let response = self.client()
            .post(format!("{}/find/email", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&serde_json::json!({
                "first_name": first_name,
                "last_name": last_name,
                "domain": domain
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

        let result: EmailFinderResponse = response.json().await?;
        Ok(result)
    }

    pub async fn verify_email(&self, email: &str) -> Result<EmailVerificationResponse> {
        let response = self.client()
            .post(format!("{}/verify/email", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&serde_json::json!({ "email": email }))
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

        let result: EmailVerificationResponse = response.json().await?;
        Ok(result)
    }

    pub async fn find_linkedin(&self, first_name: &str, last_name: &str, company: &str) -> Result<LinkedInFinderResponse> {
        let response = self.client()
            .post(format!("{}/find/linkedin", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&serde_json::json!({
                "first_name": first_name,
                "last_name": last_name,
                "company": company
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

        let result: LinkedInFinderResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PersonEnrichmentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonEnrichmentResponse {
    pub person: Option<EnrichedPerson>,
    pub found: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnrichedPerson {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub title: Option<String>,
    pub seniority: Option<String>,
    pub department: Option<String>,
    pub linkedin_url: Option<String>,
    pub twitter_url: Option<String>,
    pub github_url: Option<String>,
    pub facebook_url: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub company: Option<EnrichedCompanyRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnrichedCompanyRef {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub linkedin_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompanyEnrichmentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompanyEnrichmentResponse {
    pub company: Option<EnrichedCompany>,
    pub found: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnrichedCompany {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub employee_count: Option<i32>,
    pub employee_range: Option<String>,
    pub annual_revenue: Option<f64>,
    pub revenue_range: Option<String>,
    pub founded_year: Option<i32>,
    pub headquarters: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub linkedin_url: Option<String>,
    pub twitter_url: Option<String>,
    pub facebook_url: Option<String>,
    pub crunchbase_url: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailFinderResponse {
    pub email: Option<String>,
    pub confidence: Option<f64>,
    pub verified: Option<bool>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailVerificationResponse {
    pub email: String,
    pub is_valid: bool,
    pub is_deliverable: Option<bool>,
    pub is_disposable: Option<bool>,
    pub is_role_address: Option<bool>,
    pub is_free_provider: Option<bool>,
    pub mx_found: Option<bool>,
    pub smtp_check: Option<bool>,
    pub catch_all: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkedInFinderResponse {
    pub linkedin_url: Option<String>,
    pub confidence: Option<f64>,
    pub profile: Option<LinkedInProfile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkedInProfile {
    pub name: Option<String>,
    pub headline: Option<String>,
    pub location: Option<String>,
    pub industry: Option<String>,
    pub connections: Option<i32>,
}

impl From<EnrichedPerson> for Lead {
    fn from(p: EnrichedPerson) -> Self {
        Lead {
            id: None,
            email: p.email,
            first_name: p.first_name,
            last_name: p.last_name,
            company: p.company.and_then(|c| c.name),
            title: p.title,
            phone: p.phone,
            source: Some("clay".to_string()),
            status: None,
            owner_id: None,
            custom_fields: std::collections::HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

impl From<EnrichedCompany> for Company {
    fn from(c: EnrichedCompany) -> Self {
        Company {
            id: None,
            name: c.name.unwrap_or_default(),
            domain: c.domain,
            industry: c.industry,
            phone: None,
            website: c.website,
            address: Some(crate::Address {
                street: None,
                city: c.city,
                state: c.state,
                postal_code: None,
                country: c.country,
            }),
            employee_count: c.employee_count,
            annual_revenue: c.annual_revenue,
            custom_fields: std::collections::HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }
}
