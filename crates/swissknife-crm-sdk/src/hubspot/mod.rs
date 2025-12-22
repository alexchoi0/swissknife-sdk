use crate::{Activity, ActivityType, Address, Company, Contact, CrmProvider, Deal, Error, ListOptions, ListResult, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.hubapi.com";

pub struct HubSpotClient {
    access_token: String,
    http: reqwest::Client,
}

impl HubSpotClient {
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let response = self.http
            .get(format!("{}{}", API_BASE, path))
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if response.status() == 404 {
            return Err(Error::NotFound("Resource not found".into()));
        }

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: HubSpotError = response.json().await.unwrap_or_default();
            return Err(Error::Provider(error.message.unwrap_or_else(|| "Unknown error".into())));
        }

        Ok(response.json().await?)
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .post(format!("{}{}", API_BASE, path))
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: HubSpotError = response.json().await.unwrap_or_default();
            return Err(Error::Provider(error.message.unwrap_or_else(|| "Unknown error".into())));
        }

        Ok(response.json().await?)
    }

    async fn patch<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .patch(format!("{}{}", API_BASE, path))
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: HubSpotError = response.json().await.unwrap_or_default();
            return Err(Error::Provider(error.message.unwrap_or_else(|| "Unknown error".into())));
        }

        Ok(response.json().await?)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let response = self.http
            .delete(format!("{}{}", API_BASE, path))
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: HubSpotError = response.json().await.unwrap_or_default();
            return Err(Error::Provider(error.message.unwrap_or_else(|| "Unknown error".into())));
        }

        Ok(())
    }

    pub async fn search_contacts(&self, query: &str) -> Result<Vec<Contact>> {
        let body = serde_json::json!({
            "query": query,
            "limit": 100
        });

        let result: HsSearchResult = self.post("/crm/v3/objects/contacts/search", &body).await?;
        Ok(result.results.into_iter().map(|r| hs_to_contact(r)).collect())
    }

    pub async fn search_companies(&self, query: &str) -> Result<Vec<Company>> {
        let body = serde_json::json!({
            "query": query,
            "limit": 100
        });

        let result: HsSearchResult = self.post("/crm/v3/objects/companies/search", &body).await?;
        Ok(result.results.into_iter().map(|r| hs_to_company(r)).collect())
    }

    pub async fn search_deals(&self, query: &str) -> Result<Vec<Deal>> {
        let body = serde_json::json!({
            "query": query,
            "limit": 100
        });

        let result: HsSearchResult = self.post("/crm/v3/objects/deals/search", &body).await?;
        Ok(result.results.into_iter().map(|r| hs_to_deal(r)).collect())
    }
}

