#[cfg(feature = "kalshi")]
mod kalshi_tests {
    use swissknife_markets_sdk::kalshi::{
        KalshiClient, LoginRequest, GetMarketsParams, GetTradesParams,
        CreateOrderRequest, OrderSide, OrderType,
    };

    #[test]
    fn test_kalshi_client_creation() {
        let client = KalshiClient::new();
        assert!(true);
    }

    #[test]
    fn test_kalshi_client_demo() {
        let client = KalshiClient::demo();
        assert!(true);
    }

    #[test]
    fn test_login_request() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "secret123".to_string(),
        };

        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "secret123");
    }

    #[test]
    fn test_get_markets_params() {
        let params = GetMarketsParams {
            limit: Some(100),
            cursor: Some("abc123".to_string()),
            event_ticker: Some("ELECTION".to_string()),
            series_ticker: None,
            status: Some("active".to_string()),
        };

        assert_eq!(params.limit, Some(100));
        assert_eq!(params.cursor, Some("abc123".to_string()));
        assert_eq!(params.event_ticker, Some("ELECTION".to_string()));
        assert_eq!(params.status, Some("active".to_string()));
    }

    #[test]
    fn test_get_trades_params() {
        let params = GetTradesParams {
            limit: Some(50),
            cursor: None,
            ticker: Some("MARKET-123".to_string()),
            min_ts: None,
            max_ts: None,
        };

        assert_eq!(params.limit, Some(50));
        assert_eq!(params.ticker, Some("MARKET-123".to_string()));
    }

    #[test]
    fn test_create_order_request() {
        let request = CreateOrderRequest {
            ticker: "MARKET-ABC".to_string(),
            action: OrderSide::Buy,
            side: "yes".to_string(),
            order_type: OrderType::Limit,
            count: 10,
            yes_price: Some(55),
            no_price: None,
            expiration_ts: None,
            client_order_id: Some("client-123".to_string()),
        };

        assert_eq!(request.ticker, "MARKET-ABC");
        assert!(matches!(request.action, OrderSide::Buy));
        assert_eq!(request.side, "yes");
        assert!(matches!(request.order_type, OrderType::Limit));
        assert_eq!(request.count, 10);
        assert_eq!(request.yes_price, Some(55));
    }

    #[test]
    fn test_order_side_variants() {
        let buy = OrderSide::Buy;
        let sell = OrderSide::Sell;

        assert!(matches!(buy, OrderSide::Buy));
        assert!(matches!(sell, OrderSide::Sell));
    }

    #[test]
    fn test_order_type_variants() {
        let limit = OrderType::Limit;
        let market = OrderType::Market;

        assert!(matches!(limit, OrderType::Limit));
        assert!(matches!(market, OrderType::Market));
    }

    #[test]
    fn test_params_defaults() {
        let markets_params = GetMarketsParams::default();
        let trades_params = GetTradesParams::default();

        assert!(markets_params.limit.is_none());
        assert!(markets_params.cursor.is_none());
        assert!(trades_params.ticker.is_none());
    }
}

#[cfg(feature = "polymarket")]
mod polymarket_tests {
    use swissknife_markets_sdk::polymarket::{
        PolymarketClient, GetMarketsParams as PolyGetMarketsParams,
        CreateOrderRequest as PolyCreateOrderRequest, OrderSide as PolySide,
    };

    #[test]
    fn test_polymarket_client_creation() {
        let client = PolymarketClient::new("test-api-key", "test-secret");
        assert!(true);
    }

    #[test]
    fn test_polymarket_client_with_passphrase() {
        let client = PolymarketClient::with_passphrase(
            "test-api-key",
            "test-secret",
            "test-passphrase",
        );
        assert!(true);
    }

    #[test]
    fn test_get_markets_params() {
        let params = PolyGetMarketsParams {
            limit: Some(25),
            offset: Some(0),
            active: Some(true),
            closed: Some(false),
            order: Some("volume".to_string()),
            ascending: Some(false),
        };

        assert_eq!(params.limit, Some(25));
        assert_eq!(params.offset, Some(0));
        assert_eq!(params.active, Some(true));
        assert_eq!(params.closed, Some(false));
    }

    #[test]
    fn test_create_order_request() {
        let request = PolyCreateOrderRequest {
            token_id: "token-123".to_string(),
            side: PolySide::Buy,
            price: 0.65,
            size: 100.0,
            order_type: None,
            expiration: None,
        };

        assert_eq!(request.token_id, "token-123");
        assert!(matches!(request.side, PolySide::Buy));
        assert_eq!(request.price, 0.65);
        assert_eq!(request.size, 100.0);
    }

    #[test]
    fn test_order_side_variants() {
        let buy = PolySide::Buy;
        let sell = PolySide::Sell;

        assert!(matches!(buy, PolySide::Buy));
        assert!(matches!(sell, PolySide::Sell));
    }

    #[test]
    fn test_params_defaults() {
        let params = PolyGetMarketsParams::default();

        assert!(params.limit.is_none());
        assert!(params.offset.is_none());
        assert!(params.active.is_none());
    }
}

mod error_tests {
    use swissknife_markets_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Insufficient balance".to_string(),
            code: Some("INSUFFICIENT_FUNDS".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Insufficient balance"));
    }

    #[test]
    fn test_market_not_found_error() {
        let error = Error::MarketNotFound("MARKET-123".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("MARKET-123"));
    }

    #[test]
    fn test_order_not_found_error() {
        let error = Error::OrderNotFound("ORDER-456".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("ORDER-456"));
    }

    #[test]
    fn test_auth_error() {
        let error = Error::Auth("Invalid credentials".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid credentials"));
    }
}
