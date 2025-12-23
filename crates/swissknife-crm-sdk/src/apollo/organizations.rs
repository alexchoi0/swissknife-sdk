use crate::{Error, Result, Company};
use crate::apollo::ApolloClient;
use serde::Deserialize;

impl ApolloClient {
    pub async fn search_organizations(&self, request: SearchOrganizationsRequest) -> Result<SearchOrganizationsResponse> {
        let response = self.client()
            .post(format!("{}/mixed_companies/search", self.base_url()))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "api_key": self.api_key(),
                "q_organization_name": request.name,
                "organization_locations": request.locations,
                "organization_num_employees_ranges": request.employee_ranges,
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

        let result: SearchOrganizationsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_organization(&self, organization_id: &str) -> Result<ApolloOrganization> {
        let response = self.client()
            .get(format!("{}/organizations/{}", self.base_url(), organization_id))
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

        let result: OrganizationResponse = response.json().await?;
        Ok(result.organization)
    }

    pub async fn enrich_organization(&self, domain: &str) -> Result<ApolloOrganization> {
        let response = self.client()
            .get(format!("{}/organizations/enrich", self.base_url()))
            .header("Content-Type", "application/json")
            .query(&[
                ("api_key", self.api_key()),
                ("domain", domain),
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

        let result: OrganizationResponse = response.json().await?;
        Ok(result.organization)
    }

    pub async fn list_accounts(&self, page: u32, per_page: u32) -> Result<AccountsResponse> {
        let response = self.client()
            .post(format!("{}/accounts/search", self.base_url()))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "api_key": self.api_key(),
                "page": page,
                "per_page": per_page
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

        let result: AccountsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn create_account(&self, company: &Company) -> Result<ApolloAccount> {
        let body = serde_json::json!({
            "api_key": self.api_key(),
            "name": company.name,
            "domain": company.domain,
            "phone_number": company.phone,
            "raw_address": company.address.as_ref().map(|a| format!(
                "{}, {}, {} {}",
                a.street.as_deref().unwrap_or(""),
                a.city.as_deref().unwrap_or(""),
                a.state.as_deref().unwrap_or(""),
                a.postal_code.as_deref().unwrap_or("")
            ))
        });

        let response = self.client()
            .post(format!("{}/accounts", self.base_url()))
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

        let result: AccountResponse = response.json().await?;
        Ok(result.account)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchOrganizationsRequest {
    pub name: Option<String>,
    pub locations: Option<Vec<String>>,
    pub employee_ranges: Option<Vec<String>>,
    pub keywords: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchOrganizationsResponse {
    pub organizations: Vec<ApolloOrganization>,
    pub pagination: Option<super::people::ApolloPagination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationResponse {
    pub organization: ApolloOrganization,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloOrganization {
    pub id: String,
    pub name: Option<String>,
    pub website_url: Option<String>,
    pub blog_url: Option<String>,
    pub angellist_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub twitter_url: Option<String>,
    pub facebook_url: Option<String>,
    pub primary_phone: Option<ApolloOrgPhone>,
    pub languages: Option<Vec<String>>,
    pub alexa_ranking: Option<i64>,
    pub phone: Option<String>,
    pub linkedin_uid: Option<String>,
    pub founded_year: Option<i32>,
    pub publicly_traded_symbol: Option<String>,
    pub publicly_traded_exchange: Option<String>,
    pub logo_url: Option<String>,
    pub crunchbase_url: Option<String>,
    pub primary_domain: Option<String>,
    pub industry: Option<String>,
    pub estimated_num_employees: Option<i32>,
    pub raw_address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub short_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloOrgPhone {
    pub number: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountsResponse {
    pub accounts: Vec<ApolloAccount>,
    pub pagination: Option<super::people::ApolloPagination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountResponse {
    pub account: ApolloAccount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApolloAccount {
    pub id: String,
    pub name: Option<String>,
    pub domain: Option<String>,
    pub phone_number: Option<String>,
    pub owner_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<ApolloOrganization> for Company {
    fn from(o: ApolloOrganization) -> Self {
        Company {
            id: Some(o.id),
            name: o.name.unwrap_or_default(),
            domain: o.primary_domain.or(o.website_url),
            industry: o.industry,
            phone: o.phone.or(o.primary_phone.and_then(|p| p.number)),
            website: None,
            address: Some(crate::Address {
                street: o.raw_address,
                city: o.city,
                state: o.state,
                postal_code: o.postal_code,
                country: o.country,
            }),
            employee_count: o.estimated_num_employees,
            annual_revenue: None,
            custom_fields: std::collections::HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }
}