#[async_trait]
impl CrmProvider for HubSpotClient {
    async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let hs_contact = contact_to_hs(contact);
        let result: HsObject = self.post("/crm/v3/objects/contacts", &hs_contact).await?;
        Ok(hs_to_contact(result))
    }

    async fn get_contact(&self, id: &str) -> Result<Contact> {
        let result: HsObject = self.get(&format!("/crm/v3/objects/contacts/{}?properties=email,firstname,lastname,phone,company,jobtitle,address,city,state,zip,country", id)).await?;
        Ok(hs_to_contact(result))
    }

    async fn update_contact(&self, id: &str, contact: &Contact) -> Result<Contact> {
        let hs_contact = contact_to_hs(contact);
        let result: HsObject = self.patch(&format!("/crm/v3/objects/contacts/{}", id), &hs_contact).await?;
        Ok(hs_to_contact(result))
    }

    async fn delete_contact(&self, id: &str) -> Result<()> {
        self.delete(&format!("/crm/v3/objects/contacts/{}", id)).await
    }

    async fn list_contacts(&self, options: &ListOptions) -> Result<ListResult<Contact>> {
        let limit = options.limit.unwrap_or(100);
        let mut url = format!("/crm/v3/objects/contacts?limit={}&properties=email,firstname,lastname,phone,company,jobtitle,address,city,state,zip,country", limit);

        if let Some(cursor) = &options.cursor {
            url.push_str(&format!("&after={}", cursor));
        }

        let result: HsListResult = self.get(&url).await?;
        let contacts: Vec<Contact> = result.results.into_iter().map(|r| hs_to_contact(r)).collect();

        Ok(ListResult {
            items: contacts,
            total: result.total,
            has_more: result.paging.is_some(),
            next_cursor: result.paging.and_then(|p| p.next).map(|n| n.after),
        })
    }

    async fn create_company(&self, company: &Company) -> Result<Company> {
        let hs_company = company_to_hs(company);
        let result: HsObject = self.post("/crm/v3/objects/companies", &hs_company).await?;
        Ok(hs_to_company(result))
    }

    async fn get_company(&self, id: &str) -> Result<Company> {
        let result: HsObject = self.get(&format!("/crm/v3/objects/companies/{}?properties=name,domain,industry,phone,website,address,city,state,zip,country,numberofemployees,annualrevenue", id)).await?;
        Ok(hs_to_company(result))
    }

    async fn update_company(&self, id: &str, company: &Company) -> Result<Company> {
        let hs_company = company_to_hs(company);
        let result: HsObject = self.patch(&format!("/crm/v3/objects/companies/{}", id), &hs_company).await?;
        Ok(hs_to_company(result))
    }

    async fn delete_company(&self, id: &str) -> Result<()> {
        self.delete(&format!("/crm/v3/objects/companies/{}", id)).await
    }

    async fn list_companies(&self, options: &ListOptions) -> Result<ListResult<Company>> {
        let limit = options.limit.unwrap_or(100);
        let mut url = format!("/crm/v3/objects/companies?limit={}&properties=name,domain,industry,phone,website,address,city,state,zip,country,numberofemployees,annualrevenue", limit);

        if let Some(cursor) = &options.cursor {
            url.push_str(&format!("&after={}", cursor));
        }

        let result: HsListResult = self.get(&url).await?;
        let companies: Vec<Company> = result.results.into_iter().map(|r| hs_to_company(r)).collect();

        Ok(ListResult {
            items: companies,
            total: result.total,
            has_more: result.paging.is_some(),
            next_cursor: result.paging.and_then(|p| p.next).map(|n| n.after),
        })
    }

    async fn create_deal(&self, deal: &Deal) -> Result<Deal> {
        let hs_deal = deal_to_hs(deal);
        let result: HsObject = self.post("/crm/v3/objects/deals", &hs_deal).await?;
        Ok(hs_to_deal(result))
    }

    async fn get_deal(&self, id: &str) -> Result<Deal> {
        let result: HsObject = self.get(&format!("/crm/v3/objects/deals/{}?properties=dealname,amount,dealstage,pipeline,closedate,hubspot_owner_id", id)).await?;
        Ok(hs_to_deal(result))
    }

    async fn update_deal(&self, id: &str, deal: &Deal) -> Result<Deal> {
        let hs_deal = deal_to_hs(deal);
        let result: HsObject = self.patch(&format!("/crm/v3/objects/deals/{}", id), &hs_deal).await?;
        Ok(hs_to_deal(result))
    }

    async fn delete_deal(&self, id: &str) -> Result<()> {
        self.delete(&format!("/crm/v3/objects/deals/{}", id)).await
    }

    async fn list_deals(&self, options: &ListOptions) -> Result<ListResult<Deal>> {
        let limit = options.limit.unwrap_or(100);
        let mut url = format!("/crm/v3/objects/deals?limit={}&properties=dealname,amount,dealstage,pipeline,closedate,hubspot_owner_id", limit);

        if let Some(cursor) = &options.cursor {
            url.push_str(&format!("&after={}", cursor));
        }

        let result: HsListResult = self.get(&url).await?;
        let deals: Vec<Deal> = result.results.into_iter().map(|r| hs_to_deal(r)).collect();

        Ok(ListResult {
            items: deals,
            total: result.total,
            has_more: result.paging.is_some(),
            next_cursor: result.paging.and_then(|p| p.next).map(|n| n.after),
        })
    }

    async fn create_activity(&self, activity: &Activity) -> Result<Activity> {
        let hs_activity = activity_to_hs(activity);
        let object_type = match activity.activity_type {
            ActivityType::Call => "calls",
            ActivityType::Email => "emails",
            ActivityType::Meeting => "meetings",
            ActivityType::Task => "tasks",
            ActivityType::Note => "notes",
            ActivityType::Other => "tasks",
        };

        let result: HsObject = self.post(&format!("/crm/v3/objects/{}", object_type), &hs_activity).await?;

        Ok(Activity {
            id: Some(result.id),
            ..activity.clone()
        })
    }

    async fn list_activities(&self, options: &ListOptions) -> Result<ListResult<Activity>> {
        let limit = options.limit.unwrap_or(100);
        let mut url = format!("/crm/v3/objects/tasks?limit={}&properties=hs_task_subject,hs_task_body,hs_task_status,hs_timestamp", limit);

        if let Some(cursor) = &options.cursor {
            url.push_str(&format!("&after={}", cursor));
        }

        let result: HsListResult = self.get(&url).await?;
        let activities: Vec<Activity> = result.results.into_iter().map(|r| hs_to_activity(r)).collect();

        Ok(ListResult {
            items: activities,
            total: result.total,
            has_more: result.paging.is_some(),
            next_cursor: result.paging.and_then(|p| p.next).map(|n| n.after),
        })
    }
}

