use crate::{
    Address, CreateEmployee, CreateTimeOffRequest, Department, Employee, EmploymentStatus,
    EmploymentType, Error, HrProvider, ListOptions, ListResult, Location, PayPeriod, PayRate,
    Result, TimeOffBalance, TimeOffRequest, TimeOffStatus, TimeOffType, TimeOffUnit, UpdateEmployee,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct WorkdayClient {
    base_url: String,
    tenant: String,
    access_token: String,
    http: reqwest::Client,
}

impl WorkdayClient {
    pub fn new(
        base_url: impl Into<String>,
        tenant: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            tenant: tenant.into(),
            access_token: access_token.into(),
            http: reqwest::Client::new(),
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!(
            "{}/ccx/api/v1/{}/{}",
            self.base_url, self.tenant, path
        )
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = self.api_url(path);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(path.to_string()));
        }
        if resp.status() == 401 {
            return Err(Error::Auth("Invalid access token".to_string()));
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
        let url = self.api_url(path);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.access_token)
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid access token".to_string()));
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
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.api_url(path);
        let resp = self
            .http
            .patch(&url)
            .bearer_auth(&self.access_token)
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid access token".to_string()));
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

    pub async fn get_worker_by_wid(&self, wid: &str) -> Result<Employee> {
        let worker: WorkdayWorker = self.get(&format!("workers/{}", wid)).await?;
        Ok(worker.into())
    }

    pub async fn search_workers(&self, query: &str) -> Result<Vec<Employee>> {
        #[derive(Deserialize)]
        struct SearchResponse {
            data: Vec<WorkdayWorker>,
        }

        let resp: SearchResponse = self.get(&format!("workers?search={}", query)).await?;
        Ok(resp.data.into_iter().map(|w| w.into()).collect())
    }

    pub async fn get_organizations(&self) -> Result<Vec<WorkdayOrganization>> {
        #[derive(Deserialize)]
        struct OrgResponse {
            data: Vec<WorkdayOrganization>,
        }

        let resp: OrgResponse = self.get("organizations").await?;
        Ok(resp.data)
    }

    pub async fn get_cost_centers(&self) -> Result<Vec<WorkdayCostCenter>> {
        #[derive(Deserialize)]
        struct CostCenterResponse {
            data: Vec<WorkdayCostCenter>,
        }

        let resp: CostCenterResponse = self.get("costCenters").await?;
        Ok(resp.data)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkdayWorker {
    id: String,
    descriptor: Option<String>,
    primary_work_email: Option<String>,
    primary_work_phone: Option<String>,
    business_title: Option<String>,
    supervisory_organization: Option<WorkdayRef>,
    location: Option<WorkdayRef>,
    manager: Option<WorkdayRef>,
    hire_date: Option<String>,
    termination_date: Option<String>,
    worker_status: Option<WorkdayRef>,
    worker_type: Option<WorkdayRef>,
    primary_position: Option<WorkdayPosition>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkdayRef {
    id: Option<String>,
    descriptor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkdayPosition {
    position_id: Option<String>,
    position_title: Option<String>,
    job_profile: Option<WorkdayRef>,
    pay_rate_type: Option<WorkdayRef>,
}

impl From<WorkdayWorker> for Employee {
    fn from(w: WorkdayWorker) -> Self {
        let name_parts: Vec<&str> = w
            .descriptor
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .collect();
        let first_name = name_parts.first().map(|s| s.to_string()).unwrap_or_default();
        let last_name = name_parts.get(1..).map(|s| s.join(" ")).unwrap_or_default();

        let employment_status = w.worker_status.as_ref().and_then(|s| {
            s.descriptor.as_ref().map(|d| match d.to_lowercase().as_str() {
                "active" => EmploymentStatus::Active,
                "inactive" => EmploymentStatus::Inactive,
                "terminated" | "terminated employee" => EmploymentStatus::Terminated,
                "on leave" => EmploymentStatus::OnLeave,
                _ => EmploymentStatus::Other,
            })
        });

        let employment_type = w.worker_type.as_ref().and_then(|t| {
            t.descriptor.as_ref().map(|d| match d.to_lowercase().as_str() {
                "employee" | "regular" | "full-time" => EmploymentType::FullTime,
                "part-time" => EmploymentType::PartTime,
                "contingent worker" | "contractor" => EmploymentType::Contract,
                "temporary" | "temp" => EmploymentType::Temporary,
                "intern" => EmploymentType::Intern,
                _ => EmploymentType::Other,
            })
        });

        Employee {
            id: w.id,
            employee_number: w.primary_position.as_ref().and_then(|p| p.position_id.clone()),
            first_name,
            last_name,
            email: w.primary_work_email.clone(),
            work_email: w.primary_work_email,
            personal_email: None,
            phone: w.primary_work_phone,
            mobile_phone: None,
            job_title: w.business_title.or_else(|| {
                w.primary_position
                    .as_ref()
                    .and_then(|p| p.position_title.clone())
            }),
            department: w.supervisory_organization.as_ref().and_then(|o| o.descriptor.clone()),
            department_id: w.supervisory_organization.as_ref().and_then(|o| o.id.clone()),
            division: None,
            location: w.location.as_ref().and_then(|l| l.descriptor.clone()),
            manager_id: w.manager.as_ref().and_then(|m| m.id.clone()),
            hire_date: w.hire_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            termination_date: w.termination_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            employment_status,
            employment_type,
            pay_rate: None,
            avatar_url: None,
            created_at: None,
            updated_at: None,
            extra: w.extra,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkdayOrganization {
    pub id: String,
    pub descriptor: Option<String>,
    pub organization_type: Option<WorkdayRef>,
    pub superior_organization: Option<WorkdayRef>,
    pub manager: Option<WorkdayRef>,
}

impl From<WorkdayOrganization> for Department {
    fn from(o: WorkdayOrganization) -> Self {
        Department {
            id: o.id,
            name: o.descriptor.unwrap_or_default(),
            parent_id: o.superior_organization.and_then(|s| s.id),
            manager_id: o.manager.and_then(|m| m.id),
            employee_count: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkdayCostCenter {
    pub id: String,
    pub descriptor: Option<String>,
    pub cost_center_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkdayLocation {
    pub id: String,
    pub descriptor: Option<String>,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub time_zone: Option<String>,
}

impl From<WorkdayLocation> for Location {
    fn from(l: WorkdayLocation) -> Self {
        Location {
            id: l.id,
            name: l.descriptor.unwrap_or_default(),
            address: Some(Address {
                street: l.address_line_1,
                street2: l.address_line_2,
                city: l.city,
                state: l.region,
                postal_code: l.postal_code,
                country: l.country,
            }),
            timezone: l.time_zone,
            is_remote: false,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkdayTimeOff {
    id: String,
    worker: WorkdayRef,
    time_off_type: Option<WorkdayRef>,
    start_date: String,
    end_date: String,
    status: Option<WorkdayRef>,
    total_days: Option<f64>,
    total_hours: Option<f64>,
    comments: Option<String>,
}

impl From<WorkdayTimeOff> for TimeOffRequest {
    fn from(t: WorkdayTimeOff) -> Self {
        let status = t.status.as_ref().and_then(|s| {
            s.descriptor.as_ref().map(|d| match d.to_lowercase().as_str() {
                "approved" => TimeOffStatus::Approved,
                "denied" | "rejected" => TimeOffStatus::Denied,
                "cancelled" | "canceled" => TimeOffStatus::Cancelled,
                _ => TimeOffStatus::Pending,
            })
        }).unwrap_or(TimeOffStatus::Pending);

        let request_type = t.time_off_type.as_ref().and_then(|tt| {
            tt.descriptor.as_ref().map(|d| match d.to_lowercase().as_str() {
                s if s.contains("vacation") || s.contains("pto") => TimeOffType::Vacation,
                s if s.contains("sick") => TimeOffType::Sick,
                s if s.contains("personal") => TimeOffType::Personal,
                s if s.contains("parental") || s.contains("maternity") || s.contains("paternity") => TimeOffType::Parental,
                s if s.contains("bereavement") => TimeOffType::Bereavement,
                s if s.contains("jury") => TimeOffType::JuryDuty,
                s if s.contains("holiday") => TimeOffType::Holiday,
                s if s.contains("unpaid") => TimeOffType::Unpaid,
                _ => TimeOffType::Other,
            })
        }).unwrap_or(TimeOffType::Other);

        TimeOffRequest {
            id: t.id,
            employee_id: t.worker.id.unwrap_or_default(),
            policy_id: t.time_off_type.as_ref().and_then(|tt| tt.id.clone()),
            policy_name: t.time_off_type.and_then(|tt| tt.descriptor),
            start_date: NaiveDate::parse_from_str(&t.start_date, "%Y-%m-%d").unwrap_or_default(),
            end_date: NaiveDate::parse_from_str(&t.end_date, "%Y-%m-%d").unwrap_or_default(),
            status,
            request_type,
            hours: t.total_hours,
            days: t.total_days,
            notes: t.comments,
            approver_id: None,
            created_at: None,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl HrProvider for WorkdayClient {
    async fn list_employees(&self, options: ListOptions) -> Result<ListResult<Employee>> {
        #[derive(Deserialize)]
        struct WorkersResponse {
            data: Vec<WorkdayWorker>,
            total: Option<u32>,
        }

        let offset = options.page.unwrap_or(0) * options.per_page.unwrap_or(100);
        let limit = options.per_page.unwrap_or(100);

        let resp: WorkersResponse = self
            .get(&format!("workers?offset={}&limit={}", offset, limit))
            .await?;

        let has_more = resp.data.len() == limit as usize;

        Ok(ListResult {
            data: resp.data.into_iter().map(|w| w.into()).collect(),
            total: resp.total,
            has_more,
            next_cursor: if has_more {
                Some((offset + limit).to_string())
            } else {
                None
            },
        })
    }

    async fn get_employee(&self, id: &str) -> Result<Employee> {
        self.get_worker_by_wid(id).await
    }

    async fn create_employee(&self, _data: CreateEmployee) -> Result<Employee> {
        Err(Error::Provider(
            "Workday employee creation requires HCM Core module".to_string(),
        ))
    }

    async fn update_employee(&self, _id: &str, _data: UpdateEmployee) -> Result<Employee> {
        Err(Error::Provider(
            "Workday employee updates require HCM Core module".to_string(),
        ))
    }

    async fn terminate_employee(&self, _id: &str, _termination_date: NaiveDate) -> Result<Employee> {
        Err(Error::Provider(
            "Workday terminations require HCM Core module".to_string(),
        ))
    }

    async fn list_departments(&self, _options: ListOptions) -> Result<ListResult<Department>> {
        let orgs = self.get_organizations().await?;
        let total = orgs.len() as u32;

        Ok(ListResult {
            data: orgs.into_iter().map(|o| o.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_department(&self, id: &str) -> Result<Department> {
        let org: WorkdayOrganization = self.get(&format!("organizations/{}", id)).await?;
        Ok(org.into())
    }

    async fn list_locations(&self, _options: ListOptions) -> Result<ListResult<Location>> {
        #[derive(Deserialize)]
        struct LocationsResponse {
            data: Vec<WorkdayLocation>,
        }

        let resp: LocationsResponse = self.get("locations").await?;
        let total = resp.data.len() as u32;

        Ok(ListResult {
            data: resp.data.into_iter().map(|l| l.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_location(&self, id: &str) -> Result<Location> {
        let loc: WorkdayLocation = self.get(&format!("locations/{}", id)).await?;
        Ok(loc.into())
    }

    async fn list_time_off_requests(
        &self,
        employee_id: Option<&str>,
        _options: ListOptions,
    ) -> Result<ListResult<TimeOffRequest>> {
        #[derive(Deserialize)]
        struct TimeOffResponse {
            data: Vec<WorkdayTimeOff>,
        }

        let path = if let Some(emp_id) = employee_id {
            format!("workers/{}/timeOffRequests", emp_id)
        } else {
            "timeOffRequests".to_string()
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
        let req: WorkdayTimeOff = self.get(&format!("timeOffRequests/{}", id)).await?;
        Ok(req.into())
    }

    async fn create_time_off_request(&self, _data: CreateTimeOffRequest) -> Result<TimeOffRequest> {
        Err(Error::Provider(
            "Workday time off creation requires Absence module".to_string(),
        ))
    }

    async fn approve_time_off_request(&self, _id: &str) -> Result<TimeOffRequest> {
        Err(Error::Provider(
            "Workday time off approval requires Absence module".to_string(),
        ))
    }

    async fn deny_time_off_request(&self, _id: &str) -> Result<TimeOffRequest> {
        Err(Error::Provider(
            "Workday time off denial requires Absence module".to_string(),
        ))
    }

    async fn get_time_off_balances(&self, employee_id: &str) -> Result<Vec<TimeOffBalance>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct BalanceResponse {
            data: Vec<WorkdayBalance>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct WorkdayBalance {
            time_off_type: Option<WorkdayRef>,
            balance: Option<f64>,
            used: Option<f64>,
            unit: Option<String>,
        }

        let resp: BalanceResponse = self
            .get(&format!("workers/{}/timeOffBalances", employee_id))
            .await?;

        Ok(resp
            .data
            .into_iter()
            .map(|b| TimeOffBalance {
                employee_id: employee_id.to_string(),
                policy_id: b.time_off_type.as_ref().and_then(|t| t.id.clone()).unwrap_or_default(),
                policy_name: b.time_off_type.and_then(|t| t.descriptor).unwrap_or_default(),
                balance: b.balance.unwrap_or(0.0),
                used: b.used.unwrap_or(0.0),
                pending: 0.0,
                unit: b.unit.as_deref().map(|u| match u.to_lowercase().as_str() {
                    "hours" => TimeOffUnit::Hours,
                    "days" => TimeOffUnit::Days,
                    _ => TimeOffUnit::Other,
                }).unwrap_or(TimeOffUnit::Days),
                extra: HashMap::new(),
            })
            .collect())
    }
}
