use crate::{Activity, ActivityType, Address, Company, Contact, CrmProvider, Deal, Error, ListOptions, ListResult, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct ZohoClient {
    http: reqwest::Client,
    auth: RwLock<Option<AuthToken>>,
    client_id: String,
    client_secret: String,
    refresh_token: String,
    api_domain: String,
}

#[derive(Debug, Clone)]
struct AuthToken {
    access_token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl ZohoClient {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        refresh_token: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth: RwLock::new(None),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            refresh_token: refresh_token.into(),
            api_domain: "https://www.zohoapis.com".to_string(),
        }
    }

    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.api_domain = domain.into();
        self
    }

    pub fn eu(client_id: impl Into<String>, client_secret: impl Into<String>, refresh_token: impl Into<String>) -> Self {
        Self::new(client_id, client_secret, refresh_token).with_domain("https://www.zohoapis.eu")
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

        let response = self.http
            .post("https://accounts.zoho.com/oauth/v2/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("refresh_token", &self.refresh_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::AuthFailed("Failed to refresh token".into()));
        }

        let token_response: TokenResponse = response.json().await?;
        let token = AuthToken {
            access_token: token_response.access_token.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64 - 60),
        };

        let access_token = token.access_token.clone();
        *self.auth.write().unwrap() = Some(token);
        Ok(access_token)
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/crm/v5{}", self.api_domain, path)
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

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: ZohoErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.message.unwrap_or_else(|| "Unknown error".into());
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

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: ZohoErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.message.unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    async fn put<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let token = self.get_access_token().await?;
        let response = self.http
            .put(self.api_url(path))
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: ZohoErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.message.unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let token = self.get_access_token().await?;
        let response = self.http
            .delete(self.api_url(path))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: ZohoErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.message.unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(())
    }

    pub async fn search_contacts(&self, query: &str) -> Result<Vec<Contact>> {
        let result: ZohoListResponse<ZohoContact> = self.get(&format!("/Contacts/search?criteria=(Full_Name:contains:{})", urlencoding::encode(query))).await?;
        Ok(result.data.unwrap_or_default().into_iter().map(|c| zoho_to_contact(c)).collect())
    }

    pub async fn search_accounts(&self, query: &str) -> Result<Vec<Company>> {
        let result: ZohoListResponse<ZohoAccount> = self.get(&format!("/Accounts/search?criteria=(Account_Name:contains:{})", urlencoding::encode(query))).await?;
        Ok(result.data.unwrap_or_default().into_iter().map(|a| zoho_to_company(a)).collect())
    }
}

