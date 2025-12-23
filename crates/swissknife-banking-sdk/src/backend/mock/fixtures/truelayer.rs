use crate::backend::mock::{
    entity::{mock_request::CreateMockRequest, mock_response::CreateMockResponse, scenario::CreateScenario},
    MockBackend,
};

pub async fn happy_path() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("truelayer_happy_path", "truelayer")).await?;
    add_fixtures(&backend, "truelayer_happy_path").await?;
    backend.activate_scenario("truelayer_happy_path").await?;
    Ok(backend)
}

pub async fn add_fixtures(backend: &MockBackend, scenario_name: &str) -> crate::Result<()> {
    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/connect/token"),
        CreateMockResponse::ok(TOKEN_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/data/v1/accounts"),
        CreateMockResponse::ok(ACCOUNTS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/data/v1/accounts/{id}/balance"),
        CreateMockResponse::ok(BALANCE_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/data/v1/accounts/{id}/transactions"),
        CreateMockResponse::ok(TRANSACTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/data/v1/info"),
        CreateMockResponse::ok(IDENTITY_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/payments"),
        CreateMockResponse::created(PAYMENT_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/payments/{id}"),
        CreateMockResponse::ok(PAYMENT_GET_RESPONSE),
    ).await?;

    Ok(())
}

pub const TOKEN_RESPONSE: &str = r#"{
    "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.test",
    "token_type": "Bearer",
    "expires_in": 3600
}"#;

pub const ACCOUNTS_RESPONSE: &str = r#"{
    "results": [
        {
            "account_id": "acc_12345",
            "account_type": "TRANSACTION",
            "display_name": "Current Account",
            "currency": "GBP",
            "account_number": {
                "iban": "GB29NWBK60161331926819",
                "swift_bic": "NWBKGB2L",
                "number": "31926819",
                "sort_code": "601613"
            },
            "provider": {
                "provider_id": "ob-monzo",
                "display_name": "Monzo",
                "logo_uri": "https://truelayer.com/logos/monzo.png"
            }
        }
    ],
    "status": "Succeeded"
}"#;

pub const BALANCE_RESPONSE: &str = r#"{
    "results": [
        {
            "currency": "GBP",
            "available": 1250.50,
            "current": 1300.00,
            "overdraft": 500.00,
            "update_timestamp": "2024-01-15T10:30:00Z"
        }
    ],
    "status": "Succeeded"
}"#;

pub const TRANSACTIONS_RESPONSE: &str = r#"{
    "results": [
        {
            "transaction_id": "txn_001",
            "timestamp": "2024-01-15T14:30:00Z",
            "description": "Tesco Stores",
            "amount": -45.67,
            "currency": "GBP",
            "transaction_type": "DEBIT",
            "transaction_category": "PURCHASE",
            "merchant_name": "Tesco",
            "running_balance": {
                "currency": "GBP",
                "amount": 1254.33
            }
        },
        {
            "transaction_id": "txn_002",
            "timestamp": "2024-01-14T09:00:00Z",
            "description": "Salary",
            "amount": 3500.00,
            "currency": "GBP",
            "transaction_type": "CREDIT",
            "transaction_category": "INCOME",
            "merchant_name": null
        }
    ],
    "status": "Succeeded"
}"#;

pub const IDENTITY_RESPONSE: &str = r#"{
    "results": [
        {
            "full_name": "John Smith",
            "emails": ["john.smith@example.com"],
            "phones": ["+44 7700 900123"],
            "addresses": [
                {
                    "address": "123 High Street",
                    "city": "London",
                    "state": null,
                    "zip": "SW1A 1AA",
                    "country": "GB"
                }
            ],
            "date_of_birth": "1990-05-15"
        }
    ],
    "status": "Succeeded"
}"#;

pub const PAYMENT_RESPONSE: &str = r#"{
    "id": "pay_12345",
    "resource_token": "rt_12345",
    "status": "authorization_required"
}"#;

pub const PAYMENT_GET_RESPONSE: &str = r#"{
    "id": "pay_12345",
    "status": "executed",
    "amount_in_minor": 10000,
    "currency": "GBP",
    "payment_method": {
        "type": "bank_transfer"
    },
    "created_at": "2024-01-15T10:00:00Z"
}"#;
