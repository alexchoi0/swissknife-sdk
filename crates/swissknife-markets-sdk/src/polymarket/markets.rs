use crate::{Error, Result, Market, MarketStatus, Outcome, Orderbook, OrderbookLevel};
use crate::polymarket::PolymarketClient;
use serde::Deserialize;

impl PolymarketClient {
    pub async fn list_markets(&self, params: Option<ListMarketsParams>) -> Result<Vec<PolymarketMarket>> {
        let mut request = self.client()
            .get(format!("{}/markets", self.gamma_url()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
            if let Some(active) = p.active {
                query.push(("active", active.to_string()));
            }
            if let Some(closed) = p.closed {
                query.push(("closed", closed.to_string()));
            }
            if let Some(tag) = p.tag {
                query.push(("tag", tag));
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

        let result: Vec<PolymarketMarket> = response.json().await?;
        Ok(result)
    }

    pub async fn get_market(&self, condition_id: &str) -> Result<PolymarketMarket> {
        let response = self.client()
            .get(format!("{}/markets/{}", self.gamma_url(), condition_id))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::MarketNotFound(condition_id.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: PolymarketMarket = response.json().await?;
        Ok(result)
    }

    pub async fn get_orderbook(&self, token_id: &str) -> Result<PolymarketOrderbook> {
        let response = self.client()
            .get(format!("{}/book", self.clob_url()))
            .query(&[("token_id", token_id)])
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

        let result: PolymarketOrderbook = response.json().await?;
        Ok(result)
    }

    pub async fn get_price(&self, token_id: &str) -> Result<PriceInfo> {
        let response = self.client()
            .get(format!("{}/price", self.clob_url()))
            .query(&[("token_id", token_id)])
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

        let result: PriceInfo = response.json().await?;
        Ok(result)
    }

    pub async fn get_midpoint(&self, token_id: &str) -> Result<f64> {
        let response = self.client()
            .get(format!("{}/midpoint", self.clob_url()))
            .query(&[("token_id", token_id)])
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

        let result: MidpointResponse = response.json().await?;
        Ok(result.mid)
    }

    pub async fn get_spread(&self, token_id: &str) -> Result<SpreadInfo> {
        let response = self.client()
            .get(format!("{}/spread", self.clob_url()))
            .query(&[("token_id", token_id)])
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

        let result: SpreadInfo = response.json().await?;
        Ok(result)
    }

    pub async fn get_trades_history(&self, params: Option<TradesParams>) -> Result<Vec<PolymarketTrade>> {
        let mut request = self.client()
            .get(format!("{}/trades", self.clob_url()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(maker) = p.maker {
                query.push(("maker", maker));
            }
            if let Some(market) = p.market {
                query.push(("market", market));
            }
            if let Some(asset_id) = p.asset_id {
                query.push(("asset_id", asset_id));
            }
            if let Some(before) = p.before {
                query.push(("before", before.to_string()));
            }
            if let Some(after) = p.after {
                query.push(("after", after.to_string()));
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

        let result: Vec<PolymarketTrade> = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListMarketsParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TradesParams {
    pub maker: Option<String>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub before: Option<i64>,
    pub after: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolymarketMarket {
    pub id: Option<String>,
    pub condition_id: String,
    pub question_id: Option<String>,
    pub question: String,
    pub description: Option<String>,
    pub end_date_iso: Option<String>,
    pub game_start_time: Option<String>,
    pub volume: Option<String>,
    pub liquidity: Option<String>,
    pub active: bool,
    pub closed: bool,
    pub archived: Option<bool>,
    pub accepting_orders: Option<bool>,
    pub tokens: Option<Vec<MarketToken>>,
    pub tags: Option<Vec<Tag>>,
    pub outcomes: Option<Vec<String>>,
    pub outcome_prices: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketToken {
    pub token_id: String,
    pub outcome: String,
    pub price: Option<String>,
    pub winner: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    pub id: Option<String>,
    pub label: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolymarketOrderbook {
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub hash: Option<String>,
    pub bids: Vec<OrderbookEntry>,
    pub asks: Vec<OrderbookEntry>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookEntry {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceInfo {
    pub price: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MidpointResponse {
    pub mid: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpreadInfo {
    pub spread: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolymarketTrade {
    pub id: Option<String>,
    pub taker_order_id: Option<String>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub side: String,
    pub size: String,
    pub price: String,
    pub status: Option<String>,
    pub match_time: Option<String>,
    pub last_update: Option<String>,
    pub trade_owner: Option<String>,
    pub maker_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub bucket_index: Option<i32>,
    pub fee_rate_bps: Option<String>,
}

impl From<PolymarketMarket> for Market {
    fn from(m: PolymarketMarket) -> Self {
        let outcomes = if let Some(tokens) = &m.tokens {
            tokens.iter().map(|t| {
                let price = t.price.as_ref()
                    .and_then(|p| p.parse::<f64>().ok())
                    .unwrap_or(0.0);
                Outcome {
                    id: t.token_id.clone(),
                    title: t.outcome.clone(),
                    price,
                    probability: Some(price),
                }
            }).collect()
        } else if let (Some(outcomes_list), Some(prices)) = (&m.outcomes, &m.outcome_prices) {
            outcomes_list.iter().zip(prices.iter()).enumerate().map(|(i, (outcome, price))| {
                let p = price.parse::<f64>().unwrap_or(0.0);
                Outcome {
                    id: format!("{}_{}", m.condition_id, i),
                    title: outcome.clone(),
                    price: p,
                    probability: Some(p),
                }
            }).collect()
        } else {
            Vec::new()
        };

        Self {
            id: m.condition_id.clone(),
            title: m.question,
            description: m.description,
            status: if m.closed {
                MarketStatus::Closed
            } else if m.active {
                MarketStatus::Open
            } else {
                MarketStatus::Pending
            },
            category: m.tags.and_then(|t| t.first().and_then(|tag| tag.label.clone())),
            end_date: m.end_date_iso,
            created_at: None,
            volume: m.volume.and_then(|v| v.parse().ok()),
            liquidity: m.liquidity.and_then(|l| l.parse().ok()),
            outcomes,
        }
    }
}

impl From<PolymarketOrderbook> for Orderbook {
    fn from(ob: PolymarketOrderbook) -> Self {
        Self {
            market_id: ob.asset_id.unwrap_or_default(),
            bids: ob.bids.iter().map(|e| OrderbookLevel {
                price: e.price.parse().unwrap_or(0.0),
                quantity: e.size.parse().unwrap_or(0) as u32,
            }).collect(),
            asks: ob.asks.iter().map(|e| OrderbookLevel {
                price: e.price.parse().unwrap_or(0.0),
                quantity: e.size.parse().unwrap_or(0) as u32,
            }).collect(),
        }
    }
}
