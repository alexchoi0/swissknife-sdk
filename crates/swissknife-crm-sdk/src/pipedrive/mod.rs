use crate::{Activity, ActivityType, Address, Company, Contact, CrmProvider, Deal, Error, ListOptions, ListResult, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.pipedrive.com/v1";

pub struct PipedriveClient {
    api_token: String,
    http: reqwest::Client,
}

impl PipedriveClient {
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            http: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        if path.contains('?') {
            format!("{}{}&api_token={}", API_BASE, path, self.api_token)
        } else {
            format!("{}{}?api_token={}", API_BASE, path, self.api_token)
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let response = self.http
            .get(self.url(path))
            .send()
            .await?;

        if response.status() == 404 {
            return Err(Error::NotFound("Resource not found".into()));
        }

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        let result: PdResponse<T> = response.json().await?;
        if !result.success {
            return Err(Error::Provider(result.error.unwrap_or_else(|| "Unknown error".into())));
        }

        result.data.ok_or_else(|| Error::NotFound("No data".into()))
    }

    async fn get_list<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<PdListResponse<T>> {
        let response = self.http
            .get(self.url(path))
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        let result: PdListResponse<T> = response.json().await?;
        if !result.success {
            return Err(Error::Provider(result.error.unwrap_or_else(|| "Unknown error".into())));
        }

        Ok(result)
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .post(self.url(path))
            .json(&body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        let result: PdResponse<T> = response.json().await?;
        if !result.success {
            return Err(Error::Provider(result.error.unwrap_or_else(|| "Unknown error".into())));
        }

        result.data.ok_or_else(|| Error::Provider("No data returned".into()))
    }

    async fn put<T: for<'de> Deserialize<'de>>(&self, path: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .put(self.url(path))
            .json(&body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        let result: PdResponse<T> = response.json().await?;
        if !result.success {
            return Err(Error::Provider(result.error.unwrap_or_else(|| "Unknown error".into())));
        }

        result.data.ok_or_else(|| Error::Provider("No data returned".into()))
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let response = self.http
            .delete(self.url(path))
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::RateLimited);
        }

        let result: PdResponse<serde_json::Value> = response.json().await?;
        if !result.success {
            return Err(Error::Provider(result.error.unwrap_or_else(|| "Unknown error".into())));
        }

        Ok(())
    }

    pub async fn search_persons(&self, query: &str) -> Result<Vec<Contact>> {
        let result: PdListResponse<PdSearchResult> = self.get_list(&format!("/persons/search?term={}", urlencoding::encode(query))).await?;
        let contacts = result.data.unwrap_or_default()
            .into_iter()
            .filter_map(|r| r.item)
            .map(|p| pd_person_to_contact(p))
            .collect();
        Ok(contacts)
    }

    pub async fn search_organizations(&self, query: &str) -> Result<Vec<Company>> {
        let result: PdListResponse<PdOrgSearchResult> = self.get_list(&format!("/organizations/search?term={}", urlencoding::encode(query))).await?;
        let companies = result.data.unwrap_or_default()
            .into_iter()
            .filter_map(|r| r.item)
            .map(|o| pd_org_to_company(o))
            .collect();
        Ok(companies)
    }
}

#[async_trait]
impl CrmProvider for PipedriveClient {
    async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        let pd_person = contact_to_pd_person(contact);
        let result: PdPerson = self.post("/persons", &pd_person).await?;
        Ok(pd_person_to_contact(result))
    }

    async fn get_contact(&self, id: &str) -> Result<Contact> {
        let result: PdPerson = self.get(&format!("/persons/{}", id)).await?;
        Ok(pd_person_to_contact(result))
    }

    async fn update_contact(&self, id: &str, contact: &Contact) -> Result<Contact> {
        let pd_person = contact_to_pd_person(contact);
        let result: PdPerson = self.put(&format!("/persons/{}", id), &pd_person).await?;
        Ok(pd_person_to_contact(result))
    }

    async fn delete_contact(&self, id: &str) -> Result<()> {
        self.delete(&format!("/persons/{}", id)).await
    }

    async fn list_contacts(&self, options: &ListOptions) -> Result<ListResult<Contact>> {
        let limit = options.limit.unwrap_or(100);
        let start = options.offset.unwrap_or(0);

        let result: PdListResponse<PdPerson> = self.get_list(&format!("/persons?limit={}&start={}", limit, start)).await?;
        let contacts: Vec<Contact> = result.data.unwrap_or_default()
            .into_iter()
            .map(|p| pd_person_to_contact(p))
            .collect();

        let has_more = result.additional_data
            .and_then(|a| a.pagination)
            .map(|p| p.more_items_in_collection)
            .unwrap_or(false);

        Ok(ListResult {
            items: contacts,
            total: None,
            next_cursor: None,
            has_more,
        })
    }

    async fn create_company(&self, company: &Company) -> Result<Company> {
        let pd_org = company_to_pd_org(company);
        let result: PdOrganization = self.post("/organizations", &pd_org).await?;
        Ok(pd_org_to_company(result))
    }

    async fn get_company(&self, id: &str) -> Result<Company> {
        let result: PdOrganization = self.get(&format!("/organizations/{}", id)).await?;
        Ok(pd_org_to_company(result))
    }

    async fn update_company(&self, id: &str, company: &Company) -> Result<Company> {
        let pd_org = company_to_pd_org(company);
        let result: PdOrganization = self.put(&format!("/organizations/{}", id), &pd_org).await?;
        Ok(pd_org_to_company(result))
    }

    async fn delete_company(&self, id: &str) -> Result<()> {
        self.delete(&format!("/organizations/{}", id)).await
    }

    async fn list_companies(&self, options: &ListOptions) -> Result<ListResult<Company>> {
        let limit = options.limit.unwrap_or(100);
        let start = options.offset.unwrap_or(0);

        let result: PdListResponse<PdOrganization> = self.get_list(&format!("/organizations?limit={}&start={}", limit, start)).await?;
        let companies: Vec<Company> = result.data.unwrap_or_default()
            .into_iter()
            .map(|o| pd_org_to_company(o))
            .collect();

        let has_more = result.additional_data
            .and_then(|a| a.pagination)
            .map(|p| p.more_items_in_collection)
            .unwrap_or(false);

        Ok(ListResult {
            items: companies,
            total: None,
            next_cursor: None,
            has_more,
        })
    }

    async fn create_deal(&self, deal: &Deal) -> Result<Deal> {
        let pd_deal = deal_to_pd_deal(deal);
        let result: PdDeal = self.post("/deals", &pd_deal).await?;
        Ok(pd_deal_to_deal(result))
    }

    async fn get_deal(&self, id: &str) -> Result<Deal> {
        let result: PdDeal = self.get(&format!("/deals/{}", id)).await?;
        Ok(pd_deal_to_deal(result))
    }

    async fn update_deal(&self, id: &str, deal: &Deal) -> Result<Deal> {
        let pd_deal = deal_to_pd_deal(deal);
        let result: PdDeal = self.put(&format!("/deals/{}", id), &pd_deal).await?;
        Ok(pd_deal_to_deal(result))
    }

    async fn delete_deal(&self, id: &str) -> Result<()> {
        self.delete(&format!("/deals/{}", id)).await
    }

    async fn list_deals(&self, options: &ListOptions) -> Result<ListResult<Deal>> {
        let limit = options.limit.unwrap_or(100);
        let start = options.offset.unwrap_or(0);

        let result: PdListResponse<PdDeal> = self.get_list(&format!("/deals?limit={}&start={}", limit, start)).await?;
        let deals: Vec<Deal> = result.data.unwrap_or_default()
            .into_iter()
            .map(|d| pd_deal_to_deal(d))
            .collect();

        let has_more = result.additional_data
            .and_then(|a| a.pagination)
            .map(|p| p.more_items_in_collection)
            .unwrap_or(false);

        Ok(ListResult {
            items: deals,
            total: None,
            next_cursor: None,
            has_more,
        })
    }

    async fn create_activity(&self, activity: &Activity) -> Result<Activity> {
        let pd_activity = activity_to_pd_activity(activity);
        let result: PdActivity = self.post("/activities", &pd_activity).await?;
        Ok(pd_activity_to_activity(result))
    }

    async fn list_activities(&self, options: &ListOptions) -> Result<ListResult<Activity>> {
        let limit = options.limit.unwrap_or(100);
        let start = options.offset.unwrap_or(0);

        let result: PdListResponse<PdActivity> = self.get_list(&format!("/activities?limit={}&start={}", limit, start)).await?;
        let activities: Vec<Activity> = result.data.unwrap_or_default()
            .into_iter()
            .map(|a| pd_activity_to_activity(a))
            .collect();

        let has_more = result.additional_data
            .and_then(|a| a.pagination)
            .map(|p| p.more_items_in_collection)
            .unwrap_or(false);

        Ok(ListResult {
            items: activities,
            total: None,
            next_cursor: None,
            has_more,
        })
    }
}

