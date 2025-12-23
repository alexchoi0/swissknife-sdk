use crate::{
    AccessToken, Account, AccountBalance, AccountIdentity, BankingProvider, CreateLinkTokenRequest,
    CreatePaymentRequest, Institution, InstitutionOptions, Item, LinkToken, Payment, Transaction,
    TransactionList, TransactionOptions, Error, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct TrueLayerClient {
    client: Client,
    client_id: String,
    client_secret: String,
    auth_base_url: String,
    api_base_url: String,
}

impl TrueLayerClient {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self::with_environment(client_id, client_secret, TrueLayerEnvironment::Sandbox)
    }

    pub fn with_environment(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        environment: TrueLayerEnvironment,
    ) -> Self {
        let (auth_base_url, api_base_url) = environment.base_urls();
        Self {
            client: Client::new(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_base_url: auth_base_url.to_string(),
            api_base_url: api_base_url.to_string(),
        }
    }

    pub fn production(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self::with_environment(client_id, client_secret, TrueLayerEnvironment::Production)
    }

    async fn get_access_token(&self) -> Result<String> {
        let response = self
            .client
            .post(format!("{}/connect/token", self.auth_base_url))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("scope", "payments"),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let token: TokenResponse = response.json().await?;
            Ok(token.access_token)
        } else {
            let error: TrueLayerError = response.json().await?;
            Err(Error::Api {
                code: error.error.unwrap_or_else(|| "unknown".to_string()),
                message: error.error_description.unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, access_token: &str, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.api_base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: TrueLayerError = response.json().await?;
            Err(Error::Api {
                code: error.error.unwrap_or_else(|| "unknown".to_string()),
                message: error.error_description.unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        access_token: &str,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.api_base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: TrueLayerError = response.json().await?;
            Err(Error::Api {
                code: error.error.unwrap_or_else(|| "unknown".to_string()),
                message: error.error_description.unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TrueLayerEnvironment {
    Sandbox,
    Production,
}

impl TrueLayerEnvironment {
    fn base_urls(&self) -> (&'static str, &'static str) {
        match self {
            TrueLayerEnvironment::Sandbox => (
                "https://auth.truelayer-sandbox.com",
                "https://api.truelayer-sandbox.com",
            ),
            TrueLayerEnvironment::Production => (
                "https://auth.truelayer.com",
                "https://api.truelayer.com",
            ),
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct TrueLayerError {
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct TrueLayerResults<T> {
    results: Vec<T>,
    status: String,
}

#[derive(Deserialize)]
struct TrueLayerResult<T> {
    result: T,
    status: String,
}

#[derive(Deserialize)]
struct TrueLayerAccount {
    account_id: String,
    account_type: String,
    display_name: String,
    currency: String,
    account_number: Option<TrueLayerAccountNumber>,
    provider: Option<TrueLayerProvider>,
}

#[derive(Deserialize)]
struct TrueLayerAccountNumber {
    iban: Option<String>,
    swift_bic: Option<String>,
    number: Option<String>,
    sort_code: Option<String>,
}

#[derive(Deserialize)]
struct TrueLayerProvider {
    provider_id: String,
    display_name: String,
    logo_uri: Option<String>,
}

#[derive(Deserialize)]
struct TrueLayerBalance {
    currency: String,
    available: f64,
    current: f64,
    overdraft: Option<f64>,
    update_timestamp: String,
}

#[derive(Deserialize)]
struct TrueLayerTransaction {
    transaction_id: String,
    timestamp: String,
    description: String,
    amount: f64,
    currency: String,
    transaction_type: String,
    transaction_category: String,
    merchant_name: Option<String>,
    running_balance: Option<TrueLayerRunningBalance>,
    meta: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize)]
struct TrueLayerRunningBalance {
    currency: String,
    amount: f64,
}

#[derive(Clone, Deserialize)]
struct TrueLayerIdentity {
    full_name: Option<String>,
    emails: Option<Vec<String>>,
    phones: Option<Vec<String>>,
    addresses: Option<Vec<TrueLayerAddress>>,
    date_of_birth: Option<String>,
}

#[derive(Clone, Deserialize)]
struct TrueLayerAddress {
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
    country: Option<String>,
}

#[derive(Serialize)]
struct CreatePaymentRequestBody {
    amount_in_minor: u64,
    currency: String,
    payment_method: PaymentMethod,
    beneficiary: Beneficiary,
}

#[derive(Serialize)]
struct PaymentMethod {
    #[serde(rename = "type")]
    method_type: String,
    provider_selection: ProviderSelection,
}

#[derive(Serialize)]
struct ProviderSelection {
    #[serde(rename = "type")]
    selection_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_id: Option<String>,
}

#[derive(Serialize)]
struct Beneficiary {
    #[serde(rename = "type")]
    beneficiary_type: String,
    merchant_account_id: String,
}

#[derive(Deserialize)]
struct CreatePaymentResponse {
    id: String,
    resource_token: String,
    status: String,
}

#[derive(Deserialize)]
struct GetPaymentResponse {
    id: String,
    status: String,
    amount_in_minor: u64,
    currency: String,
    payment_method: PaymentMethodResponse,
    created_at: String,
}

#[derive(Deserialize)]
struct PaymentMethodResponse {
    #[serde(rename = "type")]
    method_type: String,
}

impl TrueLayerClient {
    fn convert_account(&self, account: TrueLayerAccount, balance: Option<TrueLayerBalance>) -> Account {
        let balances = balance.map(|b| AccountBalance {
            account_id: account.account_id.clone(),
            current: b.current,
            available: Some(b.available),
            limit: b.overdraft,
            currency: b.currency,
            last_updated: chrono::DateTime::parse_from_rfc3339(&b.update_timestamp)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }).unwrap_or(AccountBalance {
            account_id: account.account_id.clone(),
            current: 0.0,
            available: None,
            limit: None,
            currency: account.currency.clone(),
            last_updated: None,
        });

        Account {
            id: account.account_id,
            name: account.display_name,
            official_name: None,
            account_type: match account.account_type.as_str() {
                "TRANSACTION" | "CURRENT" => crate::AccountType::Depository,
                "SAVINGS" => crate::AccountType::Depository,
                "CREDIT_CARD" => crate::AccountType::Credit,
                "LOAN" => crate::AccountType::Loan,
                "INVESTMENT" => crate::AccountType::Investment,
                _ => crate::AccountType::Other,
            },
            subtype: Some(account.account_type),
            mask: None,
            currency: account.currency,
            institution_id: account.provider.as_ref().map(|p| p.provider_id.clone()),
            balances,
            account_number: account.account_number.map(|an| crate::AccountNumber {
                account_number: an.number,
                routing_number: None,
                sort_code: an.sort_code,
                iban: an.iban,
                bic: an.swift_bic,
            }),
            extra: HashMap::new(),
        }
    }

    fn convert_transaction(&self, tx: TrueLayerTransaction, account_id: &str) -> Transaction {
        Transaction {
            id: tx.transaction_id,
            account_id: account_id.to_string(),
            amount: tx.amount.abs(),
            currency: tx.currency,
            date: chrono::DateTime::parse_from_rfc3339(&tx.timestamp)
                .map(|dt| dt.date_naive())
                .unwrap_or_else(|_| chrono::Utc::now().date_naive()),
            datetime: chrono::DateTime::parse_from_rfc3339(&tx.timestamp)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            name: tx.description.clone(),
            merchant_name: tx.merchant_name,
            merchant_id: None,
            category: Some(vec![tx.transaction_category]),
            category_id: None,
            pending: false,
            transaction_type: if tx.amount < 0.0 {
                crate::TransactionType::Debit
            } else {
                crate::TransactionType::Credit
            },
            payment_channel: None,
            location: None,
            counterparty: None,
            extra: tx.meta.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl BankingProvider for TrueLayerClient {
    async fn create_link_token(&self, request: CreateLinkTokenRequest) -> Result<LinkToken> {
        let redirect_uri = request.redirect_uri.ok_or_else(|| {
            Error::InvalidRequest("redirect_uri is required for TrueLayer".to_string())
        })?;

        let scopes = vec!["info", "accounts", "balance", "transactions", "offline_access"];
        let providers = request.extra.get("providers")
            .and_then(|v| v.as_str())
            .unwrap_or("uk-ob-all uk-oauth-all");

        let auth_url = format!(
            "{}/connect/auth?response_type=code&client_id={}&redirect_uri={}&scope={}&providers={}",
            self.auth_base_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&scopes.join(" ")),
            urlencoding::encode(providers)
        );

        Ok(LinkToken {
            link_token: auth_url,
            expiration: chrono::Utc::now() + chrono::Duration::hours(1),
            request_id: None,
        })
    }

    async fn exchange_public_token(&self, code: &str) -> Result<AccessToken> {
        let redirect_uri = "https://console.truelayer.com/redirect-page";

        let response = self
            .client
            .post(format!("{}/connect/token", self.auth_base_url))
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("redirect_uri", redirect_uri),
                ("code", code),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let token: TokenResponse = response.json().await?;
            Ok(AccessToken {
                access_token: token.access_token,
                item_id: None,
                request_id: None,
            })
        } else {
            let error: TrueLayerError = response.json().await?;
            Err(Error::Api {
                code: error.error.unwrap_or_else(|| "unknown".to_string()),
                message: error.error_description.unwrap_or_else(|| "Token exchange failed".to_string()),
            })
        }
    }

    async fn list_accounts(&self, access_token: &str) -> Result<Vec<Account>> {
        let accounts_response: TrueLayerResults<TrueLayerAccount> = self
            .get(access_token, "/data/v1/accounts")
            .await?;

        let mut accounts = Vec::new();
        for acc in accounts_response.results {
            let balance_response: std::result::Result<TrueLayerResults<TrueLayerBalance>, _> = self
                .get(access_token, &format!("/data/v1/accounts/{}/balance", acc.account_id))
                .await;

            let balance = balance_response.ok().and_then(|r| r.results.into_iter().next());
            accounts.push(self.convert_account(acc, balance));
        }

        Ok(accounts)
    }

    async fn get_account(&self, access_token: &str, account_id: &str) -> Result<Account> {
        let accounts = self.list_accounts(access_token).await?;
        accounts
            .into_iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| Error::AccountNotFound(account_id.to_string()))
    }

    async fn get_balances(&self, access_token: &str) -> Result<Vec<AccountBalance>> {
        let accounts = self.list_accounts(access_token).await?;
        Ok(accounts.into_iter().map(|a| a.balances).collect())
    }

    async fn list_transactions(
        &self,
        access_token: &str,
        options: TransactionOptions,
    ) -> Result<TransactionList> {
        let accounts: TrueLayerResults<TrueLayerAccount> = self
            .get(access_token, "/data/v1/accounts")
            .await?;

        let today = chrono::Utc::now().date_naive();
        let from = options.start_date.unwrap_or_else(|| today - chrono::Duration::days(90));
        let to = options.end_date.unwrap_or(today);

        let mut all_transactions = Vec::new();

        for account in accounts.results {
            if let Some(ref account_ids) = options.account_ids {
                if !account_ids.contains(&account.account_id) {
                    continue;
                }
            }

            let endpoint = format!(
                "/data/v1/accounts/{}/transactions?from={}&to={}",
                account.account_id,
                from.format("%Y-%m-%d"),
                to.format("%Y-%m-%d")
            );

            if let Ok(response) = self.get::<TrueLayerResults<TrueLayerTransaction>>(access_token, &endpoint).await {
                for tx in response.results {
                    all_transactions.push(self.convert_transaction(tx, &account.account_id));
                }
            }
        }

        let total = all_transactions.len() as u32;

        Ok(TransactionList {
            transactions: all_transactions,
            total_transactions: total,
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_transaction(
        &self,
        access_token: &str,
        transaction_id: &str,
    ) -> Result<Transaction> {
        let transactions = self
            .list_transactions(access_token, TransactionOptions::default())
            .await?;

        transactions
            .transactions
            .into_iter()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| Error::Api {
                code: "TRANSACTION_NOT_FOUND".to_string(),
                message: format!("Transaction {} not found", transaction_id),
            })
    }

    async fn list_institutions(&self, _options: InstitutionOptions) -> Result<Vec<Institution>> {
        Ok(vec![])
    }

    async fn get_institution(&self, institution_id: &str) -> Result<Institution> {
        Err(Error::InstitutionNotSupported(institution_id.to_string()))
    }

    async fn get_identity(&self, access_token: &str) -> Result<Vec<AccountIdentity>> {
        let response: TrueLayerResults<TrueLayerIdentity> = self
            .get(access_token, "/data/v1/info")
            .await?;

        let accounts: TrueLayerResults<TrueLayerAccount> = self
            .get(access_token, "/data/v1/accounts")
            .await?;

        let identity = response.results.into_iter().next();

        Ok(accounts
            .results
            .into_iter()
            .map(|acc| {
                let owner = identity.as_ref().map(|id| crate::Owner {
                    names: id.full_name.clone().map(|n| vec![n]).unwrap_or_default(),
                    phone_numbers: id
                        .phones
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| crate::PhoneNumber {
                            data: p,
                            primary: i == 0,
                            phone_type: None,
                        })
                        .collect(),
                    emails: id
                        .emails
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .enumerate()
                        .map(|(i, e)| crate::Email {
                            data: e,
                            primary: i == 0,
                            email_type: None,
                        })
                        .collect(),
                    addresses: id
                        .addresses
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .enumerate()
                        .map(|(i, a)| crate::Address {
                            street: a.address,
                            city: a.city,
                            region: a.state,
                            postal_code: a.zip,
                            country: a.country,
                            primary: i == 0,
                        })
                        .collect(),
                });

                AccountIdentity {
                    account_id: acc.account_id,
                    owners: owner.map(|o| vec![o]).unwrap_or_default(),
                }
            })
            .collect())
    }

    async fn create_payment(
        &self,
        _access_token: &str,
        request: CreatePaymentRequest,
    ) -> Result<Payment> {
        let client_token = self.get_access_token().await?;

        let amount_in_minor = (request.amount.value * 100.0) as u64;

        let merchant_account_id = request.recipient_id.clone().ok_or_else(|| {
            Error::InvalidRequest("merchant_account_id (recipient_id) is required".to_string())
        })?;

        let body = CreatePaymentRequestBody {
            amount_in_minor,
            currency: request.amount.currency.clone(),
            payment_method: PaymentMethod {
                method_type: "bank_transfer".to_string(),
                provider_selection: ProviderSelection {
                    selection_type: "user_selected".to_string(),
                    provider_id: None,
                },
            },
            beneficiary: Beneficiary {
                beneficiary_type: "merchant_account".to_string(),
                merchant_account_id,
            },
        };

        let response: CreatePaymentResponse = self
            .post(&client_token, "/payments", &body)
            .await?;

        Ok(Payment {
            id: response.id,
            status: match response.status.as_str() {
                "authorization_required" => crate::PaymentStatus::InputNeeded,
                "authorizing" => crate::PaymentStatus::Processing,
                "authorized" => crate::PaymentStatus::Initiated,
                "executed" => crate::PaymentStatus::Executed,
                "settled" => crate::PaymentStatus::Completed,
                "failed" => crate::PaymentStatus::Failed,
                _ => crate::PaymentStatus::Processing,
            },
            amount: request.amount,
            reference: request.reference,
            recipient_id: request.recipient_id,
            created_at: chrono::Utc::now(),
            last_status_update: None,
            schedule: request.schedule,
            extra: HashMap::new(),
        })
    }

    async fn get_payment(&self, _access_token: &str, payment_id: &str) -> Result<Payment> {
        let client_token = self.get_access_token().await?;

        let response: GetPaymentResponse = self
            .get(&client_token, &format!("/payments/{}", payment_id))
            .await?;

        Ok(Payment {
            id: response.id,
            status: match response.status.as_str() {
                "authorization_required" => crate::PaymentStatus::InputNeeded,
                "authorizing" => crate::PaymentStatus::Processing,
                "authorized" => crate::PaymentStatus::Initiated,
                "executed" => crate::PaymentStatus::Executed,
                "settled" => crate::PaymentStatus::Completed,
                "failed" => crate::PaymentStatus::Failed,
                _ => crate::PaymentStatus::Processing,
            },
            amount: crate::PaymentAmount {
                value: response.amount_in_minor as f64 / 100.0,
                currency: response.currency,
            },
            reference: String::new(),
            recipient_id: None,
            created_at: chrono::DateTime::parse_from_rfc3339(&response.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            last_status_update: None,
            schedule: None,
            extra: HashMap::new(),
        })
    }

    async fn remove_item(&self, _access_token: &str) -> Result<()> {
        Ok(())
    }

    async fn get_item(&self, access_token: &str) -> Result<Item> {
        let accounts: TrueLayerResults<TrueLayerAccount> = self
            .get(access_token, "/data/v1/accounts")
            .await?;

        let institution_id = accounts
            .results
            .first()
            .and_then(|a| a.provider.as_ref())
            .map(|p| p.provider_id.clone());

        Ok(Item {
            id: "truelayer_connection".to_string(),
            institution_id,
            webhook: None,
            error: None,
            available_products: vec![
                crate::Product::Transactions,
                crate::Product::Auth,
                crate::Product::Identity,
                crate::Product::PaymentInitiation,
            ],
            billed_products: vec![],
            consent_expiration_time: Some(chrono::Utc::now() + chrono::Duration::days(90)),
            update_type: None,
            extra: HashMap::new(),
        })
    }
}
