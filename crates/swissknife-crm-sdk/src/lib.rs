mod error;

pub use error::{Error, Result};

#[cfg(feature = "salesforce")]
pub mod salesforce;

#[cfg(feature = "hubspot")]
pub mod hubspot;

#[cfg(feature = "pipedrive")]
pub mod pipedrive;

#[cfg(feature = "zoho")]
pub mod zoho;

#[cfg(feature = "zendesk")]
pub mod zendesk;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub address: Option<Address>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Contact {
    pub fn new() -> Self {
        Self {
            id: None,
            email: None,
            first_name: None,
            last_name: None,
            phone: None,
            company: None,
            title: None,
            address: None,
            custom_fields: HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_name(mut self, first: impl Into<String>, last: impl Into<String>) -> Self {
        self.first_name = Some(first.into());
        self.last_name = Some(last.into());
        self
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn with_company(mut self, company: impl Into<String>) -> Self {
        self.company = Some(company.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn full_name(&self) -> Option<String> {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => None,
        }
    }
}

impl Default for Contact {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Option<String>,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address: Option<Address>,
    pub employee_count: Option<i32>,
    pub annual_revenue: Option<f64>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Company {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            domain: None,
            industry: None,
            phone: None,
            website: None,
            address: None,
            employee_count: None,
            annual_revenue: None,
            custom_fields: HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn with_industry(mut self, industry: impl Into<String>) -> Self {
        self.industry = Some(industry.into());
        self
    }

    pub fn with_website(mut self, website: impl Into<String>) -> Self {
        self.website = Some(website.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: Option<String>,
    pub name: String,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub stage: Option<String>,
    pub pipeline: Option<String>,
    pub probability: Option<f64>,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub contact_id: Option<String>,
    pub company_id: Option<String>,
    pub owner_id: Option<String>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Deal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            amount: None,
            currency: None,
            stage: None,
            pipeline: None,
            probability: None,
            expected_close_date: None,
            contact_id: None,
            company_id: None,
            owner_id: None,
            custom_fields: HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn with_amount(mut self, amount: f64, currency: impl Into<String>) -> Self {
        self.amount = Some(amount);
        self.currency = Some(currency.into());
        self
    }

    pub fn with_stage(mut self, stage: impl Into<String>) -> Self {
        self.stage = Some(stage.into());
        self
    }

    pub fn with_contact(mut self, contact_id: impl Into<String>) -> Self {
        self.contact_id = Some(contact_id.into());
        self
    }

    pub fn with_company(mut self, company_id: impl Into<String>) -> Self {
        self.company_id = Some(company_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<String>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Lead {
    pub fn new() -> Self {
        Self {
            id: None,
            email: None,
            first_name: None,
            last_name: None,
            company: None,
            title: None,
            phone: None,
            source: None,
            status: None,
            owner_id: None,
            custom_fields: HashMap::new(),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_name(mut self, first: impl Into<String>, last: impl Into<String>) -> Self {
        self.first_name = Some(first.into());
        self.last_name = Some(last.into());
        self
    }

    pub fn with_company(mut self, company: impl Into<String>) -> Self {
        self.company = Some(company.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

impl Default for Lead {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Option<String>,
    pub activity_type: ActivityType,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub completed: bool,
    pub contact_id: Option<String>,
    pub company_id: Option<String>,
    pub deal_id: Option<String>,
    pub owner_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Call,
    Email,
    Meeting,
    Task,
    Note,
    Other,
}

impl Activity {
    pub fn call(subject: impl Into<String>) -> Self {
        Self {
            id: None,
            activity_type: ActivityType::Call,
            subject: Some(subject.into()),
            body: None,
            due_date: None,
            completed: false,
            contact_id: None,
            company_id: None,
            deal_id: None,
            owner_id: None,
            created_at: None,
        }
    }

    pub fn email(subject: impl Into<String>) -> Self {
        Self {
            id: None,
            activity_type: ActivityType::Email,
            subject: Some(subject.into()),
            body: None,
            due_date: None,
            completed: false,
            contact_id: None,
            company_id: None,
            deal_id: None,
            owner_id: None,
            created_at: None,
        }
    }

    pub fn meeting(subject: impl Into<String>) -> Self {
        Self {
            id: None,
            activity_type: ActivityType::Meeting,
            subject: Some(subject.into()),
            body: None,
            due_date: None,
            completed: false,
            contact_id: None,
            company_id: None,
            deal_id: None,
            owner_id: None,
            created_at: None,
        }
    }

    pub fn task(subject: impl Into<String>) -> Self {
        Self {
            id: None,
            activity_type: ActivityType::Task,
            subject: Some(subject.into()),
            body: None,
            due_date: None,
            completed: false,
            contact_id: None,
            company_id: None,
            deal_id: None,
            owner_id: None,
            created_at: None,
        }
    }

    pub fn note(body: impl Into<String>) -> Self {
        Self {
            id: None,
            activity_type: ActivityType::Note,
            subject: None,
            body: Some(body.into()),
            due_date: None,
            completed: false,
            contact_id: None,
            company_id: None,
            deal_id: None,
            owner_id: None,
            created_at: None,
        }
    }

    pub fn with_contact(mut self, contact_id: impl Into<String>) -> Self {
        self.contact_id = Some(contact_id.into());
        self
    }

    pub fn with_deal(mut self, deal_id: impl Into<String>) -> Self {
        self.deal_id = Some(deal_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

impl Address {
    pub fn new() -> Self {
        Self {
            street: None,
            city: None,
            state: None,
            postal_code: None,
            country: None,
        }
    }
}

impl Default for Address {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
    pub query: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub total: Option<u64>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[async_trait]
pub trait CrmProvider: Send + Sync {
    async fn create_contact(&self, contact: &Contact) -> Result<Contact>;
    async fn get_contact(&self, id: &str) -> Result<Contact>;
    async fn update_contact(&self, id: &str, contact: &Contact) -> Result<Contact>;
    async fn delete_contact(&self, id: &str) -> Result<()>;
    async fn list_contacts(&self, options: &ListOptions) -> Result<ListResult<Contact>>;

    async fn create_company(&self, company: &Company) -> Result<Company>;
    async fn get_company(&self, id: &str) -> Result<Company>;
    async fn update_company(&self, id: &str, company: &Company) -> Result<Company>;
    async fn delete_company(&self, id: &str) -> Result<()>;
    async fn list_companies(&self, options: &ListOptions) -> Result<ListResult<Company>>;

    async fn create_deal(&self, deal: &Deal) -> Result<Deal>;
    async fn get_deal(&self, id: &str) -> Result<Deal>;
    async fn update_deal(&self, id: &str, deal: &Deal) -> Result<Deal>;
    async fn delete_deal(&self, id: &str) -> Result<()>;
    async fn list_deals(&self, options: &ListOptions) -> Result<ListResult<Deal>>;

    async fn create_activity(&self, activity: &Activity) -> Result<Activity>;
    async fn list_activities(&self, options: &ListOptions) -> Result<ListResult<Activity>>;
}