fn contact_to_pd_person(contact: &Contact) -> PdPersonCreate {
    let mut emails = Vec::new();
    if let Some(email) = &contact.email {
        emails.push(PdEmail { value: email.clone(), primary: true });
    }

    let mut phones = Vec::new();
    if let Some(phone) = &contact.phone {
        phones.push(PdPhone { value: phone.clone(), primary: true });
    }

    PdPersonCreate {
        name: contact.full_name(),
        email: if emails.is_empty() { None } else { Some(emails) },
        phone: if phones.is_empty() { None } else { Some(phones) },
        org_id: None,
    }
}

fn pd_person_to_contact(person: PdPerson) -> Contact {
    let email = person.email.and_then(|emails| emails.into_iter().find(|e| e.primary).or_else(|| None).map(|e| e.value));
    let phone = person.phone.and_then(|phones| phones.into_iter().find(|p| p.primary).or_else(|| None).map(|p| p.value));

    let (first_name, last_name) = person.name.map(|n| {
        let parts: Vec<&str> = n.splitn(2, ' ').collect();
        (parts.first().map(|s| s.to_string()), parts.get(1).map(|s| s.to_string()))
    }).unwrap_or((None, None));

    Contact {
        id: Some(person.id.to_string()),
        email,
        first_name,
        last_name,
        phone,
        company: person.org_name,
        title: None,
        address: None,
        custom_fields: HashMap::new(),
        created_at: person.add_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: person.update_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn company_to_pd_org(company: &Company) -> PdOrgCreate {
    PdOrgCreate {
        name: company.name.clone(),
        address: company.address.as_ref().and_then(|a| a.street.clone()),
    }
}

fn pd_org_to_company(org: PdOrganization) -> Company {
    Company {
        id: Some(org.id.to_string()),
        name: org.name.unwrap_or_default(),
        domain: None,
        industry: None,
        phone: None,
        website: None,
        address: org.address.map(|a| Address {
            street: Some(a),
            city: None,
            state: None,
            postal_code: None,
            country: None,
        }),
        employee_count: org.people_count,
        annual_revenue: None,
        custom_fields: HashMap::new(),
        created_at: org.add_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: org.update_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn deal_to_pd_deal(deal: &Deal) -> PdDealCreate {
    PdDealCreate {
        title: deal.name.clone(),
        value: deal.amount,
        currency: deal.currency.clone(),
        stage_id: deal.stage.as_ref().and_then(|s| s.parse().ok()),
        person_id: deal.contact_id.as_ref().and_then(|s| s.parse().ok()),
        org_id: deal.company_id.as_ref().and_then(|s| s.parse().ok()),
        expected_close_date: deal.expected_close_date.map(|d| d.to_string()),
    }
}

fn pd_deal_to_deal(deal: PdDeal) -> Deal {
    Deal {
        id: Some(deal.id.to_string()),
        name: deal.title.unwrap_or_default(),
        amount: deal.value,
        currency: deal.currency,
        stage: deal.stage_id.map(|s| s.to_string()),
        pipeline: deal.pipeline_id.map(|p| p.to_string()),
        probability: deal.probability,
        expected_close_date: deal.expected_close_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        contact_id: deal.person_id.map(|p| p.to_string()),
        company_id: deal.org_id.map(|o| o.to_string()),
        owner_id: deal.user_id.map(|u| u.to_string()),
        custom_fields: HashMap::new(),
        created_at: deal.add_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
        updated_at: deal.update_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

fn activity_to_pd_activity(activity: &Activity) -> PdActivityCreate {
    let activity_type = match activity.activity_type {
        ActivityType::Call => "call",
        ActivityType::Email => "email",
        ActivityType::Meeting => "meeting",
        ActivityType::Task => "task",
        ActivityType::Note => "task",
        ActivityType::Other => "task",
    };

    PdActivityCreate {
        subject: activity.subject.clone(),
        activity_type: activity_type.to_string(),
        note: activity.body.clone(),
        done: if activity.completed { Some(1) } else { Some(0) },
        due_date: activity.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
        due_time: activity.due_date.map(|d| d.format("%H:%M").to_string()),
        person_id: activity.contact_id.as_ref().and_then(|s| s.parse().ok()),
        deal_id: activity.deal_id.as_ref().and_then(|s| s.parse().ok()),
        org_id: activity.company_id.as_ref().and_then(|s| s.parse().ok()),
    }
}

fn pd_activity_to_activity(activity: PdActivity) -> Activity {
    let activity_type = match activity.activity_type.as_deref() {
        Some("call") => ActivityType::Call,
        Some("email") => ActivityType::Email,
        Some("meeting") => ActivityType::Meeting,
        _ => ActivityType::Task,
    };

    Activity {
        id: Some(activity.id.to_string()),
        activity_type,
        subject: activity.subject,
        body: activity.note,
        due_date: activity.due_date.and_then(|d| {
            let time = activity.due_time.unwrap_or_else(|| "00:00".to_string());
            chrono::NaiveDateTime::parse_from_str(&format!("{} {}", d, time), "%Y-%m-%d %H:%M").ok()
        }).map(|dt| dt.and_utc()),
        completed: activity.done == Some(true),
        contact_id: activity.person_id.map(|p| p.to_string()),
        company_id: activity.org_id.map(|o| o.to_string()),
        deal_id: activity.deal_id.map(|d| d.to_string()),
        owner_id: activity.user_id.map(|u| u.to_string()),
        created_at: activity.add_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc)),
    }
}

#[derive(Debug, Deserialize)]
struct PdResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PdListResponse<T> {
    success: bool,
    data: Option<Vec<T>>,
    error: Option<String>,
    additional_data: Option<PdAdditionalData>,
}

#[derive(Debug, Deserialize)]
struct PdAdditionalData {
    pagination: Option<PdPagination>,
}

#[derive(Debug, Deserialize)]
struct PdPagination {
    more_items_in_collection: bool,
}

#[derive(Debug, Deserialize)]
struct PdSearchResult {
    item: Option<PdPerson>,
}

#[derive(Debug, Deserialize)]
struct PdOrgSearchResult {
    item: Option<PdOrganization>,
}

#[derive(Debug, Serialize)]
struct PdPersonCreate {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<Vec<PdEmail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<Vec<PdPhone>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    org_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PdEmail {
    value: String,
    primary: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PdPhone {
    value: String,
    primary: bool,
}

#[derive(Debug, Deserialize)]
struct PdPerson {
    id: i64,
    name: Option<String>,
    email: Option<Vec<PdEmail>>,
    phone: Option<Vec<PdPhone>>,
    org_name: Option<String>,
    add_time: Option<String>,
    update_time: Option<String>,
}

#[derive(Debug, Serialize)]
struct PdOrgCreate {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PdOrganization {
    id: i64,
    name: Option<String>,
    address: Option<String>,
    people_count: Option<i32>,
    add_time: Option<String>,
    update_time: Option<String>,
}

#[derive(Debug, Serialize)]
struct PdDealCreate {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    person_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    org_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_close_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PdDeal {
    id: i64,
    title: Option<String>,
    value: Option<f64>,
    currency: Option<String>,
    stage_id: Option<i64>,
    pipeline_id: Option<i64>,
    probability: Option<f64>,
    expected_close_date: Option<String>,
    person_id: Option<i64>,
    org_id: Option<i64>,
    user_id: Option<i64>,
    add_time: Option<String>,
    update_time: Option<String>,
}

#[derive(Debug, Serialize)]
struct PdActivityCreate {
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    #[serde(rename = "type")]
    activity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    done: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    person_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deal_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    org_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PdActivity {
    id: i64,
    subject: Option<String>,
    #[serde(rename = "type")]
    activity_type: Option<String>,
    note: Option<String>,
    done: Option<bool>,
    due_date: Option<String>,
    due_time: Option<String>,
    person_id: Option<i64>,
    deal_id: Option<i64>,
    org_id: Option<i64>,
    user_id: Option<i64>,
    add_time: Option<String>,
}
