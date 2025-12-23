#![recursion_limit = "256"]

mod error;

pub use error::{Error, Result};

pub mod backend;

#[cfg(feature = "plaid")]
pub mod plaid;

#[cfg(feature = "truelayer")]
pub mod truelayer;

#[cfg(feature = "teller")]
pub mod teller;

#[cfg(feature = "gocardless")]
pub mod gocardless;

#[cfg(feature = "yapily")]
pub mod yapily;

#[cfg(feature = "mx")]
pub mod mx;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[async_trait]
pub trait BankingProvider: Send + Sync {
    async fn create_link_token(&self, request: CreateLinkTokenRequest) -> Result<LinkToken>;
    async fn exchange_public_token(&self, public_token: &str) -> Result<AccessToken>;

    async fn list_accounts(&self, access_token: &str) -> Result<Vec<Account>>;
    async fn get_account(&self, access_token: &str, account_id: &str) -> Result<Account>;
    async fn get_balances(&self, access_token: &str) -> Result<Vec<AccountBalance>>;

    async fn list_transactions(
        &self,
        access_token: &str,
        options: TransactionOptions,
    ) -> Result<TransactionList>;

    async fn get_transaction(
        &self,
        access_token: &str,
        transaction_id: &str,
    ) -> Result<Transaction>;

    async fn list_institutions(&self, options: InstitutionOptions) -> Result<Vec<Institution>>;
    async fn get_institution(&self, institution_id: &str) -> Result<Institution>;

    async fn get_identity(&self, access_token: &str) -> Result<Vec<AccountIdentity>>;

    async fn create_payment(
        &self,
        access_token: &str,
        request: CreatePaymentRequest,
    ) -> Result<Payment>;
    async fn get_payment(&self, access_token: &str, payment_id: &str) -> Result<Payment>;

    async fn remove_item(&self, access_token: &str) -> Result<()>;

