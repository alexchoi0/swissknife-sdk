use crate::{
    Address, CreateEmployee, CreateTimeOffRequest, Department, Employee, EmploymentStatus,
    EmploymentType, Error, HrProvider, ListOptions, ListResult, Location, PayPeriod, PayRate,
    Result, TimeOffBalance, TimeOffRequest, TimeOffStatus, TimeOffType, TimeOffUnit, UpdateEmployee,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.personio.de/v1";

pub struct PersonioClient {
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
    http: reqwest::Client,
}

impl PersonioClient {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            access_token: None,
            http: reqwest::Client::new(),
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    async fn ensure_token(&mut self) -> Result<String> {
        if let Some(ref token) = self.access_token {
            return Ok(token.clone());
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            data: TokenData,
        }

        #[derive(Deserialize)]
        struct TokenData {
            token: String,
        }

        let resp = self
            .http
            .post(&format!("{}/auth", API_BASE))
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(Error::Auth("Failed to authenticate".to_string()));
        }

        let token_resp: TokenResponse = resp.json().await?;
        self.access_token = Some(token_resp.data.token.clone());
        Ok(token_resp.data.token)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&mut self, path: &str) -> Result<T> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", API_BASE, path);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(path.to_string()));
        }
        if resp.status() == 401 {
            self.access_token = None;
            return Err(Error::Auth("Token expired".to_string()));
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
        &mut self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", API_BASE, path);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            self.access_token = None;
            return Err(Error::Auth("Token expired".to_string()));
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

    async fn patch<T: for<'de> Deserialize<'de>, B: Serialize>(
        &mut self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", API_BASE, path);

        let resp = self
            .http
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            self.access_token = None;
            return Err(Error::Auth("Token expired".to_string()));
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

    pub async fn get_company_employees(&mut self) -> Result<Vec<Employee>> {
        #[derive(Deserialize)]
        struct EmployeesResponse {
            data: Vec<PersonioEmployee>,
        }

        let resp: EmployeesResponse = self.get("/company/employees").await?;
        Ok(resp.data.into_iter().map(|e| e.into()).collect())
    }

    pub async fn get_attendances(&mut self, start: NaiveDate, end: NaiveDate) -> Result<Vec<PersonioAttendance>> {
        #[derive(Deserialize)]
        struct AttendanceResponse {
            data: Vec<PersonioAttendance>,
        }

        let resp: AttendanceResponse = self
            .get(&format!("/company/attendances?start_date={}&end_date={}", start, end))
            .await?;
        Ok(resp.data)
    }

    pub async fn get_absence_types(&mut self) -> Result<Vec<PersonioAbsenceType>> {
        #[derive(Deserialize)]
        struct TypesResponse {
            data: Vec<PersonioAbsenceType>,
        }

        let resp: TypesResponse = self.get("/company/time-off-types").await?;
        Ok(resp.data)
    }
}

#[derive(Debug, Deserialize)]
struct PersonioEmployee {
    attributes: PersonioAttributes,
}

#[derive(Debug, Deserialize)]
struct PersonioAttributes {
    id: PersonioValue<i64>,
    first_name: Option<PersonioValue<String>>,
    last_name: Option<PersonioValue<String>>,
    email: Option<PersonioValue<String>>,
    position: Option<PersonioValue<String>>,
    department: Option<PersonioNestedValue>,
    office: Option<PersonioNestedValue>,
    supervisor: Option<PersonioNestedValue>,
    hire_date: Option<PersonioValue<String>>,
    termination_date: Option<PersonioValue<String>>,
    status: Option<PersonioValue<String>>,
    employment_type: Option<PersonioValue<String>>,
    fix_salary: Option<PersonioValue<f64>>,
    fix_salary_interval: Option<PersonioValue<String>>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PersonioValue<T> {
    value: T,
}

#[derive(Debug, Deserialize)]
struct PersonioNestedValue {
    value: Option<PersonioNestedInner>,
}

#[derive(Debug, Deserialize)]
struct PersonioNestedInner {
    id: Option<i64>,
    name: Option<String>,
}

impl From<PersonioEmployee> for Employee {
    fn from(e: PersonioEmployee) -> Self {
        let a = e.attributes;

        let employment_status = a.status.as_ref().map(|s| match s.value.to_lowercase().as_str() {
            "active" => EmploymentStatus::Active,
            "inactive" => EmploymentStatus::Inactive,
            "onboarding" => EmploymentStatus::Onboarding,
            "terminated" | "offboarded" => EmploymentStatus::Terminated,
            "leave" | "on leave" => EmploymentStatus::OnLeave,
            _ => EmploymentStatus::Other,
        });

        let employment_type = a.employment_type.as_ref().map(|t| match t.value.to_lowercase().as_str() {
            "internal" | "full-time" | "permanent" => EmploymentType::FullTime,
            "part-time" => EmploymentType::PartTime,
            "external" | "contractor" => EmploymentType::Contract,
            "intern" | "trainee" => EmploymentType::Intern,
            "temporary" => EmploymentType::Temporary,
            "freelance" => EmploymentType::Freelance,
            _ => EmploymentType::Other,
        });

        let pay_rate = a.fix_salary.as_ref().map(|s| PayRate {
            amount: s.value,
            currency: "EUR".to_string(),
            period: a.fix_salary_interval.as_ref().map(|i| match i.value.to_lowercase().as_str() {
                "hour" => PayPeriod::Hourly,
                "day" => PayPeriod::Daily,
                "week" => PayPeriod::Weekly,
                "month" => PayPeriod::Monthly,
                "year" => PayPeriod::Annually,
                _ => PayPeriod::Monthly,
            }).unwrap_or(PayPeriod::Monthly),
        });

        Employee {
            id: a.id.value.to_string(),
            employee_number: None,
            first_name: a.first_name.map(|v| v.value).unwrap_or_default(),
            last_name: a.last_name.map(|v| v.value).unwrap_or_default(),
            email: a.email.as_ref().map(|v| v.value.clone()),
            work_email: a.email.map(|v| v.value),
            personal_email: None,
            phone: None,
            mobile_phone: None,
            job_title: a.position.map(|v| v.value),
            department: a.department.as_ref().and_then(|d| d.value.as_ref().and_then(|v| v.name.clone())),
            department_id: a.department.as_ref().and_then(|d| d.value.as_ref().and_then(|v| v.id.map(|i| i.to_string()))),
            division: None,
            location: a.office.as_ref().and_then(|o| o.value.as_ref().and_then(|v| v.name.clone())),
            manager_id: a.supervisor.as_ref().and_then(|s| s.value.as_ref().and_then(|v| v.id.map(|i| i.to_string()))),
            hire_date: a.hire_date.and_then(|v| NaiveDate::parse_from_str(&v.value, "%Y-%m-%d").ok()),
            termination_date: a.termination_date.and_then(|v| NaiveDate::parse_from_str(&v.value, "%Y-%m-%d").ok()),
            employment_status,
            employment_type,
            pay_rate,
            avatar_url: None,
            created_at: None,
            updated_at: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonioAttendance {
    pub id: i64,
    pub employee_id: i64,
    pub date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub break_duration: Option<i32>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonioAbsenceType {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PersonioAbsence {
    attributes: PersonioAbsenceAttributes,
}

#[derive(Debug, Deserialize)]
struct PersonioAbsenceAttributes {
    id: PersonioValue<i64>,
    employee: PersonioNestedValue,
    time_off_type: Option<PersonioNestedValue>,
    start_date: Option<PersonioValue<String>>,
    end_date: Option<PersonioValue<String>>,
    status: Option<PersonioValue<String>>,
    days_count: Option<PersonioValue<f64>>,
    comment: Option<PersonioValue<String>>,
    created_at: Option<PersonioValue<String>>,
}

impl From<PersonioAbsence> for TimeOffRequest {
    fn from(a: PersonioAbsence) -> Self {
        let attrs = a.attributes;

        let status = attrs.status.as_ref().map(|s| match s.value.to_lowercase().as_str() {
            "approved" => TimeOffStatus::Approved,
            "declined" | "rejected" => TimeOffStatus::Denied,
            "cancelled" => TimeOffStatus::Cancelled,
            _ => TimeOffStatus::Pending,
        }).unwrap_or(TimeOffStatus::Pending);

        let request_type = attrs.time_off_type.as_ref().and_then(|t| {
            t.value.as_ref().and_then(|v| v.name.as_ref().map(|n| match n.to_lowercase().as_str() {
                s if s.contains("vacation") || s.contains("holiday") || s.contains("urlaub") => TimeOffType::Vacation,
                s if s.contains("sick") || s.contains("krank") => TimeOffType::Sick,
                s if s.contains("personal") => TimeOffType::Personal,
                s if s.contains("parental") || s.contains("eltern") => TimeOffType::Parental,
                s if s.contains("unpaid") || s.contains("unbezahlt") => TimeOffType::Unpaid,
                _ => TimeOffType::Other,
            }))
        }).unwrap_or(TimeOffType::Other);

        TimeOffRequest {
            id: attrs.id.value.to_string(),
            employee_id: attrs.employee.value.as_ref().and_then(|v| v.id).map(|i| i.to_string()).unwrap_or_default(),
            policy_id: attrs.time_off_type.as_ref().and_then(|t| t.value.as_ref().and_then(|v| v.id.map(|i| i.to_string()))),
            policy_name: attrs.time_off_type.and_then(|t| t.value.and_then(|v| v.name)),
            start_date: attrs.start_date.and_then(|v| NaiveDate::parse_from_str(&v.value, "%Y-%m-%d").ok()).unwrap_or_default(),
            end_date: attrs.end_date.and_then(|v| NaiveDate::parse_from_str(&v.value, "%Y-%m-%d").ok()).unwrap_or_default(),
            status,
            request_type,
            hours: None,
            days: attrs.days_count.map(|v| v.value),
            notes: attrs.comment.map(|v| v.value),
            approver_id: None,
            created_at: attrs.created_at.and_then(|v| DateTime::parse_from_rfc3339(&v.value).ok().map(|dt| dt.with_timezone(&Utc))),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PersonioDepartment {
    attributes: PersonioDepartmentAttributes,
}

#[derive(Debug, Deserialize)]
struct PersonioDepartmentAttributes {
    id: PersonioValue<i64>,
    name: PersonioValue<String>,
}

impl From<PersonioDepartment> for Department {
    fn from(d: PersonioDepartment) -> Self {
        Department {
            id: d.attributes.id.value.to_string(),
            name: d.attributes.name.value,
            parent_id: None,
            manager_id: None,
            employee_count: None,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl HrProvider for PersonioClient {
    async fn list_employees(&self, options: ListOptions) -> Result<ListResult<Employee>> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        let offset = options.page.unwrap_or(0) * options.per_page.unwrap_or(200);
        let limit = options.per_page.unwrap_or(200);

        #[derive(Deserialize)]
        struct EmployeesResponse {
            data: Vec<PersonioEmployee>,
        }

        let resp: EmployeesResponse = client
            .get(&format!("/company/employees?offset={}&limit={}", offset, limit))
            .await?;

        let has_more = resp.data.len() == limit as usize;

        Ok(ListResult {
            data: resp.data.into_iter().map(|e| e.into()).collect(),
            total: None,
            has_more,
            next_cursor: if has_more {
                Some((offset + limit).to_string())
            } else {
                None
            },
        })
    }

    async fn get_employee(&self, id: &str) -> Result<Employee> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Deserialize)]
        struct EmployeeResponse {
            data: PersonioEmployee,
        }

        let resp: EmployeeResponse = client.get(&format!("/company/employees/{}", id)).await?;
        Ok(resp.data.into())
    }

    async fn create_employee(&self, data: CreateEmployee) -> Result<Employee> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Serialize)]
        struct CreateBody {
            employee: CreateEmployeeInner,
        }

        #[derive(Serialize)]
        struct CreateEmployeeInner {
            first_name: String,
            last_name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            position: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            hire_date: Option<String>,
        }

        let body = CreateBody {
            employee: CreateEmployeeInner {
                first_name: data.first_name,
                last_name: data.last_name,
                email: data.email.or(data.work_email),
                position: data.job_title,
                hire_date: data.hire_date.map(|d| d.to_string()),
            },
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            data: PersonioEmployee,
        }

        let resp: CreateResponse = client.post("/company/employees", &body).await?;
        Ok(resp.data.into())
    }

    async fn update_employee(&self, id: &str, data: UpdateEmployee) -> Result<Employee> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Serialize)]
        struct UpdateBody {
            employee: UpdateEmployeeInner,
        }

        #[derive(Serialize)]
        struct UpdateEmployeeInner {
            #[serde(skip_serializing_if = "Option::is_none")]
            first_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            last_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            position: Option<String>,
        }

        let body = UpdateBody {
            employee: UpdateEmployeeInner {
                first_name: data.first_name,
                last_name: data.last_name,
                email: data.email.or(data.work_email),
                position: data.job_title,
            },
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            data: PersonioEmployee,
        }

        let resp: UpdateResponse = client.patch(&format!("/company/employees/{}", id), &body).await?;
        Ok(resp.data.into())
    }

    async fn terminate_employee(&self, _id: &str, _termination_date: NaiveDate) -> Result<Employee> {
        Err(Error::Provider(
            "Personio employee termination requires offboarding workflow".to_string(),
        ))
    }

    async fn list_departments(&self, _options: ListOptions) -> Result<ListResult<Department>> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Deserialize)]
        struct DepartmentsResponse {
            data: Vec<PersonioDepartment>,
        }

        let resp: DepartmentsResponse = client.get("/company/departments").await?;
        let total = resp.data.len() as u32;

        Ok(ListResult {
            data: resp.data.into_iter().map(|d| d.into()).collect(),
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
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Deserialize)]
        struct OfficesResponse {
            data: Vec<PersonioOffice>,
        }

        #[derive(Deserialize)]
        struct PersonioOffice {
            attributes: PersonioOfficeAttributes,
        }

        #[derive(Deserialize)]
        struct PersonioOfficeAttributes {
            id: PersonioValue<i64>,
            name: PersonioValue<String>,
        }

        let resp: OfficesResponse = client.get("/company/offices").await?;
        let total = resp.data.len() as u32;

        Ok(ListResult {
            data: resp
                .data
                .into_iter()
                .map(|o| Location {
                    id: o.attributes.id.value.to_string(),
                    name: o.attributes.name.value,
                    address: None,
                    timezone: None,
                    is_remote: false,
                    extra: HashMap::new(),
                })
                .collect(),
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
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Deserialize)]
        struct AbsencesResponse {
            data: Vec<PersonioAbsence>,
        }

        let path = if let Some(emp_id) = employee_id {
            format!("/company/time-offs?employees[]={}", emp_id)
        } else {
            "/company/time-offs".to_string()
        };

        let resp: AbsencesResponse = client.get(&path).await?;
        let total = resp.data.len() as u32;

        Ok(ListResult {
            data: resp.data.into_iter().map(|a| a.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Deserialize)]
        struct AbsenceResponse {
            data: PersonioAbsence,
        }

        let resp: AbsenceResponse = client.get(&format!("/company/time-offs/{}", id)).await?;
        Ok(resp.data.into())
    }

    async fn create_time_off_request(&self, data: CreateTimeOffRequest) -> Result<TimeOffRequest> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Serialize)]
        struct CreateBody {
            employee_id: String,
            time_off_type_id: Option<String>,
            start_date: String,
            end_date: String,
            half_day_start: bool,
            half_day_end: bool,
            comment: Option<String>,
        }

        let body = CreateBody {
            employee_id: data.employee_id,
            time_off_type_id: data.policy_id,
            start_date: data.start_date.to_string(),
            end_date: data.end_date.to_string(),
            half_day_start: false,
            half_day_end: false,
            comment: data.notes,
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            data: PersonioAbsence,
        }

        let resp: CreateResponse = client.post("/company/time-offs", &body).await?;
        Ok(resp.data.into())
    }

    async fn approve_time_off_request(&self, _id: &str) -> Result<TimeOffRequest> {
        Err(Error::Provider(
            "Personio time off approval requires approval workflow API".to_string(),
        ))
    }

    async fn deny_time_off_request(&self, _id: &str) -> Result<TimeOffRequest> {
        Err(Error::Provider(
            "Personio time off denial requires approval workflow API".to_string(),
        ))
    }

    async fn get_time_off_balances(&self, employee_id: &str) -> Result<Vec<TimeOffBalance>> {
        let mut client = PersonioClient::new(&self.client_id, &self.client_secret);
        if let Some(ref token) = self.access_token {
            client.access_token = Some(token.clone());
        }

        #[derive(Deserialize)]
        struct BalancesResponse {
            data: Vec<PersonioBalance>,
        }

        #[derive(Deserialize)]
        struct PersonioBalance {
            time_off_type_id: i64,
            time_off_type_name: String,
            balance: f64,
            used: Option<f64>,
        }

        let resp: BalancesResponse = client
            .get(&format!("/company/employees/{}/absences/balance", employee_id))
            .await?;

        Ok(resp
            .data
            .into_iter()
            .map(|b| TimeOffBalance {
                employee_id: employee_id.to_string(),
                policy_id: b.time_off_type_id.to_string(),
                policy_name: b.time_off_type_name,
                balance: b.balance,
                used: b.used.unwrap_or(0.0),
                pending: 0.0,
                unit: TimeOffUnit::Days,
                extra: HashMap::new(),
            })
            .collect())
    }
}
