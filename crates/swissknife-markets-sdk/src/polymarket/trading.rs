use crate::{Error, Result, Order, OrderSide, OrderStatus, OrderType, Position};
use crate::polymarket::PolymarketClient;
use serde::{Deserialize, Serialize};

impl PolymarketClient {
    pub async fn create_order(&self, order: CreateOrderRequest) -> Result<OrderResponse> {
        if !self.is_authenticated() {
            return Err(Error::Auth("Not authenticated".to_string()));
        }

        let response = self.client()
            .post(format!("{}/order", self.clob_url()))
            .header("POLY_API_KEY", self.api_key().unwrap_or_default())
            .header("POLY_SECRET", self.api_secret().unwrap_or_default())
            .header("POLY_PASSPHRASE", self.api_passphrase().unwrap_or_default())
            .json(&order)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();

            if text.contains("insufficient") {
                return Err(Error::InsufficientBalance(text));
            }

            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: OrderResponse = response.json().await?;
        Ok(result)
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelResponse> {
        if !self.is_authenticated() {
            return Err(Error::Auth("Not authenticated".to_string()));
        }

        let body = serde_json::json!({
            "orderID": order_id
        });

        let response = self.client()
            .delete(format!("{}/order", self.clob_url()))
            .header("POLY_API_KEY", self.api_key().unwrap_or_default())
            .header("POLY_SECRET", self.api_secret().unwrap_or_default())
            .header("POLY_PASSPHRASE", self.api_passphrase().unwrap_or_default())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: CancelResponse = response.json().await?;
        Ok(result)
    }

    pub async fn cancel_all_orders(&self) -> Result<CancelAllResponse> {
        if !self.is_authenticated() {
            return Err(Error::Auth("Not authenticated".to_string()));
        }

        let response = self.client()
            .delete(format!("{}/cancel-all", self.clob_url()))
            .header("POLY_API_KEY", self.api_key().unwrap_or_default())
            .header("POLY_SECRET", self.api_secret().unwrap_or_default())
            .header("POLY_PASSPHRASE", self.api_passphrase().unwrap_or_default())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: CancelAllResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<PolymarketOrder> {
        if !self.is_authenticated() {
            return Err(Error::Auth("Not authenticated".to_string()));
        }

        let response = self.client()
            .get(format!("{}/order/{}", self.clob_url(), order_id))
            .header("POLY_API_KEY", self.api_key().unwrap_or_default())
            .header("POLY_SECRET", self.api_secret().unwrap_or_default())
            .header("POLY_PASSPHRASE", self.api_passphrase().unwrap_or_default())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: PolymarketOrder = response.json().await?;
        Ok(result)
    }

    pub async fn get_open_orders(&self, params: Option<OpenOrdersParams>) -> Result<Vec<PolymarketOrder>> {
        if !self.is_authenticated() {
            return Err(Error::Auth("Not authenticated".to_string()));
        }

        let mut request = self.client()
            .get(format!("{}/orders", self.clob_url()))
            .header("POLY_API_KEY", self.api_key().unwrap_or_default())
            .header("POLY_SECRET", self.api_secret().unwrap_or_default())
            .header("POLY_PASSPHRASE", self.api_passphrase().unwrap_or_default());

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(market) = p.market {
                query.push(("market", market));
            }
            if let Some(asset_id) = p.asset_id {
                query.push(("asset_id", asset_id));
            }
            if !query.is_empty() {
                request = request.query(&query);
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: Vec<PolymarketOrder> = response.json().await?;
        Ok(result)
    }

    pub async fn get_positions(&self) -> Result<Vec<PolymarketPosition>> {
        if !self.is_authenticated() {
            return Err(Error::Auth("Not authenticated".to_string()));
        }

        let response = self.client()
            .get(format!("{}/positions", self.clob_url()))
            .header("POLY_API_KEY", self.api_key().unwrap_or_default())
            .header("POLY_SECRET", self.api_secret().unwrap_or_default())
            .header("POLY_PASSPHRASE", self.api_passphrase().unwrap_or_default())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: Vec<PolymarketPosition> = response.json().await?;
        Ok(result)
    }

    pub async fn get_api_keys(&self) -> Result<ApiKeysResponse> {
        if !self.is_authenticated() {
            return Err(Error::Auth("Not authenticated".to_string()));
        }

        let response = self.client()
            .get(format!("{}/api-keys", self.clob_url()))
            .header("POLY_API_KEY", self.api_key().unwrap_or_default())
            .header("POLY_SECRET", self.api_secret().unwrap_or_default())
            .header("POLY_PASSPHRASE", self.api_passphrase().unwrap_or_default())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ApiKeysResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct OpenOrdersParams {
    pub market: Option<String>,
    pub asset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    pub order: SignedOrder,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignedOrder {
    pub salt: String,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "makerAmount")]
    pub maker_amount: String,
    #[serde(rename = "takerAmount")]
    pub taker_amount: String,
    pub expiration: String,
    pub nonce: String,
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: String,
    pub side: String,
    #[serde(rename = "signatureType")]
    pub signature_type: i32,
    pub signature: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub success: bool,
    #[serde(rename = "errorMsg")]
    pub error_msg: Option<String>,
    #[serde(rename = "orderID")]
    pub order_id: Option<String>,
    #[serde(rename = "transactionsHashes")]
    pub transaction_hashes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelResponse {
    pub canceled: Option<String>,
    pub not_canceled: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelAllResponse {
    pub canceled: Option<Vec<String>>,
    pub not_canceled: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolymarketOrder {
    pub id: String,
    pub status: String,
    pub owner: Option<String>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub side: String,
    pub original_size: Option<String>,
    pub size_matched: Option<String>,
    pub price: Option<String>,
    pub associate_trades: Option<Vec<String>>,
    pub outcome: Option<String>,
    pub created_at: Option<String>,
    pub expiration: Option<String>,
    #[serde(rename = "type")]
    pub order_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolymarketPosition {
    pub asset_id: String,
    pub market: Option<String>,
    pub size: String,
    pub avg_price: Option<String>,
    pub side: Option<String>,
    pub cur_price: Option<String>,
    pub realized_pnl: Option<String>,
    pub unrealized_pnl: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeysResponse {
    #[serde(rename = "apiKeys")]
    pub api_keys: Vec<ApiKeyInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyInfo {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

impl From<PolymarketPosition> for Position {
    fn from(p: PolymarketPosition) -> Self {
        Self {
            market_id: p.market.unwrap_or_default(),
            outcome_id: p.asset_id,
            quantity: p.size.parse().unwrap_or(0),
            average_price: p.avg_price.and_then(|p| p.parse().ok()).unwrap_or(0.0),
            current_value: p.cur_price.and_then(|p| p.parse().ok()),
            profit_loss: p.realized_pnl.and_then(|pnl| pnl.parse().ok()),
        }
    }
}

impl From<PolymarketOrder> for Order {
    fn from(o: PolymarketOrder) -> Self {
        Self {
            id: o.id,
            market_id: o.market.unwrap_or_default(),
            outcome_id: o.asset_id.unwrap_or_default(),
            side: match o.side.as_str() {
                "BUY" | "buy" => OrderSide::Buy,
                _ => OrderSide::Sell,
            },
            order_type: match o.order_type.as_deref() {
                Some("MARKET") => OrderType::Market,
                _ => OrderType::Limit,
            },
            price: o.price.and_then(|p| p.parse().ok()).unwrap_or(0.0),
            quantity: o.original_size.and_then(|s| s.parse().ok()).unwrap_or(0),
            filled_quantity: o.size_matched.and_then(|s| s.parse().ok()).unwrap_or(0),
            status: match o.status.as_str() {
                "LIVE" | "live" => OrderStatus::Open,
                "MATCHED" | "matched" => OrderStatus::Filled,
                "CANCELED" | "canceled" => OrderStatus::Cancelled,
                _ => OrderStatus::Pending,
            },
            created_at: o.created_at,
        }
    }
}
