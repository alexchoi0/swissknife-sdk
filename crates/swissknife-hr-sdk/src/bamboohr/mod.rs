use crate::{
    Address, CreateEmployee, CreateTimeOffRequest, Department, Employee, EmploymentStatus,
    EmploymentType, Error, HrProvider, ListOptions, ListResult, Location, PayPeriod, PayRate,
    Result, TimeOffBalance, TimeOffRequest, TimeOffStatus, TimeOffType, TimeOffUnit, UpdateEmployee,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.bamboohr.com/api/gateway.php";

pub struct BambooHrClient {
    company_domain: String,
    api_key: String,
    http: reqwest::Client,
}

impl BambooHrClient {
    pub fn new(company_domain: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            company_domain: company_domain.into(),
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    fn base_url(&self) -> String {
        format!("{}/{}/v1", API_BASE, self.company_domain)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url(), path);
        let resp = self
            .http
            .get(&url)
            .basic_auth(&self.api_key, Some("x"))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(path.to_string()));
        }
        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API key".to_string()));
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
        let url = format!("{}{}", self.base_url(), path);
        let resp = self
            .http
            .post(&url)
            .basic_auth(&self.api_key, Some("x"))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API key".to_string()));
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
        let url = format!("{}{}", self.base_url(), path);
        let resp = self
            .http
            .put(&url)
            .basic_auth(&self.api_key, Some("x"))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API key".to_string()));
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

    pub async fn get_employee_directory(&self) -> Result<Vec<Employee>> {
        #[derive(Deserialize)]
        struct DirectoryResponse {
            employees: Vec<BambooEmployee>,
        }

        let resp: DirectoryResponse = self.get("/employees/directory").await?;
        Ok(resp.employees.into_iter().map(|e| e.into()).collect())
    }

    pub async fn get_employee_with_fields(&self, id: &str, fields: &[&str]) -> Result<Employee> {
        let fields_param = fields.join(",");
        let path = format!("/employees/{}?fields={}", id, fields_param);
        let emp: BambooEmployee = self.get(&path).await?;
        Ok(emp.into())
    }

    pub async fn get_time_off_policies(&self) -> Result<Vec<TimeOffPolicy>> {
        let resp: Vec<BambooTimeOffPolicy> = self.get("/meta/time_off/types").await?;
        Ok(resp.into_iter().map(|p| p.into()).collect())
    }

    pub async fn who_is_out(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<TimeOffRequest>> {
        let path = format!("/time_off/whos_out/?start={}&end={}", start, end);
        let resp: Vec<BambooTimeOff> = self.get(&path).await?;
        Ok(resp.into_iter().map(|t| t.into()).collect())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BambooEmployee {
    id: String,
    employee_number: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    display_name: Option<String>,
    work_email: Option<String>,
    personal_email: Option<String>,
    home_email: Option<String>,
    mobile_phone: Option<String>,
    work_phone: Option<String>,
    job_title: Option<String>,
    department: Option<String>,
    division: Option<String>,
    location: Option<String>,
    supervisor_id: Option<String>,
    hire_date: Option<String>,
    termination_date: Option<String>,
    status: Option<String>,
    employment_status: Option<String>,
    pay_rate: Option<String>,
    pay_type: Option<String>,
    pay_per: Option<String>,
    photo_url: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl From<BambooEmployee> for Employee {
    fn from(e: BambooEmployee) -> Self {
        let employment_status = e.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "active" => EmploymentStatus::Active,
            "inactive" => EmploymentStatus::Inactive,
            "terminated" => EmploymentStatus::Terminated,
            _ => EmploymentStatus::Other,
        });

        let employment_type = e.employment_status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "full-time" | "fulltime" => EmploymentType::FullTime,
            "part-time" | "parttime" => EmploymentType::PartTime,
            "contract" | "contractor" => EmploymentType::Contract,
            "temporary" | "temp" => EmploymentType::Temporary,
            "intern" | "internship" => EmploymentType::Intern,
            _ => EmploymentType::Other,
        });

        let pay_rate = e.pay_rate.as_ref().and_then(|rate| {
            rate.parse::<f64>().ok().map(|amount| PayRate {
                amount,
                currency: "USD".to_string(),
                period: e.pay_per.as_deref().map(|p| match p.to_lowercase().as_str() {
                    "hour" => PayPeriod::Hourly,
                    "day" => PayPeriod::Daily,
                    "week" => PayPeriod::Weekly,
                    "month" => PayPeriod::Monthly,
                    "year" => PayPeriod::Annually,
                    _ => PayPeriod::Other,
                }).unwrap_or(PayPeriod::Annually),
            })
        });

        Employee {
            id: e.id,
            employee_number: e.employee_number,
            first_name: e.first_name.unwrap_or_default(),
            last_name: e.last_name.unwrap_or_default(),
            email: e.work_email.clone().or(e.personal_email.clone()),
            work_email: e.work_email,
            personal_email: e.personal_email.or(e.home_email),
            phone: e.work_phone,
            mobile_phone: e.mobile_phone,
            job_title: e.job_title,
            department: e.department,
            department_id: None,
            division: e.division,
            location: e.location,
            manager_id: e.supervisor_id,
            hire_date: e.hire_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            termination_date: e.termination_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            employment_status,
            employment_type,
            pay_rate,
            avatar_url: e.photo_url,
            created_at: None,
            updated_at: None,
            extra: e.extra,
        }
    }
}

