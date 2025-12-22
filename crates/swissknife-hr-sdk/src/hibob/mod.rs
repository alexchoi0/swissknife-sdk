use crate::{
    Address, CreateEmployee, CreateTimeOffRequest, Department, Employee, EmploymentStatus,
    EmploymentType, Error, HrProvider, ListOptions, ListResult, Location, PayPeriod, PayRate,
    Result, TimeOffBalance, TimeOffRequest, TimeOffStatus, TimeOffType, TimeOffUnit, UpdateEmployee,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.hibob.com/v1";

pub struct HiBobClient {
    api_token: String,
    http: reqwest::Client,
}

impl HiBobClient {
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_service_user(service_id: impl Into<String>, service_token: impl Into<String>) -> Self {
        let credentials = format!("{}:{}", service_id.into(), service_token.into());
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, credentials);
        Self {
            api_token: format!("Basic {}", encoded),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", &self.api_token)
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(path.to_string()));
        }
        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API token".to_string()));
        }
        if resp.status() == 429 {
            return Err(Error::RateLimited);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(resp.json().await?)
    }

    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", &self.api_token)
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API token".to_string()));
        }
        if resp.status() == 429 {
            return Err(Error::RateLimited);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(resp.json().await?)
    }

    async fn put<B: Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .put(&url)
            .header("Authorization", &self.api_token)
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API token".to_string()));
        }
        if resp.status() == 429 {
            return Err(Error::RateLimited);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(())
    }

    pub async fn get_my_profile(&self) -> Result<Employee> {
        let emp: BobEmployee = self.get("/my/profile").await?;
        Ok(emp.into())
    }

    pub async fn search_employees(&self, query: &str) -> Result<Vec<Employee>> {
        #[derive(Deserialize)]
        struct SearchResponse {
            employees: Vec<BobEmployee>,
        }

        let resp: SearchResponse = self.get(&format!("/people?search={}", query)).await?;
        Ok(resp.employees.into_iter().map(|e| e.into()).collect())
    }

    pub async fn get_company_fields(&self) -> Result<Vec<BobField>> {
        #[derive(Deserialize)]
        struct FieldsResponse {
            fields: Vec<BobField>,
        }

        let resp: FieldsResponse = self.get("/company/people/fields").await?;
        Ok(resp.fields)
    }

    pub async fn get_metadata(&self) -> Result<BobMetadata> {
        self.get("/metadata").await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobEmployee {
    id: String,
    display_name: Option<String>,
    first_name: Option<String>,
    surname: Option<String>,
    email: Option<String>,
    personal_email: Option<String>,
    work_phone: Option<String>,
    mobile_phone: Option<String>,
    work: Option<BobWork>,
    home: Option<BobHome>,
    avatar_url: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobWork {
    title: Option<String>,
    department: Option<String>,
    department_id: Option<String>,
    site: Option<String>,
    site_id: Option<String>,
    reports_to: Option<BobReportsTo>,
    start_date: Option<String>,
    termination_date: Option<String>,
    employee_id_in_company: Option<String>,
    employment_status: Option<String>,
    employment_type: Option<String>,
    custom: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobReportsTo {
    id: Option<String>,
    display_name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobHome {
    address: Option<BobAddress>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobAddress {
    street_line_1: Option<String>,
    street_line_2: Option<String>,
    city: Option<String>,
    state_province: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

impl From<BobEmployee> for Employee {
    fn from(e: BobEmployee) -> Self {
        let work = e.work.as_ref();

        let employment_status = work.and_then(|w| w.employment_status.as_ref()).map(|s| match s.to_lowercase().as_str() {
            "active" => EmploymentStatus::Active,
            "inactive" => EmploymentStatus::Inactive,
            "terminated" => EmploymentStatus::Terminated,
            "onboarding" => EmploymentStatus::Onboarding,
            "on leave" | "leave" => EmploymentStatus::OnLeave,
            _ => EmploymentStatus::Other,
        });

        let employment_type = work.and_then(|w| w.employment_type.as_ref()).map(|t| match t.to_lowercase().as_str() {
            "full-time" | "full time" | "permanent" => EmploymentType::FullTime,
            "part-time" | "part time" => EmploymentType::PartTime,
            "contractor" | "contract" => EmploymentType::Contract,
            "intern" | "internship" => EmploymentType::Intern,
            "temporary" | "temp" => EmploymentType::Temporary,
            "freelancer" | "freelance" => EmploymentType::Freelance,
            _ => EmploymentType::Other,
        });

        Employee {
            id: e.id,
            employee_number: work.and_then(|w| w.employee_id_in_company.clone()),
            first_name: e.first_name.unwrap_or_default(),
            last_name: e.surname.unwrap_or_default(),
            email: e.email.clone(),
            work_email: e.email,
            personal_email: e.personal_email,
            phone: e.work_phone,
            mobile_phone: e.mobile_phone,
            job_title: work.and_then(|w| w.title.clone()),
            department: work.and_then(|w| w.department.clone()),
            department_id: work.and_then(|w| w.department_id.clone()),
            division: None,
            location: work.and_then(|w| w.site.clone()),
            manager_id: work.and_then(|w| w.reports_to.as_ref().and_then(|r| r.id.clone())),
            hire_date: work.and_then(|w| w.start_date.as_ref().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())),
            termination_date: work.and_then(|w| w.termination_date.as_ref().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())),
            employment_status,
            employment_type,
            pay_rate: None,
            avatar_url: e.avatar_url,
            created_at: e.created_at.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc))),
            updated_at: e.updated_at.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc))),
            extra: e.extra,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BobField {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub field_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BobMetadata {
    pub lists: Option<Vec<BobList>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BobList {
    pub id: String,
    pub name: String,
    pub items: Option<Vec<BobListItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BobListItem {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobTimeOff {
    id: String,
    employee_id: String,
    policy_type: Option<String>,
    policy_type_display_name: Option<String>,
    start_date: String,
    end_date: String,
    status: Option<String>,
    requested_days: Option<f64>,
    requested_hours: Option<f64>,
    description: Option<String>,
    approver_id: Option<String>,
    created_at: Option<String>,
}

impl From<BobTimeOff> for TimeOffRequest {
    fn from(t: BobTimeOff) -> Self {
        let status = t.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "approved" => TimeOffStatus::Approved,
            "declined" | "rejected" => TimeOffStatus::Denied,
            "cancelled" | "canceled" => TimeOffStatus::Cancelled,
            _ => TimeOffStatus::Pending,
        }).unwrap_or(TimeOffStatus::Pending);

        let request_type = t.policy_type.as_deref().map(|s| match s.to_lowercase().as_str() {
            s if s.contains("holiday") || s.contains("vacation") || s.contains("annual") => TimeOffType::Vacation,
            s if s.contains("sick") => TimeOffType::Sick,
            s if s.contains("personal") => TimeOffType::Personal,
            s if s.contains("parental") || s.contains("maternity") || s.contains("paternity") => TimeOffType::Parental,
            s if s.contains("bereavement") => TimeOffType::Bereavement,
            s if s.contains("jury") => TimeOffType::JuryDuty,
            s if s.contains("unpaid") => TimeOffType::Unpaid,
            _ => TimeOffType::Other,
        }).unwrap_or(TimeOffType::Other);

        TimeOffRequest {
            id: t.id,
            employee_id: t.employee_id,
            policy_id: t.policy_type.clone(),
            policy_name: t.policy_type_display_name.or(t.policy_type),
            start_date: NaiveDate::parse_from_str(&t.start_date, "%Y-%m-%d").unwrap_or_default(),
            end_date: NaiveDate::parse_from_str(&t.end_date, "%Y-%m-%d").unwrap_or_default(),
            status,
            request_type,
            hours: t.requested_hours,
            days: t.requested_days,
            notes: t.description,
            approver_id: t.approver_id,
            created_at: t.created_at.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc))),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobDepartment {
    id: String,
    name: String,
}

impl From<BobDepartment> for Department {
    fn from(d: BobDepartment) -> Self {
        Department {
            id: d.id,
            name: d.name,
            parent_id: None,
            manager_id: None,
            employee_count: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobSite {
    id: String,
    name: String,
    address: Option<BobAddress>,
    timezone: Option<String>,
}

impl From<BobSite> for Location {
    fn from(s: BobSite) -> Self {
        Location {
            id: s.id,
            name: s.name,
            address: s.address.map(|a| Address {
                street: a.street_line_1,
                street2: a.street_line_2,
                city: a.city,
                state: a.state_province,
                postal_code: a.postal_code,
                country: a.country,
            }),
            timezone: s.timezone,
            is_remote: false,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl HrProvider for HiBobClient {
    async fn list_employees(&self, _options: ListOptions) -> Result<ListResult<Employee>> {
        #[derive(Deserialize)]
        struct EmployeesResponse {
            employees: Vec<BobEmployee>,
        }

        let resp: EmployeesResponse = self.get("/people").await?;
        let total = resp.employees.len() as u32;

        Ok(ListResult {
            data: resp.employees.into_iter().map(|e| e.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_employee(&self, id: &str) -> Result<Employee> {
        let emp: BobEmployee = self.get(&format!("/people/{}", id)).await?;
        Ok(emp.into())
    }

    async fn create_employee(&self, data: CreateEmployee) -> Result<Employee> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateBody {
            first_name: String,
            surname: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            work: Option<CreateWork>,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateWork {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            department: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start_date: Option<String>,
        }

        let body = CreateBody {
            first_name: data.first_name,
            surname: data.last_name,
            email: data.email.or(data.work_email),
            work: Some(CreateWork {
                title: data.job_title,
                department: None,
                start_date: data.hire_date.map(|d| d.to_string()),
            }),
        };

        let emp: BobEmployee = self.post("/people", &body).await?;
        Ok(emp.into())
    }

    async fn update_employee(&self, id: &str, data: UpdateEmployee) -> Result<Employee> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateBody {
            #[serde(skip_serializing_if = "Option::is_none")]
            first_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            surname: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            work: Option<UpdateWork>,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateWork {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<String>,
        }

        let body = UpdateBody {
            first_name: data.first_name,
            surname: data.last_name,
            email: data.email.or(data.work_email),
            work: data.job_title.map(|t| UpdateWork { title: Some(t) }),
        };

        self.put(&format!("/people/{}", id), &body).await?;
        self.get_employee(id).await
    }

    async fn terminate_employee(&self, id: &str, termination_date: NaiveDate) -> Result<Employee> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TerminateBody {
            termination_date: String,
        }

        let body = TerminateBody {
            termination_date: termination_date.to_string(),
        };

        self.put(&format!("/people/{}/terminate", id), &body).await?;
        self.get_employee(id).await
    }

    async fn list_departments(&self, _options: ListOptions) -> Result<ListResult<Department>> {
        #[derive(Deserialize)]
        struct DepartmentsResponse {
            departments: Vec<BobDepartment>,
        }

        let resp: DepartmentsResponse = self.get("/metadata/objects/department").await?;
        let total = resp.departments.len() as u32;

        Ok(ListResult {
            data: resp.departments.into_iter().map(|d| d.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_department(&self, id: &str) -> Result<Department> {
        let result = self.list_departments(ListOptions::default()).await?;
        result
            .data
            .into_iter()
            .find(|d| d.id == id)
            .ok_or_else(|| Error::NotFound(format!("Department {}", id)))
    }

    async fn list_locations(&self, _options: ListOptions) -> Result<ListResult<Location>> {
        #[derive(Deserialize)]
        struct SitesResponse {
            sites: Vec<BobSite>,
        }

        let resp: SitesResponse = self.get("/metadata/objects/site").await?;
        let total = resp.sites.len() as u32;

        Ok(ListResult {
            data: resp.sites.into_iter().map(|s| s.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_location(&self, id: &str) -> Result<Location> {
        let result = self.list_locations(ListOptions::default()).await?;
        result
            .data
            .into_iter()
            .find(|l| l.id == id)
            .ok_or_else(|| Error::NotFound(format!("Location {}", id)))
    }

    async fn list_time_off_requests(
        &self,
        employee_id: Option<&str>,
        _options: ListOptions,
    ) -> Result<ListResult<TimeOffRequest>> {
        #[derive(Deserialize)]
        struct TimeOffResponse {
            requests: Vec<BobTimeOff>,
        }

        let path = if let Some(emp_id) = employee_id {
            format!("/timeoff/employees/{}/requests", emp_id)
        } else {
            "/timeoff/requests".to_string()
        };

        let resp: TimeOffResponse = self.get(&path).await?;
        let total = resp.requests.len() as u32;

        Ok(ListResult {
            data: resp.requests.into_iter().map(|t| t.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        let req: BobTimeOff = self.get(&format!("/timeoff/requests/{}", id)).await?;
        Ok(req.into())
    }

    async fn create_time_off_request(&self, data: CreateTimeOffRequest) -> Result<TimeOffRequest> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateBody {
            employee_id: String,
            policy_type: Option<String>,
            start_date: String,
            end_date: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            hours: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
        }

        let body = CreateBody {
            employee_id: data.employee_id,
            policy_type: data.policy_id,
            start_date: data.start_date.to_string(),
            end_date: data.end_date.to_string(),
            hours: data.hours,
            description: data.notes,
        };

        let req: BobTimeOff = self.post("/timeoff/requests", &body).await?;
        Ok(req.into())
    }

    async fn approve_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        self.put(&format!("/timeoff/requests/{}/approve", id), &serde_json::json!({})).await?;
        self.get_time_off_request(id).await
    }

    async fn deny_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        self.put(&format!("/timeoff/requests/{}/decline", id), &serde_json::json!({})).await?;
        self.get_time_off_request(id).await
    }

    async fn get_time_off_balances(&self, employee_id: &str) -> Result<Vec<TimeOffBalance>> {
        #[derive(Deserialize)]
        struct BalancesResponse {
            balances: Vec<BobBalance>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct BobBalance {
            policy_type: String,
            policy_type_display_name: Option<String>,
            balance: f64,
            used: Option<f64>,
            pending: Option<f64>,
        }

        let resp: BalancesResponse = self
            .get(&format!("/timeoff/employees/{}/balance", employee_id))
            .await?;

        Ok(resp
            .balances
            .into_iter()
            .map(|b| TimeOffBalance {
                employee_id: employee_id.to_string(),
                policy_id: b.policy_type.clone(),
                policy_name: b.policy_type_display_name.unwrap_or(b.policy_type),
                balance: b.balance,
                used: b.used.unwrap_or(0.0),
                pending: b.pending.unwrap_or(0.0),
                unit: TimeOffUnit::Days,
                extra: HashMap::new(),
            })
            .collect())
    }
}
