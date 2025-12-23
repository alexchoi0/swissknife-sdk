mod error;

pub use error::{Error, Result};

#[cfg(feature = "kalshi")]
pub mod kalshi;

#[cfg(feature = "polymarket")]
pub mod polymarket;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: MarketStatus,
    pub category: Option<String>,
    pub end_date: Option<String>,
    pub created_at: Option<String>,
    pub volume: Option<f64>,
    pub liquidity: Option<f64>,
    pub outcomes: Vec<Outcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    Open,
    Closed,
    Resolved,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub id: String,
    pub title: String,
    pub price: f64,
    pub probability: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub market_id: String,
    pub outcome_id: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: u32,
    pub filled_quantity: u32,
    pub status: OrderStatus,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Open,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub market_id: String,
    pub outcome_id: String,
    pub quantity: i32,
    pub average_price: f64,
    pub current_value: Option<f64>,
    pub profit_loss: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub market_id: String,
    pub outcome_id: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: u32,
    pub timestamp: String,
}

#[async_trait]
pub trait PredictionMarket: Send + Sync {
    async fn list_markets(&self, category: Option<&str>, status: Option<MarketStatus>) -> Result<Vec<Market>>;
    async fn get_market(&self, market_id: &str) -> Result<Market>;
    async fn get_orderbook(&self, market_id: &str) -> Result<Orderbook>;
    async fn place_order(&self, order: &OrderRequest) -> Result<Order>;
    async fn cancel_order(&self, order_id: &str) -> Result<()>;
    async fn get_positions(&self) -> Result<Vec<Position>>;
    async fn get_trades(&self, market_id: Option<&str>) -> Result<Vec<Trade>>;
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderRequest {
    pub market_id: String,
    pub outcome_id: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub market_id: String,
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevel {
    pub price: f64,
    pub quantity: u32,
}