#[derive(Debug, Deserialize)]
struct BambooTimeOff {
    id: Option<i64>,
    #[serde(rename = "employeeId")]
    employee_id: String,
    #[serde(rename = "type")]
    time_off_type: Option<String>,
    #[serde(rename = "typeId")]
    type_id: Option<String>,
    start: String,
    end: String,
    status: Option<String>,
    amount: Option<serde_json::Value>,
    notes: Option<String>,
}

impl From<BambooTimeOff> for TimeOffRequest {
    fn from(t: BambooTimeOff) -> Self {
        let status = t.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "approved" => TimeOffStatus::Approved,
            "denied" | "declined" => TimeOffStatus::Denied,
            "cancelled" | "canceled" => TimeOffStatus::Cancelled,
            _ => TimeOffStatus::Pending,
        }).unwrap_or(TimeOffStatus::Pending);

        let request_type = t.time_off_type.as_deref().map(|s| match s.to_lowercase().as_str() {
            "vacation" | "pto" | "paid time off" => TimeOffType::Vacation,
            "sick" | "sick leave" => TimeOffType::Sick,
            "personal" => TimeOffType::Personal,
            "parental" | "maternity" | "paternity" => TimeOffType::Parental,
            "bereavement" => TimeOffType::Bereavement,
            "jury duty" => TimeOffType::JuryDuty,
            "holiday" => TimeOffType::Holiday,
            "unpaid" => TimeOffType::Unpaid,
            _ => TimeOffType::Other,
        }).unwrap_or(TimeOffType::Other);

        let days = t.amount.and_then(|a| match a {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse().ok(),
            _ => None,
        });

        TimeOffRequest {
            id: t.id.map(|i| i.to_string()).unwrap_or_default(),
            employee_id: t.employee_id,
            policy_id: t.type_id.clone(),
            policy_name: t.time_off_type,
            start_date: NaiveDate::parse_from_str(&t.start, "%Y-%m-%d").unwrap_or_default(),
            end_date: NaiveDate::parse_from_str(&t.end, "%Y-%m-%d").unwrap_or_default(),
            status,
            request_type,
            hours: None,
            days,
            notes: t.notes,
            approver_id: None,
            created_at: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BambooTimeOffPolicy {
    id: String,
    name: String,
    units: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TimeOffPolicy {
    pub id: String,
    pub name: String,
    pub unit: TimeOffUnit,
}

impl From<BambooTimeOffPolicy> for TimeOffPolicy {
    fn from(p: BambooTimeOffPolicy) -> Self {
        TimeOffPolicy {
            id: p.id,
            name: p.name,
            unit: p.units.as_deref().map(|u| match u.to_lowercase().as_str() {
                "hours" => TimeOffUnit::Hours,
                "days" => TimeOffUnit::Days,
                _ => TimeOffUnit::Other,
            }).unwrap_or(TimeOffUnit::Days),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BambooTimeOffBalance {
    #[serde(rename = "timeOffTypeId")]
    type_id: String,
    #[serde(rename = "name")]
    type_name: String,
    balance: String,
    used: Option<String>,
    #[serde(rename = "pending")]
    pending: Option<String>,
    units: Option<String>,
}

#[async_trait]
impl HrProvider for BambooHrClient {
    async fn list_employees(&self, _options: ListOptions) -> Result<ListResult<Employee>> {
        let employees = self.get_employee_directory().await?;
        let total = employees.len() as u32;
        Ok(ListResult {
            data: employees,
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_employee(&self, id: &str) -> Result<Employee> {
        let fields = [
            "id", "employeeNumber", "firstName", "lastName", "displayName",
            "workEmail", "personalEmail", "mobilePhone", "workPhone",
            "jobTitle", "department", "division", "location", "supervisorId",
            "hireDate", "terminationDate", "status", "employmentStatus",
            "payRate", "payType", "payPer", "photoUrl",
        ];
        self.get_employee_with_fields(id, &fields).await
    }

    async fn create_employee(&self, data: CreateEmployee) -> Result<Employee> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateBody {
            first_name: String,
            last_name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            work_email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            mobile_phone: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            job_title: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            department: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            hire_date: Option<String>,
        }

        let body = CreateBody {
            first_name: data.first_name,
            last_name: data.last_name,
            work_email: data.work_email.or(data.email),
            mobile_phone: data.phone,
            job_title: data.job_title,
            department: None,
            hire_date: data.hire_date.map(|d| d.to_string()),
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            id: String,
        }

        let resp: CreateResponse = self.post("/employees", &body).await?;
        self.get_employee(&resp.id).await
    }

    async fn update_employee(&self, id: &str, data: UpdateEmployee) -> Result<Employee> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateBody {
            #[serde(skip_serializing_if = "Option::is_none")]
            first_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            last_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            work_email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            mobile_phone: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            job_title: Option<String>,
        }

        let body = UpdateBody {
            first_name: data.first_name,
            last_name: data.last_name,
            work_email: data.work_email.or(data.email),
            mobile_phone: data.phone,
            job_title: data.job_title,
        };

        self.put(&format!("/employees/{}", id), &body).await?;
        self.get_employee(id).await
    }

    async fn terminate_employee(&self, id: &str, termination_date: NaiveDate) -> Result<Employee> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TerminateBody {
            termination_date: String,
            status: String,
        }

        let body = TerminateBody {
            termination_date: termination_date.to_string(),
            status: "Inactive".to_string(),
        };

        self.put(&format!("/employees/{}", id), &body).await?;
        self.get_employee(id).await
    }

    async fn list_departments(&self, _options: ListOptions) -> Result<ListResult<Department>> {
        Err(Error::Provider("BambooHR departments require custom field configuration".to_string()))
    }

    async fn get_department(&self, _id: &str) -> Result<Department> {
        Err(Error::Provider("BambooHR departments require custom field configuration".to_string()))
    }

    async fn list_locations(&self, _options: ListOptions) -> Result<ListResult<Location>> {
        Err(Error::Provider("BambooHR locations require custom field configuration".to_string()))
    }

    async fn get_location(&self, _id: &str) -> Result<Location> {
        Err(Error::Provider("BambooHR locations require custom field configuration".to_string()))
    }

    async fn list_time_off_requests(
        &self,
        employee_id: Option<&str>,
        _options: ListOptions,
    ) -> Result<ListResult<TimeOffRequest>> {
        let today = chrono::Utc::now().date_naive();
        let start = today - chrono::Duration::days(365);
        let end = today + chrono::Duration::days(365);

        let mut requests = self.who_is_out(start, end).await?;

        if let Some(emp_id) = employee_id {
            requests.retain(|r| r.employee_id == emp_id);
        }

        let total = requests.len() as u32;
        Ok(ListResult {
            data: requests,
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        let path = format!("/time_off/requests/{}", id);
        let req: BambooTimeOff = self.get(&path).await?;
        Ok(req.into())
    }

    async fn create_time_off_request(&self, data: CreateTimeOffRequest) -> Result<TimeOffRequest> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateBody {
            employee_id: String,
            time_off_type_id: Option<String>,
            start: String,
            end: String,
            amount: Option<f64>,
            notes: Option<String>,
            status: String,
        }

        let body = CreateBody {
            employee_id: data.employee_id,
            time_off_type_id: data.policy_id,
            start: data.start_date.to_string(),
            end: data.end_date.to_string(),
            amount: data.hours,
            notes: data.notes,
            status: "requested".to_string(),
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            id: String,
        }

        let resp: CreateResponse = self.post("/time_off/requests", &body).await?;
        self.get_time_off_request(&resp.id).await
    }

    async fn approve_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        self.put(&format!("/time_off/requests/{}/status", id), &serde_json::json!({"status": "approved"})).await?;
        self.get_time_off_request(id).await
    }

    async fn deny_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        self.put(&format!("/time_off/requests/{}/status", id), &serde_json::json!({"status": "denied"})).await?;
        self.get_time_off_request(id).await
    }

    async fn get_time_off_balances(&self, employee_id: &str) -> Result<Vec<TimeOffBalance>> {
        let path = format!("/employees/{}/time_off/calculator", employee_id);
        let balances: Vec<BambooTimeOffBalance> = self.get(&path).await?;

        Ok(balances
            .into_iter()
            .map(|b| TimeOffBalance {
                employee_id: employee_id.to_string(),
                policy_id: b.type_id,
                policy_name: b.type_name,
                balance: b.balance.parse().unwrap_or(0.0),
                used: b.used.and_then(|u| u.parse().ok()).unwrap_or(0.0),
                pending: b.pending.and_then(|p| p.parse().ok()).unwrap_or(0.0),
                unit: b.units.as_deref().map(|u| match u.to_lowercase().as_str() {
                    "hours" => TimeOffUnit::Hours,
                    "days" => TimeOffUnit::Days,
                    _ => TimeOffUnit::Other,
                }).unwrap_or(TimeOffUnit::Days),
                extra: HashMap::new(),
            })
            .collect())
    }
}
