use crate::backend::mock::{
    entity::{mock_request::CreateMockRequest, mock_response::CreateMockResponse, scenario::CreateScenario},
    MockBackend,
};

pub async fn happy_path() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("plaid_happy_path", "plaid")).await?;
    add_fixtures(&backend, "plaid_happy_path").await?;
    backend.activate_scenario("plaid_happy_path").await?;
    Ok(backend)
}

pub async fn add_fixtures(backend: &MockBackend, scenario_name: &str) -> crate::Result<()> {
    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/link/token/create"),
        CreateMockResponse::ok(LINK_TOKEN_CREATE_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/item/public_token/exchange"),
        CreateMockResponse::ok(PUBLIC_TOKEN_EXCHANGE_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/accounts/get"),
        CreateMockResponse::ok(ACCOUNTS_GET_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/transactions/get"),
        CreateMockResponse::ok(TRANSACTIONS_GET_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/institutions/search"),
        CreateMockResponse::ok(INSTITUTIONS_SEARCH_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/institutions/get_by_id"),
        CreateMockResponse::ok(INSTITUTION_GET_BY_ID_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/identity/get"),
        CreateMockResponse::ok(IDENTITY_GET_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/item/get"),
        CreateMockResponse::ok(ITEM_GET_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/item/remove"),
        CreateMockResponse::ok(ITEM_REMOVE_RESPONSE),
    ).await?;

    Ok(())
}

pub const LINK_TOKEN_CREATE_RESPONSE: &str = r#"{
    "link_token": "link-sandbox-af1a0311-da53-4636-b754-dd15cc058176",
    "expiration": "2024-12-31T23:59:59Z",
    "request_id": "HNTDNrA8F1shFEW"
}"#;

pub const PUBLIC_TOKEN_EXCHANGE_RESPONSE: &str = r#"{
    "access_token": "access-sandbox-de3ce8ef-33f8-452c-a685-8671031fc0f6",
    "item_id": "M5eVJqLnv3tbzdngLDp9FL5OlDNxlNhlE55op",
    "request_id": "Aim3b"
}"#;

pub const ACCOUNTS_GET_RESPONSE: &str = r#"{
    "accounts": [
        {
            "account_id": "BxBXxLj1m4HMXBm9WZZmCWVbPjX16EHwv99vp",
            "balances": {
                "available": 100.00,
                "current": 110.00,
                "iso_currency_code": "USD",
                "limit": null,
                "unofficial_currency_code": null
            },
            "mask": "0000",
            "name": "Plaid Checking",
            "official_name": "Plaid Gold Standard 0% Interest Checking",
            "subtype": "checking",
            "type": "depository"
        },
        {
            "account_id": "dVzbVMLjrxTnLjX4G66XUp5GLklm4oiZy88yK",
            "balances": {
                "available": 200.00,
                "current": 210.00,
                "iso_currency_code": "USD",
                "limit": null,
                "unofficial_currency_code": null
            },
            "mask": "1111",
            "name": "Plaid Saving",
            "official_name": "Plaid Silver Standard 0.1% Interest Saving",
            "subtype": "savings",
            "type": "depository"
        }
    ],
    "item": {
        "available_products": ["balance", "identity"],
        "billed_products": ["transactions"],
        "consent_expiration_time": null,
        "error": null,
        "institution_id": "ins_3",
        "item_id": "M5eVJqLnv3tbzdngLDp9FL5OlDNxlNhlE55op",
        "update_type": "background",
        "webhook": "https://www.example.com"
    },
    "request_id": "bkVE1BHWMAZ9Rnr"
}"#;

pub const TRANSACTIONS_GET_RESPONSE: &str = r#"{
    "accounts": [],
    "transactions": [
        {
            "account_id": "BxBXxLj1m4HMXBm9WZZmCWVbPjX16EHwv99vp",
            "amount": 25.00,
            "iso_currency_code": "USD",
            "unofficial_currency_code": null,
            "category": ["Food and Drink", "Restaurants"],
            "category_id": "13005000",
            "check_number": null,
            "date": "2024-01-15",
            "datetime": null,
            "location": {
                "address": "123 Main St",
                "city": "San Francisco",
                "region": "CA",
                "postal_code": "94102",
                "country": "US",
                "lat": 37.7749,
                "lon": -122.4194,
                "store_number": null
            },
            "merchant_name": "Starbucks",
            "merchant_entity_id": "starbucks",
            "name": "Starbucks",
            "payment_channel": "in store",
            "pending": false,
            "pending_transaction_id": null,
            "transaction_id": "lPNjeW1nR6CDn5okmGQ6hEpMo4lLNoSrzqDje",
            "transaction_type": "place",
            "counterparties": []
        },
        {
            "account_id": "BxBXxLj1m4HMXBm9WZZmCWVbPjX16EHwv99vp",
            "amount": -500.00,
            "iso_currency_code": "USD",
            "unofficial_currency_code": null,
            "category": ["Transfer", "Deposit"],
            "category_id": "21001000",
            "check_number": null,
            "date": "2024-01-14",
            "datetime": null,
            "location": {},
            "merchant_name": null,
            "merchant_entity_id": null,
            "name": "Direct Deposit - ACME Corp",
            "payment_channel": "other",
            "pending": false,
            "pending_transaction_id": null,
            "transaction_id": "aP4NjeW1nR6CDn5okmGQ6hEpMo4lLNoSrzqXYz",
            "transaction_type": "special",
            "counterparties": []
        }
    ],
    "total_transactions": 2,
    "request_id": "45QSn"
}"#;

pub const INSTITUTIONS_SEARCH_RESPONSE: &str = r#"{
    "institutions": [
        {
            "country_codes": ["US"],
            "institution_id": "ins_3",
            "name": "Chase",
            "oauth": false,
            "products": ["transactions", "auth", "balance", "identity"],
            "routing_numbers": ["021000021", "022000046"],
            "url": "https://www.chase.com",
            "primary_color": "#117ACA",
            "logo": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
        },
        {
            "country_codes": ["US"],
            "institution_id": "ins_4",
            "name": "Wells Fargo",
            "oauth": false,
            "products": ["transactions", "auth", "balance"],
            "routing_numbers": ["121000248"],
            "url": "https://www.wellsfargo.com",
            "primary_color": "#D71E28",
            "logo": null
        }
    ],
    "request_id": "m8MDnv9okwxFNBV"
}"#;

pub const INSTITUTION_GET_BY_ID_RESPONSE: &str = r#"{
    "institution": {
        "country_codes": ["US"],
        "institution_id": "ins_3",
        "name": "Chase",
        "oauth": false,
        "products": ["transactions", "auth", "balance", "identity"],
        "routing_numbers": ["021000021", "022000046"],
        "url": "https://www.chase.com",
        "primary_color": "#117ACA",
        "logo": null
    },
    "request_id": "m8MDnv9okwxFNBV"
}"#;

pub const IDENTITY_GET_RESPONSE: &str = r#"{
    "accounts": [
        {
            "account_id": "BxBXxLj1m4HMXBm9WZZmCWVbPjX16EHwv99vp",
            "owners": [
                {
                    "addresses": [
                        {
                            "data": {
                                "city": "San Francisco",
                                "country": "US",
                                "postal_code": "94102",
                                "region": "CA",
                                "street": "123 Main St"
                            },
                            "primary": true
                        }
                    ],
                    "emails": [
                        {
                            "data": "john.doe@example.com",
                            "primary": true,
                            "type": "primary"
                        }
                    ],
                    "names": ["John Doe"],
                    "phone_numbers": [
                        {
                            "data": "+1 415-555-0123",
                            "primary": true,
                            "type": "mobile"
                        }
                    ]
                }
            ]
        }
    ],
    "request_id": "3nARps6TOYtbACO"
}"#;