#[async_trait]
impl CrmProvider for ZohoClient {
    async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let zoho_contact = contact_to_zoho(contact);
        let body = ZohoCreateRequest { data: vec![zoho_contact] };
        let result: ZohoCreateResponse = self.post("/Contacts", &body).await?;
        let id = result.data.first().and_then(|d| d.details.id.clone()).ok_or_else(|| Error::Provider("No ID returned".into()))?;
        self.get_contact(&id).await
    }

    async fn get_contact(&self, id: &str) -> Result<Contact> {
        let result: ZohoRecordResponse<ZohoContact> = self.get(&format!("/Contacts/{}", id)).await?;
        result.data.into_iter().next().map(|c| zoho_to_contact(c)).ok_or_else(|| Error::NotFound("Contact not found".into()))
    }

    async fn update_contact(&self, id: &str, contact: &Contact) -> Result<Contact> {
        let mut zoho_contact = contact_to_zoho(contact);
        zoho_contact.id = Some(id.to_string());
        let body = ZohoCreateRequest { data: vec![zoho_contact] };
        let _: ZohoCreateResponse = self.put("/Contacts", &body).await?;
        self.get_contact(id).await
    }

    async fn delete_contact(&self, id: &str) -> Result<()> {
        self.delete(&format!("/Contacts?ids={}", id)).await
    }

    async fn list_contacts(&self, options: &ListOptions) -> Result<ListResult<Contact>> {
        let per_page = options.limit.unwrap_or(200);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZohoListResponse<ZohoContact> = self.get(&format!("/Contacts?per_page={}&page={}", per_page, page)).await?;
        let contacts: Vec<Contact> = result.data.unwrap_or_default().into_iter().map(|c| zoho_to_contact(c)).collect();

        let has_more = result.info.as_ref().map(|i| i.more_records).unwrap_or(false);

        Ok(ListResult {
            items: contacts,
            total: result.info.as_ref().and_then(|i| i.count).map(|c| c as u64),
            next_cursor: None,
            has_more,
        })
    }

    async fn create_company(&self, company: &Company) -> Result<Company> {
        let zoho_account = company_to_zoho(company);
        let body = ZohoCreateRequest { data: vec![zoho_account] };
        let result: ZohoCreateResponse = self.post("/Accounts", &body).await?;
        let id = result.data.first().and_then(|d| d.details.id.clone()).ok_or_else(|| Error::Provider("No ID returned".into()))?;
        self.get_company(&id).await
    }

    async fn get_company(&self, id: &str) -> Result<Company> {
        let result: ZohoRecordResponse<ZohoAccount> = self.get(&format!("/Accounts/{}", id)).await?;
        result.data.into_iter().next().map(|a| zoho_to_company(a)).ok_or_else(|| Error::NotFound("Account not found".into()))
    }

    async fn update_company(&self, id: &str, company: &Company) -> Result<Company> {
        let mut zoho_account = company_to_zoho(company);
        zoho_account.id = Some(id.to_string());
        let body = ZohoCreateRequest { data: vec![zoho_account] };
        let _: ZohoCreateResponse = self.put("/Accounts", &body).await?;
        self.get_company(id).await
    }

    async fn delete_company(&self, id: &str) -> Result<()> {
        self.delete(&format!("/Accounts?ids={}", id)).await
    }

    async fn list_companies(&self, options: &ListOptions) -> Result<ListResult<Company>> {
        let per_page = options.limit.unwrap_or(200);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZohoListResponse<ZohoAccount> = self.get(&format!("/Accounts?per_page={}&page={}", per_page, page)).await?;
        let companies: Vec<Company> = result.data.unwrap_or_default().into_iter().map(|a| zoho_to_company(a)).collect();

        let has_more = result.info.as_ref().map(|i| i.more_records).unwrap_or(false);

        Ok(ListResult {
            items: companies,
            total: result.info.as_ref().and_then(|i| i.count).map(|c| c as u64),
            next_cursor: None,
            has_more,
        })
    }

    async fn create_deal(&self, deal: &Deal) -> Result<Deal> {
        let zoho_deal = deal_to_zoho(deal);
        let body = ZohoCreateRequest { data: vec![zoho_deal] };
        let result: ZohoCreateResponse = self.post("/Deals", &body).await?;
        let id = result.data.first().and_then(|d| d.details.id.clone()).ok_or_else(|| Error::Provider("No ID returned".into()))?;
        self.get_deal(&id).await
    }

    async fn get_deal(&self, id: &str) -> Result<Deal> {
        let result: ZohoRecordResponse<ZohoDeal> = self.get(&format!("/Deals/{}", id)).await?;
        result.data.into_iter().next().map(|d| zoho_to_deal(d)).ok_or_else(|| Error::NotFound("Deal not found".into()))
    }

    async fn update_deal(&self, id: &str, deal: &Deal) -> Result<Deal> {
        let mut zoho_deal = deal_to_zoho(deal);
        zoho_deal.id = Some(id.to_string());
        let body = ZohoCreateRequest { data: vec![zoho_deal] };
        let _: ZohoCreateResponse = self.put("/Deals", &body).await?;
        self.get_deal(id).await
    }

    async fn delete_deal(&self, id: &str) -> Result<()> {
        self.delete(&format!("/Deals?ids={}", id)).await
    }

    async fn list_deals(&self, options: &ListOptions) -> Result<ListResult<Deal>> {
        let per_page = options.limit.unwrap_or(200);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZohoListResponse<ZohoDeal> = self.get(&format!("/Deals?per_page={}&page={}", per_page, page)).await?;
        let deals: Vec<Deal> = result.data.unwrap_or_default().into_iter().map(|d| zoho_to_deal(d)).collect();

        let has_more = result.info.as_ref().map(|i| i.more_records).unwrap_or(false);

        Ok(ListResult {
            items: deals,
            total: result.info.as_ref().and_then(|i| i.count).map(|c| c as u64),
            next_cursor: None,
            has_more,
        })
    }

    async fn create_activity(&self, activity: &Activity) -> Result<Activity> {
        let zoho_task = activity_to_zoho(activity);
        let body = ZohoCreateRequest { data: vec![zoho_task] };
        let result: ZohoCreateResponse = self.post("/Tasks", &body).await?;
        let id = result.data.first().and_then(|d| d.details.id.clone()).ok_or_else(|| Error::Provider("No ID returned".into()))?;

        Ok(Activity {
            id: Some(id),
            ..activity.clone()
        })
    }

    async fn list_activities(&self, options: &ListOptions) -> Result<ListResult<Activity>> {
        let per_page = options.limit.unwrap_or(200);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZohoListResponse<ZohoTask> = self.get(&format!("/Tasks?per_page={}&page={}", per_page, page)).await?;
        let activities: Vec<Activity> = result.data.unwrap_or_default().into_iter().map(|t| zoho_to_activity(t)).collect();

        let has_more = result.info.as_ref().map(|i| i.more_records).unwrap_or(false);

        Ok(ListResult {
            items: activities,
            total: result.info.as_ref().and_then(|i| i.count).map(|c| c as u64),
            next_cursor: None,
            has_more,
        })
    }
}

fn contact_to_zoho(contact: &Contact) -> ZohoContact {
    ZohoContact {
        id: None,
        email: contact.email.clone(),
        first_name: contact.first_name.clone(),
        last_name: contact.last_name.clone().unwrap_or_else(|| "-".to_string()),
        phone: contact.phone.clone(),
        title: contact.title.clone(),
        mailing_street: contact.address.as_ref().and_then(|a| a.street.clone()),
        mailing_city: contact.address.as_ref().and_then(|a| a.city.clone()),
        mailing_state: contact.address.as_ref().and_then(|a| a.state.clone()),
        mailing_zip: contact.address.as_ref().and_then(|a| a.postal_code.clone()),
        mailing_country: contact.address.as_ref().and_then(|a| a.country.clone()),
        created_time: None,
        modified_time: None,
    }
}

