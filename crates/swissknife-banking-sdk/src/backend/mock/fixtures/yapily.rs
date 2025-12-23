use crate::backend::mock::{
    entity::{mock_request::CreateMockRequest, mock_response::CreateMockResponse, scenario::CreateScenario},
    MockBackend,
};

pub async fn happy_path() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("yapily_happy_path", "yapily")).await?;
    add_fixtures(&backend, "yapily_happy_path").await?;
    backend.activate_scenario("yapily_happy_path").await?;
    Ok(backend)
}

pub async fn add_fixtures(backend: &MockBackend, scenario_name: &str) -> crate::Result<()> {
    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/institutions"),
        CreateMockResponse::ok(INSTITUTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/institutions/{id}"),
        CreateMockResponse::ok(INSTITUTION_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/account-auth-requests"),
        CreateMockResponse::created(AUTH_REQUEST_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/consents"),
        CreateMockResponse::created(CONSENT_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/consents/{id}"),
        CreateMockResponse::ok(CONSENT_GET_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::delete("/consents/{id}"),
        CreateMockResponse::no_content(),
    ).await?;

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
        CreateMockRequest::get("/accounts/{id}/transactions"),
        CreateMockResponse::ok(TRANSACTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/identity"),
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

pub const INSTITUTIONS_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": [
        {
            "id": "modelo-sandbox",
            "name": "Modelo Sandbox",
            "fullName": "Modelo Sandbox Bank",
            "countries": [
                {"displayName": "United Kingdom", "countryCode2": "GB"}
            ],
            "environmentType": "SANDBOX",
            "credentialsType": "OPEN_BANKING_UK_AUTO",
            "media": [
                {"source": "https://images.yapily.com/image/modelo-sandbox/icon", "type": "icon"}
            ],
            "features": ["ACCOUNT_STATEMENT", "ACCOUNTS", "IDENTITY", "TRANSACTIONS"]
        },
        {
            "id": "natwest-sandbox",
            "name": "NatWest Sandbox",
            "fullName": "NatWest Sandbox Bank",
            "countries": [
                {"displayName": "United Kingdom", "countryCode2": "GB"}
            ],
            "environmentType": "SANDBOX",
            "credentialsType": "OPEN_BANKING_UK_AUTO",
            "media": [
                {"source": "https://images.yapily.com/image/natwest-sandbox/icon", "type": "icon"}
            ],
            "features": ["ACCOUNTS", "TRANSACTIONS", "PAYMENTS"]
        }
    ]
}"#;

pub const INSTITUTION_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "modelo-sandbox",
        "name": "Modelo Sandbox",
        "fullName": "Modelo Sandbox Bank",
        "countries": [
            {"displayName": "United Kingdom", "countryCode2": "GB"}
        ],
        "environmentType": "SANDBOX",
        "credentialsType": "OPEN_BANKING_UK_AUTO",
        "media": [
            {"source": "https://images.yapily.com/image/modelo-sandbox/icon", "type": "icon"}
        ],
        "features": ["ACCOUNT_STATEMENT", "ACCOUNTS", "IDENTITY", "TRANSACTIONS"]
    }
}"#;

pub const AUTH_REQUEST_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "auth_12345",
        "userUuid": "user_12345",
        "applicationUserId": "app_user_123",
        "institutionId": "modelo-sandbox",
        "status": "AWAITING_AUTHORIZATION",
        "createdAt": "2024-01-15T10:00:00Z",
        "featureScope": ["ACCOUNTS", "TRANSACTIONS"],
        "authorisationUrl": "https://ob.modelo.yapily.com/authorize?request=auth_12345",
        "qrCodeUrl": "https://ob.modelo.yapily.com/qr/auth_12345"
    }
}"#;

pub const CONSENT_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "consent_12345",
        "userUuid": "user_12345",
        "applicationUserId": "app_user_123",
        "institutionId": "modelo-sandbox",
        "status": "AUTHORIZED",
        "createdAt": "2024-01-15T10:00:00Z",
        "expiresAt": "2024-04-15T10:00:00Z",
        "featureScope": ["ACCOUNTS", "TRANSACTIONS", "IDENTITY"],
        "consentToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.consent_token"
    }
}"#;

pub const CONSENT_GET_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "consent_12345",
        "userUuid": "user_12345",
        "applicationUserId": "app_user_123",
        "institutionId": "modelo-sandbox",
        "status": "AUTHORIZED",
        "createdAt": "2024-01-15T10:00:00Z",
        "expiresAt": "2024-04-15T10:00:00Z",
        "featureScope": ["ACCOUNTS", "TRANSACTIONS", "IDENTITY"],
        "consentToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.consent_token"
    }
}"#;

