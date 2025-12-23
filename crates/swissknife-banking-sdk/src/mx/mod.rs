use crate::{
    AccessToken, Account, AccountBalance, AccountIdentity, BankingProvider, CreateLinkTokenRequest,
    CreatePaymentRequest, Institution, InstitutionOptions, Item, LinkToken, Payment, Transaction,
    TransactionList, TransactionOptions, Error, Result,
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct MxClient {
    client: Client,
    client_id: String,
    api_key: String,
    base_url: String,
}

impl MxClient {
    pub fn new(client_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self::with_environment(client_id, api_key, MxEnvironment::Development)
    }

    pub fn with_environment(
        client_id: impl Into<String>,
        api_key: impl Into<String>,
        environment: MxEnvironment,
    ) -> Self {
        Self {
            client: Client::new(),
            client_id: client_id.into(),
            api_key: api_key.into(),
            base_url: environment.base_url().to_string(),
        }
    }

    pub fn production(client_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self::with_environment(client_id, api_key, MxEnvironment::Production)
    }

    fn basic_auth(&self) -> String {
        let credentials = format!("{}:{}", self.client_id, self.api_key);
        format!("Basic {}", BASE64.encode(credentials.as_bytes()))
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/vnd.mx.api.v1+json")
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: MxError = response.json().await?;
            Err(Error::Api {
                code: error.error.as_ref().map(|e| e.error_type.clone()).unwrap_or_else(|| "unknown".to_string()),
                message: error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/vnd.mx.api.v1+json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: MxError = response.json().await?;
            Err(Error::Api {
                code: error.error.as_ref().map(|e| e.error_type.clone()).unwrap_or_else(|| "unknown".to_string()),
                message: error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn delete(&self, endpoint: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/vnd.mx.api.v1+json")
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            let error: MxError = response.json().await?;
            Err(Error::Api {
                code: error.error.as_ref().map(|e| e.error_type.clone()).unwrap_or_else(|| "unknown".to_string()),
                message: error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MxEnvironment {
    Development,
    Production,
}

impl MxEnvironment {
    fn base_url(&self) -> &'static str {
        match self {
            MxEnvironment::Development => "https://int-api.mx.com",
            MxEnvironment::Production => "https://api.mx.com",
        }
    }
}

#[derive(Deserialize)]
struct MxError {
    error: Option<MxErrorDetail>,
}

#[derive(Deserialize)]
struct MxErrorDetail {
    error_type: String,
    message: String,
}

#[derive(Clone, Deserialize)]
struct MxPagination {
    current_page: u32,
    per_page: u32,
    total_entries: u32,
    total_pages: u32,
}

#[derive(Serialize)]
struct CreateUserRequest {
    user: CreateUserData,
}

#[derive(Serialize)]
struct CreateUserData {
    id: Option<String>,
    is_disabled: Option<bool>,
    metadata: Option<String>,
}

#[derive(Deserialize)]
struct UserResponse {
    user: MxUser,
}

#[derive(Deserialize)]
struct MxUser {
    guid: String,
    id: Option<String>,
    is_disabled: Option<bool>,
    metadata: Option<String>,
}

#[derive(Serialize)]
struct ConnectWidgetRequest {
    widget_url: ConnectWidgetData,
}

#[derive(Serialize)]
struct ConnectWidgetData {
    #[serde(skip_serializing_if = "Option::is_none")]
    color_scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_institution_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_member_guid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_institution_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_transactions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ui_message_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    update_credentials: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for_full_aggregation: Option<bool>,
}

#[derive(Deserialize)]
struct WidgetUrlResponse {
    widget_url: WidgetUrl,
}

#[derive(Deserialize)]
struct WidgetUrl {
    type_: Option<String>,
    url: String,
    user_id: String,
}

#[derive(Deserialize)]
struct MembersResponse {
    members: Vec<MxMember>,
    pagination: Option<MxPagination>,
}

#[derive(Deserialize)]
struct MemberResponse {
    member: MxMember,
}

#[derive(Deserialize)]
struct MxMember {
    aggregated_at: Option<String>,
    background_aggregation_is_disabled: Option<bool>,
    connection_status: Option<String>,
    guid: String,
    id: Option<String>,
    institution_code: Option<String>,
    is_being_aggregated: Option<bool>,
    is_managed_by_user: Option<bool>,
    is_oauth: Option<bool>,
    metadata: Option<String>,
    name: Option<String>,
    oauth_window_uri: Option<String>,
    successfully_aggregated_at: Option<String>,
    user_guid: String,
    user_id: Option<String>,
}

#[derive(Deserialize)]
struct AccountsResponse {
    accounts: Vec<MxAccount>,
    pagination: Option<MxPagination>,
}

#[derive(Deserialize)]
struct AccountResponse {
    account: MxAccount,
}

#[derive(Deserialize)]
struct MxAccount {
    account_number: Option<String>,
    apr: Option<f64>,
    apy: Option<f64>,
    available_balance: Option<f64>,
    available_credit: Option<f64>,
    balance: Option<f64>,
    cash_balance: Option<f64>,
    cash_surrender_value: Option<f64>,
    created_at: Option<String>,
    credit_limit: Option<f64>,
    currency_code: Option<String>,
    day_payment_is_due: Option<u8>,
    death_benefit: Option<f64>,
    guid: String,
    holdings_value: Option<f64>,
    id: Option<String>,
    imported_at: Option<String>,
    institution_code: Option<String>,
    insured_name: Option<String>,
    interest_rate: Option<f64>,
    is_closed: Option<bool>,
    is_hidden: Option<bool>,
    last_payment: Option<f64>,
    last_payment_at: Option<String>,
    loan_amount: Option<f64>,
    matures_on: Option<String>,
    member_guid: String,
    member_id: Option<String>,
    member_is_managed_by_user: Option<bool>,
    metadata: Option<String>,
    minimum_balance: Option<f64>,
    minimum_payment: Option<f64>,
    name: Option<String>,
    nickname: Option<String>,
    original_balance: Option<f64>,
    pay_out_amount: Option<f64>,
    payment_due_at: Option<String>,
    payoff_balance: Option<f64>,
    premium_amount: Option<f64>,
    routing_number: Option<String>,
    started_on: Option<String>,
    subtype: Option<String>,
    total_account_value: Option<f64>,
    #[serde(rename = "type")]
    account_type: Option<String>,
    updated_at: Option<String>,
    user_guid: String,
    user_id: Option<String>,
}

#[derive(Deserialize)]
struct TransactionsResponse {
    transactions: Vec<MxTransaction>,
    pagination: Option<MxPagination>,
}

#[derive(Deserialize)]
struct TransactionResponse {
    transaction: MxTransaction,
}

#[derive(Deserialize)]
struct MxTransaction {
    account_guid: String,
    account_id: Option<String>,
    amount: f64,
    category: Option<String>,
    category_guid: Option<String>,
    check_number_string: Option<String>,
    created_at: Option<String>,
    currency_code: Option<String>,
    date: Option<String>,
    description: Option<String>,
    extended_transaction_type: Option<String>,
    guid: String,
    id: Option<String>,
    is_bill_pay: Option<bool>,
    is_direct_deposit: Option<bool>,
    is_expense: Option<bool>,
    is_fee: Option<bool>,
    is_income: Option<bool>,
    is_international: Option<bool>,
    is_overdraft_fee: Option<bool>,
    is_payroll_advance: Option<bool>,
    is_recurring: Option<bool>,
    is_subscription: Option<bool>,
    latitude: Option<f64>,
    localized_description: Option<String>,
    localized_memo: Option<String>,
    longitude: Option<f64>,
    member_guid: String,
    member_is_managed_by_user: Option<bool>,
    memo: Option<String>,
    merchant_category_code: Option<u32>,
    merchant_guid: Option<String>,
    merchant_location_guid: Option<String>,
    metadata: Option<String>,
    original_description: Option<String>,
    posted_at: Option<String>,
    status: Option<String>,
    top_level_category: Option<String>,
    transacted_at: Option<String>,
    #[serde(rename = "type")]
    transaction_type: Option<String>,
    updated_at: Option<String>,
    user_guid: String,
    user_id: Option<String>,
}

#[derive(Deserialize)]
struct InstitutionsResponse {
    institutions: Vec<MxInstitution>,
    pagination: Option<MxPagination>,
}

#[derive(Deserialize)]
struct InstitutionResponse {
    institution: MxInstitution,
}

#[derive(Deserialize)]
struct MxInstitution {
    code: String,
    medium_logo_url: Option<String>,
    name: String,
    small_logo_url: Option<String>,
    supports_account_identification: Option<bool>,
    supports_account_statement: Option<bool>,
    supports_account_verification: Option<bool>,
    supports_oauth: Option<bool>,
    supports_transaction_history: Option<bool>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct AccountOwnersResponse {
    account_owners: Vec<MxAccountOwner>,
}

#[derive(Deserialize)]
struct MxAccountOwner {
    account_guid: String,
    address: Option<String>,
    city: Option<String>,
    country: Option<String>,
    email: Option<String>,
    first_name: Option<String>,
    guid: String,
    last_name: Option<String>,
    member_guid: String,
    owner_name: Option<String>,
    phone: Option<String>,
    postal_code: Option<String>,
    state: Option<String>,
}

impl MxClient {
    fn convert_account(&self, account: MxAccount) -> Account {
        let currency = account.currency_code.clone().unwrap_or_else(|| "USD".to_string());
        let current = account.balance.or(account.total_account_value).unwrap_or(0.0);

        Account {
            id: account.guid.clone(),
            name: account.name.clone()
                .or(account.nickname.clone())
                .unwrap_or_else(|| "Account".to_string()),
            official_name: account.name,
            account_type: match account.account_type.as_deref() {
                Some("CHECKING") | Some("SAVINGS") | Some("MONEY_MARKET") => crate::AccountType::Depository,
                Some("CREDIT_CARD") | Some("LINE_OF_CREDIT") => crate::AccountType::Credit,
                Some("LOAN") | Some("MORTGAGE") | Some("AUTO") | Some("STUDENT_LOAN") => crate::AccountType::Loan,
                Some("INVESTMENT") | Some("BROKERAGE") | Some("RETIREMENT") | Some("401K") | Some("IRA") => crate::AccountType::Investment,
                _ => crate::AccountType::Other,
            },
            subtype: account.subtype.or(account.account_type),
            mask: account.account_number.as_ref().map(|n| {
                n.chars().rev().take(4).collect::<String>().chars().rev().collect()
            }),
            currency: currency.clone(),
            institution_id: account.institution_code,
            balances: AccountBalance {
                account_id: account.guid,
                current,
                available: account.available_balance.or(account.available_credit),
                limit: account.credit_limit,
                currency,
                last_updated: account.updated_at.and_then(|dt| {
                    chrono::DateTime::parse_from_rfc3339(&dt).ok().map(|d| d.with_timezone(&chrono::Utc))
                }),
            },
            account_number: Some(crate::AccountNumber {
                account_number: account.account_number,
                routing_number: account.routing_number,
                sort_code: None,
                iban: None,
                bic: None,
            }),
            extra: HashMap::new(),
        }
    }

    fn convert_transaction(&self, tx: MxTransaction) -> Transaction {
        let is_expense = tx.is_expense.unwrap_or(tx.amount < 0.0);
        let currency = tx.currency_code.clone().unwrap_or_else(|| "USD".to_string());

        Transaction {
            id: tx.guid,
            account_id: tx.account_guid,
            amount: tx.amount.abs(),
            currency,
            date: tx.date
                .or(tx.transacted_at.as_ref().map(|d| d.split('T').next().unwrap_or(d).to_string()))
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                .unwrap_or_else(|| chrono::Utc::now().date_naive()),
            datetime: tx.transacted_at.as_ref()
                .and_then(|dt| chrono::DateTime::parse_from_rfc3339(dt).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            name: tx.description.clone()
                .or(tx.original_description.clone())
                .unwrap_or_else(|| "Transaction".to_string()),
            merchant_name: tx.localized_description.or(tx.description),
            merchant_id: tx.merchant_guid,
            category: tx.category.map(|c| vec![c]),
            category_id: tx.category_guid,
            pending: tx.status.as_deref() == Some("PENDING"),
            transaction_type: if is_expense {
                crate::TransactionType::Debit
            } else if tx.is_income.unwrap_or(false) {
                crate::TransactionType::Credit
            } else if tx.is_fee.unwrap_or(false) {
                crate::TransactionType::Fee
            } else {
                crate::TransactionType::Other
            },
            payment_channel: Some(if tx.is_international.unwrap_or(false) {
                crate::PaymentChannel::Other
            } else {
                crate::PaymentChannel::Other
            }),
            location: if tx.latitude.is_some() || tx.longitude.is_some() {
                Some(crate::TransactionLocation {
                    address: None,
                    city: None,
                    region: None,
                    postal_code: None,
                    country: None,
                    lat: tx.latitude,
                    lon: tx.longitude,
                    store_number: None,
                })
            } else {
                None
            },
            counterparty: None,
            extra: HashMap::new(),
        }
    }

    fn convert_institution(&self, inst: MxInstitution) -> Institution {
        let mut products = vec![];
        if inst.supports_transaction_history.unwrap_or(false) {
            products.push(crate::Product::Transactions);
        }
        if inst.supports_account_identification.unwrap_or(false) || inst.supports_account_verification.unwrap_or(false) {
            products.push(crate::Product::Auth);
        }

        Institution {
            id: inst.code,
            name: inst.name,
            url: inst.url,
            logo: inst.medium_logo_url.or(inst.small_logo_url),
            primary_color: None,
            country_codes: vec!["US".to_string()],
            products,
            routing_numbers: None,
            oauth: inst.supports_oauth.unwrap_or(false),
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl BankingProvider for MxClient {
    async fn create_link_token(&self, request: CreateLinkTokenRequest) -> Result<LinkToken> {
        let user_response: UserResponse = match self
            .post("/users", &CreateUserRequest {
                user: CreateUserData {
                    id: Some(request.user_id.clone()),
                    is_disabled: Some(false),
                    metadata: None,
                },
            })
            .await
        {
            Ok(response) => response,
            Err(_) => self.get::<UserResponse>(&format!("/users/{}", request.user_id)).await?,
        };

        let user_guid = user_response.user.guid;

        let widget_response: WidgetUrlResponse = self
            .post(
                &format!("/users/{}/connect_widget_url", user_guid),
                &ConnectWidgetRequest {
                    widget_url: ConnectWidgetData {
                        color_scheme: None,
                        current_institution_code: request.extra.get("institution_code")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        current_member_guid: None,
                        disable_institution_search: None,
                        include_transactions: Some(true),
                        mode: Some("verification".to_string()),
                        ui_message_version: Some(4),
                        update_credentials: None,
                        wait_for_full_aggregation: Some(false),
                    },
                },
            )
            .await?;

        Ok(LinkToken {
            link_token: widget_response.widget_url.url,
            expiration: chrono::Utc::now() + chrono::Duration::minutes(10),
            request_id: Some(user_guid),
        })
    }

    async fn exchange_public_token(&self, user_guid: &str) -> Result<AccessToken> {
        Ok(AccessToken {
            access_token: user_guid.to_string(),
            item_id: None,
            request_id: None,
        })
    }

    async fn list_accounts(&self, user_guid: &str) -> Result<Vec<Account>> {
        let response: AccountsResponse = self
            .get(&format!("/users/{}/accounts", user_guid))
            .await?;

        Ok(response.accounts.into_iter().map(|a| self.convert_account(a)).collect())
    }

    async fn get_account(&self, user_guid: &str, account_id: &str) -> Result<Account> {
        let response: AccountResponse = self
            .get(&format!("/users/{}/accounts/{}", user_guid, account_id))
            .await?;

        Ok(self.convert_account(response.account))
    }

    async fn get_balances(&self, user_guid: &str) -> Result<Vec<AccountBalance>> {
        let accounts = self.list_accounts(user_guid).await?;
        Ok(accounts.into_iter().map(|a| a.balances).collect())
    }

    async fn list_transactions(
        &self,
        user_guid: &str,
        options: TransactionOptions,
    ) -> Result<TransactionList> {
        let today = chrono::Utc::now().date_naive();
        let from_date = options.start_date.unwrap_or_else(|| today - chrono::Duration::days(90));
        let to_date = options.end_date.unwrap_or(today);

        let mut endpoint = format!(
            "/users/{}/transactions?from_date={}&to_date={}",
            user_guid,
            from_date.format("%Y-%m-%d"),
            to_date.format("%Y-%m-%d")
        );

        if let Some(count) = options.count {
            endpoint.push_str(&format!("&records_per_page={}", count));
        }

        if let Some(offset) = options.offset {
            let page = (offset / options.count.unwrap_or(25)) + 1;
            endpoint.push_str(&format!("&page={}", page));
        }

        let response: TransactionsResponse = self.get(&endpoint).await?;

        let mut transactions: Vec<Transaction> = response
            .transactions
            .into_iter()
            .map(|t| self.convert_transaction(t))
            .collect();

        if let Some(ref account_ids) = options.account_ids {
            transactions.retain(|t| account_ids.contains(&t.account_id));
        }

        let total = response.pagination.as_ref().map(|p| p.total_entries).unwrap_or(transactions.len() as u32);
        let has_more = response.pagination.as_ref()
            .map(|p| p.current_page < p.total_pages)
            .unwrap_or(false);

        Ok(TransactionList {
            transactions,
            total_transactions: total,
            has_more,
            next_cursor: None,
        })
    }

    async fn get_transaction(
        &self,
        user_guid: &str,
        transaction_id: &str,
    ) -> Result<Transaction> {
        let response: TransactionResponse = self
            .get(&format!("/users/{}/transactions/{}", user_guid, transaction_id))
            .await?;

        Ok(self.convert_transaction(response.transaction))
    }

    async fn list_institutions(&self, options: InstitutionOptions) -> Result<Vec<Institution>> {
        let mut endpoint = "/institutions".to_string();

        if let Some(query) = options.query {
            endpoint.push_str(&format!("?name={}", urlencoding::encode(&query)));
        }

        let response: InstitutionsResponse = self.get(&endpoint).await?;

        Ok(response
            .institutions
            .into_iter()
            .map(|i| self.convert_institution(i))
            .collect())
    }

    async fn get_institution(&self, institution_code: &str) -> Result<Institution> {
        let response: InstitutionResponse = self
            .get(&format!("/institutions/{}", institution_code))
            .await?;

        Ok(self.convert_institution(response.institution))
    }

    async fn get_identity(&self, user_guid: &str) -> Result<Vec<AccountIdentity>> {
        let members: MembersResponse = self
            .get(&format!("/users/{}/members", user_guid))
            .await?;

        let mut identities = Vec::new();

        for member in members.members {
            let endpoint = format!("/users/{}/members/{}/account_owners", user_guid, member.guid);
            if let Ok(response) = self.get::<AccountOwnersResponse>(&endpoint).await {
                for owner in response.account_owners {
                    let name = owner.owner_name.or_else(|| {
                        match (&owner.first_name, &owner.last_name) {
                            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                            (Some(first), None) => Some(first.clone()),
                            (None, Some(last)) => Some(last.clone()),
                            _ => None,
                        }
                    });

                    identities.push(AccountIdentity {
                        account_id: owner.account_guid,
                        owners: vec![crate::Owner {
                            names: name.into_iter().collect(),
                            phone_numbers: owner.phone.map(|p| vec![crate::PhoneNumber {
                                data: p,
                                primary: true,
                                phone_type: None,
                            }]).unwrap_or_default(),
                            emails: owner.email.map(|e| vec![crate::Email {
                                data: e,
                                primary: true,
                                email_type: None,
                            }]).unwrap_or_default(),
                            addresses: vec![crate::Address {
                                street: owner.address,
                                city: owner.city,
                                region: owner.state,
                                postal_code: owner.postal_code,
                                country: owner.country,
                                primary: true,
                            }],
                        }],
                    });
                }
            }
        }

        Ok(identities)
    }

    async fn create_payment(
        &self,
        _user_guid: &str,
        _request: CreatePaymentRequest,
    ) -> Result<Payment> {
        Err(Error::Provider(
            "MX does not support payment initiation".to_string(),
        ))
    }

    async fn get_payment(&self, _user_guid: &str, _payment_id: &str) -> Result<Payment> {
        Err(Error::Provider(
            "MX does not support payment initiation".to_string(),
        ))
    }

    async fn remove_item(&self, user_guid: &str) -> Result<()> {
        self.delete(&format!("/users/{}", user_guid)).await
    }

    async fn get_item(&self, user_guid: &str) -> Result<Item> {
        let members: MembersResponse = self
            .get(&format!("/users/{}/members", user_guid))
            .await?;

        let member = members.members.into_iter().next();

        Ok(Item {
            id: user_guid.to_string(),
            institution_id: member.as_ref().and_then(|m| m.institution_code.clone()),
            webhook: None,
            error: member.as_ref().and_then(|m| {
                if m.connection_status.as_deref() == Some("FAILED") {
                    Some(crate::ItemError {
                        error_type: "CONNECTION_ERROR".to_string(),
                        error_code: m.connection_status.clone().unwrap_or_default(),
                        error_message: "Connection failed".to_string(),
                        display_message: Some("Please reconnect your account".to_string()),
                    })
                } else {
                    None
                }
            }),
            available_products: vec![
                crate::Product::Transactions,
                crate::Product::Auth,
                crate::Product::Identity,
            ],
            billed_products: vec![],
            consent_expiration_time: None,
            update_type: member.and_then(|m| m.connection_status),
            extra: HashMap::new(),
        })
    }
}
