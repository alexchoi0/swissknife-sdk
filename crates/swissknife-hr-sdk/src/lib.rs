mod error;

pub use error::{Error, Result};

#[cfg(feature = "bamboohr")]
pub mod bamboohr;

#[cfg(feature = "gusto")]
pub mod gusto;

#[cfg(feature = "workday")]
pub mod workday;

#[cfg(feature = "deel")]
pub mod deel;

#[cfg(feature = "personio")]
pub mod personio;

#[cfg(feature = "hibob")]
pub mod hibob;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: String,
    pub employee_number: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub personal_email: Option<String>,
    pub phone: Option<String>,
    pub mobile_phone: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub department_id: Option<String>,
    pub division: Option<String>,
    pub location: Option<String>,
    pub manager_id: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub employment_status: Option<EmploymentStatus>,
    pub employment_type: Option<EmploymentType>,
    pub pay_rate: Option<PayRate>,
    pub avatar_url: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmploymentStatus {
    Active,
    Inactive,
    Onboarding,
    Terminated,
    OnLeave,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Temporary,
    Intern,
    Freelance,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayRate {
    pub amount: f64,
    pub currency: String,
    pub period: PayPeriod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PayPeriod {
    Hourly,
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Annually,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub manager_id: Option<String>,
    pub employee_count: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub lead_id: Option<String>,
    pub member_ids: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub address: Option<Address>,
    pub timezone: Option<String>,
    pub is_remote: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub street2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeOffRequest {
    pub id: String,
    pub employee_id: String,
    pub policy_id: Option<String>,
    pub policy_name: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: TimeOffStatus,
    pub request_type: TimeOffType,
    pub hours: Option<f64>,
    pub days: Option<f64>,
    pub notes: Option<String>,
    pub approver_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeOffStatus {
    Pending,
    Approved,
    Denied,
    Cancelled,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeOffType {
    Vacation,
    Sick,
    Personal,
    Parental,
    Bereavement,
    JuryDuty,
    Holiday,
    Unpaid,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeOffBalance {
    pub employee_id: String,
    pub policy_id: String,
    pub policy_name: String,
    pub balance: f64,
    pub used: f64,
    pub pending: f64,
    pub unit: TimeOffUnit,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeOffUnit {
    Hours,
    Days,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollRun {
    pub id: String,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub pay_date: NaiveDate,
    pub status: PayrollStatus,
    pub total_gross: Option<f64>,
    pub total_net: Option<f64>,
    pub total_taxes: Option<f64>,
    pub total_deductions: Option<f64>,
    pub currency: Option<String>,
    pub employee_count: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PayrollStatus {
    Draft,
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payslip {
    pub id: String,
    pub employee_id: String,
    pub payroll_run_id: Option<String>,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub pay_date: NaiveDate,
    pub gross_pay: f64,
    pub net_pay: f64,
    pub taxes: f64,
    pub deductions: f64,
    pub currency: String,
    pub earnings: Vec<PayComponent>,
    pub tax_items: Vec<PayComponent>,
    pub deduction_items: Vec<PayComponent>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayComponent {
    pub name: String,
    pub amount: f64,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benefit {
    pub id: String,
    pub name: String,
    pub benefit_type: BenefitType,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub employee_cost: Option<f64>,
    pub employer_cost: Option<f64>,
    pub currency: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenefitType {
    Health,
    Dental,
    Vision,
    Life,
    Disability,
    Retirement,
    Hsa,
    Fsa,
    Commuter,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitEnrollment {
    pub id: String,
    pub employee_id: String,
    pub benefit_id: String,
    pub benefit_name: String,
    pub plan_name: Option<String>,
    pub coverage_level: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub employee_contribution: Option<f64>,
    pub employer_contribution: Option<f64>,
    pub currency: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPosting {
    pub id: String,
    pub title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: Option<EmploymentType>,
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub salary_min: Option<f64>,
    pub salary_max: Option<f64>,
    pub currency: Option<String>,
    pub status: JobPostingStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobPostingStatus {
    Draft,
    Open,
    Closed,
    OnHold,
    Filled,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListResult<T> {
    pub data: Vec<T>,
    pub total: Option<u32>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateEmployee {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone: Option<String>,
    pub job_title: Option<String>,
    pub department_id: Option<String>,
    pub manager_id: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub employment_type: Option<EmploymentType>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateEmployee {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone: Option<String>,
    pub job_title: Option<String>,
    pub department_id: Option<String>,
    pub manager_id: Option<String>,
    pub employment_type: Option<EmploymentType>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeOffRequest {
    pub employee_id: String,
    pub policy_id: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub request_type: TimeOffType,
    pub hours: Option<f64>,
    pub notes: Option<String>,
}

#[async_trait]
pub trait HrProvider: Send + Sync {
    async fn list_employees(&self, options: ListOptions) -> Result<ListResult<Employee>>;
    async fn get_employee(&self, id: &str) -> Result<Employee>;
    async fn create_employee(&self, data: CreateEmployee) -> Result<Employee>;
    async fn update_employee(&self, id: &str, data: UpdateEmployee) -> Result<Employee>;
    async fn terminate_employee(&self, id: &str, termination_date: NaiveDate) -> Result<Employee>;

    async fn list_departments(&self, options: ListOptions) -> Result<ListResult<Department>>;
    async fn get_department(&self, id: &str) -> Result<Department>;

    async fn list_locations(&self, options: ListOptions) -> Result<ListResult<Location>>;
    async fn get_location(&self, id: &str) -> Result<Location>;

    async fn list_time_off_requests(&self, employee_id: Option<&str>, options: ListOptions) -> Result<ListResult<TimeOffRequest>>;
    async fn get_time_off_request(&self, id: &str) -> Result<TimeOffRequest>;
    async fn create_time_off_request(&self, data: CreateTimeOffRequest) -> Result<TimeOffRequest>;
    async fn approve_time_off_request(&self, id: &str) -> Result<TimeOffRequest>;
    async fn deny_time_off_request(&self, id: &str) -> Result<TimeOffRequest>;

    async fn get_time_off_balances(&self, employee_id: &str) -> Result<Vec<TimeOffBalance>>;
}