    async fn get_item(&self, access_token: &str) -> Result<Item>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLinkTokenRequest {
    pub user_id: String,
    pub client_name: String,
    pub products: Vec<Product>,
    pub country_codes: Vec<String>,
    pub language: Option<String>,
    pub webhook: Option<String>,
    pub redirect_uri: Option<String>,
    pub account_filters: Option<AccountFilters>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountFilters {
    pub depository: Option<AccountSubtypeFilter>,
    pub credit: Option<AccountSubtypeFilter>,
    pub loan: Option<AccountSubtypeFilter>,
    pub investment: Option<AccountSubtypeFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSubtypeFilter {
    pub account_subtypes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkToken {
    pub link_token: String,
    pub expiration: DateTime<Utc>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub item_id: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Product {
    Transactions,
    Auth,
    Identity,
    Assets,
    Investments,
    Liabilities,
    PaymentInitiation,
    IdentityVerification,
    Transfer,
    Employment,
    Income,
    Standing,
    RecurringTransactions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub official_name: Option<String>,
    pub account_type: AccountType,
    pub subtype: Option<String>,
    pub mask: Option<String>,
    pub currency: String,
    pub institution_id: Option<String>,
    pub balances: AccountBalance,
    pub account_number: Option<AccountNumber>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Depository,
    Credit,
    Loan,
    Investment,
    Brokerage,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account_id: String,
    pub current: f64,
    pub available: Option<f64>,
    pub limit: Option<f64>,
    pub currency: String,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountNumber {
    pub account_number: Option<String>,
    pub routing_number: Option<String>,
    pub sort_code: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub amount: f64,
    pub currency: String,
    pub date: NaiveDate,
    pub datetime: Option<DateTime<Utc>>,
    pub name: String,
    pub merchant_name: Option<String>,
    pub merchant_id: Option<String>,
    pub category: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub pending: bool,
    pub transaction_type: TransactionType,
    pub payment_channel: Option<PaymentChannel>,
    pub location: Option<TransactionLocation>,
    pub counterparty: Option<Counterparty>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Debit,
    Credit,
    Transfer,
    Fee,
    Interest,
    Dividend,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentChannel {
    Online,
    InStore,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLocation {
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub store_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterparty {
    pub name: Option<String>,
    pub entity_id: Option<String>,
    #[serde(rename = "type")]
    pub counterparty_type: Option<String>,
    pub logo_url: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionOptions {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub account_ids: Option<Vec<String>>,
    pub count: Option<u32>,
    pub offset: Option<u32>,
    pub include_pending: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionList {
    pub transactions: Vec<Transaction>,
    pub total_transactions: u32,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub logo: Option<String>,
    pub primary_color: Option<String>,
    pub country_codes: Vec<String>,
    pub products: Vec<Product>,
    pub routing_numbers: Option<Vec<String>>,
    pub oauth: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstitutionOptions {
    pub country_codes: Option<Vec<String>>,
    pub products: Option<Vec<Product>>,
    pub query: Option<String>,
    pub count: Option<u32>,
    pub offset: Option<u32>,
    pub include_optional_metadata: Option<bool>,
    pub oauth: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountIdentity {
    pub account_id: String,
    pub owners: Vec<Owner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub names: Vec<String>,
    pub phone_numbers: Vec<PhoneNumber>,
    pub emails: Vec<Email>,
    pub addresses: Vec<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub data: String,
    pub primary: bool,
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub data: String,
    pub primary: bool,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub recipient_id: Option<String>,
    pub recipient: Option<PaymentRecipient>,
    pub amount: PaymentAmount,
    pub reference: String,
    pub schedule: Option<PaymentSchedule>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecipient {
    pub name: String,
    pub iban: Option<String>,
    pub account_number: Option<String>,
    pub sort_code: Option<String>,
    pub routing_number: Option<String>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAmount {
    pub value: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSchedule {
    pub interval: PaymentInterval,
    pub interval_execution_day: Option<u8>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentInterval {
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub status: PaymentStatus,
    pub amount: PaymentAmount,
    pub reference: String,
    pub recipient_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_status_update: Option<DateTime<Utc>>,
    pub schedule: Option<PaymentSchedule>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    InputNeeded,
    Processing,
    Initiated,
    Completed,
    InsufficientFunds,
    Failed,
    Blocked,
    Cancelled,
    Established,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub institution_id: Option<String>,
    pub webhook: Option<String>,
    pub error: Option<ItemError>,
    pub available_products: Vec<Product>,
    pub billed_products: Vec<Product>,
    pub consent_expiration_time: Option<DateTime<Utc>>,
    pub update_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemError {
    pub error_type: String,
    pub error_code: String,
    pub error_message: String,
    pub display_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub webhook_type: String,
    pub webhook_code: String,
    pub item_id: Option<String>,
    pub error: Option<ItemError>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentHolding {
    pub account_id: String,
    pub security_id: String,
    pub institution_value: f64,
    pub institution_price: f64,
    pub quantity: f64,
    pub cost_basis: Option<f64>,
    pub currency: String,
    pub iso_currency_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    pub id: String,
    pub name: Option<String>,
    pub ticker_symbol: Option<String>,
    pub isin: Option<String>,
    pub cusip: Option<String>,
    pub sedol: Option<String>,
    pub security_type: Option<String>,
    pub close_price: Option<f64>,
    pub close_price_as_of: Option<NaiveDate>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTransaction {
    pub id: String,
    pub account_id: String,
    pub stream_id: String,
    pub category: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub description: String,
    pub merchant_name: Option<String>,
    pub first_date: NaiveDate,
    pub last_date: NaiveDate,
    pub frequency: RecurringFrequency,
    pub average_amount: PaymentAmount,
    pub last_amount: PaymentAmount,
    pub is_active: bool,
    pub status: RecurringStatus,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecurringFrequency {
    Weekly,
    Biweekly,
    SemiMonthly,
    Monthly,
    Annually,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecurringStatus {
    Mature,
    EarlyDetection,
    TombStoned,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liability {
    pub account_id: String,
    pub liability_type: LiabilityType,
    pub balance: f64,
    pub currency: String,
    pub minimum_payment_amount: Option<f64>,
    pub next_payment_due_date: Option<NaiveDate>,
    pub interest_rate: Option<InterestRate>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiabilityType {
    Credit,
    Mortgage,
    Student,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestRate {
    pub percentage: f64,
    #[serde(rename = "type")]
    pub rate_type: String,
}
