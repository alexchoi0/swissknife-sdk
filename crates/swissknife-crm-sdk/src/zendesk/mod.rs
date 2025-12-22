use crate::{Activity, ActivityType, Address, Company, Contact, CrmProvider, Deal, Error, ListOptions, ListResult, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.getbase.com/v2";

pub struct ZendeskSellClient {
    access_token: String,
    http: reqwest::Client,
}

impl ZendeskSellClient {
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
            .header("Accept", "application/json")
            .send()
            .await?;

        if response.status() == 404 {
            return Err(Error::NotFound("Resource not found".into()));
        }

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: ZendeskErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.errors.first().map(|e| e.error.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .post(format!("{}{}", API_BASE, path))
            .bearer_auth(&self.access_token)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: ZendeskErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.errors.first().map(|e| e.error.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    async fn put<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .put(format!("{}{}", API_BASE, path))
            .bearer_auth(&self.access_token)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() {
            let error: ZendeskErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.errors.first().map(|e| e.error.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(response.json().await?)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let response = self.http
            .delete(format!("{}{}", API_BASE, path))
            .bearer_auth(&self.access_token)
            .header("Accept", "application/json")
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        if !response.status().is_success() && response.status() != 204 {
            let error: ZendeskErrorResponse = response.json().await.unwrap_or_default();
            let msg = error.errors.first().map(|e| e.error.message.clone()).unwrap_or_else(|| "Unknown error".into());
            return Err(Error::Provider(msg));
        }

        Ok(())
    }

    pub async fn search_contacts(&self, query: &str) -> Result<Vec<Contact>> {
        let result: ZdListResponse<ZdContact> = self.get(&format!("/contacts?name={}", urlencoding::encode(query))).await?;
        Ok(result.items.into_iter().map(|i| zd_to_contact(i.data)).collect())
    }

    pub async fn search_leads(&self, query: &str) -> Result<Vec<Contact>> {
        let result: ZdListResponse<ZdLead> = self.get(&format!("/leads?name={}", urlencoding::encode(query))).await?;
        Ok(result.items.into_iter().map(|i| zd_lead_to_contact(i.data)).collect())
    }
}

#[async_trait]
impl CrmProvider for ZendeskSellClient {
    async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let zd_contact = contact_to_zd(contact);
        let body = ZdCreateRequest { data: zd_contact };
        let result: ZdItemResponse<ZdContact> = self.post("/contacts", &body).await?;
        Ok(zd_to_contact(result.data))
    }

    async fn get_contact(&self, id: &str) -> Result<Contact> {
        let result: ZdItemResponse<ZdContact> = self.get(&format!("/contacts/{}", id)).await?;
        Ok(zd_to_contact(result.data))
    }

    async fn update_contact(&self, id: &str, contact: &Contact) -> Result<Contact> {
        let zd_contact = contact_to_zd(contact);
        let body = ZdCreateRequest { data: zd_contact };
        let result: ZdItemResponse<ZdContact> = self.put(&format!("/contacts/{}", id), &body).await?;
        Ok(zd_to_contact(result.data))
    }

    async fn delete_contact(&self, id: &str) -> Result<()> {
        self.delete(&format!("/contacts/{}", id)).await
    }

    async fn list_contacts(&self, options: &ListOptions) -> Result<ListResult<Contact>> {
        let per_page = options.limit.unwrap_or(100);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZdListResponse<ZdContact> = self.get(&format!("/contacts?per_page={}&page={}", per_page, page)).await?;
        let contacts: Vec<Contact> = result.items.into_iter().map(|i| zd_to_contact(i.data)).collect();

        Ok(ListResult {
            items: contacts,
            total: result.meta.as_ref().and_then(|m| m.count),
            next_cursor: None,
            has_more: result.meta.as_ref().map(|m| m.count.unwrap_or(0) > (page * per_page) as u64).unwrap_or(false),
        })
    }

    async fn create_company(&self, company: &Company) -> Result<Company> {
        let zd_contact = ZdContact {
            id: None,
            is_organization: Some(true),
            name: Some(company.name.clone()),
            email: None,
            first_name: None,
            last_name: None,
            phone: company.phone.clone(),
            title: None,
            industry: company.industry.clone(),
            website: company.website.clone(),
            address: company.address.as_ref().and_then(|a| {
                let parts: Vec<String> = [
                    a.street.clone(),
                    a.city.clone(),
                    a.state.clone(),
                    a.postal_code.clone(),
                    a.country.clone(),
                ].into_iter().flatten().collect();
                if parts.is_empty() { None } else { Some(parts.join(", ")) }
            }),
            created_at: None,
            updated_at: None,
        };
        let body = ZdCreateRequest { data: zd_contact };
        let result: ZdItemResponse<ZdContact> = self.post("/contacts", &body).await?;
        Ok(zd_contact_to_company(result.data))
    }

    async fn get_company(&self, id: &str) -> Result<Company> {
        let result: ZdItemResponse<ZdContact> = self.get(&format!("/contacts/{}", id)).await?;
        Ok(zd_contact_to_company(result.data))
    }

    async fn update_company(&self, id: &str, company: &Company) -> Result<Company> {
        let zd_contact = ZdContact {
            id: None,
            is_organization: Some(true),
            name: Some(company.name.clone()),
            email: None,
            first_name: None,
            last_name: None,
            phone: company.phone.clone(),
            title: None,
            industry: company.industry.clone(),
            website: company.website.clone(),
            address: None,
            created_at: None,
            updated_at: None,
        };
        let body = ZdCreateRequest { data: zd_contact };
        let result: ZdItemResponse<ZdContact> = self.put(&format!("/contacts/{}", id), &body).await?;
        Ok(zd_contact_to_company(result.data))
    }

    async fn delete_company(&self, id: &str) -> Result<()> {
        self.delete(&format!("/contacts/{}", id)).await
    }

    async fn list_companies(&self, options: &ListOptions) -> Result<ListResult<Company>> {
        let per_page = options.limit.unwrap_or(100);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZdListResponse<ZdContact> = self.get(&format!("/contacts?is_organization=true&per_page={}&page={}", per_page, page)).await?;
        let companies: Vec<Company> = result.items.into_iter().map(|i| zd_contact_to_company(i.data)).collect();

        Ok(ListResult {
            items: companies,
            total: result.meta.as_ref().and_then(|m| m.count),
            next_cursor: None,
            has_more: result.meta.as_ref().map(|m| m.count.unwrap_or(0) > (page * per_page) as u64).unwrap_or(false),
        })
    }

    async fn create_deal(&self, deal: &Deal) -> Result<Deal> {
        let zd_deal = deal_to_zd(deal);
        let body = ZdCreateRequest { data: zd_deal };
        let result: ZdItemResponse<ZdDeal> = self.post("/deals", &body).await?;
        Ok(zd_to_deal(result.data))
    }

    async fn get_deal(&self, id: &str) -> Result<Deal> {
        let result: ZdItemResponse<ZdDeal> = self.get(&format!("/deals/{}", id)).await?;
        Ok(zd_to_deal(result.data))
    }

    async fn update_deal(&self, id: &str, deal: &Deal) -> Result<Deal> {
        let zd_deal = deal_to_zd(deal);
        let body = ZdCreateRequest { data: zd_deal };
        let result: ZdItemResponse<ZdDeal> = self.put(&format!("/deals/{}", id), &body).await?;
        Ok(zd_to_deal(result.data))
    }

    async fn delete_deal(&self, id: &str) -> Result<()> {
        self.delete(&format!("/deals/{}", id)).await
    }

    async fn list_deals(&self, options: &ListOptions) -> Result<ListResult<Deal>> {
        let per_page = options.limit.unwrap_or(100);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZdListResponse<ZdDeal> = self.get(&format!("/deals?per_page={}&page={}", per_page, page)).await?;
        let deals: Vec<Deal> = result.items.into_iter().map(|i| zd_to_deal(i.data)).collect();

        Ok(ListResult {
            items: deals,
            total: result.meta.as_ref().and_then(|m| m.count),
            next_cursor: None,
            has_more: result.meta.as_ref().map(|m| m.count.unwrap_or(0) > (page * per_page) as u64).unwrap_or(false),
        })
    }

    async fn create_activity(&self, activity: &Activity) -> Result<Activity> {
        let zd_task = activity_to_zd(activity);
        let body = ZdCreateRequest { data: zd_task };
        let result: ZdItemResponse<ZdTask> = self.post("/tasks", &body).await?;
        Ok(zd_to_activity(result.data))
    }

    async fn list_activities(&self, options: &ListOptions) -> Result<ListResult<Activity>> {
        let per_page = options.limit.unwrap_or(100);
        let page = (options.offset.unwrap_or(0) / per_page) + 1;

        let result: ZdListResponse<ZdTask> = self.get(&format!("/tasks?per_page={}&page={}", per_page, page)).await?;
        let activities: Vec<Activity> = result.items.into_iter().map(|i| zd_to_activity(i.data)).collect();

        Ok(ListResult {
            items: activities,
            total: result.meta.as_ref().and_then(|m| m.count),
            next_cursor: None,
            has_more: result.meta.as_ref().map(|m| m.count.unwrap_or(0) > (page * per_page) as u64).unwrap_or(false),
        })
    }
}

fn contact_to_zd(contact: &Contact) -> ZdContact {
    ZdContact {
        id: None,
        is_organization: Some(false),
        name: contact.full_name(),
        email: contact.email.clone(),
        first_name: contact.first_name.clone(),
        last_name: contact.last_name.clone(),
        phone: contact.phone.clone(),
        title: contact.title.clone(),
        industry: None,
        website: None,
        address: contact.address.as_ref().and_then(|a| {
            let parts: Vec<String> = [
                a.street.clone(),
                a.city.clone(),
                a.state.clone(),
                a.postal_code.clone(),
                a.country.clone(),
            ].into_iter().flatten().collect();
            if parts.is_empty() { None } else { Some(parts.join(", ")) }
        }),
        created_at: None,
        updated_at: None,
    }
}

fn zd_to_contact(contact: ZdContact) -> Contact {
    Contact {
        id: contact.id.map(|i| i.to_string()),
        email: contact.email,
        first_name: contact.first_name,
        last_name: contact.last_name,
        phone: contact.phone,
        company: None,
        title: contact.title,
        address: contact.address.map(|a| Address {
            street: Some(a),
            city: None,
            state: None,
            postal_code: None,
            country: None,
        }),
        custom_fields: HashMap::new(),
        created_at: contact.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: contact.updated_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn zd_contact_to_company(contact: ZdContact) -> Company {
    Company {
        id: contact.id.map(|i| i.to_string()),
        name: contact.name.unwrap_or_default(),
        domain: None,
        industry: contact.industry,
        phone: contact.phone,
        website: contact.website,
        address: contact.address.map(|a| Address {
            street: Some(a),
            city: None,
            state: None,
            postal_code: None,
            country: None,
        }),
        employee_count: None,
        annual_revenue: None,
        custom_fields: HashMap::new(),
        created_at: contact.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: contact.updated_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn zd_lead_to_contact(lead: ZdLead) -> Contact {
    Contact {
        id: lead.id.map(|i| i.to_string()),
        email: lead.email,
        first_name: lead.first_name,
        last_name: lead.last_name,
        phone: lead.phone,
        company: lead.organization_name,
        title: lead.title,
        address: None,
        custom_fields: HashMap::new(),
        created_at: lead.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: lead.updated_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn deal_to_zd(deal: &Deal) -> ZdDeal {
    ZdDeal {
        id: None,
        name: deal.name.clone(),
        value: deal.amount,
        currency: deal.currency.clone(),
        stage_id: deal.stage.as_ref().and_then(|s| s.parse().ok()),
        contact_id: deal.contact_id.as_ref().and_then(|s| s.parse().ok()),
        estimated_close_date: deal.expected_close_date.map(|d| d.to_string()),
        owner_id: deal.owner_id.as_ref().and_then(|s| s.parse().ok()),
        created_at: None,
        updated_at: None,
    }
}

fn zd_to_deal(deal: ZdDeal) -> Deal {
    Deal {
        id: deal.id.map(|i| i.to_string()),
        name: deal.name,
        amount: deal.value,
        currency: deal.currency,
        stage: deal.stage_id.map(|s| s.to_string()),
        pipeline: None,
        probability: None,
        expected_close_date: deal.estimated_close_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        contact_id: deal.contact_id.map(|c| c.to_string()),
        company_id: None,
        owner_id: deal.owner_id.map(|o| o.to_string()),
        custom_fields: HashMap::new(),
        created_at: deal.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: deal.updated_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn activity_to_zd(activity: &Activity) -> ZdTask {
    ZdTask {
        id: None,
        content: activity.subject.clone().or_else(|| activity.body.clone()).unwrap_or_default(),
        due_date: activity.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
        completed: activity.completed,
        resource_type: activity.contact_id.as_ref().map(|_| "contact".to_string()),
        resource_id: activity.contact_id.as_ref().and_then(|s| s.parse().ok()),
        owner_id: activity.owner_id.as_ref().and_then(|s| s.parse().ok()),
        created_at: None,
    }
}

fn zd_to_activity(task: ZdTask) -> Activity {
    Activity {
        id: task.id.map(|i| i.to_string()),
        activity_type: ActivityType::Task,
        subject: Some(task.content.clone()),
        body: Some(task.content),
        due_date: task.due_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()).map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()),
        completed: task.completed,
        contact_id: task.resource_id.filter(|_| task.resource_type.as_deref() == Some("contact")).map(|i| i.to_string()),
        company_id: None,
        deal_id: task.resource_id.filter(|_| task.resource_type.as_deref() == Some("deal")).map(|i| i.to_string()),
        owner_id: task.owner_id.map(|o| o.to_string()),
        created_at: task.created_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

#[derive(Debug, Default, Deserialize)]
struct ZendeskErrorResponse {
    #[serde(default)]
    errors: Vec<ZdErrorItem>,
}

#[derive(Debug, Deserialize)]
struct ZdErrorItem {
    error: ZdError,
}

#[derive(Debug, Deserialize)]
struct ZdError {
    message: String,
}

#[derive(Debug, Serialize)]
struct ZdCreateRequest<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct ZdItemResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct ZdListResponse<T> {
    items: Vec<ZdListItem<T>>,
    meta: Option<ZdMeta>,
}

#[derive(Debug, Deserialize)]
struct ZdListItem<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct ZdMeta {
    count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZdContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_organization: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
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
    industry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZdLead {
    id: Option<i64>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
    title: Option<String>,
    organization_name: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZdDeal {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_close_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZdTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(default)]
    completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
}
