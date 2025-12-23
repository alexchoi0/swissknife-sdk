use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "banking")]
use swissknife_banking_sdk as banking;

#[derive(Clone)]
pub struct BankingTools {
    #[cfg(feature = "plaid")]
    pub plaid: Option<banking::plaid::PlaidClient>,
    #[cfg(feature = "teller")]
    pub teller: Option<banking::teller::TellerClient>,
}

impl BankingTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "plaid")]
            plaid: None,
            #[cfg(feature = "teller")]
            teller: None,
        }
    }

    #[cfg(feature = "plaid")]
    pub fn with_plaid(mut self, client: banking::plaid::PlaidClient) -> Self {
        self.plaid = Some(client);
        self
    }

    #[cfg(feature = "teller")]
    pub fn with_teller(mut self, client: banking::teller::TellerClient) -> Self {
        self.teller = Some(client);
        self
    }
}

impl Default for BankingTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlaidGetAccountsRequest {
    pub access_token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlaidGetTransactionsRequest {
    pub access_token: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub count: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlaidGetBalanceRequest {
    pub access_token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlaidGetIdentityRequest {
    pub access_token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlaidCreateLinkTokenRequest {
    pub client_name: String,
    pub user_client_id: String,
    pub products: Vec<String>,
    pub country_codes: Vec<String>,
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TellerGetAccountsRequest {
    pub access_token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TellerGetAccountRequest {
    pub access_token: String,
    pub account_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TellerGetTransactionsRequest {
    pub access_token: String,
    pub account_id: String,
    #[serde(default)]
    pub count: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TellerGetBalanceRequest {
    pub access_token: String,
    pub account_id: String,
}

#[tool_box]
impl BankingTools {
    #[cfg(feature = "plaid")]
    #[rmcp::tool(description = "Get accounts from Plaid")]
    pub async fn plaid_get_accounts(
        &self,
        #[rmcp::tool(aggr)] req: PlaidGetAccountsRequest,
    ) -> Result<String, String> {
        let client = self.plaid.as_ref()
            .ok_or_else(|| "Plaid client not configured".to_string())?;

        let accounts = client.get_accounts(&req.access_token).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&accounts).map_err(|e| e.to_string())
    }

    #[cfg(feature = "plaid")]
    #[rmcp::tool(description = "Get transactions from Plaid")]
    pub async fn plaid_get_transactions(
        &self,
        #[rmcp::tool(aggr)] req: PlaidGetTransactionsRequest,
    ) -> Result<String, String> {
        let client = self.plaid.as_ref()
            .ok_or_else(|| "Plaid client not configured".to_string())?;

        let transactions = client.get_transactions(
            &req.access_token,
            &req.start_date,
            &req.end_date,
            req.count,
            req.offset,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&transactions).map_err(|e| e.to_string())
    }

    #[cfg(feature = "plaid")]
    #[rmcp::tool(description = "Get account balances from Plaid")]
    pub async fn plaid_get_balance(
        &self,
        #[rmcp::tool(aggr)] req: PlaidGetBalanceRequest,
    ) -> Result<String, String> {
        let client = self.plaid.as_ref()
            .ok_or_else(|| "Plaid client not configured".to_string())?;

        let balance = client.get_balance(&req.access_token).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&balance).map_err(|e| e.to_string())
    }

    #[cfg(feature = "plaid")]
    #[rmcp::tool(description = "Get identity information from Plaid")]
    pub async fn plaid_get_identity(
        &self,
        #[rmcp::tool(aggr)] req: PlaidGetIdentityRequest,
    ) -> Result<String, String> {
        let client = self.plaid.as_ref()
            .ok_or_else(|| "Plaid client not configured".to_string())?;

        let identity = client.get_identity(&req.access_token).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&identity).map_err(|e| e.to_string())
    }

    #[cfg(feature = "plaid")]
    #[rmcp::tool(description = "Create a Plaid Link token for onboarding")]
    pub async fn plaid_create_link_token(
        &self,
        #[rmcp::tool(aggr)] req: PlaidCreateLinkTokenRequest,
    ) -> Result<String, String> {
        let client = self.plaid.as_ref()
            .ok_or_else(|| "Plaid client not configured".to_string())?;

        let token = client.create_link_token(
            &req.client_name,
            &req.user_client_id,
            &req.products,
            &req.country_codes,
            &req.language,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&token).map_err(|e| e.to_string())
    }

    #[cfg(feature = "teller")]
    #[rmcp::tool(description = "Get accounts from Teller")]
    pub async fn teller_get_accounts(
        &self,
        #[rmcp::tool(aggr)] req: TellerGetAccountsRequest,
    ) -> Result<String, String> {
        let client = self.teller.as_ref()
            .ok_or_else(|| "Teller client not configured".to_string())?;

        let accounts = client.get_accounts(&req.access_token).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&accounts).map_err(|e| e.to_string())
    }

    #[cfg(feature = "teller")]
    #[rmcp::tool(description = "Get a specific account from Teller")]
    pub async fn teller_get_account(
        &self,
        #[rmcp::tool(aggr)] req: TellerGetAccountRequest,
    ) -> Result<String, String> {
        let client = self.teller.as_ref()
            .ok_or_else(|| "Teller client not configured".to_string())?;

        let account = client.get_account(&req.access_token, &req.account_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&account).map_err(|e| e.to_string())
    }

    #[cfg(feature = "teller")]
    #[rmcp::tool(description = "Get transactions from Teller")]
    pub async fn teller_get_transactions(
        &self,
        #[rmcp::tool(aggr)] req: TellerGetTransactionsRequest,
    ) -> Result<String, String> {
        let client = self.teller.as_ref()
            .ok_or_else(|| "Teller client not configured".to_string())?;

        let transactions = client.get_transactions(
            &req.access_token,
            &req.account_id,
            req.count,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&transactions).map_err(|e| e.to_string())
    }

    #[cfg(feature = "teller")]
    #[rmcp::tool(description = "Get account balance from Teller")]
    pub async fn teller_get_balance(
        &self,
        #[rmcp::tool(aggr)] req: TellerGetBalanceRequest,
    ) -> Result<String, String> {
        let client = self.teller.as_ref()
            .ok_or_else(|| "Teller client not configured".to_string())?;

        let balance = client.get_balance(&req.access_token, &req.account_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&balance).map_err(|e| e.to_string())
    }
}
