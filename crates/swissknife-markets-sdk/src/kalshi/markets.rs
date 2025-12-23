use crate::{Error, Result, Market, MarketStatus, Outcome, Orderbook, OrderbookLevel};
use crate::kalshi::KalshiClient;
use serde::Deserialize;

impl KalshiClient {
    pub async fn login(&mut self, email: &str, password: &str) -> Result<LoginResponse> {
        let body = serde_json::json!({
            "email": email,
            "password": password
        });

        let response = self.client()
            .post(format!("{}/login", self.base_url()))
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

        let result: LoginResponse = response.json().await?;
        self.set_token(&result.token);
        Ok(result)
    }

    pub async fn list_markets(&self, params: Option<ListMarketsParams>) -> Result<MarketsResponse> {
        let mut request = self.client()
            .get(format!("{}/markets", self.base_url()));

        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(cursor) = p.cursor {
                query.push(("cursor", cursor));
            }
            if let Some(event_ticker) = p.event_ticker {
                query.push(("event_ticker", event_ticker));
            }
            if let Some(series_ticker) = p.series_ticker {
                query.push(("series_ticker", series_ticker));
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

        let result: MarketsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_market(&self, ticker: &str) -> Result<KalshiMarket> {
        let mut request = self.client()
            .get(format!("{}/markets/{}", self.base_url(), ticker));

        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::MarketNotFound(ticker.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: MarketResponse = response.json().await?;
        Ok(result.market)
    }

    pub async fn get_orderbook(&self, ticker: &str, depth: Option<u32>) -> Result<KalshiOrderbook> {
        let mut request = self.client()
            .get(format!("{}/markets/{}/orderbook", self.base_url(), ticker));

        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        if let Some(d) = depth {
            request = request.query(&[("depth", d.to_string())]);
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

        let result: OrderbookResponse = response.json().await?;
        Ok(result.orderbook)
    }

    pub async fn get_market_history(&self, ticker: &str, params: Option<HistoryParams>) -> Result<Vec<HistoryPoint>> {
        let mut request = self.client()
            .get(format!("{}/markets/{}/history", self.base_url(), ticker));

        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(min_ts) = p.min_ts {
                query.push(("min_ts", min_ts.to_string()));
            }
            if let Some(max_ts) = p.max_ts {
                query.push(("max_ts", max_ts.to_string()));
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

        let result: HistoryResponse = response.json().await?;
        Ok(result.history)
    }

    pub async fn list_events(&self, params: Option<ListEventsParams>) -> Result<EventsResponse> {
        let mut request = self.client()
            .get(format!("{}/events", self.base_url()));

        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(cursor) = p.cursor {
                query.push(("cursor", cursor));
            }
            if let Some(status) = p.status {
                query.push(("status", status));
            }
            if let Some(series_ticker) = p.series_ticker {
                query.push(("series_ticker", series_ticker));
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

        let result: EventsResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListMarketsParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub event_ticker: Option<String>,
    pub series_ticker: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryParams {
    pub limit: Option<u32>,
    pub min_ts: Option<i64>,
    pub max_ts: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct ListEventsParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub status: Option<String>,
    pub series_ticker: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub member_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketsResponse {
    pub markets: Vec<KalshiMarket>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketResponse {
    pub market: KalshiMarket,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KalshiMarket {
    pub ticker: String,
    pub event_ticker: String,
    pub subtitle: Option<String>,
    pub open_time: Option<String>,
    pub close_time: Option<String>,
    pub expiration_time: Option<String>,
    pub status: String,
    pub yes_bid: Option<i32>,
    pub yes_ask: Option<i32>,
    pub no_bid: Option<i32>,
    pub no_ask: Option<i32>,
    pub last_price: Option<i32>,
    pub previous_yes_bid: Option<i32>,
    pub previous_yes_ask: Option<i32>,
    pub previous_price: Option<i32>,
    pub volume: Option<i64>,
    pub volume_24h: Option<i64>,
    pub liquidity: Option<i64>,
    pub open_interest: Option<i64>,
    pub result: Option<String>,
    pub strike_type: Option<String>,
    pub floor_strike: Option<f64>,
    pub cap_strike: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookResponse {
    pub orderbook: KalshiOrderbook,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KalshiOrderbook {
    pub yes: Vec<OrderbookEntry>,
    pub no: Vec<OrderbookEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookEntry {
    pub price: i32,
    pub quantity: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryResponse {
    pub history: Vec<HistoryPoint>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryPoint {
    pub ts: i64,
    pub yes_price: Option<i32>,
    pub yes_bid: Option<i32>,
    pub yes_ask: Option<i32>,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsResponse {
    pub events: Vec<KalshiEvent>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KalshiEvent {
    pub event_ticker: String,
    pub series_ticker: String,
    pub sub_title: Option<String>,
    pub title: String,
    pub category: Option<String>,
    pub mutually_exclusive: bool,
    pub strike_date: Option<String>,
    pub strike_period: Option<String>,
}

impl From<KalshiMarket> for Market {
    fn from(m: KalshiMarket) -> Self {
        let yes_prob = m.yes_bid.map(|b| b as f64 / 100.0);

        Self {
            id: m.ticker.clone(),
            title: m.subtitle.unwrap_or_else(|| m.ticker.clone()),
            description: None,
            status: match m.status.as_str() {
                "active" => MarketStatus::Open,
                "closed" => MarketStatus::Closed,
                "settled" => MarketStatus::Resolved,
                _ => MarketStatus::Pending,
            },
            category: None,
            end_date: m.expiration_time,
            created_at: m.open_time,
            volume: m.volume.map(|v| v as f64),
            liquidity: m.liquidity.map(|l| l as f64),
            outcomes: vec![
                Outcome {
                    id: format!("{}_yes", m.ticker),
                    title: "Yes".to_string(),
                    price: m.yes_bid.unwrap_or(0) as f64 / 100.0,
                    probability: yes_prob,
                },
                Outcome {
                    id: format!("{}_no", m.ticker),
                    title: "No".to_string(),
                    price: m.no_bid.unwrap_or(0) as f64 / 100.0,
                    probability: yes_prob.map(|p| 1.0 - p),
                },
            ],
        }
    }
}

impl From<KalshiOrderbook> for Orderbook {
    fn from(ob: KalshiOrderbook) -> Self {
        Self {
            market_id: String::new(),
            bids: ob.yes.iter().map(|e| OrderbookLevel {
                price: e.price as f64 / 100.0,
                quantity: e.quantity as u32,
            }).collect(),
            asks: ob.no.iter().map(|e| OrderbookLevel {
                price: e.price as f64 / 100.0,
                quantity: e.quantity as u32,
            }).collect(),
        }
    }
}
