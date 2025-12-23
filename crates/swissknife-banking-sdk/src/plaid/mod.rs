use crate::{
    AccessToken, Account, AccountBalance, AccountIdentity, BankingProvider, CreateLinkTokenRequest,
    CreatePaymentRequest, Institution, InstitutionOptions, Item, LinkToken, Payment, Transaction,
    TransactionList, TransactionOptions, Error, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct PlaidClient {
    client: Client,
    client_id: String,
    secret: String,
    base_url: String,
}

impl PlaidClient {
    pub fn new(client_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::with_environment(client_id, secret, PlaidEnvironment::Sandbox)
    }

    pub fn with_environment(
        client_id: impl Into<String>,
        secret: impl Into<String>,
        environment: PlaidEnvironment,
    ) -> Self {
        Self {
            client: Client::new(),
            client_id: client_id.into(),
            secret: secret.into(),
            base_url: environment.base_url().to_string(),
        }
    }

    pub fn production(client_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::with_environment(client_id, secret, PlaidEnvironment::Production)
    }

    pub fn development(client_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::with_environment(client_id, secret, PlaidEnvironment::Development)
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
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: PlaidError = response.json().await?;
            Err(Error::Api {
                code: error.error_code,
                message: error.error_message,
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlaidEnvironment {
    Sandbox,
    Development,
    Production,
}

impl PlaidEnvironment {
    fn base_url(&self) -> &'static str {
        match self {
            PlaidEnvironment::Sandbox => "https://sandbox.plaid.com",
            PlaidEnvironment::Development => "https://development.plaid.com",
            PlaidEnvironment::Production => "https://production.plaid.com",
        }
    }
}

#[derive(Debug, Deserialize)]
struct PlaidError {
    error_type: String,
    error_code: String,
    error_message: String,
    display_message: Option<String>,
}

#[derive(Serialize)]
struct PlaidRequest<T: Serialize> {
    client_id: String,
    secret: String,
    #[serde(flatten)]
    inner: T,
}

impl PlaidClient {
    fn wrap_request<T: Serialize>(&self, inner: T) -> PlaidRequest<T> {
        PlaidRequest {
            client_id: self.client_id.clone(),
            secret: self.secret.clone(),
            inner,
        }
    }
}

#[derive(Serialize)]
struct LinkTokenCreateRequest {
    user: LinkTokenUser,
    client_name: String,
    products: Vec<String>,
    country_codes: Vec<String>,
    language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uri: Option<String>,
}

#[derive(Serialize)]
struct LinkTokenUser {
    client_user_id: String,
}

#[derive(Deserialize)]
struct LinkTokenCreateResponse {
    link_token: String,
    expiration: chrono::DateTime<chrono::Utc>,
    request_id: String,
}

#[derive(Serialize)]
struct ItemPublicTokenExchangeRequest {
    public_token: String,
}

#[derive(Deserialize)]
struct ItemPublicTokenExchangeResponse {
    access_token: String,
    item_id: String,
    request_id: String,
}

#[derive(Serialize)]
struct AccountsGetRequest {
    access_token: String,
}

#[derive(Deserialize)]
struct AccountsGetResponse {
    accounts: Vec<PlaidAccount>,
    item: PlaidItem,
    request_id: String,
}

#[derive(Deserialize)]
struct PlaidAccount {
    account_id: String,
    name: String,
    official_name: Option<String>,
    #[serde(rename = "type")]
    account_type: String,
    subtype: Option<String>,
    mask: Option<String>,
    balances: PlaidBalances,
}

#[derive(Deserialize)]
struct PlaidBalances {
    available: Option<f64>,
    current: Option<f64>,
    limit: Option<f64>,
    iso_currency_code: Option<String>,
    unofficial_currency_code: Option<String>,
}

#[derive(Deserialize)]
struct PlaidItem {
    item_id: String,
    institution_id: Option<String>,
    webhook: Option<String>,
    error: Option<PlaidItemError>,
    available_products: Vec<String>,
    billed_products: Vec<String>,
    consent_expiration_time: Option<chrono::DateTime<chrono::Utc>>,
    update_type: Option<String>,
}

#[derive(Deserialize)]
struct PlaidItemError {
    error_type: String,
    error_code: String,
    error_message: String,
    display_message: Option<String>,
}

#[derive(Serialize)]
struct TransactionsGetRequest {
    access_token: String,
    start_date: String,
    end_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<TransactionsGetOptions>,
}

#[derive(Serialize)]
struct TransactionsGetOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    account_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_personal_finance_category: Option<bool>,
}

#[derive(Deserialize)]
struct TransactionsGetResponse {
    accounts: Vec<PlaidAccount>,
    transactions: Vec<PlaidTransaction>,
    total_transactions: u32,
    request_id: String,
}

#[derive(Deserialize)]
struct PlaidTransaction {
    transaction_id: String,
    account_id: String,
    amount: f64,
    iso_currency_code: Option<String>,
    unofficial_currency_code: Option<String>,
    date: String,
    datetime: Option<String>,
    name: String,
    merchant_name: Option<String>,
    merchant_entity_id: Option<String>,
    category: Option<Vec<String>>,
    category_id: Option<String>,
    pending: bool,
    transaction_type: Option<String>,
    payment_channel: Option<String>,
    location: Option<PlaidLocation>,
    counterparties: Option<Vec<PlaidCounterparty>>,
}

#[derive(Deserialize)]
struct PlaidLocation {
    address: Option<String>,
    city: Option<String>,
    region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    store_number: Option<String>,
}

#[derive(Deserialize)]
struct PlaidCounterparty {
    name: Option<String>,
    entity_id: Option<String>,
    #[serde(rename = "type")]
    counterparty_type: Option<String>,
    logo_url: Option<String>,
    website: Option<String>,
}

#[derive(Serialize)]
struct InstitutionsSearchRequest {
    query: String,
    products: Vec<String>,
    country_codes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<InstitutionsSearchOptions>,
}

#[derive(Serialize)]
struct InstitutionsSearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    include_optional_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth: Option<bool>,
}

#[derive(Serialize)]
struct InstitutionsGetByIdRequest {
    institution_id: String,
    country_codes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<InstitutionsGetByIdOptions>,
}

#[derive(Serialize)]
struct InstitutionsGetByIdOptions {
    include_optional_metadata: bool,
}

#[derive(Deserialize)]
struct InstitutionsSearchResponse {
    institutions: Vec<PlaidInstitution>,
    request_id: String,
}

#[derive(Deserialize)]
struct InstitutionGetByIdResponse {
    institution: PlaidInstitution,
    request_id: String,
}

#[derive(Deserialize)]
struct PlaidInstitution {
    institution_id: String,
    name: String,
    url: Option<String>,
    logo: Option<String>,
    primary_color: Option<String>,
    country_codes: Vec<String>,
    products: Vec<String>,
    routing_numbers: Option<Vec<String>>,
    oauth: bool,
}

#[derive(Serialize)]
struct IdentityGetRequest {
    access_token: String,
}

#[derive(Deserialize)]
struct IdentityGetResponse {
    accounts: Vec<PlaidIdentityAccount>,
    request_id: String,
}

#[derive(Deserialize)]
struct PlaidIdentityAccount {
    account_id: String,
    owners: Vec<PlaidOwner>,
}

#[derive(Deserialize)]
struct PlaidOwner {
    names: Vec<String>,
    phone_numbers: Vec<PlaidPhoneNumber>,
    emails: Vec<PlaidEmail>,
    addresses: Vec<PlaidAddress>,
}

#[derive(Deserialize)]
struct PlaidPhoneNumber {
    data: String,
    primary: bool,
    #[serde(rename = "type")]
    phone_type: Option<String>,
}

#[derive(Deserialize)]
struct PlaidEmail {
    data: String,
    primary: bool,
    #[serde(rename = "type")]
    email_type: Option<String>,
}

#[derive(Deserialize)]
struct PlaidAddress {
    data: PlaidAddressData,
    primary: bool,
}

#[derive(Deserialize)]
struct PlaidAddressData {
    street: Option<String>,
    city: Option<String>,
    region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

#[derive(Serialize)]
struct ItemRemoveRequest {
    access_token: String,
}

#[derive(Deserialize)]
struct ItemRemoveResponse {
    request_id: String,
}

#[derive(Serialize)]
struct ItemGetRequest {
    access_token: String,
}

#[derive(Deserialize)]
struct ItemGetResponse {
    item: PlaidItem,
    request_id: String,
}

impl PlaidClient {
    fn convert_account(&self, account: PlaidAccount) -> Account {
        let currency = account
            .balances
            .iso_currency_code
            .or(account.balances.unofficial_currency_code)
            .unwrap_or_else(|| "USD".to_string());

        Account {
            id: account.account_id.clone(),
            name: account.name,
            official_name: account.official_name,
            account_type: match account.account_type.as_str() {
                "depository" => crate::AccountType::Depository,
                "credit" => crate::AccountType::Credit,
                "loan" => crate::AccountType::Loan,
                "investment" => crate::AccountType::Investment,
                "brokerage" => crate::AccountType::Brokerage,
                _ => crate::AccountType::Other,
            },
            subtype: account.subtype,
            mask: account.mask,
            currency: currency.clone(),
            institution_id: None,
            balances: AccountBalance {
                account_id: account.account_id,
                current: account.balances.current.unwrap_or(0.0),
                available: account.balances.available,
                limit: account.balances.limit,
                currency,
                last_updated: None,
            },
            account_number: None,
            extra: std::collections::HashMap::new(),
        }
    }

    fn convert_transaction(&self, tx: PlaidTransaction) -> Transaction {
        let currency = tx
            .iso_currency_code
            .or(tx.unofficial_currency_code)
            .unwrap_or_else(|| "USD".to_string());

        Transaction {
            id: tx.transaction_id,
            account_id: tx.account_id,
            amount: tx.amount,
            currency,
            date: chrono::NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive()),
            datetime: tx.datetime.and_then(|dt| {
                chrono::DateTime::parse_from_rfc3339(&dt)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
            name: tx.name,
            merchant_name: tx.merchant_name,
            merchant_id: tx.merchant_entity_id,
            category: tx.category,
            category_id: tx.category_id,
            pending: tx.pending,
            transaction_type: match tx.transaction_type.as_deref() {
                Some("special") | Some("place") => crate::TransactionType::Debit,
                _ => if tx.amount > 0.0 {
                    crate::TransactionType::Debit
                } else {
                    crate::TransactionType::Credit
                },
            },
            payment_channel: tx.payment_channel.map(|ch| match ch.as_str() {
                "online" => crate::PaymentChannel::Online,
                "in store" => crate::PaymentChannel::InStore,
                _ => crate::PaymentChannel::Other,
            }),
            location: tx.location.map(|loc| crate::TransactionLocation {
                address: loc.address,
                city: loc.city,
                region: loc.region,
                postal_code: loc.postal_code,
                country: loc.country,
                lat: loc.lat,
                lon: loc.lon,
                store_number: loc.store_number,
            }),
            counterparty: tx.counterparties.and_then(|cps| {
                cps.into_iter().next().map(|cp| crate::Counterparty {
                    name: cp.name,
                    entity_id: cp.entity_id,
                    counterparty_type: cp.counterparty_type,
                    logo_url: cp.logo_url,
                    website: cp.website,
                })
            }),
            extra: std::collections::HashMap::new(),
        }
    }

    fn convert_institution(&self, inst: PlaidInstitution) -> Institution {
        Institution {
            id: inst.institution_id,
            name: inst.name,
            url: inst.url,
            logo: inst.logo,
            primary_color: inst.primary_color,
            country_codes: inst.country_codes,
            products: inst
                .products
                .into_iter()
                .filter_map(|p| match p.as_str() {
                    "transactions" => Some(crate::Product::Transactions),
                    "auth" => Some(crate::Product::Auth),
                    "identity" => Some(crate::Product::Identity),
                    "assets" => Some(crate::Product::Assets),
                    "investments" => Some(crate::Product::Investments),
                    "liabilities" => Some(crate::Product::Liabilities),
                    "payment_initiation" => Some(crate::Product::PaymentInitiation),
                    "identity_verification" => Some(crate::Product::IdentityVerification),
                    "transfer" => Some(crate::Product::Transfer),
                    "employment" => Some(crate::Product::Employment),
                    "income" => Some(crate::Product::Income),
                    "standing_orders" => Some(crate::Product::Standing),
                    "recurring_transactions" => Some(crate::Product::RecurringTransactions),
                    _ => None,
                })
                .collect(),
            routing_numbers: inst.routing_numbers,
            oauth: inst.oauth,
            extra: std::collections::HashMap::new(),
        }
    }
}

#[async_trait]
impl BankingProvider for PlaidClient {
    async fn create_link_token(&self, request: CreateLinkTokenRequest) -> Result<LinkToken> {
        let products: Vec<String> = request
            .products
            .iter()
            .map(|p| match p {
                crate::Product::Transactions => "transactions",
                crate::Product::Auth => "auth",
                crate::Product::Identity => "identity",
                crate::Product::Assets => "assets",
                crate::Product::Investments => "investments",
                crate::Product::Liabilities => "liabilities",
                crate::Product::PaymentInitiation => "payment_initiation",
                crate::Product::IdentityVerification => "identity_verification",
                crate::Product::Transfer => "transfer",
                crate::Product::Employment => "employment",
                crate::Product::Income => "income",
                crate::Product::Standing => "standing_orders",
                crate::Product::RecurringTransactions => "recurring_transactions",
            })
            .map(String::from)
            .collect();

        let plaid_request = self.wrap_request(LinkTokenCreateRequest {
            user: LinkTokenUser {
                client_user_id: request.user_id,
            },
            client_name: request.client_name,
            products,
            country_codes: request.country_codes,
            language: request.language.unwrap_or_else(|| "en".to_string()),
            webhook: request.webhook,
            redirect_uri: request.redirect_uri,
        });

        let response: LinkTokenCreateResponse = self
            .post("/link/token/create", &plaid_request)
            .await?;

        Ok(LinkToken {
            link_token: response.link_token,
            expiration: response.expiration,
            request_id: Some(response.request_id),
        })
    }

    async fn exchange_public_token(&self, public_token: &str) -> Result<AccessToken> {
        let request = self.wrap_request(ItemPublicTokenExchangeRequest {
            public_token: public_token.to_string(),
        });

        let response: ItemPublicTokenExchangeResponse = self
            .post("/item/public_token/exchange", &request)
            .await?;

        Ok(AccessToken {
            access_token: response.access_token,
            item_id: Some(response.item_id),
            request_id: Some(response.request_id),
        })
    }

    async fn list_accounts(&self, access_token: &str) -> Result<Vec<Account>> {
        let request = self.wrap_request(AccountsGetRequest {
            access_token: access_token.to_string(),
        });

        let response: AccountsGetResponse = self.post("/accounts/get", &request).await?;

        Ok(response
            .accounts
            .into_iter()
            .map(|a| self.convert_account(a))
            .collect())
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
        let today = chrono::Utc::now().date_naive();
        let start_date = options
            .start_date
            .unwrap_or_else(|| today - chrono::Duration::days(30));
        let end_date = options.end_date.unwrap_or(today);

        let request = self.wrap_request(TransactionsGetRequest {
            access_token: access_token.to_string(),
            start_date: start_date.format("%Y-%m-%d").to_string(),
            end_date: end_date.format("%Y-%m-%d").to_string(),
            options: Some(TransactionsGetOptions {
                account_ids: options.account_ids,
                count: options.count,
                offset: options.offset,
                include_personal_finance_category: Some(true),
            }),
        });

        let response: TransactionsGetResponse = self
            .post("/transactions/get", &request)
            .await?;

        let count = options.count.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);
        let has_more = offset + count < response.total_transactions;

        Ok(TransactionList {
            transactions: response
                .transactions
                .into_iter()
                .map(|t| self.convert_transaction(t))
                .collect(),
            total_transactions: response.total_transactions,
            has_more,
            next_cursor: None,
        })
    }

    async fn get_transaction(
        &self,
        access_token: &str,
        transaction_id: &str,
    ) -> Result<Transaction> {
        let transactions = self
            .list_transactions(
                access_token,
                TransactionOptions {
                    count: Some(500),
                    ..Default::default()
                },
            )
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
        let query = options.query.clone().unwrap_or_default();
        let country_codes = options
            .country_codes
            .clone()
            .unwrap_or_else(|| vec!["US".to_string()]);
        let products: Vec<String> = options
            .products
            .clone()
            .unwrap_or_else(|| vec![crate::Product::Transactions])
            .iter()
            .map(|p| match p {
                crate::Product::Transactions => "transactions",
                crate::Product::Auth => "auth",
                crate::Product::Identity => "identity",
                crate::Product::Assets => "assets",
                crate::Product::Investments => "investments",
                crate::Product::Liabilities => "liabilities",
                crate::Product::PaymentInitiation => "payment_initiation",
                crate::Product::IdentityVerification => "identity_verification",
                crate::Product::Transfer => "transfer",
                crate::Product::Employment => "employment",
                crate::Product::Income => "income",
                crate::Product::Standing => "standing_orders",
                crate::Product::RecurringTransactions => "recurring_transactions",
            })
            .map(String::from)
            .collect();

        let request = self.wrap_request(InstitutionsSearchRequest {
            query,
            products,
            country_codes,
            options: Some(InstitutionsSearchOptions {
                include_optional_metadata: options.include_optional_metadata,
                oauth: options.oauth,
            }),
        });

        let response: InstitutionsSearchResponse = self
            .post("/institutions/search", &request)
            .await?;

        Ok(response
            .institutions
            .into_iter()
            .map(|i| self.convert_institution(i))
            .collect())
    }

    async fn get_institution(&self, institution_id: &str) -> Result<Institution> {
        let request = self.wrap_request(InstitutionsGetByIdRequest {
            institution_id: institution_id.to_string(),
            country_codes: vec!["US".to_string()],
            options: Some(InstitutionsGetByIdOptions {
                include_optional_metadata: true,
            }),
        });

        let response: InstitutionGetByIdResponse = self
            .post("/institutions/get_by_id", &request)
            .await?;

        Ok(self.convert_institution(response.institution))
    }

    async fn get_identity(&self, access_token: &str) -> Result<Vec<AccountIdentity>> {
        let request = self.wrap_request(IdentityGetRequest {
            access_token: access_token.to_string(),
        });

        let response: IdentityGetResponse = self.post("/identity/get", &request).await?;

        Ok(response
            .accounts
            .into_iter()
            .map(|a| AccountIdentity {
                account_id: a.account_id,
                owners: a
                    .owners
                    .into_iter()
                    .map(|o| crate::Owner {
                        names: o.names,
                        phone_numbers: o
                            .phone_numbers
                            .into_iter()
                            .map(|p| crate::PhoneNumber {
                                data: p.data,
                                primary: p.primary,
                                phone_type: p.phone_type,
                            })
                            .collect(),
                        emails: o
                            .emails
                            .into_iter()
                            .map(|e| crate::Email {
                                data: e.data,
                                primary: e.primary,
                                email_type: e.email_type,
                            })
                            .collect(),
                        addresses: o
                            .addresses
                            .into_iter()
                            .map(|a| crate::Address {
                                street: a.data.street,
                                city: a.data.city,
                                region: a.data.region,
                                postal_code: a.data.postal_code,
                                country: a.data.country,
                                primary: a.primary,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect())
    }

    async fn create_payment(
        &self,
        _access_token: &str,
        _request: CreatePaymentRequest,
    ) -> Result<Payment> {
        Err(Error::Provider(
            "Payment initiation requires UK/EU configuration".to_string(),
        ))
    }

    async fn get_payment(&self, _access_token: &str, _payment_id: &str) -> Result<Payment> {
        Err(Error::Provider(
            "Payment initiation requires UK/EU configuration".to_string(),
        ))
    }

    async fn remove_item(&self, access_token: &str) -> Result<()> {
        let request = self.wrap_request(ItemRemoveRequest {
            access_token: access_token.to_string(),
        });

        let _response: ItemRemoveResponse = self.post("/item/remove", &request).await?;
        Ok(())
    }

    async fn get_item(&self, access_token: &str) -> Result<Item> {
        let request = self.wrap_request(ItemGetRequest {
            access_token: access_token.to_string(),
        });

        let response: ItemGetResponse = self.post("/item/get", &request).await?;
        let item = response.item;

        Ok(Item {
            id: item.item_id,
            institution_id: item.institution_id,
            webhook: item.webhook,
            error: item.error.map(|e| crate::ItemError {
                error_type: e.error_type,
                error_code: e.error_code,
                error_message: e.error_message,
                display_message: e.display_message,
            }),
            available_products: item
                .available_products
                .into_iter()
                .filter_map(|p| match p.as_str() {
                    "transactions" => Some(crate::Product::Transactions),
                    "auth" => Some(crate::Product::Auth),
                    "identity" => Some(crate::Product::Identity),
                    "assets" => Some(crate::Product::Assets),
                    "investments" => Some(crate::Product::Investments),
                    "liabilities" => Some(crate::Product::Liabilities),
                    "payment_initiation" => Some(crate::Product::PaymentInitiation),
                    "identity_verification" => Some(crate::Product::IdentityVerification),
                    "transfer" => Some(crate::Product::Transfer),
                    "employment" => Some(crate::Product::Employment),
                    "income" => Some(crate::Product::Income),
                    "standing_orders" => Some(crate::Product::Standing),
                    "recurring_transactions" => Some(crate::Product::RecurringTransactions),
                    _ => None,
                })
                .collect(),
            billed_products: item
                .billed_products
                .into_iter()
                .filter_map(|p| match p.as_str() {
                    "transactions" => Some(crate::Product::Transactions),
                    "auth" => Some(crate::Product::Auth),
                    "identity" => Some(crate::Product::Identity),
                    "assets" => Some(crate::Product::Assets),
                    "investments" => Some(crate::Product::Investments),
                    "liabilities" => Some(crate::Product::Liabilities),
                    "payment_initiation" => Some(crate::Product::PaymentInitiation),
                    "identity_verification" => Some(crate::Product::IdentityVerification),
                    "transfer" => Some(crate::Product::Transfer),
                    "employment" => Some(crate::Product::Employment),
                    "income" => Some(crate::Product::Income),
                    "standing_orders" => Some(crate::Product::Standing),
                    "recurring_transactions" => Some(crate::Product::RecurringTransactions),
                    _ => None,
                })
                .collect(),
            consent_expiration_time: item.consent_expiration_time,
            update_type: item.update_type,
            extra: std::collections::HashMap::new(),
        })
    }
}
