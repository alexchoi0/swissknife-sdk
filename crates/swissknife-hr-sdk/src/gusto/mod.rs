use crate::{
    Address, CreateEmployee, CreateTimeOffRequest, Department, Employee, EmploymentStatus,
    EmploymentType, Error, HrProvider, ListOptions, ListResult, Location, PayPeriod, PayRate,
    Payslip, PayComponent, PayrollRun, PayrollStatus, Result, TimeOffBalance, TimeOffRequest,
    TimeOffStatus, TimeOffType, TimeOffUnit, UpdateEmployee,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.gusto.com/v1";

pub struct GustoClient {
    access_token: String,
    company_id: String,
    http: reqwest::Client,
}

impl GustoClient {
    pub fn new(access_token: impl Into<String>, company_id: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            company_id: company_id.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
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
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.access_token)
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

    async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .put(&url)
            .bearer_auth(&self.access_token)
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

    pub async fn get_company(&self) -> Result<GustoCompany> {
        self.get(&format!("/companies/{}", self.company_id)).await
    }

    pub async fn list_payrolls(&self) -> Result<Vec<PayrollRun>> {
        let payrolls: Vec<GustoPayroll> = self
            .get(&format!("/companies/{}/payrolls", self.company_id))
            .await?;
        Ok(payrolls.into_iter().map(|p| p.into()).collect())
    }

    pub async fn get_payroll(&self, payroll_id: &str) -> Result<PayrollRun> {
        let payroll: GustoPayroll = self
            .get(&format!(
                "/companies/{}/payrolls/{}",
                self.company_id, payroll_id
            ))
            .await?;
        Ok(payroll.into())
    }

    pub async fn list_payslips(&self, payroll_id: &str) -> Result<Vec<Payslip>> {
        #[derive(Deserialize)]
        struct PayrollDetail {
            employee_compensations: Vec<GustoEmployeeCompensation>,
            pay_period: GustoPayPeriod,
            check_date: String,
        }

        let detail: PayrollDetail = self
            .get(&format!(
                "/companies/{}/payrolls/{}",
                self.company_id, payroll_id
            ))
            .await?;

        Ok(detail
            .employee_compensations
            .into_iter()
            .map(|ec| {
                let pay_period_start =
                    NaiveDate::parse_from_str(&detail.pay_period.start_date, "%Y-%m-%d")
                        .unwrap_or_default();
                let pay_period_end =
                    NaiveDate::parse_from_str(&detail.pay_period.end_date, "%Y-%m-%d")
                        .unwrap_or_default();
                let pay_date =
                    NaiveDate::parse_from_str(&detail.check_date, "%Y-%m-%d").unwrap_or_default();

                ec.into_payslip(payroll_id, pay_period_start, pay_period_end, pay_date)
            })
            .collect())
    }

    pub async fn list_benefits(&self) -> Result<Vec<GustoBenefit>> {
        self.get(&format!("/companies/{}/company_benefits", self.company_id))
            .await
    }

    pub async fn list_time_off_policies(&self) -> Result<Vec<GustoTimeOffPolicy>> {
        self.get(&format!(
            "/companies/{}/time_off_policies",
            self.company_id
        ))
        .await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GustoCompany {
    pub uuid: String,
    pub name: String,
    pub trade_name: Option<String>,
    pub ein: Option<String>,
    pub entity_type: Option<String>,
    pub company_status: Option<String>,
    pub primary_signatory: Option<GustoSignatory>,
    pub primary_payroll_admin: Option<GustoAdmin>,
    pub locations: Option<Vec<GustoLocation>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GustoSignatory {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GustoAdmin {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GustoLocation {
    pub uuid: String,
    pub street_1: Option<String>,
    pub street_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    pub active: bool,
}

impl From<GustoLocation> for Location {
    fn from(l: GustoLocation) -> Self {
        Location {
            id: l.uuid,
            name: format!(
                "{}, {}",
                l.city.clone().unwrap_or_default(),
                l.state.clone().unwrap_or_default()
            ),
            address: Some(Address {
                street: l.street_1,
                street2: l.street_2,
                city: l.city,
                state: l.state,
                postal_code: l.zip,
                country: l.country,
            }),
            timezone: None,
            is_remote: false,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GustoEmployee {
    uuid: String,
    first_name: String,
    last_name: String,
    email: Option<String>,
    personal_email: Option<String>,
    phone: Option<String>,
    date_of_birth: Option<String>,
    ssn: Option<String>,
    jobs: Option<Vec<GustoJob>>,
    home_address: Option<GustoAddress>,
    terminated: bool,
    termination_date: Option<String>,
    onboarded: bool,
    two_percent_shareholder: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GustoJob {
    uuid: String,
    title: Option<String>,
    rate: Option<String>,
    payment_unit: Option<String>,
    current_compensation_uuid: Option<String>,
    hire_date: Option<String>,
    location_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GustoAddress {
    street_1: Option<String>,
    street_2: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
    country: Option<String>,
}

impl From<GustoEmployee> for Employee {
    fn from(e: GustoEmployee) -> Self {
        let job = e.jobs.as_ref().and_then(|j| j.first());

        let employment_status = if e.terminated {
            Some(EmploymentStatus::Terminated)
        } else if e.onboarded {
            Some(EmploymentStatus::Active)
        } else {
            Some(EmploymentStatus::Onboarding)
        };

        let pay_rate = job.and_then(|j| {
            j.rate.as_ref().and_then(|r| {
                r.parse::<f64>().ok().map(|amount| PayRate {
                    amount,
                    currency: "USD".to_string(),
                    period: j
                        .payment_unit
                        .as_deref()
                        .map(|u| match u.to_lowercase().as_str() {
                            "hour" => PayPeriod::Hourly,
                            "week" => PayPeriod::Weekly,
                            "month" => PayPeriod::Monthly,
                            "year" => PayPeriod::Annually,
                            _ => PayPeriod::Other,
                        })
                        .unwrap_or(PayPeriod::Annually),
                })
            })
        });

        Employee {
            id: e.uuid,
            employee_number: None,
            first_name: e.first_name,
            last_name: e.last_name,
            email: e.email.clone(),
            work_email: e.email,
            personal_email: e.personal_email,
            phone: e.phone,
            mobile_phone: None,
            job_title: job.and_then(|j| j.title.clone()),
            department: None,
            department_id: None,
            division: None,
            location: None,
            manager_id: None,
            hire_date: job.and_then(|j| {
                j.hire_date
                    .as_ref()
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            }),
            termination_date: e
                .termination_date
                .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            employment_status,
            employment_type: Some(EmploymentType::FullTime),
            pay_rate,
            avatar_url: None,
            created_at: None,
            updated_at: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GustoPayroll {
    payroll_uuid: String,
    pay_period: GustoPayPeriod,
    check_date: String,
    processed: bool,
    totals: Option<GustoPayrollTotals>,
}

#[derive(Debug, Deserialize)]
struct GustoPayPeriod {
    start_date: String,
    end_date: String,
}

#[derive(Debug, Deserialize)]
struct GustoPayrollTotals {
    gross_pay: Option<String>,
    net_pay: Option<String>,
    employer_taxes: Option<String>,
    employee_taxes: Option<String>,
    benefits: Option<String>,
}

impl From<GustoPayroll> for PayrollRun {
    fn from(p: GustoPayroll) -> Self {
        let status = if p.processed {
            PayrollStatus::Completed
        } else {
            PayrollStatus::Pending
        };

        PayrollRun {
            id: p.payroll_uuid,
            pay_period_start: NaiveDate::parse_from_str(&p.pay_period.start_date, "%Y-%m-%d")
                .unwrap_or_default(),
            pay_period_end: NaiveDate::parse_from_str(&p.pay_period.end_date, "%Y-%m-%d")
                .unwrap_or_default(),
            pay_date: NaiveDate::parse_from_str(&p.check_date, "%Y-%m-%d").unwrap_or_default(),
            status,
            total_gross: p
                .totals
                .as_ref()
                .and_then(|t| t.gross_pay.as_ref())
                .and_then(|v| v.parse().ok()),
            total_net: p
                .totals
                .as_ref()
                .and_then(|t| t.net_pay.as_ref())
                .and_then(|v| v.parse().ok()),
            total_taxes: p.totals.as_ref().and_then(|t| {
                let employer: f64 = t
                    .employer_taxes
                    .as_ref()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0);
                let employee: f64 = t
                    .employee_taxes
                    .as_ref()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0);
                Some(employer + employee)
            }),
            total_deductions: p
                .totals
                .as_ref()
                .and_then(|t| t.benefits.as_ref())
                .and_then(|v| v.parse().ok()),
            currency: Some("USD".to_string()),
            employee_count: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GustoEmployeeCompensation {
    employee_uuid: String,
    gross_pay: Option<String>,
    net_pay: Option<String>,
    fixed_compensations: Option<Vec<GustoCompensationItem>>,
    hourly_compensations: Option<Vec<GustoCompensationItem>>,
    taxes: Option<Vec<GustoTaxItem>>,
    benefits: Option<Vec<GustoBenefitItem>>,
    deductions: Option<Vec<GustoDeductionItem>>,
}

#[derive(Debug, Deserialize)]
struct GustoCompensationItem {
    name: String,
    amount: String,
}

#[derive(Debug, Deserialize)]
struct GustoTaxItem {
    name: String,
    amount: String,
    employer: bool,
}

#[derive(Debug, Deserialize)]
struct GustoBenefitItem {
    name: String,
    employee_deduction: String,
    company_contribution: String,
}

#[derive(Debug, Deserialize)]
struct GustoDeductionItem {
    name: String,
    amount: String,
}

impl GustoEmployeeCompensation {
    fn into_payslip(
        self,
        payroll_id: &str,
        pay_period_start: NaiveDate,
        pay_period_end: NaiveDate,
        pay_date: NaiveDate,
    ) -> Payslip {
        let mut earnings = Vec::new();
        if let Some(fixed) = self.fixed_compensations {
            for item in fixed {
                earnings.push(PayComponent {
                    name: item.name,
                    amount: item.amount.parse().unwrap_or(0.0),
                    category: Some("fixed".to_string()),
                });
            }
        }
        if let Some(hourly) = self.hourly_compensations {
            for item in hourly {
                earnings.push(PayComponent {
                    name: item.name,
                    amount: item.amount.parse().unwrap_or(0.0),
                    category: Some("hourly".to_string()),
                });
            }
        }

        let mut tax_items = Vec::new();
        let mut total_taxes = 0.0;
        if let Some(taxes) = self.taxes {
            for item in taxes {
                if !item.employer {
                    let amount: f64 = item.amount.parse().unwrap_or(0.0);
                    total_taxes += amount;
                    tax_items.push(PayComponent {
                        name: item.name,
                        amount,
                        category: Some("tax".to_string()),
                    });
                }
            }
        }

        let mut deduction_items = Vec::new();
        let mut total_deductions = 0.0;
        if let Some(benefits) = self.benefits {
            for item in benefits {
                let amount: f64 = item.employee_deduction.parse().unwrap_or(0.0);
                total_deductions += amount;
                deduction_items.push(PayComponent {
                    name: item.name,
                    amount,
                    category: Some("benefit".to_string()),
                });
            }
        }
        if let Some(deductions) = self.deductions {
            for item in deductions {
                let amount: f64 = item.amount.parse().unwrap_or(0.0);
                total_deductions += amount;
                deduction_items.push(PayComponent {
                    name: item.name,
                    amount,
                    category: Some("deduction".to_string()),
                });
            }
        }

        Payslip {
            id: format!("{}-{}", payroll_id, self.employee_uuid),
            employee_id: self.employee_uuid,
            payroll_run_id: Some(payroll_id.to_string()),
            pay_period_start,
            pay_period_end,
            pay_date,
            gross_pay: self.gross_pay.and_then(|v| v.parse().ok()).unwrap_or(0.0),
            net_pay: self.net_pay.and_then(|v| v.parse().ok()).unwrap_or(0.0),
            taxes: total_taxes,
            deductions: total_deductions,
            currency: "USD".to_string(),
            earnings,
            tax_items,
            deduction_items,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GustoBenefit {
    pub uuid: String,
    pub benefit_type: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub responsible_for_employer_taxes: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GustoTimeOffPolicy {
    pub uuid: String,
    pub name: String,
    pub policy_type: Option<String>,
    pub accrual_method: Option<String>,
    pub accrual_rate: Option<String>,
    pub accrual_rate_unit: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GustoTimeOffRequest {
    uuid: String,
    employee_uuid: String,
    time_off_policy_uuid: Option<String>,
    request_type: String,
    status: String,
    start_date: String,
    end_date: String,
    days: Option<f64>,
    hours: Option<f64>,
    notes: Option<String>,
    initiator_uuid: Option<String>,
    approver_uuid: Option<String>,
}

impl From<GustoTimeOffRequest> for TimeOffRequest {
    fn from(r: GustoTimeOffRequest) -> Self {
        let status = match r.status.to_lowercase().as_str() {
            "approved" => TimeOffStatus::Approved,
            "denied" | "rejected" => TimeOffStatus::Denied,
            "cancelled" | "canceled" => TimeOffStatus::Cancelled,
            _ => TimeOffStatus::Pending,
        };

        let request_type = match r.request_type.to_lowercase().as_str() {
            "vacation" => TimeOffType::Vacation,
            "sick" => TimeOffType::Sick,
            "personal" => TimeOffType::Personal,
            "parental" => TimeOffType::Parental,
            "bereavement" => TimeOffType::Bereavement,
            "jury_duty" => TimeOffType::JuryDuty,
            "holiday" => TimeOffType::Holiday,
            "unpaid" => TimeOffType::Unpaid,
            _ => TimeOffType::Other,
        };

        TimeOffRequest {
            id: r.uuid,
            employee_id: r.employee_uuid,
            policy_id: r.time_off_policy_uuid,
            policy_name: None,
            start_date: NaiveDate::parse_from_str(&r.start_date, "%Y-%m-%d").unwrap_or_default(),
            end_date: NaiveDate::parse_from_str(&r.end_date, "%Y-%m-%d").unwrap_or_default(),
            status,
            request_type,
            hours: r.hours,
            days: r.days,
            notes: r.notes,
            approver_id: r.approver_uuid,
            created_at: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GustoTimeOffBalance {
    policy_uuid: String,
    policy_name: String,
    balance: String,
    used: Option<String>,
    unit: Option<String>,
}

#[async_trait]
impl HrProvider for GustoClient {
    async fn list_employees(&self, options: ListOptions) -> Result<ListResult<Employee>> {
        let page = options.page.unwrap_or(1);
        let per_page = options.per_page.unwrap_or(25);

        let employees: Vec<GustoEmployee> = self
            .get(&format!(
                "/companies/{}/employees?page={}&per={}",
                self.company_id, page, per_page
            ))
            .await?;

        let has_more = employees.len() == per_page as usize;
        let data: Vec<Employee> = employees.into_iter().map(|e| e.into()).collect();

        Ok(ListResult {
            total: None,
            data,
            has_more,
            next_cursor: if has_more {
                Some((page + 1).to_string())
            } else {
                None
            },
        })
    }

    async fn get_employee(&self, id: &str) -> Result<Employee> {
        let emp: GustoEmployee = self.get(&format!("/employees/{}", id)).await?;
        Ok(emp.into())
    }

    async fn create_employee(&self, data: CreateEmployee) -> Result<Employee> {
        #[derive(Serialize)]
        struct CreateBody {
            first_name: String,
            last_name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            date_of_birth: Option<String>,
        }

        let body = CreateBody {
            first_name: data.first_name,
            last_name: data.last_name,
            email: data.email.or(data.work_email),
            date_of_birth: None,
        };

        let emp: GustoEmployee = self
            .post(&format!("/companies/{}/employees", self.company_id), &body)
            .await?;
        Ok(emp.into())
    }

    async fn update_employee(&self, id: &str, data: UpdateEmployee) -> Result<Employee> {
        #[derive(Serialize)]
        struct UpdateBody {
            #[serde(skip_serializing_if = "Option::is_none")]
            first_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            last_name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
        }

        let body = UpdateBody {
            first_name: data.first_name,
            last_name: data.last_name,
            email: data.email.or(data.work_email),
        };

        let emp: GustoEmployee = self.put(&format!("/employees/{}", id), &body).await?;
        Ok(emp.into())
    }

    async fn terminate_employee(&self, id: &str, termination_date: NaiveDate) -> Result<Employee> {
        #[derive(Serialize)]
        struct TerminateBody {
            effective_date: String,
        }

        let body = TerminateBody {
            effective_date: termination_date.to_string(),
        };

        let emp: GustoEmployee = self
            .put(&format!("/employees/{}/terminations", id), &body)
            .await?;
        Ok(emp.into())
    }

    async fn list_departments(&self, _options: ListOptions) -> Result<ListResult<Department>> {
        Err(Error::Provider(
            "Gusto does not have a departments API".to_string(),
        ))
    }

    async fn get_department(&self, _id: &str) -> Result<Department> {
        Err(Error::Provider(
            "Gusto does not have a departments API".to_string(),
        ))
    }

    async fn list_locations(&self, _options: ListOptions) -> Result<ListResult<Location>> {
        let company = self.get_company().await?;
        let locations: Vec<Location> = company
            .locations
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.into())
            .collect();
        let total = locations.len() as u32;

        Ok(ListResult {
            data: locations,
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_location(&self, id: &str) -> Result<Location> {
        let company = self.get_company().await?;
        company
            .locations
            .unwrap_or_default()
            .into_iter()
            .find(|l| l.uuid == id)
            .map(|l| l.into())
            .ok_or_else(|| Error::NotFound(format!("Location {}", id)))
    }

    async fn list_time_off_requests(
        &self,
        employee_id: Option<&str>,
        _options: ListOptions,
    ) -> Result<ListResult<TimeOffRequest>> {
        let path = if let Some(emp_id) = employee_id {
            format!("/employees/{}/time_off_requests", emp_id)
        } else {
            format!("/companies/{}/time_off_requests", self.company_id)
        };

        let requests: Vec<GustoTimeOffRequest> = self.get(&path).await?;
        let total = requests.len() as u32;

        Ok(ListResult {
            data: requests.into_iter().map(|r| r.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        let req: GustoTimeOffRequest = self.get(&format!("/time_off_requests/{}", id)).await?;
        Ok(req.into())
    }

    async fn create_time_off_request(&self, data: CreateTimeOffRequest) -> Result<TimeOffRequest> {
        #[derive(Serialize)]
        struct CreateBody {
            time_off_policy_uuid: Option<String>,
            start_date: String,
            end_date: String,
            hours: Option<f64>,
            request_type: String,
            notes: Option<String>,
        }

        let request_type = match data.request_type {
            TimeOffType::Vacation => "vacation",
            TimeOffType::Sick => "sick",
            TimeOffType::Personal => "personal",
            TimeOffType::Parental => "parental",
            TimeOffType::Bereavement => "bereavement",
            TimeOffType::JuryDuty => "jury_duty",
            TimeOffType::Holiday => "holiday",
            TimeOffType::Unpaid => "unpaid",
            TimeOffType::Other => "other",
        };

        let body = CreateBody {
            time_off_policy_uuid: data.policy_id,
            start_date: data.start_date.to_string(),
            end_date: data.end_date.to_string(),
            hours: data.hours,
            request_type: request_type.to_string(),
            notes: data.notes,
        };

        let req: GustoTimeOffRequest = self
            .post(
                &format!("/employees/{}/time_off_requests", data.employee_id),
                &body,
            )
            .await?;
        Ok(req.into())
    }

    async fn approve_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        let req: GustoTimeOffRequest = self
            .put(
                &format!("/time_off_requests/{}/approve", id),
                &serde_json::json!({}),
            )
            .await?;
        Ok(req.into())
    }

    async fn deny_time_off_request(&self, id: &str) -> Result<TimeOffRequest> {
        let req: GustoTimeOffRequest = self
            .put(
                &format!("/time_off_requests/{}/deny", id),
                &serde_json::json!({}),
            )
            .await?;
        Ok(req.into())
    }

    async fn get_time_off_balances(&self, employee_id: &str) -> Result<Vec<TimeOffBalance>> {
        let balances: Vec<GustoTimeOffBalance> = self
            .get(&format!("/employees/{}/time_off_balances", employee_id))
            .await?;

        Ok(balances
            .into_iter()
            .map(|b| TimeOffBalance {
                employee_id: employee_id.to_string(),
                policy_id: b.policy_uuid,
                policy_name: b.policy_name,
                balance: b.balance.parse().unwrap_or(0.0),
                used: b.used.and_then(|u| u.parse().ok()).unwrap_or(0.0),
                pending: 0.0,
                unit: b
                    .unit
                    .as_deref()
                    .map(|u| match u.to_lowercase().as_str() {
                        "hours" => TimeOffUnit::Hours,
                        "days" => TimeOffUnit::Days,
                        _ => TimeOffUnit::Other,
                    })
                    .unwrap_or(TimeOffUnit::Hours),
                extra: HashMap::new(),
            })
            .collect())
    }
}