fn contact_to_hs(contact: &Contact) -> HsCreateObject {
    let mut properties = HashMap::new();

    if let Some(email) = &contact.email {
        properties.insert("email".to_string(), email.clone());
    }
    if let Some(first) = &contact.first_name {
        properties.insert("firstname".to_string(), first.clone());
    }
    if let Some(last) = &contact.last_name {
        properties.insert("lastname".to_string(), last.clone());
    }
    if let Some(phone) = &contact.phone {
        properties.insert("phone".to_string(), phone.clone());
    }
    if let Some(company) = &contact.company {
        properties.insert("company".to_string(), company.clone());
    }
    if let Some(title) = &contact.title {
        properties.insert("jobtitle".to_string(), title.clone());
    }
    if let Some(addr) = &contact.address {
        if let Some(street) = &addr.street {
            properties.insert("address".to_string(), street.clone());
        }
        if let Some(city) = &addr.city {
            properties.insert("city".to_string(), city.clone());
        }
        if let Some(state) = &addr.state {
            properties.insert("state".to_string(), state.clone());
        }
        if let Some(zip) = &addr.postal_code {
            properties.insert("zip".to_string(), zip.clone());
        }
        if let Some(country) = &addr.country {
            properties.insert("country".to_string(), country.clone());
        }
    }

    HsCreateObject { properties }
}

