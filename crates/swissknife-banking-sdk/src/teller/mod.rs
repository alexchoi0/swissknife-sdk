use crate::{
    AccessToken, Account, AccountBalance, AccountIdentity, BankingProvider, CreateLinkTokenRequest,
    CreatePaymentRequest, Institution, InstitutionOptions, Item, LinkToken, Payment, Transaction,
    TransactionList, TransactionOptions, Error, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct TellerClient {
    client: Client,
    application_id: String,
    api_base_url: String,
    certificate: Option<TellerCertificate>,
}

pub struct TellerCertificate {
    pub cert_pem: String,
    pub key_pem: String,
}

impl TellerClient {
    pub fn new(application_id: impl Into<String>) -> Self {
        Self::with_environment(application_id, TellerEnvironment::Sandbox)
    }

    pub fn with_environment(
        application_id: impl Into<String>,
        environment: TellerEnvironment,
    ) -> Self {
        Self {
            client: Client::new(),
            application_id: application_id.into(),
            api_base_url: environment.base_url().to_string(),
            certificate: None,
        }
    }

    pub fn production(application_id: impl Into<String>, certificate: TellerCertificate) -> Self {
        Self {
            client: Client::new(),
            application_id: application_id.into(),
            api_base_url: TellerEnvironment::Production.base_url().to_string(),
            certificate: Some(certificate),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, access_token: &str, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.api_base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .basic_auth(access_token, Option::<&str>::None)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: TellerError = response.json().await?;
            Err(Error::Api {
                code: error.error.code,
                message: error.error.message,
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
            .basic_auth(access_token, Option::<&str>::None)
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: TellerError = response.json().await?;
            Err(Error::Api {
                code: error.error.code,
                message: error.error.message,
            })
        }
    }

    async fn delete(&self, access_token: &str, endpoint: &str) -> Result<()> {
        let url = format!("{}{}", self.api_base_url, endpoint);
        let response = self
            .client
            .delete(&url)
            .basic_auth(access_token, Option::<&str>::None)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: TellerError = response.json().await?;
            Err(Error::Api {
                code: error.error.code,
                message: error.error.message,
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TellerEnvironment {
    Sandbox,
    Development,
    Production,
}

impl TellerEnvironment {
    fn base_url(&self) -> &'static str {
        match self {
            TellerEnvironment::Sandbox => "https://api.teller.io",
            TellerEnvironment::Development => "https://api.teller.io",
            TellerEnvironment::Production => "https://api.teller.io",
        }
    }
}

#[derive(Deserialize)]
struct TellerError {
    error: TellerErrorDetail,
}

#[derive(Deserialize)]
struct TellerErrorDetail {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct TellerAccount {
    id: String,
    name: String,
    #[serde(rename = "type")]
    account_type: String,
    subtype: String,
    currency: String,
    enrollment_id: String,
    institution: TellerInstitution,
    last_four: Option<String>,
    status: String,
    links: TellerAccountLinks,
}

#[derive(Deserialize)]
struct TellerInstitution {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct TellerAccountLinks {
    balances: String,
    transactions: String,
    details: Option<String>,
    #[serde(rename = "self")]
    self_link: String,
}

#[derive(Deserialize)]
struct TellerBalances {
    account_id: String,
    available: Option<String>,
    ledger: String,
    links: TellerBalanceLinks,
}

#[derive(Deserialize)]
struct TellerBalanceLinks {
    account: String,
    #[serde(rename = "self")]
    self_link: String,
}

#[derive(Deserialize)]
struct TellerAccountDetails {
    account_id: String,
    account_number: String,
    routing_numbers: TellerRoutingNumbers,
    links: TellerDetailLinks,
}

#[derive(Deserialize)]
struct TellerRoutingNumbers {
    ach: Option<String>,
    wire: Option<String>,
}

#[derive(Deserialize)]
struct TellerDetailLinks {
    account: String,
    #[serde(rename = "self")]
    self_link: String,
}

#[derive(Deserialize)]
struct TellerTransaction {
    id: String,
    account_id: String,
    date: String,
    description: String,
    details: TellerTransactionDetails,
    status: String,
    amount: String,
    running_balance: Option<String>,
    #[serde(rename = "type")]
    transaction_type: String,
    links: TellerTransactionLinks,
}

#[derive(Deserialize)]
struct TellerTransactionDetails {
    processing_status: String,
    category: Option<String>,
    counterparty: Option<TellerCounterparty>,
}

#[derive(Deserialize)]
struct TellerCounterparty {
    name: Option<String>,
    #[serde(rename = "type")]
    counterparty_type: Option<String>,
}

#[derive(Deserialize)]
struct TellerTransactionLinks {
    account: String,
    #[serde(rename = "self")]
    self_link: String,
}

#[derive(Deserialize)]
struct TellerIdentity {
    emails: Vec<TellerEmail>,
    names: Vec<TellerName>,
    phone_numbers: Vec<TellerPhone>,
    addresses: Vec<TellerAddressWrapper>,
}

#[derive(Deserialize)]
struct TellerEmail {
    data: String,
    #[serde(rename = "type")]
    email_type: String,
}

#[derive(Deserialize)]
struct TellerName {
    data: String,
}

#[derive(Deserialize)]
struct TellerPhone {
    data: String,
    #[serde(rename = "type")]
    phone_type: String,
}

#[derive(Deserialize)]
struct TellerAddressWrapper {
    data: TellerAddress,
    #[serde(rename = "type")]
    address_type: String,
}

#[derive(Deserialize)]
struct TellerAddress {
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

impl TellerClient {
    fn convert_account(&self, account: TellerAccount, balance: Option<TellerBalances>, details: Option<TellerAccountDetails>) -> Account {
        let current = balance
            .as_ref()
            .map(|b| b.ledger.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let available = balance
            .as_ref()
            .and_then(|b| b.available.as_ref())
            .and_then(|a| a.parse::<f64>().ok());

        Account {
            id: account.id.clone(),
            name: account.name,
            official_name: None,
            account_type: match account.account_type.as_str() {
                "depository" => crate::AccountType::Depository,
                "credit" => crate::AccountType::Credit,
                _ => crate::AccountType::Other,
            },
            subtype: Some(account.subtype),
            mask: account.last_four,
            currency: account.currency.clone(),
            institution_id: Some(account.institution.id),
            balances: AccountBalance {
                account_id: account.id,
                current,
                available,
                limit: None,
                currency: account.currency,
                last_updated: None,
            },
            account_number: details.map(|d| crate::AccountNumber {
                account_number: Some(d.account_number),
                routing_number: d.routing_numbers.ach,
                sort_code: None,
                iban: None,
                bic: None,
            }),
            extra: HashMap::new(),
        }
    }

    fn convert_transaction(&self, tx: TellerTransaction) -> Transaction {
        let amount = tx.amount.parse::<f64>().unwrap_or(0.0);

        Transaction {
            id: tx.id,
            account_id: tx.account_id,
            amount: amount.abs(),
            currency: "USD".to_string(),
            date: chrono::NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive()),
            datetime: None,
            name: tx.description.clone(),
            merchant_name: tx.details.counterparty.as_ref().and_then(|c| c.name.clone()),
            merchant_id: None,
            category: tx.details.category.map(|c| vec![c]),
            category_id: None,
            pending: tx.status == "pending",
            transaction_type: if amount < 0.0 {
                crate::TransactionType::Debit
            } else {
                crate::TransactionType::Credit
            },
            payment_channel: None,
            location: None,
            counterparty: tx.details.counterparty.map(|c| crate::Counterparty {
                name: c.name,
                entity_id: None,
                counterparty_type: c.counterparty_type,
                logo_url: None,
                website: None,
            }),
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl BankingProvider for TellerClient {
    async fn create_link_token(&self, request: CreateLinkTokenRequest) -> Result<LinkToken> {
        let products: Vec<&str> = request
            .products
            .iter()
            .filter_map(|p| match p {
                crate::Product::Transactions => Some("transactions"),
                crate::Product::Auth => Some("identity"),
                crate::Product::Identity => Some("identity"),
                _ => None,
            })
            .collect();

        let select_account = request.extra.get("select_account")
            .and_then(|v| v.as_str())
            .unwrap_or("single");

        let institution = request.extra.get("institution")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut url = format!(
            "https://teller.io/connect?application_id={}&products={}",
            urlencoding::encode(&self.application_id),
            urlencoding::encode(&products.join(","))
        );

        if !institution.is_empty() {
            url.push_str(&format!("&institution={}", urlencoding::encode(institution)));
        }

        url.push_str(&format!("&select_account={}", select_account));

        Ok(LinkToken {
            link_token: url,
            expiration: chrono::Utc::now() + chrono::Duration::hours(24),
            request_id: None,
        })
    }

    async fn exchange_public_token(&self, access_token: &str) -> Result<AccessToken> {
        Ok(AccessToken {
            access_token: access_token.to_string(),
            item_id: None,
            request_id: None,
        })
    }

    async fn list_accounts(&self, access_token: &str) -> Result<Vec<Account>> {
        let accounts: Vec<TellerAccount> = self.get(access_token, "/accounts").await?;

        let mut result = Vec::new();
        for account in accounts {
            let balance: std::result::Result<TellerBalances, _> = self
                .get(access_token, &format!("/accounts/{}/balances", account.id))
                .await;

            let details: std::result::Result<TellerAccountDetails, _> = self
                .get(access_token, &format!("/accounts/{}/details", account.id))
                .await;

            result.push(self.convert_account(account, balance.ok(), details.ok()));
        }

        Ok(result)
    }

    async fn get_account(&self, access_token: &str, account_id: &str) -> Result<Account> {
        let account: TellerAccount = self
            .get(access_token, &format!("/accounts/{}", account_id))
            .await?;

        let balance: std::result::Result<TellerBalances, _> = self
            .get(access_token, &format!("/accounts/{}/balances", account_id))
            .await;

        let details: std::result::Result<TellerAccountDetails, _> = self
            .get(access_token, &format!("/accounts/{}/details", account_id))
            .await;

        Ok(self.convert_account(account, balance.ok(), details.ok()))
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
        let accounts: Vec<TellerAccount> = self.get(access_token, "/accounts").await?;

        let mut all_transactions = Vec::new();

        for account in accounts {
            if let Some(ref account_ids) = options.account_ids {
                if !account_ids.contains(&account.id) {
                    continue;
                }
            }

            let endpoint = format!("/accounts/{}/transactions", account.id);
            if let Ok(transactions) = self.get::<Vec<TellerTransaction>>(access_token, &endpoint).await {
                for tx in transactions {
                    let transaction = self.convert_transaction(tx);

                    if let Some(start) = options.start_date {
                        if transaction.date < start {
                            continue;
                        }
                    }
                    if let Some(end) = options.end_date {
                        if transaction.date > end {
                            continue;
                        }
                    }

                    all_transactions.push(transaction);
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
        let accounts: Vec<TellerAccount> = self.get(access_token, "/accounts").await?;

        for account in accounts {
            let endpoint = format!("/accounts/{}/transactions/{}", account.id, transaction_id);
            if let Ok(tx) = self.get::<TellerTransaction>(access_token, &endpoint).await {
                return Ok(self.convert_transaction(tx));
            }
        }

        Err(Error::Api {
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
        let accounts: Vec<TellerAccount> = self.get(access_token, "/accounts").await?;

        let mut identities = Vec::new();

        for account in accounts {
            let endpoint = format!("/accounts/{}/identity", account.id);
            if let Ok(identity) = self.get::<TellerIdentity>(access_token, &endpoint).await {
                identities.push(AccountIdentity {
                    account_id: account.id,
                    owners: vec![crate::Owner {
                        names: identity.names.into_iter().map(|n| n.data).collect(),
                        phone_numbers: identity
                            .phone_numbers
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| crate::PhoneNumber {
                                data: p.data,
                                primary: i == 0,
                                phone_type: Some(p.phone_type),
                            })
                            .collect(),
                        emails: identity
                            .emails
                            .into_iter()
                            .enumerate()
                            .map(|(i, e)| crate::Email {
                                data: e.data,
                                primary: i == 0,
                                email_type: Some(e.email_type),
                            })
                            .collect(),
                        addresses: identity
                            .addresses
                            .into_iter()
                            .enumerate()
                            .map(|(i, a)| crate::Address {
                                street: a.data.street,
                                city: a.data.city,
                                region: a.data.state,
                                postal_code: a.data.postal_code,
                                country: a.data.country,
                                primary: i == 0,
                            })
                            .collect(),
                    }],
                });
            }
        }

        Ok(identities)
    }

    async fn create_payment(
        &self,
        _access_token: &str,
        _request: CreatePaymentRequest,
    ) -> Result<Payment> {
        Err(Error::Provider(
            "Teller does not support payment initiation".to_string(),
        ))
    }

    async fn get_payment(&self, _access_token: &str, _payment_id: &str) -> Result<Payment> {
        Err(Error::Provider(
            "Teller does not support payment initiation".to_string(),
        ))
    }

    async fn remove_item(&self, access_token: &str) -> Result<()> {
        let accounts: Vec<TellerAccount> = self.get(access_token, "/accounts").await?;

        if let Some(account) = accounts.first() {
            self.delete(access_token, &format!("/accounts/{}", account.id)).await?;
        }

        Ok(())
    }

    async fn get_item(&self, access_token: &str) -> Result<Item> {
        let accounts: Vec<TellerAccount> = self.get(access_token, "/accounts").await?;

        let institution_id = accounts.first().map(|a| a.institution.id.clone());
        let enrollment_id = accounts.first().map(|a| a.enrollment_id.clone());

        Ok(Item {
            id: enrollment_id.unwrap_or_else(|| "teller_enrollment".to_string()),
            institution_id,
            webhook: None,
            error: None,
            available_products: vec![
                crate::Product::Transactions,
                crate::Product::Auth,
                crate::Product::Identity,
            ],
            billed_products: vec![],
            consent_expiration_time: None,
            update_type: None,
            extra: HashMap::new(),
        })
    }
}
