pub mod domain;
pub mod formatter;
pub mod generator;

use crate::backend::{Backend, HttpMethod, HttpRequest, HttpResponse};
use async_trait::async_trait;
use domain::{account, institution, transaction, user};
use formatter::ProviderFormatter;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
    QueryFilter, Schema, Set,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Plaid,
    TrueLayer,
    Teller,
    GoCardless,
    Yapily,
    Mx,
}

impl Provider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "plaid" => Some(Self::Plaid),
            "truelayer" => Some(Self::TrueLayer),
            "teller" => Some(Self::Teller),
            "gocardless" => Some(Self::GoCardless),
            "yapily" => Some(Self::Yapily),
            "mx" => Some(Self::Mx),
            _ => None,
        }
    }
}

pub struct FakeBackend {
    db: DatabaseConnection,
    provider: Provider,
    formatter: Arc<dyn ProviderFormatter>,
    #[allow(dead_code)]
    access_tokens: Arc<RwLock<HashMap<String, String>>>,
}

impl FakeBackend {
    pub async fn new(provider: Provider) -> crate::Result<Self> {
        let db = Database::connect("sqlite::memory:")
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to connect to SQLite: {}", e)))?;

        let formatter: Arc<dyn ProviderFormatter> = match provider {
            Provider::Plaid => Arc::new(formatter::PlaidFormatter),
            Provider::TrueLayer => Arc::new(formatter::TrueLayerFormatter),
            Provider::Teller => Arc::new(formatter::TellerFormatter),
            Provider::GoCardless => Arc::new(formatter::GoCardlessFormatter),
            Provider::Yapily => Arc::new(formatter::YapilyFormatter),
            Provider::Mx => Arc::new(formatter::MxFormatter),
        };

        let backend = Self {
            db,
            provider,
            formatter,
            access_tokens: Arc::new(RwLock::new(HashMap::new())),
        };

        backend.create_tables().await?;

        Ok(backend)
    }

    async fn create_tables(&self) -> crate::Result<()> {
        let builder = self.db.get_database_backend();
        let schema = Schema::new(builder);

        let tables = vec![
            schema.create_table_from_entity(user::Entity),
            schema.create_table_from_entity(institution::Entity),
            schema.create_table_from_entity(account::Entity),
            schema.create_table_from_entity(transaction::Entity),
        ];

        for table in tables {
            let stmt = builder.build(&table);
            self.db
                .execute(stmt)
                .await
                .map_err(|e| crate::Error::Provider(format!("Failed to create table: {}", e)))?;
        }

        Ok(())
    }