pub const ITEM_GET_RESPONSE: &str = r#"{
    "item": {
        "available_products": ["balance", "identity"],
        "billed_products": ["transactions"],
        "consent_expiration_time": null,
        "error": null,
        "institution_id": "ins_3",
        "item_id": "M5eVJqLnv3tbzdngLDp9FL5OlDNxlNhlE55op",
        "update_type": "background",
        "webhook": "https://www.example.com"
    },
    "request_id": "m8MDnv9okwxFNBV"
}"#;

pub const ITEM_REMOVE_RESPONSE: &str = r#"{
    "request_id": "m8MDnv9okwxFNBV"
}"#;

pub async fn error_scenario() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("plaid_error", "plaid")).await?;

    backend.add_mock(
        "plaid_error",
        CreateMockRequest::post("/accounts/get"),
        CreateMockResponse::bad_request(r#"{
            "error_type": "ITEM_ERROR",
            "error_code": "ITEM_LOGIN_REQUIRED",
            "error_message": "the login details of this item have changed",
            "display_message": "The login details of this item have changed. Please update your credentials."
        }"#),
    ).await?;

    backend.activate_scenario("plaid_error").await?;
    Ok(backend)
}

pub async fn rate_limited_scenario() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("plaid_rate_limited", "plaid")).await?;

    backend.add_mock(
        "plaid_rate_limited",
        CreateMockRequest::post("/accounts/get"),
        CreateMockResponse::ok(r#"{
            "error_type": "RATE_LIMIT_EXCEEDED",
            "error_code": "RATE_LIMIT",
            "error_message": "Rate limit exceeded",
            "display_message": "Too many requests. Please try again later."
        }"#).with_status(429),
    ).await?;

    backend.activate_scenario("plaid_rate_limited").await?;
    Ok(backend)
}
