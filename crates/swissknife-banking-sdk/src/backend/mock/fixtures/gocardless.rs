use crate::backend::mock::{
    entity::{mock_request::CreateMockRequest, mock_response::CreateMockResponse, scenario::CreateScenario},
    MockBackend,
};

pub async fn happy_path() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("gocardless_happy_path", "gocardless")).await?;
    add_fixtures(&backend, "gocardless_happy_path").await?;
    backend.activate_scenario("gocardless_happy_path").await?;
    Ok(backend)
}

pub async fn add_fixtures(backend: &MockBackend, scenario_name: &str) -> crate::Result<()> {
    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/api/v2/token/new/"),
        CreateMockResponse::ok(TOKEN_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/api/v2/requisitions/"),
        CreateMockResponse::created(REQUISITION_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/api/v2/requisitions/{id}/"),
        CreateMockResponse::ok(REQUISITION_GET_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/api/v2/accounts/{id}/"),
        CreateMockResponse::ok(ACCOUNT_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/api/v2/accounts/{id}/details/"),
        CreateMockResponse::ok(ACCOUNT_DETAILS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/api/v2/accounts/{id}/balances/"),
        CreateMockResponse::ok(BALANCES_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/api/v2/accounts/{id}/transactions/"),
        CreateMockResponse::ok(TRANSACTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/api/v2/institutions/"),
        CreateMockResponse::ok(INSTITUTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/api/v2/institutions/{id}/"),
        CreateMockResponse::ok(INSTITUTION_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::delete("/api/v2/requisitions/{id}/"),
        CreateMockResponse::no_content(),
    ).await?;

    Ok(())
}

pub const TOKEN_RESPONSE: &str = r#"{
    "access": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test_access",
    "access_expires": 86400,
    "refresh": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test_refresh",
    "refresh_expires": 2592000
}"#;

pub const REQUISITION_RESPONSE: &str = r#"{
    "id": "req_12345",
    "created": "2024-01-15T10:00:00Z",
    "redirect": "https://example.com/callback",
    "status": "LN",
    "institution_id": "SANDBOXFINANCE_SFIN0000",
    "agreement": null,
    "reference": "user_123",
    "accounts": [],
    "link": "https://ob.gocardless.com/psd2/start/req_12345"
}"#;

pub const REQUISITION_GET_RESPONSE: &str = r#"{
    "id": "req_12345",
    "created": "2024-01-15T10:00:00Z",
    "redirect": "https://example.com/callback",
    "status": "LN",
    "institution_id": "SANDBOXFINANCE_SFIN0000",
    "agreement": null,
    "reference": "user_123",
    "accounts": ["acc_12345", "acc_67890"],
    "link": "https://ob.gocardless.com/psd2/start/req_12345"
}"#;

pub const ACCOUNT_RESPONSE: &str = r#"{
    "id": "acc_12345",
    "created": "2024-01-15T10:30:00Z",
    "last_accessed": "2024-01-15T12:00:00Z",
    "iban": "DE89370400440532013000",
    "institution_id": "SANDBOXFINANCE_SFIN0000",
    "status": "READY",
    "owner_name": "Max Mustermann"
}"#;

pub const ACCOUNT_DETAILS_RESPONSE: &str = r#"{
    "account": {
        "resource_id": "res_12345",
        "iban": "DE89370400440532013000",
        "bban": "370400440532013000",
        "currency": "EUR",
        "owner_name": "Max Mustermann",
        "name": "Main Account",
        "product": "Current Account",
        "cash_account_type": "CACC"
    }
}"#;

pub const BALANCES_RESPONSE: &str = r#"{
    "balances": [
        {
            "balance_amount": {
                "amount": "2500.00",
                "currency": "EUR"
            },
            "balance_type": "closingBooked",
            "reference_date": "2024-01-15"
        },
        {
            "balance_amount": {
                "amount": "2450.00",
                "currency": "EUR"
            },
            "balance_type": "interimAvailable",
            "reference_date": "2024-01-15"
        }
    ]
}"#;

pub const TRANSACTIONS_RESPONSE: &str = r#"{
    "transactions": {
        "booked": [
            {
                "transaction_id": "txn_001",
                "booking_date": "2024-01-15",
                "value_date": "2024-01-15",
                "transaction_amount": {
                    "amount": "-50.00",
                    "currency": "EUR"
                },
                "remittance_information_unstructured": "Amazon.de",
                "creditor_name": "Amazon EU S.a.r.l.",
                "bank_transaction_code": "PMNT-ICDT-STDO"
            },
            {
                "transaction_id": "txn_002",
                "booking_date": "2024-01-14",
                "value_date": "2024-01-14",
                "transaction_amount": {
                    "amount": "3000.00",
                    "currency": "EUR"
                },
                "remittance_information_unstructured": "Salary January",
                "debtor_name": "ACME GmbH",
                "bank_transaction_code": "PMNT-RCDT-SALA"
            }
        ],
        "pending": [
            {
                "transaction_amount": {
                    "amount": "-25.00",
                    "currency": "EUR"
                },
                "remittance_information_unstructured": "Pending payment"
            }
        ]
    }
}"#;

pub const INSTITUTIONS_RESPONSE: &str = r#"[
    {
        "id": "SANDBOXFINANCE_SFIN0000",
        "name": "Sandbox Finance",
        "bic": "SFIN0000",
        "transaction_total_days": "90",
        "countries": ["DE", "AT"],
        "logo": "https://cdn.gocardless.com/logos/sandbox.png"
    },
    {
        "id": "DEUTSCHE_BANK_DEUTDEFF",
        "name": "Deutsche Bank",
        "bic": "DEUTDEFF",
        "transaction_total_days": "90",
        "countries": ["DE"],
        "logo": "https://cdn.gocardless.com/logos/deutsche_bank.png"
    }
]"#;

pub const INSTITUTION_RESPONSE: &str = r#"{
    "id": "SANDBOXFINANCE_SFIN0000",
    "name": "Sandbox Finance",
    "bic": "SFIN0000",
    "transaction_total_days": "90",
    "countries": ["DE", "AT"],
    "logo": "https://cdn.gocardless.com/logos/sandbox.png"
}"#;