fn hs_to_contact(obj: HsObject) -> Contact {
    let props = obj.properties;
    Contact {
        id: Some(obj.id),
        email: props.get("email").cloned(),
        first_name: props.get("firstname").cloned(),
        last_name: props.get("lastname").cloned(),
        phone: props.get("phone").cloned(),
        company: props.get("company").cloned(),
        title: props.get("jobtitle").cloned(),
        address: Some(Address {
            street: props.get("address").cloned(),
            city: props.get("city").cloned(),
            state: props.get("state").cloned(),
            postal_code: props.get("zip").cloned(),
            country: props.get("country").cloned(),
        }),
        custom_fields: HashMap::new(),
        created_at: obj.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: obj.updated_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn company_to_hs(company: &Company) -> HsCreateObject {
    let mut properties = HashMap::new();

    properties.insert("name".to_string(), company.name.clone());

    if let Some(domain) = &company.domain {
        properties.insert("domain".to_string(), domain.clone());
    }
    if let Some(industry) = &company.industry {
        properties.insert("industry".to_string(), industry.clone());
    }
    if let Some(phone) = &company.phone {
        properties.insert("phone".to_string(), phone.clone());
    }
    if let Some(website) = &company.website {
        properties.insert("website".to_string(), website.clone());
    }
    if let Some(count) = company.employee_count {
        properties.insert("numberofemployees".to_string(), count.to_string());
    }
    if let Some(revenue) = company.annual_revenue {
        properties.insert("annualrevenue".to_string(), revenue.to_string());
    }

    HsCreateObject { properties }
}

fn hs_to_company(obj: HsObject) -> Company {
    let props = obj.properties;
    Company {
        id: Some(obj.id),
        name: props.get("name").cloned().unwrap_or_default(),
        domain: props.get("domain").cloned(),
        industry: props.get("industry").cloned(),
        phone: props.get("phone").cloned(),
        website: props.get("website").cloned(),
        address: Some(Address {
            street: props.get("address").cloned(),
            city: props.get("city").cloned(),
            state: props.get("state").cloned(),
            postal_code: props.get("zip").cloned(),
            country: props.get("country").cloned(),
        }),
        employee_count: props.get("numberofemployees").and_then(|s| s.parse().ok()),
        annual_revenue: props.get("annualrevenue").and_then(|s| s.parse().ok()),
        custom_fields: HashMap::new(),
        created_at: obj.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: obj.updated_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn deal_to_hs(deal: &Deal) -> HsCreateObject {
    let mut properties = HashMap::new();

    properties.insert("dealname".to_string(), deal.name.clone());

    if let Some(amount) = deal.amount {
        properties.insert("amount".to_string(), amount.to_string());
    }
    if let Some(stage) = &deal.stage {
        properties.insert("dealstage".to_string(), stage.clone());
    }
    if let Some(pipeline) = &deal.pipeline {
        properties.insert("pipeline".to_string(), pipeline.clone());
    }
    if let Some(close_date) = deal.expected_close_date {
        properties.insert("closedate".to_string(), close_date.to_string());
    }
    if let Some(owner) = &deal.owner_id {
        properties.insert("hubspot_owner_id".to_string(), owner.clone());
    }

    HsCreateObject { properties }
}

fn hs_to_deal(obj: HsObject) -> Deal {
    let props = obj.properties;
    Deal {
        id: Some(obj.id),
        name: props.get("dealname").cloned().unwrap_or_default(),
        amount: props.get("amount").and_then(|s| s.parse().ok()),
        currency: None,
        stage: props.get("dealstage").cloned(),
        pipeline: props.get("pipeline").cloned(),
        probability: None,
        expected_close_date: props.get("closedate").and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        contact_id: None,
        company_id: None,
        owner_id: props.get("hubspot_owner_id").cloned(),
        custom_fields: HashMap::new(),
        created_at: obj.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: obj.updated_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn activity_to_hs(activity: &Activity) -> HsCreateObject {
    let mut properties = HashMap::new();

    if let Some(subject) = &activity.subject {
        properties.insert("hs_task_subject".to_string(), subject.clone());
    }
    if let Some(body) = &activity.body {
        properties.insert("hs_task_body".to_string(), body.clone());
    }
    if activity.completed {
        properties.insert("hs_task_status".to_string(), "COMPLETED".to_string());
    } else {
        properties.insert("hs_task_status".to_string(), "NOT_STARTED".to_string());
    }
    if let Some(due) = activity.due_date {
        properties.insert("hs_timestamp".to_string(), due.timestamp_millis().to_string());
    }

    HsCreateObject { properties }
}

fn hs_to_activity(obj: HsObject) -> Activity {
    let props = obj.properties;
    Activity {
        id: Some(obj.id),
        activity_type: ActivityType::Task,
        subject: props.get("hs_task_subject").cloned(),
        body: props.get("hs_task_body").cloned(),
        due_date: props.get("hs_timestamp").and_then(|s| s.parse::<i64>().ok()).and_then(|ts| chrono::DateTime::from_timestamp_millis(ts)),
        completed: props.get("hs_task_status").map(|s| s == "COMPLETED").unwrap_or(false),
        contact_id: None,
        company_id: None,
        deal_id: None,
        owner_id: None,
        created_at: obj.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

#[derive(Debug, Default, Deserialize)]
struct HubSpotError {
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct HsCreateObject {
    properties: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HsObject {
    id: String,
    #[serde(default)]
    properties: HashMap<String, String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HsListResult {
    results: Vec<HsObject>,
    paging: Option<HsPaging>,
    total: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HsSearchResult {
    results: Vec<HsObject>,
    total: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct HsPaging {
    next: Option<HsPageLink>,
}

#[derive(Debug, Clone, Deserialize)]
struct HsPageLink {
    after: String,
}