pub const ACCOUNTS_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": [
        {
            "id": "acc_12345",
            "type": "PERSONAL",
            "description": "Current Account",
            "balance": 2500.00,
            "currency": "GBP",
            "usageType": "PERSONAL",
            "accountType": "CURRENT",
            "nickname": "My Current Account",
            "accountNames": [
                {"name": "John Smith"}
            ],
            "accountIdentifications": [
                {"type": "SORT_CODE", "identification": "040004"},
                {"type": "ACCOUNT_NUMBER", "identification": "12345678"},
                {"type": "IBAN", "identification": "GB29NWBK60161331926819"}
            ],
            "accountBalances": [
                {
                    "type": "CLOSING_AVAILABLE",
                    "dateTime": "2024-01-15T10:00:00Z",
                    "balanceAmount": {"amount": 2500.00, "currency": "GBP"},
                    "creditLineIncluded": false
                }
            ]
        }
    ]
}"#;

pub const ACCOUNT_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "acc_12345",
        "type": "PERSONAL",
        "description": "Current Account",
        "balance": 2500.00,
        "currency": "GBP",
        "usageType": "PERSONAL",
        "accountType": "CURRENT",
        "nickname": "My Current Account",
        "accountNames": [
            {"name": "John Smith"}
        ],
        "accountIdentifications": [
            {"type": "SORT_CODE", "identification": "040004"},
            {"type": "ACCOUNT_NUMBER", "identification": "12345678"},
            {"type": "IBAN", "identification": "GB29NWBK60161331926819"}
        ]
    }
}"#;

pub const BALANCES_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "accountBalances": [
            {
                "type": "CLOSING_AVAILABLE",
                "dateTime": "2024-01-15T10:00:00Z",
                "balanceAmount": {"amount": 2500.00, "currency": "GBP"},
                "creditLineIncluded": false
            },
            {
                "type": "INTERIM_BOOKED",
                "dateTime": "2024-01-15T10:00:00Z",
                "balanceAmount": {"amount": 2450.00, "currency": "GBP"},
                "creditLineIncluded": false
            }
        ]
    }
}"#;

pub const TRANSACTIONS_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": [
        {
            "id": "txn_001",
            "date": "2024-01-15",
            "bookingDateTime": "2024-01-15T14:30:00Z",
            "valueDateTime": "2024-01-15T14:30:00Z",
            "status": "BOOKED",
            "amount": -75.50,
            "currency": "GBP",
            "transactionAmount": {"amount": -75.50, "currency": "GBP"},
            "reference": "Payment to Sainsbury's",
            "description": "Sainsbury's Supermarkets",
            "transactionInformation": ["Sainsbury's Supermarkets Ltd"],
            "proprietaryBankTransactionCode": {"code": "CARD", "issuer": "MODELO"}
        },
        {
            "id": "txn_002",
            "date": "2024-01-14",
            "bookingDateTime": "2024-01-14T09:00:00Z",
            "valueDateTime": "2024-01-14T09:00:00Z",
            "status": "BOOKED",
            "amount": 3500.00,
            "currency": "GBP",
            "transactionAmount": {"amount": 3500.00, "currency": "GBP"},
            "reference": "Salary",
            "description": "ACME Ltd Salary",
            "transactionInformation": ["ACME Ltd Monthly Salary"],
            "proprietaryBankTransactionCode": {"code": "TRANSFER", "issuer": "MODELO"}
        }
    ]
}"#;

pub const IDENTITY_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "identity_12345",
        "fullName": "John Smith",
        "firstName": "John",
        "lastName": "Smith",
        "dateOfBirth": "1990-05-15",
        "addresses": [
            {
                "addressLines": ["123 High Street"],
                "city": "London",
                "postCode": "SW1A 1AA",
                "country": "GB",
                "addressType": "HOME"
            }
        ],
        "emails": ["john.smith@example.com"],
        "phones": ["+44 7700 900123"]
    }
}"#;

pub const PAYMENT_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "payment_12345",
        "institutionId": "modelo-sandbox",
        "paymentIdempotencyId": "idempotency_12345",
        "paymentLifecycleId": "lifecycle_12345",
        "status": "PENDING",
        "statusDetails": {
            "status": "PENDING",
            "statusReason": "AWAITING_AUTHORIZATION"
        },
        "amount": {"amount": 100.00, "currency": "GBP"},
        "reference": "Test Payment",
        "createdAt": "2024-01-15T10:00:00Z"
    }
}"#;

pub const PAYMENT_GET_RESPONSE: &str = r#"{
    "meta": {
        "tracingId": "trace_12345"
    },
    "data": {
        "id": "payment_12345",
        "institutionId": "modelo-sandbox",
        "paymentIdempotencyId": "idempotency_12345",
        "paymentLifecycleId": "lifecycle_12345",
        "status": "COMPLETED",
        "statusDetails": {
            "status": "COMPLETED",
            "statusReason": "PAYMENT_ACCEPTED"
        },
        "amount": {"amount": 100.00, "currency": "GBP"},
        "reference": "Test Payment",
        "createdAt": "2024-01-15T10:00:00Z",
        "updatedAt": "2024-01-15T10:05:00Z"
    }
}"#;
