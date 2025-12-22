use crate::{
    Address, CreateEmployee, CreateTimeOffRequest, Department, Employee, EmploymentStatus,
    EmploymentType, Error, HrProvider, ListOptions, ListResult, Location, PayPeriod, PayRate,
    Result, TimeOffBalance, TimeOffRequest, TimeOffStatus, TimeOffType, TimeOffUnit, UpdateEmployee,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.deel.com/rest/v2";

pub struct DeelClient {
    api_token: String,
    http: reqwest::Client,
}

impl DeelClient {
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
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
            .header("Authorization", format!("Bearer {}", self.api_token))
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

    pub async fn list_contracts(&self) -> Result<Vec<DeelContract>> {
        #[derive(Deserialize)]
        struct ContractsResponse {
            data: Vec<DeelContract>,
        }

        let resp: ContractsResponse = self.get("/contracts").await?;
        Ok(resp.data)
    }

    pub async fn get_contract(&self, id: &str) -> Result<DeelContract> {
        #[derive(Deserialize)]
        struct ContractResponse {
            data: DeelContract,
        }

        let resp: ContractResponse = self.get(&format!("/contracts/{}", id)).await?;
        Ok(resp.data)
    }

    pub async fn list_invoices(&self) -> Result<Vec<DeelInvoice>> {
        #[derive(Deserialize)]
        struct InvoicesResponse {
            data: Vec<DeelInvoice>,
        }

        let resp: InvoicesResponse = self.get("/invoices").await?;
        Ok(resp.data)
    }

    pub async fn list_payments(&self) -> Result<Vec<DeelPayment>> {
        #[derive(Deserialize)]
        struct PaymentsResponse {
            data: Vec<DeelPayment>,
        }

        let resp: PaymentsResponse = self.get("/payments").await?;
        Ok(resp.data)
    }

    pub async fn get_worker(&self, worker_id: &str) -> Result<DeelWorker> {
        #[derive(Deserialize)]
        struct WorkerResponse {
            data: DeelWorker,
        }

        let resp: WorkerResponse = self.get(&format!("/people/{}", worker_id)).await?;
        Ok(resp.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelContract {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub contract_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub worker: Option<DeelWorkerRef>,
    pub client: Option<DeelClientRef>,
    pub compensation: Option<DeelCompensation>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelWorkerRef {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelClientRef {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelCompensation {
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub frequency: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelWorker {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub personal_email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<String>,
    pub nationality: Option<String>,
    pub country: Option<String>,
    pub address: Option<DeelAddress>,
    pub contracts: Option<Vec<DeelContractRef>>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelAddress {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelContractRef {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<String>,
}

impl From<DeelWorker> for Employee {
    fn from(w: DeelWorker) -> Self {
        let active_contract = w.contracts.as_ref().and_then(|c| {
            c.iter().find(|c| c.status.as_deref() == Some("active"))
        });

        let employment_status = active_contract.map(|_| EmploymentStatus::Active)
            .unwrap_or(EmploymentStatus::Inactive);

        Employee {
            id: w.id,
            employee_number: None,
            first_name: w.first_name.unwrap_or_default(),
            last_name: w.last_name.unwrap_or_default(),
            email: w.email.clone(),
            work_email: w.email,
            personal_email: w.personal_email,
            phone: w.phone,
            mobile_phone: None,
            job_title: active_contract.and_then(|c| c.title.clone()),
            department: None,
            department_id: None,
            division: None,
            location: w.country.clone(),
            manager_id: None,
            hire_date: None,
            termination_date: None,
            employment_status: Some(employment_status),
            employment_type: Some(EmploymentType::Contract),
            pay_rate: None,
            avatar_url: None,
            created_at: w.created_at.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc))),
            updated_at: None,
            extra: HashMap::new(),
        }
    }
}

impl From<DeelContract> for Employee {
    fn from(c: DeelContract) -> Self {
        let worker = c.worker.as_ref();

        let employment_status = c.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "active" | "in_progress" => EmploymentStatus::Active,
            "terminated" | "ended" | "completed" => EmploymentStatus::Terminated,
            "pending" | "draft" => EmploymentStatus::Onboarding,
            _ => EmploymentStatus::Other,
        });

        let employment_type = c.contract_type.as_deref().map(|t| match t.to_lowercase().as_str() {
            "contractor" | "independent_contractor" => EmploymentType::Contract,
            "employee" | "eor" => EmploymentType::FullTime,
            "freelancer" => EmploymentType::Freelance,
            _ => EmploymentType::Other,
        });

        let pay_rate = c.compensation.as_ref().map(|comp| PayRate {
            amount: comp.amount.unwrap_or(0.0),
            currency: comp.currency.clone().unwrap_or_else(|| "USD".to_string()),
            period: comp.frequency.as_deref().map(|f| match f.to_lowercase().as_str() {
                "hourly" => PayPeriod::Hourly,
                "daily" => PayPeriod::Daily,
                "weekly" => PayPeriod::Weekly,
                "biweekly" => PayPeriod::Biweekly,
                "monthly" => PayPeriod::Monthly,
                "annually" | "yearly" => PayPeriod::Annually,
                _ => PayPeriod::Monthly,
            }).unwrap_or(PayPeriod::Monthly),
        });

        Employee {
            id: c.id.clone(),
            employee_number: Some(c.id),
            first_name: worker.and_then(|w| w.first_name.clone()).unwrap_or_default(),
            last_name: worker.and_then(|w| w.last_name.clone()).unwrap_or_default(),
            email: worker.and_then(|w| w.email.clone()),
            work_email: worker.and_then(|w| w.email.clone()),
            personal_email: None,
            phone: None,
            mobile_phone: None,
            job_title: c.title,
            department: None,
            department_id: None,
            division: None,
            location: c.country,
            manager_id: None,
            hire_date: c.start_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            termination_date: c.end_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
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
pub struct DeelInvoice {
    pub id: String,
    pub status: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub due_date: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeelPayment {
    pub id: String,
    pub status: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub payment_date: Option<String>,
    pub contract_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeelTimeOff {
    id: String,
    worker_id: String,
    start_date: String,
    end_date: String,
    status: Option<String>,
    time_off_type: Option<String>,
    days: Option<f64>,
    notes: Option<String>,
}

impl From<DeelTimeOff> for TimeOffRequest {
    fn from(t: DeelTimeOff) -> Self {
        let status = t.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "approved" => TimeOffStatus::Approved,
            "denied" | "rejected" => TimeOffStatus::Denied,
            "cancelled" | "canceled" => TimeOffStatus::Cancelled,
            _ => TimeOffStatus::Pending,
        }).unwrap_or(TimeOffStatus::Pending);

        let request_type = t.time_off_type.as_deref().map(|s| match s.to_lowercase().as_str() {
            "vacation" | "pto" | "annual_leave" => TimeOffType::Vacation,
            "sick" | "sick_leave" => TimeOffType::Sick,
            "personal" => TimeOffType::Personal,
            "parental" | "maternity" | "paternity" => TimeOffType::Parental,
            "unpaid" => TimeOffType::Unpaid,
            _ => TimeOffType::Other,
        }).unwrap_or(TimeOffType::Other);

        TimeOffRequest {
            id: t.id,
            employee_id: t.worker_id,
            policy_id: None,
            policy_name: t.time_off_type,
            start_date: NaiveDate::parse_from_str(&t.start_date, "%Y-%m-%d").unwrap_or_default(),
            end_date: NaiveDate::parse_from_str(&t.end_date, "%Y-%m-%d").unwrap_or_default(),
            status,
            request_type,
            hours: None,
            days: t.days,
            notes: t.notes,
            approver_id: None,
            created_at: None,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl HrProvider for DeelClient {
    async fn list_employees(&self, _options: ListOptions) -> Result<ListResult<Employee>> {
        let contracts = self.list_contracts().await?;
        let employees: Vec<Employee> = contracts.into_iter().map(|c| c.into()).collect();
        let total = employees.len() as u32;

        Ok(ListResult {
            data: employees,
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_employee(&self, id: &str) -> Result<Employee> {
        let contract = self.get_contract(id).await?;
        Ok(contract.into())
    }

    async fn create_employee(&self, _data: CreateEmployee) -> Result<Employee> {
        Err(Error::Provider(
            "Use Deel dashboard to create contracts".to_string(),
        ))
    }

    async fn update_employee(&self, _id: &str, _data: UpdateEmployee) -> Result<Employee> {
        Err(Error::Provider(
            "Use Deel dashboard to update contracts".to_string(),
        ))
    }

    async fn terminate_employee(&self, _id: &str, _termination_date: NaiveDate) -> Result<Employee> {
        Err(Error::Provider(
            "Use Deel dashboard to terminate contracts".to_string(),
        ))
    }

    async fn list_departments(&self, _options: ListOptions) -> Result<ListResult<Department>> {
        Err(Error::Provider("Deel does not have departments".to_string()))
    }

    async fn get_department(&self, _id: &str) -> Result<Department> {
        Err(Error::Provider("Deel does not have departments".to_string()))
    }

    async fn list_locations(&self, _options: ListOptions) -> Result<ListResult<Location>> {
        Err(Error::Provider(
            "Deel uses worker countries instead of locations".to_string(),
        ))
    }

    async fn get_location(&self, _id: &str) -> Result<Location> {
        Err(Error::Provider(
            "Deel uses worker countries instead of locations".to_string(),
        ))
    }

    async fn list_time_off_requests(
        &self,
        employee_id: Option<&str>,
        _options: ListOptions,
    ) -> Result<ListResult<TimeOffRequest>> {
        #[derive(Deserialize)]
        struct TimeOffResponse {
            data: Vec<DeelTimeOff>,
        }

        let path = if let Some(emp_id) = employee_id {
            format!("/people/{}/time-off", emp_id)
        } else {
            "/time-off".to_string()
        };

        let resp: TimeOffResponse = self.get(&path).await?;
        let total = resp.data.len() as u32;

        Ok(ListResult {
            data: resp.data.into_iter().map(|t| t.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        #[derive(Deserialize)]
        struct TimeOffResponse {
            data: DeelTimeOff,
        }

        let resp: TimeOffResponse = self.get(&format!("/time-off/{}", id)).await?;
        Ok(resp.data.into())
    }

    async fn create_time_off_request(&self, data: CreateTimeOffRequest) -> Result<TimeOffRequest> {
        #[derive(Serialize)]
        struct CreateBody {
            worker_id: String,
            start_date: String,
            end_date: String,
            time_off_type: String,
            notes: Option<String>,
        }

        let time_off_type = match data.request_type {
            TimeOffType::Vacation => "vacation",
            TimeOffType::Sick => "sick",
            TimeOffType::Personal => "personal",
            TimeOffType::Parental => "parental",
            TimeOffType::Unpaid => "unpaid",
            _ => "other",
        };

        let body = CreateBody {
            worker_id: data.employee_id,
            start_date: data.start_date.to_string(),
            end_date: data.end_date.to_string(),
            time_off_type: time_off_type.to_string(),
            notes: data.notes,
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            data: DeelTimeOff,
        }

        let resp: CreateResponse = self.post("/time-off", &body).await?;
        Ok(resp.data.into())
    }

    async fn approve_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        #[derive(Deserialize)]
        struct ApproveResponse {
            data: DeelTimeOff,
        }

        let resp: ApproveResponse = self
            .post(&format!("/time-off/{}/approve", id), &serde_json::json!({}))
            .await?;
        Ok(resp.data.into())
    }

    async fn deny_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        #[derive(Deserialize)]
        struct DenyResponse {
            data: DeelTimeOff,
        }

        let resp: DenyResponse = self
            .post(&format!("/time-off/{}/deny", id), &serde_json::json!({}))
            .await?;
        Ok(resp.data.into())
    }

    async fn get_time_off_balances(&self, employee_id: &str) -> Result<Vec<TimeOffBalance>> {
        #[derive(Deserialize)]
        struct BalanceResponse {
            data: Vec<DeelBalance>,
        }

        #[derive(Deserialize)]
        struct DeelBalance {
            policy_type: Option<String>,
            balance: Option<f64>,
            used: Option<f64>,
        }

        let resp: BalanceResponse = self
            .get(&format!("/people/{}/time-off/balance", employee_id))
            .await?;

        Ok(resp
            .data
            .into_iter()
            .map(|b| TimeOffBalance {
                employee_id: employee_id.to_string(),
                policy_id: b.policy_type.clone().unwrap_or_default(),
                policy_name: b.policy_type.unwrap_or_default(),
                balance: b.balance.unwrap_or(0.0),
                used: b.used.unwrap_or(0.0),
                pending: 0.0,
                unit: TimeOffUnit::Days,
                extra: HashMap::new(),
            })
            .collect())
    }
}
