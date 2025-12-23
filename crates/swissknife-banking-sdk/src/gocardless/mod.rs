use crate::{
    AccessToken, Account, AccountBalance, AccountIdentity, BankingProvider, CreateLinkTokenRequest,
    CreatePaymentRequest, Institution, InstitutionOptions, Item, LinkToken, Payment, Transaction,
    TransactionList, TransactionOptions, Error, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct GoCardlessClient {
    client: Client,
    secret_id: String,
    secret_key: String,
    base_url: String,
    access_token: Option<String>,
    token_expiry: Option<chrono::DateTime<chrono::Utc>>,
}

impl GoCardlessClient {
    pub fn new(secret_id: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self::with_environment(secret_id, secret_key, GoCardlessEnvironment::Sandbox)
    }

    pub fn with_environment(
        secret_id: impl Into<String>,
        secret_key: impl Into<String>,
        environment: GoCardlessEnvironment,
    ) -> Self {
        Self {
            client: Client::new(),
            secret_id: secret_id.into(),
            secret_key: secret_key.into(),
            base_url: environment.base_url().to_string(),
            access_token: None,
            token_expiry: None,
        }
    }

    pub fn production(secret_id: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self::with_environment(secret_id, secret_key, GoCardlessEnvironment::Production)
    }

    async fn ensure_token(&mut self) -> Result<String> {
        let needs_refresh = self.token_expiry
            .map(|exp| chrono::Utc::now() >= exp)
            .unwrap_or(true);

        if needs_refresh || self.access_token.is_none() {
            let response = self
                .client
                .post(format!("{}/api/v2/token/new/", self.base_url))
                .json(&TokenRequest {
                    secret_id: self.secret_id.clone(),
                    secret_key: self.secret_key.clone(),
                })
                .send()
                .await?;

            if response.status().is_success() {
                let token: TokenResponse = response.json().await?;
                self.access_token = Some(token.access.clone());
                self.token_expiry = Some(chrono::Utc::now() + chrono::Duration::seconds(token.access_expires as i64 - 60));
                Ok(token.access)
            } else {
                let error: GoCardlessError = response.json().await?;
                Err(Error::Authentication(error.detail.unwrap_or_else(|| "Token refresh failed".to_string())))
            }
        } else {
            Ok(self.access_token.clone().unwrap())
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&mut self, endpoint: &str) -> Result<T> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: GoCardlessError = response.json().await?;
            Err(Error::Api {
                code: error.status_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()),
                message: error.detail.unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &mut self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: GoCardlessError = response.json().await?;
            Err(Error::Api {
                code: error.status_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()),
                message: error.detail.unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn delete(&mut self, endpoint: &str) -> Result<()> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            let error: GoCardlessError = response.json().await?;
            Err(Error::Api {
                code: error.status_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()),
                message: error.detail.unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GoCardlessEnvironment {
    Sandbox,
    Production,
}

impl GoCardlessEnvironment {
    fn base_url(&self) -> &'static str {
        match self {
            GoCardlessEnvironment::Sandbox => "https://bankaccountdata.gocardless.com",
            GoCardlessEnvironment::Production => "https://bankaccountdata.gocardless.com",
        }
    }
}

#[derive(Serialize)]
struct TokenRequest {
    secret_id: String,
    secret_key: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access: String,
    access_expires: u64,
    refresh: String,
    refresh_expires: u64,
}

#[derive(Deserialize)]
struct GoCardlessError {
    status_code: Option<u16>,
    detail: Option<String>,
    summary: Option<String>,
}

#[derive(Serialize)]
struct CreateRequisitionRequest {
    redirect: String,
    institution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agreement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account_selection: Option<bool>,
}

#[derive(Deserialize)]
struct RequisitionResponse {
    id: String,
    created: String,
    redirect: String,
    status: String,
    institution_id: String,
    agreement: Option<String>,
    reference: Option<String>,
    accounts: Vec<String>,
    link: String,
}

#[derive(Deserialize)]
struct InstitutionListResponse(Vec<GCInstitution>);

#[derive(Deserialize)]
struct GCInstitution {
    id: String,
    name: String,
    bic: Option<String>,
    transaction_total_days: Option<String>,
    countries: Vec<String>,
    logo: Option<String>,
}

#[derive(Deserialize)]
struct GCAccountDetails {
    id: String,
    created: String,
    last_accessed: Option<String>,
    iban: Option<String>,
    institution_id: String,
    status: String,
    owner_name: Option<String>,
}

#[derive(Deserialize)]
struct GCAccountData {
    account: GCAccountInfo,
}

#[derive(Deserialize)]
struct GCAccountInfo {
    resource_id: Option<String>,
    iban: Option<String>,
    bban: Option<String>,
    currency: Option<String>,
    owner_name: Option<String>,
    name: Option<String>,
    product: Option<String>,
    cash_account_type: Option<String>,
}

#[derive(Deserialize)]
struct GCBalancesResponse {
    balances: Vec<GCBalance>,
}

#[derive(Deserialize)]
struct GCBalance {
    balance_amount: GCAmount,
    balance_type: String,
    reference_date: Option<String>,
}

#[derive(Deserialize)]
struct GCAmount {
    amount: String,
    currency: String,
}

#[derive(Deserialize)]
struct GCTransactionsResponse {
    transactions: GCTransactionsList,
}

#[derive(Deserialize)]
struct GCTransactionsList {
    booked: Vec<GCTransaction>,
    pending: Option<Vec<GCTransaction>>,
}

#[derive(Deserialize)]
struct GCTransaction {
    transaction_id: Option<String>,
    booking_date: Option<String>,
    value_date: Option<String>,
    transaction_amount: GCAmount,
    remittance_information_unstructured: Option<String>,
    remittance_information_structured: Option<String>,
    creditor_name: Option<String>,
    creditor_account: Option<GCTransactionAccount>,
    debtor_name: Option<String>,
    debtor_account: Option<GCTransactionAccount>,
    bank_transaction_code: Option<String>,
    internal_transaction_id: Option<String>,
}

#[derive(Deserialize)]
struct GCTransactionAccount {
    iban: Option<String>,
    bban: Option<String>,
}

impl GoCardlessClient {
    fn convert_account(&self, details: GCAccountDetails, data: Option<GCAccountData>, balances: Option<GCBalancesResponse>) -> Account {
        let account_info = data.map(|d| d.account);
        let currency = account_info.as_ref()
            .and_then(|a| a.currency.clone())
            .unwrap_or_else(|| "EUR".to_string());

        let balance = balances.and_then(|b| {
            b.balances.into_iter().find(|bal| {
                bal.balance_type == "interimAvailable" || bal.balance_type == "closingBooked"
            })
        });

        let current = balance.as_ref()
            .and_then(|b| b.balance_amount.amount.parse::<f64>().ok())
            .unwrap_or(0.0);

        Account {
            id: details.id.clone(),
            name: account_info.as_ref()
                .and_then(|a| a.name.clone())
                .or(account_info.as_ref().and_then(|a| a.product.clone()))
                .unwrap_or_else(|| "Account".to_string()),
            official_name: account_info.as_ref().and_then(|a| a.product.clone()),
            account_type: match account_info.as_ref().and_then(|a| a.cash_account_type.as_deref()) {
                Some("CACC") | Some("CARD") => crate::AccountType::Depository,
                Some("SVGS") => crate::AccountType::Depository,
                Some("LOAN") => crate::AccountType::Loan,
                _ => crate::AccountType::Other,
            },
            subtype: account_info.as_ref().and_then(|a| a.cash_account_type.clone()),
            mask: details.iban.as_ref().map(|i| i.chars().rev().take(4).collect::<String>().chars().rev().collect()),
            currency: currency.clone(),
            institution_id: Some(details.institution_id),
            balances: AccountBalance {
                account_id: details.id,
                current,
                available: Some(current),
                limit: None,
                currency,
                last_updated: details.last_accessed.and_then(|la| {
                    chrono::DateTime::parse_from_rfc3339(&la).ok().map(|dt| dt.with_timezone(&chrono::Utc))
                }),
            },
            account_number: Some(crate::AccountNumber {
                account_number: account_info.as_ref().and_then(|a| a.bban.clone()),
                routing_number: None,
                sort_code: None,
                iban: details.iban.or(account_info.and_then(|a| a.iban)),
                bic: None,
            }),
            extra: HashMap::new(),
        }
    }

    fn convert_transaction(&self, tx: GCTransaction, account_id: &str, pending: bool) -> Transaction {
        let amount = tx.transaction_amount.amount.parse::<f64>().unwrap_or(0.0);
        let is_credit = amount > 0.0;

        let description = tx.remittance_information_unstructured
            .or(tx.remittance_information_structured)
            .unwrap_or_else(|| "Transaction".to_string());

        let counterparty_name = if is_credit {
            tx.debtor_name.clone()
        } else {
            tx.creditor_name.clone()
        };

        Transaction {
            id: tx.transaction_id
                .or(tx.internal_transaction_id)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            account_id: account_id.to_string(),
            amount: amount.abs(),
            currency: tx.transaction_amount.currency,
            date: tx.booking_date
                .or(tx.value_date)
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                .unwrap_or_else(|| chrono::Utc::now().date_naive()),
            datetime: None,
            name: description,
            merchant_name: counterparty_name.clone(),
            merchant_id: None,
            category: tx.bank_transaction_code.map(|c| vec![c]),
            category_id: None,
            pending,
            transaction_type: if is_credit {
                crate::TransactionType::Credit
            } else {
                crate::TransactionType::Debit
            },
            payment_channel: None,
            location: None,
            counterparty: counterparty_name.map(|name| crate::Counterparty {
                name: Some(name),
                entity_id: None,
                counterparty_type: None,
                logo_url: None,
                website: None,
            }),
            extra: HashMap::new(),
        }
    }

    fn convert_institution(&self, inst: GCInstitution) -> Institution {
        Institution {
            id: inst.id,
            name: inst.name,
            url: None,
            logo: inst.logo,
            primary_color: None,
            country_codes: inst.countries,
            products: vec![
                crate::Product::Transactions,
                crate::Product::Auth,
            ],
            routing_numbers: inst.bic.map(|b| vec![b]),
            oauth: true,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl BankingProvider for GoCardlessClient {
    async fn create_link_token(&self, request: CreateLinkTokenRequest) -> Result<LinkToken> {
        let mut client = Self::with_environment(
            self.secret_id.clone(),
            self.secret_key.clone(),
            GoCardlessEnvironment::Production,
        );

        let redirect_uri = request.redirect_uri.ok_or_else(|| {
            Error::InvalidRequest("redirect_uri is required".to_string())
        })?;

        let institution_id = request.extra.get("institution_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("institution_id is required".to_string()))?;

        let requisition: RequisitionResponse = client
            .post("/api/v2/requisitions/", &CreateRequisitionRequest {
                redirect: redirect_uri,
                institution_id: institution_id.to_string(),
                reference: Some(request.user_id),
                user_language: request.language,
                agreement: None,
                account_selection: Some(true),
            })
            .await?;

        Ok(LinkToken {
            link_token: requisition.link,
            expiration: chrono::Utc::now() + chrono::Duration::days(7),
            request_id: Some(requisition.id),
        })
    }

    async fn exchange_public_token(&self, requisition_id: &str) -> Result<AccessToken> {
        Ok(AccessToken {
            access_token: requisition_id.to_string(),
            item_id: Some(requisition_id.to_string()),
            request_id: None,
        })
    }

    async fn list_accounts(&self, requisition_id: &str) -> Result<Vec<Account>> {
        let mut client = Self::with_environment(
            self.secret_id.clone(),
            self.secret_key.clone(),
            GoCardlessEnvironment::Production,
        );

        let requisition: RequisitionResponse = client
            .get(&format!("/api/v2/requisitions/{}/", requisition_id))
            .await?;

        let mut accounts = Vec::new();

        for account_id in requisition.accounts {
            let details: GCAccountDetails = client
                .get(&format!("/api/v2/accounts/{}/", account_id))
                .await?;

            let data: std::result::Result<GCAccountData, _> = client
                .get(&format!("/api/v2/accounts/{}/details/", account_id))
                .await;

            let balances: std::result::Result<GCBalancesResponse, _> = client
                .get(&format!("/api/v2/accounts/{}/balances/", account_id))
                .await;

            accounts.push(client.convert_account(details, data.ok(), balances.ok()));
        }

        Ok(accounts)
    }

    async fn get_account(&self, requisition_id: &str, account_id: &str) -> Result<Account> {
        let accounts = self.list_accounts(requisition_id).await?;
        accounts
            .into_iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| Error::AccountNotFound(account_id.to_string()))
    }

    async fn get_balances(&self, requisition_id: &str) -> Result<Vec<AccountBalance>> {
        let accounts = self.list_accounts(requisition_id).await?;
        Ok(accounts.into_iter().map(|a| a.balances).collect())
    }

    async fn list_transactions(
        &self,
        requisition_id: &str,
        options: TransactionOptions,
    ) -> Result<TransactionList> {
        let mut client = Self::with_environment(
            self.secret_id.clone(),
            self.secret_key.clone(),
            GoCardlessEnvironment::Production,
        );

        let requisition: RequisitionResponse = client
            .get(&format!("/api/v2/requisitions/{}/", requisition_id))
            .await?;

        let mut all_transactions = Vec::new();

        for account_id in requisition.accounts {
            if let Some(ref account_ids) = options.account_ids {
                if !account_ids.contains(&account_id) {
                    continue;
                }
            }

            let today = chrono::Utc::now().date_naive();
            let date_from = options.start_date.unwrap_or_else(|| today - chrono::Duration::days(90));
            let date_to = options.end_date.unwrap_or(today);

            let endpoint = format!(
                "/api/v2/accounts/{}/transactions/?date_from={}&date_to={}",
                account_id,
                date_from.format("%Y-%m-%d"),
                date_to.format("%Y-%m-%d")
            );

            if let Ok(response) = client.get::<GCTransactionsResponse>(&endpoint).await {
                for tx in response.transactions.booked {
                    all_transactions.push(client.convert_transaction(tx, &account_id, false));
                }

                if options.include_pending.unwrap_or(false) {
                    if let Some(pending) = response.transactions.pending {
                        for tx in pending {
                            all_transactions.push(client.convert_transaction(tx, &account_id, true));
                        }
                    }
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
        requisition_id: &str,
        transaction_id: &str,
    ) -> Result<Transaction> {
        let transactions = self
            .list_transactions(requisition_id, TransactionOptions::default())
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

    async fn list_institutions(&self, options: InstitutionOptions) -> Result<Vec<Institution>> {
        let mut client = Self::with_environment(
            self.secret_id.clone(),
            self.secret_key.clone(),
            GoCardlessEnvironment::Production,
        );

        let country = options.country_codes
            .as_ref()
            .and_then(|c| c.first())
            .cloned()
            .unwrap_or_else(|| "GB".to_string());

        let endpoint = format!("/api/v2/institutions/?country={}", country);
        let institutions: Vec<GCInstitution> = client.get(&endpoint).await?;

        Ok(institutions
            .into_iter()
            .map(|i| client.convert_institution(i))
            .collect())
    }

    async fn get_institution(&self, institution_id: &str) -> Result<Institution> {
        let mut client = Self::with_environment(
            self.secret_id.clone(),
            self.secret_key.clone(),
            GoCardlessEnvironment::Production,
        );

        let endpoint = format!("/api/v2/institutions/{}/", institution_id);
        let institution: GCInstitution = client.get(&endpoint).await?;

        Ok(client.convert_institution(institution))
    }

    async fn get_identity(&self, requisition_id: &str) -> Result<Vec<AccountIdentity>> {
        let accounts = self.list_accounts(requisition_id).await?;

        Ok(accounts
            .into_iter()
            .filter_map(|acc| {
                acc.extra.get("owner_name").and_then(|v| v.as_str()).map(|name| {
                    AccountIdentity {
                        account_id: acc.id,
                        owners: vec![crate::Owner {
                            names: vec![name.to_string()],
                            phone_numbers: vec![],
                            emails: vec![],
                            addresses: vec![],
                        }],
                    }
                })
            })
            .collect())
    }

    async fn create_payment(
        &self,
        _access_token: &str,
        _request: CreatePaymentRequest,
    ) -> Result<Payment> {
        Err(Error::Provider(
            "GoCardless Bank Account Data does not support payment initiation. Use GoCardless Payments API separately.".to_string(),
        ))
    }

    async fn get_payment(&self, _access_token: &str, _payment_id: &str) -> Result<Payment> {
        Err(Error::Provider(
            "GoCardless Bank Account Data does not support payment initiation".to_string(),
        ))
    }

    async fn remove_item(&self, requisition_id: &str) -> Result<()> {
        let mut client = Self::with_environment(
            self.secret_id.clone(),
            self.secret_key.clone(),
            GoCardlessEnvironment::Production,
        );

        client.delete(&format!("/api/v2/requisitions/{}/", requisition_id)).await
    }

    async fn get_item(&self, requisition_id: &str) -> Result<Item> {
        let mut client = Self::with_environment(
            self.secret_id.clone(),
            self.secret_key.clone(),
            GoCardlessEnvironment::Production,
        );

        let requisition: RequisitionResponse = client
            .get(&format!("/api/v2/requisitions/{}/", requisition_id))
            .await?;

        Ok(Item {
            id: requisition.id,
            institution_id: Some(requisition.institution_id),
            webhook: None,
            error: None,
            available_products: vec![
                crate::Product::Transactions,
                crate::Product::Auth,
            ],
            billed_products: vec![],
            consent_expiration_time: Some(chrono::Utc::now() + chrono::Duration::days(90)),
            update_type: Some(requisition.status),
            extra: HashMap::new(),
        })
    }
}
