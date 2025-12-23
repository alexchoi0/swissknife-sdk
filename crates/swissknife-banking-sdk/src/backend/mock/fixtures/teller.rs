use crate::backend::mock::{
    entity::{mock_request::CreateMockRequest, mock_response::CreateMockResponse, scenario::CreateScenario},
    MockBackend,
};

pub async fn happy_path() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("teller_happy_path", "teller")).await?;
    add_fixtures(&backend, "teller_happy_path").await?;
    backend.activate_scenario("teller_happy_path").await?;
    Ok(backend)
}

pub async fn add_fixtures(backend: &MockBackend, scenario_name: &str) -> crate::Result<()> {
    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/accounts"),
        CreateMockResponse::ok(ACCOUNTS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/accounts/{id}"),
        CreateMockResponse::ok(ACCOUNT_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/accounts/{id}/balances"),
        CreateMockResponse::ok(BALANCES_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/accounts/{id}/details"),
        CreateMockResponse::ok(DETAILS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/accounts/{id}/transactions"),
        CreateMockResponse::ok(TRANSACTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/accounts/{id}/transactions/{transaction_id}"),
        CreateMockResponse::ok(TRANSACTION_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/accounts/{id}/identity"),
        CreateMockResponse::ok(IDENTITY_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::delete("/accounts/{id}"),
        CreateMockResponse::no_content(),
    ).await?;

    Ok(())
}

pub const ACCOUNTS_RESPONSE: &str = r#"[
    {
        "id": "acc_12345",
        "name": "My Checking",
        "type": "depository",
        "subtype": "checking",
        "currency": "USD",
        "enrollment_id": "enr_12345",
        "institution": {
            "id": "chase",
            "name": "Chase"
        },
        "last_four": "1234",
        "status": "open",
        "links": {
            "balances": "https://api.teller.io/accounts/acc_12345/balances",
            "transactions": "https://api.teller.io/accounts/acc_12345/transactions",
            "details": "https://api.teller.io/accounts/acc_12345/details",
            "self": "https://api.teller.io/accounts/acc_12345"
        }
    }
]"#;

pub const ACCOUNT_RESPONSE: &str = r#"{
    "id": "acc_12345",
    "name": "My Checking",
    "type": "depository",
    "subtype": "checking",
    "currency": "USD",
    "enrollment_id": "enr_12345",
    "institution": {
        "id": "chase",
        "name": "Chase"
    },
    "last_four": "1234",
    "status": "open",
    "links": {
        "balances": "https://api.teller.io/accounts/acc_12345/balances",
        "transactions": "https://api.teller.io/accounts/acc_12345/transactions",
        "details": "https://api.teller.io/accounts/acc_12345/details",
        "self": "https://api.teller.io/accounts/acc_12345"
    }
}"#;

pub const BALANCES_RESPONSE: &str = r#"{
    "account_id": "acc_12345",
    "available": "1500.00",
    "ledger": "1600.00",
    "links": {
        "account": "https://api.teller.io/accounts/acc_12345",
        "self": "https://api.teller.io/accounts/acc_12345/balances"
    }
}"#;

pub const DETAILS_RESPONSE: &str = r#"{
    "account_id": "acc_12345",
    "account_number": "123456789",
    "routing_numbers": {
        "ach": "021000021",
        "wire": "021000021"
    },
    "links": {
        "account": "https://api.teller.io/accounts/acc_12345",
        "self": "https://api.teller.io/accounts/acc_12345/details"
    }
}"#;

pub const TRANSACTIONS_RESPONSE: &str = r#"[
    {
        "id": "txn_001",
        "account_id": "acc_12345",
        "date": "2024-01-15",
        "description": "Starbucks",
        "details": {
            "processing_status": "complete",
            "category": "food",
            "counterparty": {
                "name": "Starbucks",
                "type": "merchant"
            }
        },
        "status": "posted",
        "amount": "-5.75",
        "running_balance": "1594.25",
        "type": "card_payment",
        "links": {
            "account": "https://api.teller.io/accounts/acc_12345",
            "self": "https://api.teller.io/accounts/acc_12345/transactions/txn_001"
        }
    },
    {
        "id": "txn_002",
        "account_id": "acc_12345",
        "date": "2024-01-14",
        "description": "Direct Deposit",
        "details": {
            "processing_status": "complete",
            "category": "income",
            "counterparty": {
                "name": "ACME Corp",
                "type": "organization"
            }
        },
        "status": "posted",
        "amount": "3000.00",
        "running_balance": "1600.00",
        "type": "ach",
        "links": {
            "account": "https://api.teller.io/accounts/acc_12345",
            "self": "https://api.teller.io/accounts/acc_12345/transactions/txn_002"
        }
    }
]"#;

pub const TRANSACTION_RESPONSE: &str = r#"{
    "id": "txn_001",
    "account_id": "acc_12345",
    "date": "2024-01-15",
    "description": "Starbucks",
    "details": {
        "processing_status": "complete",
        "category": "food",
        "counterparty": {
            "name": "Starbucks",
            "type": "merchant"
        }
    },
    "status": "posted",
    "amount": "-5.75",
    "running_balance": "1594.25",
    "type": "card_payment",
    "links": {
        "account": "https://api.teller.io/accounts/acc_12345",
        "self": "https://api.teller.io/accounts/acc_12345/transactions/txn_001"
    }
}"#;

pub const IDENTITY_RESPONSE: &str = r#"{
    "emails": [
        {"data": "john.doe@example.com", "type": "primary"}
    ],
    "names": [
        {"data": "John Doe"}
    ],
    "phone_numbers": [
        {"data": "+14155550123", "type": "mobile"}
    ],
    "addresses": [
        {
            "data": {
                "street": "123 Main St",
                "city": "San Francisco",
                "state": "CA",
                "postal_code": "94102",
                "country": "US"
            },
            "type": "primary"
        }
    ]
}"#;