fn zoho_to_contact(contact: ZohoContact) -> Contact {
    Contact {
        id: contact.id,
        email: contact.email,
        first_name: contact.first_name,
        last_name: Some(contact.last_name),
        phone: contact.phone,
        company: None,
        title: contact.title,
        address: Some(Address {
            street: contact.mailing_street,
            city: contact.mailing_city,
            state: contact.mailing_state,
            postal_code: contact.mailing_zip,
            country: contact.mailing_country,
        }),
        custom_fields: HashMap::new(),
        created_at: contact.created_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: contact.modified_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn company_to_zoho(company: &Company) -> ZohoAccount {
    ZohoAccount {
        id: None,
        account_name: company.name.clone(),
        website: company.website.clone(),
        industry: company.industry.clone(),
        phone: company.phone.clone(),
        billing_street: company.address.as_ref().and_then(|a| a.street.clone()),
        billing_city: company.address.as_ref().and_then(|a| a.city.clone()),
        billing_state: company.address.as_ref().and_then(|a| a.state.clone()),
        billing_code: company.address.as_ref().and_then(|a| a.postal_code.clone()),
        billing_country: company.address.as_ref().and_then(|a| a.country.clone()),
        employees: company.employee_count,
        annual_revenue: company.annual_revenue,
        created_time: None,
        modified_time: None,
    }
}

fn zoho_to_company(account: ZohoAccount) -> Company {
    Company {
        id: account.id,
        name: account.account_name,
        domain: None,
        industry: account.industry,
        phone: account.phone,
        website: account.website,
        address: Some(Address {
            street: account.billing_street,
            city: account.billing_city,
            state: account.billing_state,
            postal_code: account.billing_code,
            country: account.billing_country,
        }),
        employee_count: account.employees,
        annual_revenue: account.annual_revenue,
        custom_fields: HashMap::new(),
        created_at: account.created_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: account.modified_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn deal_to_zoho(deal: &Deal) -> ZohoDeal {
    ZohoDeal {
        id: None,
        deal_name: deal.name.clone(),
        amount: deal.amount,
        stage: deal.stage.clone(),
        probability: deal.probability.map(|p| p as i32),
        closing_date: deal.expected_close_date.map(|d| d.to_string()),
        account_id: deal.company_id.clone(),
        contact_id: deal.contact_id.clone(),
        created_time: None,
        modified_time: None,
    }
}

fn zoho_to_deal(deal: ZohoDeal) -> Deal {
    Deal {
        id: deal.id,
        name: deal.deal_name,
        amount: deal.amount,
        currency: None,
        stage: deal.stage,
        pipeline: None,
        probability: deal.probability.map(|p| p as f64),
        expected_close_date: deal.closing_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        contact_id: deal.contact_id,
        company_id: deal.account_id,
        owner_id: None,
        custom_fields: HashMap::new(),
        created_at: deal.created_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: deal.modified_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn activity_to_zoho(activity: &Activity) -> ZohoTask {
    ZohoTask {
        id: None,
        subject: activity.subject.clone().unwrap_or_else(|| "Task".to_string()),
        description: activity.body.clone(),
        status: if activity.completed { Some("Completed".to_string()) } else { Some("Not Started".to_string()) },
        due_date: activity.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
        contact_id: activity.contact_id.clone(),
        deal_id: activity.deal_id.clone(),
        created_time: None,
    }
}

fn zoho_to_activity(task: ZohoTask) -> Activity {
    Activity {
        id: task.id,
        activity_type: ActivityType::Task,
        subject: Some(task.subject),
        body: task.description,
        due_date: task.due_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()).map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()),
        completed: task.status.as_deref() == Some("Completed"),
        contact_id: task.contact_id,
        company_id: None,
        deal_id: task.deal_id,
        owner_id: None,
        created_at: task.created_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Default, Deserialize)]
struct ZohoErrorResponse {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZohoListResponse<T> {
    data: Option<Vec<T>>,
    info: Option<ZohoInfo>,
}

#[derive(Debug, Deserialize)]
struct ZohoRecordResponse<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct ZohoInfo {
    more_records: bool,
    count: Option<i32>,
}

#[derive(Debug, Serialize)]
struct ZohoCreateRequest<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct ZohoCreateResponse {
    data: Vec<ZohoCreateResult>,
}

#[derive(Debug, Deserialize)]
struct ZohoCreateResult {
    details: ZohoCreateDetails,
}

#[derive(Debug, Deserialize)]
struct ZohoCreateDetails {
    id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ZohoContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    last_name: String,
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
    mailing_zip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mailing_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ZohoAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    account_name: String,
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
    billing_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    billing_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    employees: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annual_revenue: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ZohoDeal {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    deal_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    probability: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    closing_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ZohoTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deal_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_time: Option<String>,
}
