use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "markets")]
use swissknife_markets_sdk as markets;

#[derive(Clone)]
pub struct MarketsTools {
    #[cfg(feature = "kalshi")]
    pub kalshi: Option<markets::kalshi::KalshiClient>,
    #[cfg(feature = "polymarket")]
    pub polymarket: Option<markets::polymarket::PolymarketClient>,
}

impl MarketsTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "kalshi")]
            kalshi: None,
            #[cfg(feature = "polymarket")]
            polymarket: None,
        }
    }

    #[cfg(feature = "kalshi")]
    pub fn with_kalshi(mut self, client: markets::kalshi::KalshiClient) -> Self {
        self.kalshi = Some(client);
        self
    }

    #[cfg(feature = "polymarket")]
    pub fn with_polymarket(mut self, client: markets::polymarket::PolymarketClient) -> Self {
        self.polymarket = Some(client);
        self
    }
}

impl Default for MarketsTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KalshiGetMarketsRequest {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KalshiGetMarketRequest {
    pub ticker: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KalshiGetOrderbookRequest {
    pub ticker: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KalshiCreateOrderRequest {
    pub ticker: String,
    pub side: String,
    pub action: String,
    pub count: u32,
    #[serde(default)]
    pub price: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KalshiGetPositionsRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PolymarketGetMarketsRequest {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PolymarketGetMarketRequest {
    pub condition_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PolymarketSearchMarketsRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[tool_box]
impl MarketsTools {
    #[cfg(feature = "kalshi")]
    #[rmcp::tool(description = "List prediction markets from Kalshi")]
    pub async fn kalshi_get_markets(
        &self,
        #[rmcp::tool(aggr)] req: KalshiGetMarketsRequest,
    ) -> Result<String, String> {
        let client = self.kalshi.as_ref()
            .ok_or_else(|| "Kalshi client not configured".to_string())?;

        let markets = client.get_markets(
            req.limit,
            req.cursor.as_deref(),
            req.status.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&markets).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kalshi")]
    #[rmcp::tool(description = "Get a specific Kalshi market by ticker")]
    pub async fn kalshi_get_market(
        &self,
        #[rmcp::tool(aggr)] req: KalshiGetMarketRequest,
    ) -> Result<String, String> {
        let client = self.kalshi.as_ref()
            .ok_or_else(|| "Kalshi client not configured".to_string())?;

        let market = client.get_market(&req.ticker).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&market).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kalshi")]
    #[rmcp::tool(description = "Get the orderbook for a Kalshi market")]
    pub async fn kalshi_get_orderbook(
        &self,
        #[rmcp::tool(aggr)] req: KalshiGetOrderbookRequest,
    ) -> Result<String, String> {
        let client = self.kalshi.as_ref()
            .ok_or_else(|| "Kalshi client not configured".to_string())?;

        let orderbook = client.get_orderbook(&req.ticker).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&orderbook).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kalshi")]
    #[rmcp::tool(description = "Create an order on Kalshi")]
    pub async fn kalshi_create_order(
        &self,
        #[rmcp::tool(aggr)] req: KalshiCreateOrderRequest,
    ) -> Result<String, String> {
        let client = self.kalshi.as_ref()
            .ok_or_else(|| "Kalshi client not configured".to_string())?;

        let order = client.create_order(
            &req.ticker,
            &req.side,
            &req.action,
            req.count,
            req.price,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&order).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kalshi")]
    #[rmcp::tool(description = "Get your positions on Kalshi")]
    pub async fn kalshi_get_positions(
        &self,
        #[rmcp::tool(aggr)] _req: KalshiGetPositionsRequest,
    ) -> Result<String, String> {
        let client = self.kalshi.as_ref()
            .ok_or_else(|| "Kalshi client not configured".to_string())?;

        let positions = client.get_positions().await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&positions).map_err(|e| e.to_string())
    }

    #[cfg(feature = "polymarket")]
    #[rmcp::tool(description = "List prediction markets from Polymarket")]
    pub async fn polymarket_get_markets(
        &self,
        #[rmcp::tool(aggr)] req: PolymarketGetMarketsRequest,
    ) -> Result<String, String> {
        let client = self.polymarket.as_ref()
            .ok_or_else(|| "Polymarket client not configured".to_string())?;

        let markets = client.get_markets(req.limit, req.offset).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&markets).map_err(|e| e.to_string())
    }

    #[cfg(feature = "polymarket")]
    #[rmcp::tool(description = "Get a specific Polymarket market")]
    pub async fn polymarket_get_market(
        &self,
        #[rmcp::tool(aggr)] req: PolymarketGetMarketRequest,
    ) -> Result<String, String> {
        let client = self.polymarket.as_ref()
            .ok_or_else(|| "Polymarket client not configured".to_string())?;

        let market = client.get_market(&req.condition_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&market).map_err(|e| e.to_string())
    }

    #[cfg(feature = "polymarket")]
    #[rmcp::tool(description = "Search Polymarket markets")]
    pub async fn polymarket_search_markets(
        &self,
        #[rmcp::tool(aggr)] req: PolymarketSearchMarketsRequest,
    ) -> Result<String, String> {
        let client = self.polymarket.as_ref()
            .ok_or_else(|| "Polymarket client not configured".to_string())?;

        let markets = client.search_markets(&req.query, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&markets).map_err(|e| e.to_string())
    }
}
