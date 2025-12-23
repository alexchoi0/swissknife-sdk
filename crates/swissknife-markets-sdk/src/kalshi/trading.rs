use crate::{Error, Result, Order, OrderSide, OrderStatus, OrderType, Position, Trade};
use crate::kalshi::KalshiClient;
use serde::{Deserialize, Serialize};

impl KalshiClient {
    pub async fn get_balance(&self) -> Result<Balance> {
        let auth = self.auth_header().ok_or_else(|| Error::Auth("Not authenticated".to_string()))?;

        let response = self.client()
            .get(format!("{}/portfolio/balance", self.base_url()))
            .header("Authorization", auth)
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

        let result: BalanceResponse = response.json().await?;
        Ok(result.balance)
    }

    pub async fn get_positions(&self, params: Option<PositionsParams>) -> Result<PositionsResponse> {
        let auth = self.auth_header().ok_or_else(|| Error::Auth("Not authenticated".to_string()))?;

        let mut request = self.client()
            .get(format!("{}/portfolio/positions", self.base_url()))
            .header("Authorization", auth);

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(cursor) = p.cursor {
                query.push(("cursor", cursor));
            }
            if let Some(ticker) = p.ticker {
                query.push(("ticker", ticker));
            }
            if let Some(event_ticker) = p.event_ticker {
                query.push(("event_ticker", event_ticker));
            }
            if let Some(settlement_status) = p.settlement_status {
                query.push(("settlement_status", settlement_status));
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

        let result: PositionsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn create_order(&self, order: CreateOrderRequest) -> Result<KalshiOrder> {
        let auth = self.auth_header().ok_or_else(|| Error::Auth("Not authenticated".to_string()))?;

        let response = self.client()
            .post(format!("{}/portfolio/orders", self.base_url()))
            .header("Authorization", auth)
            .json(&order)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();

            if status.as_u16() == 400 && text.contains("insufficient") {
                return Err(Error::InsufficientBalance(text));
            }

            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: OrderResponse = response.json().await?;
        Ok(result.order)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<KalshiOrder> {
        let auth = self.auth_header().ok_or_else(|| Error::Auth("Not authenticated".to_string()))?;

        let response = self.client()
            .get(format!("{}/portfolio/orders/{}", self.base_url(), order_id))
            .header("Authorization", auth)
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

        let result: OrderResponse = response.json().await?;
        Ok(result.order)
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<KalshiOrder> {
        let auth = self.auth_header().ok_or_else(|| Error::Auth("Not authenticated".to_string()))?;

        let response = self.client()
            .delete(format!("{}/portfolio/orders/{}", self.base_url(), order_id))
            .header("Authorization", auth)
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

        let result: OrderResponse = response.json().await?;
        Ok(result.order)
    }

    pub async fn list_orders(&self, params: Option<ListOrdersParams>) -> Result<OrdersResponse> {
        let auth = self.auth_header().ok_or_else(|| Error::Auth("Not authenticated".to_string()))?;

        let mut request = self.client()
            .get(format!("{}/portfolio/orders", self.base_url()))
            .header("Authorization", auth);

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(cursor) = p.cursor {
                query.push(("cursor", cursor));
            }
            if let Some(ticker) = p.ticker {
                query.push(("ticker", ticker));
            }
            if let Some(status) = p.status {
                query.push(("status", status));
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

        let result: OrdersResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_fills(&self, params: Option<ListFillsParams>) -> Result<FillsResponse> {
        let auth = self.auth_header().ok_or_else(|| Error::Auth("Not authenticated".to_string()))?;

        let mut request = self.client()
            .get(format!("{}/portfolio/fills", self.base_url()))
            .header("Authorization", auth);

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(cursor) = p.cursor {
                query.push(("cursor", cursor));
            }
            if let Some(ticker) = p.ticker {
                query.push(("ticker", ticker));
            }
            if let Some(order_id) = p.order_id {
                query.push(("order_id", order_id));
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

        let result: FillsResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositionsParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub ticker: Option<String>,
    pub event_ticker: Option<String>,
    pub settlement_status: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListOrdersParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub ticker: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListFillsParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub ticker: Option<String>,
    pub order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    pub ticker: String,
    pub action: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceResponse {
    pub balance: Balance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Balance {
    pub balance: i64,
    pub payout: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionsResponse {
    pub market_positions: Vec<KalshiPosition>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KalshiPosition {
    pub ticker: String,
    pub event_ticker: String,
    pub event_title: Option<String>,
    pub market_title: Option<String>,
    pub position: i64,
    pub total_traded: i64,
    pub realized_pnl: Option<i64>,
    pub resting_orders_count: i32,
    pub settlement_status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub order: KalshiOrder,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrdersResponse {
    pub orders: Vec<KalshiOrder>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KalshiOrder {
    pub order_id: String,
    pub ticker: String,
    pub action: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub status: String,
    pub yes_price: Option<i32>,
    pub no_price: Option<i32>,
    pub created_time: Option<String>,
    pub expiration_time: Option<String>,
    pub order_group_id: Option<String>,
    pub remaining_count: i32,
    pub initial_count: i32,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FillsResponse {
    pub fills: Vec<KalshiFill>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KalshiFill {
    pub trade_id: String,
    pub order_id: String,
    pub ticker: String,
    pub side: String,
    pub action: String,
    pub count: i32,
    pub yes_price: i32,
    pub no_price: i32,
    pub is_taker: bool,
    pub created_time: String,
}

impl From<KalshiPosition> for Position {
    fn from(p: KalshiPosition) -> Self {
        Self {
            market_id: p.ticker.clone(),
            outcome_id: format!("{}_yes", p.ticker),
            quantity: p.position as i32,
            average_price: 0.0,
            current_value: None,
            profit_loss: p.realized_pnl.map(|pnl| pnl as f64 / 100.0),
        }
    }
}

impl From<KalshiOrder> for Order {
    fn from(o: KalshiOrder) -> Self {
        Self {
            id: o.order_id,
            market_id: o.ticker.clone(),
            outcome_id: format!("{}_{}", o.ticker, o.side),
            side: match o.action.as_str() {
                "buy" => OrderSide::Buy,
                _ => OrderSide::Sell,
            },
            order_type: match o.order_type.as_str() {
                "market" => OrderType::Market,
                _ => OrderType::Limit,
            },
            price: o.yes_price.unwrap_or(0) as f64 / 100.0,
            quantity: o.initial_count as u32,
            filled_quantity: (o.initial_count - o.remaining_count) as u32,
            status: match o.status.as_str() {
                "resting" => OrderStatus::Open,
                "pending" => OrderStatus::Pending,
                "executed" => OrderStatus::Filled,
                "canceled" => OrderStatus::Cancelled,
                _ => OrderStatus::Pending,
            },
            created_at: o.created_time,
        }
    }
}

impl From<KalshiFill> for Trade {
    fn from(f: KalshiFill) -> Self {
        Self {
            id: f.trade_id,
            market_id: f.ticker.clone(),
            outcome_id: format!("{}_{}", f.ticker, f.side),
            side: match f.action.as_str() {
                "buy" => OrderSide::Buy,
                _ => OrderSide::Sell,
            },
            price: f.yes_price as f64 / 100.0,
            quantity: f.count as u32,
            timestamp: f.created_time,
        }
    }
}