    pub async fn create_user(&self, name: Option<String>, email: Option<String>) -> crate::Result<user::Model> {
        let user = generator::generate_user(name, email);
        let model = user::ActiveModel {
            id: Set(user.id),
            external_id: Set(user.external_id),
            name: Set(user.name),
            email: Set(user.email),
            phone: Set(user.phone),
            address_street: Set(user.address_street),
            address_city: Set(user.address_city),
            address_state: Set(user.address_state),
            address_postal_code: Set(user.address_postal_code),
            address_country: Set(user.address_country),
            date_of_birth: Set(user.date_of_birth),
            created_at: Set(user.created_at),
        };

        model
            .insert(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create user: {}", e)))
    }

    pub async fn create_institution(&self, name: Option<String>, country: Option<String>) -> crate::Result<institution::Model> {
        let inst = generator::generate_institution(name, country);
        let model = institution::ActiveModel {
            id: Set(inst.id),
            external_id: Set(inst.external_id),
            name: Set(inst.name),
            bic: Set(inst.bic),
            logo_url: Set(inst.logo_url),
            country: Set(inst.country),
            supported_features: Set(inst.supported_features),
            created_at: Set(inst.created_at),
        };

        model
            .insert(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create institution: {}", e)))
    }

    pub async fn create_account(
        &self,
        user_id: &str,
        institution_id: &str,
        account_type: Option<String>,
        balance: Option<f64>,
    ) -> crate::Result<account::Model> {
        let acc = generator::generate_account(user_id, institution_id, account_type, balance);
        let model = account::ActiveModel {
            id: Set(acc.id),
            external_id: Set(acc.external_id),
            user_id: Set(acc.user_id),
            institution_id: Set(acc.institution_id),
            name: Set(acc.name),
            account_type: Set(acc.account_type),
            subtype: Set(acc.subtype),
            currency: Set(acc.currency),
            balance_available: Set(acc.balance_available),
            balance_current: Set(acc.balance_current),
            balance_limit: Set(acc.balance_limit),
            iban: Set(acc.iban),
            account_number: Set(acc.account_number),
            routing_number: Set(acc.routing_number),
            sort_code: Set(acc.sort_code),
            mask: Set(acc.mask),
            status: Set(acc.status),
            created_at: Set(acc.created_at),
            updated_at: Set(acc.updated_at),
        };

        model
            .insert(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create account: {}", e)))
    }

    pub async fn create_transaction(
        &self,
        account_id: &str,
        amount: Option<f64>,
        description: Option<String>,
    ) -> crate::Result<transaction::Model> {
        let txn = generator::generate_transaction(account_id, amount, description);
        let model = transaction::ActiveModel {
            id: Set(txn.id),
            external_id: Set(txn.external_id),
            account_id: Set(txn.account_id),
            amount: Set(txn.amount),
            currency: Set(txn.currency),
            description: Set(txn.description),
            merchant_name: Set(txn.merchant_name),
            category: Set(txn.category),
            transaction_type: Set(txn.transaction_type),
            status: Set(txn.status),
            date: Set(txn.date),
            posted_at: Set(txn.posted_at),
            created_at: Set(txn.created_at),
        };

        model
            .insert(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to create transaction: {}", e)))
    }

    pub async fn seed_default_data(&self) -> crate::Result<()> {
        let inst = self.create_institution(Some("Sandbox Bank".to_string()), Some("US".to_string())).await?;
        let user = self.create_user(Some("John Doe".to_string()), Some("john.doe@example.com".to_string())).await?;

        let checking = self.create_account(&user.id, &inst.id, Some("checking".to_string()), Some(2500.0)).await?;
        let savings = self.create_account(&user.id, &inst.id, Some("savings".to_string()), Some(10000.0)).await?;

        for _ in 0..10 {
            self.create_transaction(&checking.id, None, None).await?;
        }
        for _ in 0..5 {
            self.create_transaction(&savings.id, None, None).await?;
        }

        Ok(())
    }

    pub async fn get_users(&self) -> crate::Result<Vec<user::Model>> {
        user::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get users: {}", e)))
    }

    pub async fn get_user(&self, id: &str) -> crate::Result<Option<user::Model>> {
        user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get user: {}", e)))
    }

    pub async fn get_institutions(&self) -> crate::Result<Vec<institution::Model>> {
        institution::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get institutions: {}", e)))
    }

    pub async fn get_institution(&self, id: &str) -> crate::Result<Option<institution::Model>> {
        institution::Entity::find()
            .filter(institution::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get institution: {}", e)))
    }

    pub async fn get_accounts(&self, user_id: Option<&str>) -> crate::Result<Vec<account::Model>> {
        let mut query = account::Entity::find();
        if let Some(uid) = user_id {
            query = query.filter(account::Column::UserId.eq(uid));
        }
        query
            .all(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get accounts: {}", e)))
    }

    pub async fn get_account(&self, id: &str) -> crate::Result<Option<account::Model>> {
        account::Entity::find()
            .filter(account::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get account: {}", e)))
    }

    pub async fn get_transactions(&self, account_id: &str) -> crate::Result<Vec<transaction::Model>> {
        transaction::Entity::find()
            .filter(transaction::Column::AccountId.eq(account_id))
            .all(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get transactions: {}", e)))
    }

    pub async fn get_transaction(&self, id: &str) -> crate::Result<Option<transaction::Model>> {
        transaction::Entity::find()
            .filter(transaction::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to get transaction: {}", e)))
    }

    pub async fn delete_account(&self, id: &str) -> crate::Result<()> {
        transaction::Entity::delete_many()
            .filter(transaction::Column::AccountId.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to delete transactions: {}", e)))?;

        account::Entity::delete_many()
            .filter(account::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| crate::Error::Provider(format!("Failed to delete account: {}", e)))?;

        Ok(())
    }

    async fn handle_request(&self, request: &HttpRequest) -> crate::Result<HttpResponse> {
        let path = self.extract_path(&request.url);
        let method = request.method;

        match (method, self.classify_endpoint(&path)) {
            (HttpMethod::Post, Endpoint::Token) => self.handle_token_request(request).await,
            (HttpMethod::Post, Endpoint::LinkToken) => self.handle_link_token_request(request).await,
            (HttpMethod::Get, Endpoint::Accounts) => self.handle_list_accounts(request).await,
            (HttpMethod::Post, Endpoint::Accounts) => self.handle_list_accounts(request).await,
            (HttpMethod::Get, Endpoint::Account(id)) => self.handle_get_account(&id).await,
            (HttpMethod::Get, Endpoint::AccountBalances(id)) => self.handle_get_balances(&id).await,
            (HttpMethod::Get, Endpoint::AccountTransactions(id)) => self.handle_get_transactions(&id).await,
            (HttpMethod::Post, Endpoint::Transactions) => self.handle_list_transactions(request).await,
            (HttpMethod::Get, Endpoint::Institutions) => self.handle_list_institutions().await,
            (HttpMethod::Get, Endpoint::Institution(id)) => self.handle_get_institution(&id).await,
            (HttpMethod::Post, Endpoint::InstitutionsSearch) => self.handle_search_institutions(request).await,
            (HttpMethod::Get, Endpoint::Identity) => self.handle_get_identity().await,
            (HttpMethod::Post, Endpoint::Identity) => self.handle_get_identity().await,
            (HttpMethod::Delete, Endpoint::Account(id)) => self.handle_delete_account(&id).await,
            _ => Err(crate::Error::Provider(format!(
                "Unknown endpoint: {} {}",
                method, path
            ))),
        }
    }

    fn extract_path(&self, url: &str) -> String {
        url.split('?')
            .next()
            .unwrap_or(url)
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .find('/')
            .map(|_| &url[url.find('/').unwrap_or(0)..])
            .unwrap_or("/")
            .split('?')
            .next()
            .unwrap_or("/")
            .to_string()
    }

    fn classify_endpoint(&self, path: &str) -> Endpoint {
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();

        match self.provider {
            Provider::Plaid => self.classify_plaid_endpoint(&parts),
            Provider::TrueLayer => self.classify_truelayer_endpoint(&parts),
            Provider::Teller => self.classify_teller_endpoint(&parts),
            Provider::GoCardless => self.classify_gocardless_endpoint(&parts),
            Provider::Yapily => self.classify_yapily_endpoint(&parts),
            Provider::Mx => self.classify_mx_endpoint(&parts),
        }
    }

    fn classify_plaid_endpoint(&self, parts: &[&str]) -> Endpoint {
        match parts {
            ["link", "token", "create"] => Endpoint::LinkToken,
            ["item", "public_token", "exchange"] => Endpoint::Token,
            ["accounts", "get"] => Endpoint::Accounts,
            ["transactions", "get"] => Endpoint::Transactions,
            ["institutions", "search"] => Endpoint::InstitutionsSearch,
            ["institutions", "get_by_id"] => Endpoint::InstitutionsSearch,
            ["identity", "get"] => Endpoint::Identity,
            ["item", "get"] => Endpoint::Accounts,
            ["item", "remove"] => Endpoint::DeleteItem,
            _ => Endpoint::Unknown,
        }
    }

    fn classify_truelayer_endpoint(&self, parts: &[&str]) -> Endpoint {
        match parts {
            ["connect", "token"] => Endpoint::Token,
            ["data", "v1", "accounts"] => Endpoint::Accounts,
            ["data", "v1", "accounts", id, "balance"] => Endpoint::AccountBalances(id.to_string()),
            ["data", "v1", "accounts", id, "transactions"] => Endpoint::AccountTransactions(id.to_string()),
            ["data", "v1", "info"] => Endpoint::Identity,
            _ => Endpoint::Unknown,
        }
    }

    fn classify_teller_endpoint(&self, parts: &[&str]) -> Endpoint {
        match parts {
            ["accounts"] => Endpoint::Accounts,
            ["accounts", id] => Endpoint::Account(id.to_string()),
            ["accounts", id, "balances"] => Endpoint::AccountBalances(id.to_string()),
            ["accounts", id, "transactions"] => Endpoint::AccountTransactions(id.to_string()),
            ["accounts", _id, "identity"] => Endpoint::Identity,
            _ => Endpoint::Unknown,
        }
    }

    fn classify_gocardless_endpoint(&self, parts: &[&str]) -> Endpoint {
        match parts {
            ["api", "v2", "token", "new"] => Endpoint::Token,
            ["api", "v2", "requisitions"] => Endpoint::LinkToken,
            ["api", "v2", "requisitions", _id] => Endpoint::LinkToken,
            ["api", "v2", "accounts", id] => Endpoint::Account(id.to_string()),
            ["api", "v2", "accounts", id, "details"] => Endpoint::Account(id.to_string()),
            ["api", "v2", "accounts", id, "balances"] => Endpoint::AccountBalances(id.to_string()),
            ["api", "v2", "accounts", id, "transactions"] => Endpoint::AccountTransactions(id.to_string()),
            ["api", "v2", "institutions"] => Endpoint::Institutions,
            ["api", "v2", "institutions", id] => Endpoint::Institution(id.to_string()),
            _ => Endpoint::Unknown,
        }
    }

    fn classify_yapily_endpoint(&self, parts: &[&str]) -> Endpoint {
        match parts {
            ["institutions"] => Endpoint::Institutions,
            ["institutions", id] => Endpoint::Institution(id.to_string()),
            ["accounts"] => Endpoint::Accounts,
            ["accounts", id] => Endpoint::Account(id.to_string()),
            ["accounts", id, "balances"] => Endpoint::AccountBalances(id.to_string()),
            ["accounts", id, "transactions"] => Endpoint::AccountTransactions(id.to_string()),
            ["identity"] => Endpoint::Identity,
            _ => Endpoint::Unknown,
        }
    }

    fn classify_mx_endpoint(&self, parts: &[&str]) -> Endpoint {
        match parts {
            ["users"] => Endpoint::Users,
            ["users", _user_guid] => Endpoint::Users,
            ["users", _user_guid, "accounts"] => Endpoint::Accounts,
            ["users", _user_guid, "accounts", id] => Endpoint::Account(id.to_string()),
            ["users", _user_guid, "accounts", id, "transactions"] => Endpoint::AccountTransactions(id.to_string()),
            ["users", _user_guid, "transactions"] => Endpoint::Transactions,
            ["institutions"] => Endpoint::Institutions,
            ["institutions", id] => Endpoint::Institution(id.to_string()),
            _ => Endpoint::Unknown,
        }
    }

    async fn handle_token_request(&self, _request: &HttpRequest) -> crate::Result<HttpResponse> {
        let token = generator::generate_token();
        let body = self.formatter.format_token(&token);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_link_token_request(&self, _request: &HttpRequest) -> crate::Result<HttpResponse> {
        let token = generator::generate_link_token();
        let body = self.formatter.format_link_token(&token);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_list_accounts(&self, _request: &HttpRequest) -> crate::Result<HttpResponse> {
        let accounts = self.get_accounts(None).await?;
        let body = self.formatter.format_accounts(&accounts);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_get_account(&self, id: &str) -> crate::Result<HttpResponse> {
        match self.get_account(id).await? {
            Some(account) => {
                let body = self.formatter.format_account(&account);
                Ok(HttpResponse {
                    status: 200,
                    headers: HashMap::new(),
                    body,
                })
            }
            None => Ok(HttpResponse {
                status: 404,
                headers: HashMap::new(),
                body: r#"{"error": "Account not found"}"#.to_string(),
            }),
        }
    }

    async fn handle_get_balances(&self, account_id: &str) -> crate::Result<HttpResponse> {
        match self.get_account(account_id).await? {
            Some(account) => {
                let body = self.formatter.format_balances(&account);
                Ok(HttpResponse {
                    status: 200,
                    headers: HashMap::new(),
                    body,
                })
            }
            None => Ok(HttpResponse {
                status: 404,
                headers: HashMap::new(),
                body: r#"{"error": "Account not found"}"#.to_string(),
            }),
        }
    }

    async fn handle_get_transactions(&self, account_id: &str) -> crate::Result<HttpResponse> {
        let transactions = self.get_transactions(account_id).await?;
        let body = self.formatter.format_transactions(&transactions);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_list_transactions(&self, _request: &HttpRequest) -> crate::Result<HttpResponse> {
        let accounts = self.get_accounts(None).await?;
        let mut all_transactions = Vec::new();
        for account in &accounts {
            let txns = self.get_transactions(&account.id).await?;
            all_transactions.extend(txns);
        }
        let body = self.formatter.format_transactions(&all_transactions);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_list_institutions(&self) -> crate::Result<HttpResponse> {
        let institutions = self.get_institutions().await?;
        let body = self.formatter.format_institutions(&institutions);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_get_institution(&self, id: &str) -> crate::Result<HttpResponse> {
        match self.get_institution(id).await? {
            Some(inst) => {
                let body = self.formatter.format_institution(&inst);
                Ok(HttpResponse {
                    status: 200,
                    headers: HashMap::new(),
                    body,
                })
            }
            None => Ok(HttpResponse {
                status: 404,
                headers: HashMap::new(),
                body: r#"{"error": "Institution not found"}"#.to_string(),
            }),
        }
    }

    async fn handle_search_institutions(&self, _request: &HttpRequest) -> crate::Result<HttpResponse> {
        let institutions = self.get_institutions().await?;
        let body = self.formatter.format_institutions(&institutions);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_get_identity(&self) -> crate::Result<HttpResponse> {
        let users = self.get_users().await?;
        let user = users.first();
        let body = self.formatter.format_identity(user);
        Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
        })
    }

    async fn handle_delete_account(&self, id: &str) -> crate::Result<HttpResponse> {
        self.delete_account(id).await?;
        Ok(HttpResponse {
            status: 204,
            headers: HashMap::new(),
            body: String::new(),
        })
    }
}

#[derive(Debug)]
enum Endpoint {
    Token,
    LinkToken,
    Users,
    Accounts,
    Account(String),
    AccountBalances(String),
    AccountTransactions(String),
    Transactions,
    Institutions,
    Institution(String),
    InstitutionsSearch,
    Identity,
    DeleteItem,
    Unknown,
}

#[async_trait]
impl Backend for FakeBackend {
    async fn execute(&self, request: HttpRequest) -> crate::Result<HttpResponse> {
        self.handle_request(&request).await
    }
}
