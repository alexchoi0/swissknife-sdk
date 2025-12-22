use crate::{Activity, ActivityType, Address, Company, Contact, CrmProvider, Deal, Error, ListOptions, ListResult, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

pub struct SalesforceClient {
    instance_url: String,
    http: reqwest::Client,
    auth: RwLock<Option<AuthToken>>,
    client_id: String,
    client_secret: String,
    refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
struct AuthToken {
    access_token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl SalesforceClient {
    pub fn new(
        instance_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Self {
        Self {
            instance_url: instance_url.into(),
            http: reqwest::Client::new(),
            auth: RwLock::new(None),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            refresh_token: None,
        }
    }

    pub fn with_access_token(self, access_token: impl Into<String>) -> Self {
        let token = AuthToken {
            access_token: access_token.into(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        *self.auth.write().unwrap() = Some(token);
        self
    }

    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(refresh_token.into());
        self
    }

    async fn get_access_token(&self) -> Result<String> {
        {
            let guard = self.auth.read().unwrap();
            if let Some(token) = guard.as_ref() {
                if token.expires_at > chrono::Utc::now() {
                    return Ok(token.access_token.clone());
                }
            }
        }

        if let Some(refresh_token) = &self.refresh_token {
            let response = self.http
                .post(format!("{}/services/oauth2/token", self.instance_url))
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("client_id", &self.client_id),
                    ("client_secret", &self.client_secret),
                    ("refresh_token", refresh_token),
                ])
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(Error::AuthFailed("Failed to refresh token".into()));
            }

            let token_response: TokenResponse = response.json().await?;
            let token = AuthToken {
                access_token: token_response.access_token.clone(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            };

            let access_token = token.access_token.clone();
            *self.auth.write().unwrap() = Some(token);
            return Ok(access_token);
        }

        Err(Error::AuthFailed("No valid authentication".into()))
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/services/data/v59.0{}", self.instance_url, path)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let token = self.get_access_token().await?;
        let response = self.http
            .get(self.api_url(path))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status() == 404 {
            return Err(Error::NotFound("Resource not found".into()));
        }

        if !response.status().is_success() {
            let errors: Vec<SalesforceError> = response.json().await.unwrap_or_default();
            let msg = errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let token = self.get_access_token().await?;
        let response = self.http
            .post(self.api_url(path))
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let errors: Vec<SalesforceError> = response.json().await.unwrap_or_default();
            let msg = errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    async fn patch(&self, path: &str, body: impl Serialize) -> Result<()> {
        let token = self.get_access_token().await?;
        let response = self.http
            .patch(self.api_url(path))
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let errors: Vec<SalesforceError> = response.json().await.unwrap_or_default();
            let msg = errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let token = self.get_access_token().await?;
        let response = self.http
            .delete(self.api_url(path))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            let errors: Vec<SalesforceError> = response.json().await.unwrap_or_default();
            let msg = errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(())
    }

    async fn query<T: for<'de> Deserialize<'de>>(&self, soql: &str) -> Result<QueryResult<T>> {
        let token = self.get_access_token().await?;
        let url = format!("{}?q={}", self.api_url("/query"), urlencoding::encode(soql));
        let response = self.http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            let errors: Vec<SalesforceError> = response.json().await.unwrap_or_default();
            let msg = errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    pub async fn execute_soql<T: for<'de> Deserialize<'de>>(&self, soql: &str) -> Result<Vec<T>> {
        let result: QueryResult<T> = self.query(soql).await?;
        Ok(result.records)
    }
}

#[async_trait]
impl CrmProvider for SalesforceClient {
    async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let sf_contact = SfContact::from_contact(contact);
        let result: CreateResult = self.post("/sobjects/Contact", &sf_contact).await?;
        self.get_contact(&result.id).await
    }

    async fn get_contact(&self, id: &str) -> Result<Contact> {
        let sf_contact: SfContact = self.get(&format!("/sobjects/Contact/{}", id)).await?;
        Ok(sf_contact.into_contact())
    }

    async fn update_contact(&self, id: &str, contact: &Contact) -> Result<Contact> {
        let sf_contact = SfContact::from_contact(contact);
        self.patch(&format!("/sobjects/Contact/{}", id), &sf_contact).await?;
        self.get_contact(id).await
    }

    async fn delete_contact(&self, id: &str) -> Result<()> {
        self.delete(&format!("/sobjects/Contact/{}", id)).await
    }

    async fn list_contacts(&self, options: &ListOptions) -> Result<ListResult<Contact>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let mut soql = format!(
            "SELECT Id, Email, FirstName, LastName, Phone, Title, MailingStreet, MailingCity, MailingState, MailingPostalCode, MailingCountry, CreatedDate, LastModifiedDate FROM Contact ORDER BY LastModifiedDate DESC LIMIT {} OFFSET {}",
            limit, offset
        );

        if let Some(query) = &options.query {
            soql = format!(
                "SELECT Id, Email, FirstName, LastName, Phone, Title, MailingStreet, MailingCity, MailingState, MailingPostalCode, MailingCountry, CreatedDate, LastModifiedDate FROM Contact WHERE Name LIKE '%{}%' ORDER BY LastModifiedDate DESC LIMIT {} OFFSET {}",
                query.replace('\'', "\\'"), limit, offset
            );
        }

        let result: QueryResult<SfContact> = self.query(&soql).await?;
        let contacts: Vec<Contact> = result.records.into_iter().map(|c| c.into_contact()).collect();

        Ok(ListResult {
            items: contacts,
            total: Some(result.total_size as u64),
            next_cursor: result.next_records_url,
            has_more: !result.done,
        })
    }

    async fn create_company(&self, company: &Company) -> Result<Company> {
        let sf_account = SfAccount::from_company(company);
        let result: CreateResult = self.post("/sobjects/Account", &sf_account).await?;
        self.get_company(&result.id).await
    }

    async fn get_company(&self, id: &str) -> Result<Company> {
        let sf_account: SfAccount = self.get(&format!("/sobjects/Account/{}", id)).await?;
        Ok(sf_account.into_company())
    }

    async fn update_company(&self, id: &str, company: &Company) -> Result<Company> {
        let sf_account = SfAccount::from_company(company);
        self.patch(&format!("/sobjects/Account/{}", id), &sf_account).await?;
        self.get_company(id).await
    }

    async fn delete_company(&self, id: &str) -> Result<()> {
        self.delete(&format!("/sobjects/Account/{}", id)).await
    }

    async fn list_companies(&self, options: &ListOptions) -> Result<ListResult<Company>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let soql = format!(
            "SELECT Id, Name, Website, Industry, Phone, BillingStreet, BillingCity, BillingState, BillingPostalCode, BillingCountry, NumberOfEmployees, AnnualRevenue, CreatedDate, LastModifiedDate FROM Account ORDER BY LastModifiedDate DESC LIMIT {} OFFSET {}",
            limit, offset
        );

        let result: QueryResult<SfAccount> = self.query(&soql).await?;
        let companies: Vec<Company> = result.records.into_iter().map(|a| a.into_company()).collect();

        Ok(ListResult {
            items: companies,
            total: Some(result.total_size as u64),
            next_cursor: result.next_records_url,
            has_more: !result.done,
        })
    }

    async fn create_deal(&self, deal: &Deal) -> Result<Deal> {
        let sf_opp = SfOpportunity::from_deal(deal);
        let result: CreateResult = self.post("/sobjects/Opportunity", &sf_opp).await?;
        self.get_deal(&result.id).await
    }

    async fn get_deal(&self, id: &str) -> Result<Deal> {
        let sf_opp: SfOpportunity = self.get(&format!("/sobjects/Opportunity/{}", id)).await?;
        Ok(sf_opp.into_deal())
    }

    async fn update_deal(&self, id: &str, deal: &Deal) -> Result<Deal> {
        let sf_opp = SfOpportunity::from_deal(deal);
        self.patch(&format!("/sobjects/Opportunity/{}", id), &sf_opp).await?;
        self.get_deal(id).await
    }

    async fn delete_deal(&self, id: &str) -> Result<()> {
        self.delete(&format!("/sobjects/Opportunity/{}", id)).await
    }

    async fn list_deals(&self, options: &ListOptions) -> Result<ListResult<Deal>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let soql = format!(
            "SELECT Id, Name, Amount, StageName, Probability, CloseDate, AccountId, OwnerId, CreatedDate, LastModifiedDate FROM Opportunity ORDER BY LastModifiedDate DESC LIMIT {} OFFSET {}",
            limit, offset
        );

        let result: QueryResult<SfOpportunity> = self.query(&soql).await?;
        let deals: Vec<Deal> = result.records.into_iter().map(|o| o.into_deal()).collect();

        Ok(ListResult {
            items: deals,
            total: Some(result.total_size as u64),
            next_cursor: result.next_records_url,
            has_more: !result.done,
        })
    }

    async fn create_activity(&self, activity: &Activity) -> Result<Activity> {
        let sf_task = SfTask::from_activity(activity);
        let result: CreateResult = self.post("/sobjects/Task", &sf_task).await?;

        Ok(Activity {
            id: Some(result.id),
            ..activity.clone()
        })
    }

    async fn list_activities(&self, options: &ListOptions) -> Result<ListResult<Activity>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let soql = format!(
            "SELECT Id, Subject, Description, Status, ActivityDate, WhoId, WhatId, OwnerId, CreatedDate FROM Task ORDER BY CreatedDate DESC LIMIT {} OFFSET {}",
            limit, offset
        );

        let result: QueryResult<SfTask> = self.query(&soql).await?;
        let activities: Vec<Activity> = result.records.into_iter().map(|t| t.into_activity()).collect();

        Ok(ListResult {
            items: activities,
            total: Some(result.total_size as u64),
            next_cursor: result.next_records_url,
            has_more: !result.done,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Default, Deserialize)]
struct SalesforceError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct CreateResult {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryResult<T> {
    total_size: i32,
    done: bool,
    records: Vec<T>,
    next_records_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SfContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mailing_street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mailing_city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mailing_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mailing_postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mailing_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified_date: Option<String>,
}

impl SfContact {
    fn from_contact(contact: &Contact) -> Self {
        Self {
            id: None,
            email: contact.email.clone(),
            first_name: contact.first_name.clone(),
            last_name: contact.last_name.clone(),
            phone: contact.phone.clone(),
            title: contact.title.clone(),
            mailing_street: contact.address.as_ref().and_then(|a| a.street.clone()),
            mailing_city: contact.address.as_ref().and_then(|a| a.city.clone()),
            mailing_state: contact.address.as_ref().and_then(|a| a.state.clone()),
            mailing_postal_code: contact.address.as_ref().and_then(|a| a.postal_code.clone()),
            mailing_country: contact.address.as_ref().and_then(|a| a.country.clone()),
            created_date: None,
            last_modified_date: None,
        }
    }

    fn into_contact(self) -> Contact {
        Contact {
            id: self.id,
            email: self.email,
            first_name: self.first_name,
            last_name: self.last_name,
            phone: self.phone,
            company: None,
            title: self.title,
            address: Some(Address {
                street: self.mailing_street,
                city: self.mailing_city,
                state: self.mailing_state,
                postal_code: self.mailing_postal_code,
                country: self.mailing_country,
            }),
            custom_fields: std::collections::HashMap::new(),
            created_at: self.created_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
            updated_at: self.last_modified_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SfAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    industry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    billing_street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    billing_city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    billing_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    billing_postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    billing_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    number_of_employees: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annual_revenue: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified_date: Option<String>,
}

impl SfAccount {
    fn from_company(company: &Company) -> Self {
        Self {
            id: None,
            name: Some(company.name.clone()),
            website: company.website.clone(),
            industry: company.industry.clone(),
            phone: company.phone.clone(),
            billing_street: company.address.as_ref().and_then(|a| a.street.clone()),
            billing_city: company.address.as_ref().and_then(|a| a.city.clone()),
            billing_state: company.address.as_ref().and_then(|a| a.state.clone()),
            billing_postal_code: company.address.as_ref().and_then(|a| a.postal_code.clone()),
            billing_country: company.address.as_ref().and_then(|a| a.country.clone()),
            number_of_employees: company.employee_count,
            annual_revenue: company.annual_revenue,
            created_date: None,
            last_modified_date: None,
        }
    }

    fn into_company(self) -> Company {
        Company {
            id: self.id,
            name: self.name.unwrap_or_default(),
            domain: None,
            industry: self.industry,
            phone: self.phone,
            website: self.website,
            address: Some(Address {
                street: self.billing_street,
                city: self.billing_city,
                state: self.billing_state,
                postal_code: self.billing_postal_code,
                country: self.billing_country,
            }),
            employee_count: self.number_of_employees,
            annual_revenue: self.annual_revenue,
            custom_fields: std::collections::HashMap::new(),
            created_at: self.created_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
            updated_at: self.last_modified_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SfOpportunity {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    probability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    close_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified_date: Option<String>,
}

impl SfOpportunity {
    fn from_deal(deal: &Deal) -> Self {
        Self {
            id: None,
            name: Some(deal.name.clone()),
            amount: deal.amount,
            stage_name: deal.stage.clone(),
            probability: deal.probability,
            close_date: deal.expected_close_date.map(|d| d.to_string()),
            account_id: deal.company_id.clone(),
            owner_id: deal.owner_id.clone(),
            created_date: None,
            last_modified_date: None,
        }
    }

    fn into_deal(self) -> Deal {
        Deal {
            id: self.id,
            name: self.name.unwrap_or_default(),
            amount: self.amount,
            currency: None,
            stage: self.stage_name,
            pipeline: None,
            probability: self.probability,
            expected_close_date: self.close_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            contact_id: None,
            company_id: self.account_id,
            owner_id: self.owner_id,
            custom_fields: std::collections::HashMap::new(),
            created_at: self.created_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
            updated_at: self.last_modified_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SfTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activity_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    who_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    what_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_date: Option<String>,
}

impl SfTask {
    fn from_activity(activity: &Activity) -> Self {
        Self {
            id: None,
            subject: activity.subject.clone(),
            description: activity.body.clone(),
            status: if activity.completed { Some("Completed".into()) } else { Some("Not Started".into()) },
            activity_date: activity.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
            who_id: activity.contact_id.clone(),
            what_id: activity.deal_id.clone().or_else(|| activity.company_id.clone()),
            owner_id: activity.owner_id.clone(),
            created_date: None,
        }
    }

    fn into_activity(self) -> Activity {
        Activity {
            id: self.id,
            activity_type: ActivityType::Task,
            subject: self.subject,
            body: self.description,
            due_date: self.activity_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()).map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()),
            completed: self.status.as_deref() == Some("Completed"),
            contact_id: self.who_id,
            company_id: None,
            deal_id: self.what_id,
            owner_id: self.owner_id,
            created_at: self.created_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        }
    }
}
