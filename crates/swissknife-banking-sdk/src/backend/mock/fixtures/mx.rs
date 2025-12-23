use crate::backend::mock::{
    entity::{mock_request::CreateMockRequest, mock_response::CreateMockResponse, scenario::CreateScenario},
    MockBackend,
};

pub async fn happy_path() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;
    backend.create_scenario(CreateScenario::new("mx_happy_path", "mx")).await?;
    add_fixtures(&backend, "mx_happy_path").await?;
    backend.activate_scenario("mx_happy_path").await?;
    Ok(backend)
}

pub async fn add_fixtures(backend: &MockBackend, scenario_name: &str) -> crate::Result<()> {
    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/users"),
        CreateMockResponse::ok(USER_CREATE_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}"),
        CreateMockResponse::ok(USER_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::delete("/users/{user_guid}"),
        CreateMockResponse::no_content(),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/users/{user_guid}/members"),
        CreateMockResponse::ok(MEMBER_CREATE_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/members"),
        CreateMockResponse::ok(MEMBERS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/members/{member_guid}"),
        CreateMockResponse::ok(MEMBER_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/members/{member_guid}/status"),
        CreateMockResponse::ok(MEMBER_STATUS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::delete("/users/{user_guid}/members/{member_guid}"),
        CreateMockResponse::no_content(),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/accounts"),
        CreateMockResponse::ok(ACCOUNTS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/accounts/{account_guid}"),
        CreateMockResponse::ok(ACCOUNT_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/accounts/{account_guid}/transactions"),
        CreateMockResponse::ok(TRANSACTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/transactions"),
        CreateMockResponse::ok(ALL_TRANSACTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/institutions"),
        CreateMockResponse::ok(INSTITUTIONS_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/institutions/{institution_code}"),
        CreateMockResponse::ok(INSTITUTION_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::post("/users/{user_guid}/connect_widget_url"),
        CreateMockResponse::ok(CONNECT_WIDGET_URL_RESPONSE),
    ).await?;

    backend.add_mock(
        scenario_name,
        CreateMockRequest::get("/users/{user_guid}/members/{member_guid}/account_owners"),
        CreateMockResponse::ok(ACCOUNT_OWNERS_RESPONSE),
    ).await?;

    Ok(())
}

pub const USER_CREATE_RESPONSE: &str = r#"{
    "user": {
        "guid": "USR-12345678-1234-1234-1234-123456789012",
        "id": "user_123",
        "is_disabled": false,
        "email": "john.doe@example.com",
        "metadata": "{\"first_name\": \"John\", \"last_name\": \"Doe\"}"
    }
}"#;

pub const USER_RESPONSE: &str = r#"{
    "user": {
        "guid": "USR-12345678-1234-1234-1234-123456789012",
        "id": "user_123",
        "is_disabled": false,
        "email": "john.doe@example.com",
        "metadata": "{\"first_name\": \"John\", \"last_name\": \"Doe\"}"
    }
}"#;

pub const MEMBER_CREATE_RESPONSE: &str = r#"{
    "member": {
        "aggregated_at": null,
        "connection_status": "CREATED",
        "guid": "MBR-12345678-1234-1234-1234-123456789012",
        "id": "member_123",
        "institution_code": "chase",
        "is_being_aggregated": false,
        "is_managed_by_user": true,
        "is_oauth": false,
        "metadata": null,
        "name": "Chase",
        "oauth_window_uri": null,
        "successfully_aggregated_at": null,
        "user_guid": "USR-12345678-1234-1234-1234-123456789012",
        "user_id": "user_123"
    }
}"#;

pub const MEMBERS_RESPONSE: &str = r#"{
    "members": [
        {
            "aggregated_at": "2024-01-15T10:00:00Z",
            "connection_status": "CONNECTED",
            "guid": "MBR-12345678-1234-1234-1234-123456789012",
            "id": "member_123",
            "institution_code": "chase",
            "is_being_aggregated": false,
            "is_managed_by_user": true,
            "is_oauth": false,
            "metadata": null,
            "name": "Chase",
            "oauth_window_uri": null,
            "successfully_aggregated_at": "2024-01-15T10:05:00Z",
            "user_guid": "USR-12345678-1234-1234-1234-123456789012",
            "user_id": "user_123"
        }
    ],
    "pagination": {
        "current_page": 1,
        "per_page": 25,
        "total_entries": 1,
        "total_pages": 1
    }
}"#;

pub const MEMBER_RESPONSE: &str = r#"{
    "member": {
        "aggregated_at": "2024-01-15T10:00:00Z",
        "connection_status": "CONNECTED",
        "guid": "MBR-12345678-1234-1234-1234-123456789012",
        "id": "member_123",
        "institution_code": "chase",
        "is_being_aggregated": false,
        "is_managed_by_user": true,
        "is_oauth": false,
        "metadata": null,
        "name": "Chase",
        "oauth_window_uri": null,
        "successfully_aggregated_at": "2024-01-15T10:05:00Z",
        "user_guid": "USR-12345678-1234-1234-1234-123456789012",
        "user_id": "user_123"
    }
}"#;

pub const MEMBER_STATUS_RESPONSE: &str = r#"{
    "member": {
        "aggregated_at": "2024-01-15T10:00:00Z",
        "challenges": [],
        "connection_status": "CONNECTED",
        "guid": "MBR-12345678-1234-1234-1234-123456789012",
        "has_processed_accounts": true,
        "has_processed_transactions": true,
        "is_authenticated": true,
        "is_being_aggregated": false,
        "successfully_aggregated_at": "2024-01-15T10:05:00Z"
    }
}"#;

pub const ACCOUNTS_RESPONSE: &str = r#"{
    "accounts": [
        {
            "account_number": "1234567890",
            "apr": null,
            "apy": 0.01,
            "available_balance": 1500.00,
            "available_credit": null,
            "balance": 1600.00,
            "cash_balance": null,
            "cash_surrender_value": null,
            "created_at": "2024-01-15T10:00:00Z",
            "credit_limit": null,
            "currency_code": "USD",
            "day_payment_is_due": null,
            "death_benefit": null,
            "guid": "ACT-12345678-1234-1234-1234-123456789012",
            "holdings_value": null,
            "id": "account_123",
            "imported_at": "2024-01-15T10:00:00Z",
            "institution_code": "chase",
            "insured_name": null,
            "interest_rate": null,
            "is_closed": false,
            "is_hidden": false,
            "last_payment": null,
            "last_payment_at": null,
            "loan_amount": null,
            "matures_on": null,
            "member_guid": "MBR-12345678-1234-1234-1234-123456789012",
            "member_id": "member_123",
            "member_is_managed_by_user": true,
            "metadata": null,
            "minimum_balance": null,
            "minimum_payment": null,
            "name": "Chase Checking",
            "nickname": null,
            "original_balance": null,
            "pay_out_amount": null,
            "payment_due_at": null,
            "payoff_balance": null,
            "premium_amount": null,
            "routing_number": "021000021",
            "started_on": null,
            "subtype": "CHECKING",
            "total_account_value": null,
            "type": "CHECKING",
            "updated_at": "2024-01-15T12:00:00Z",
            "user_guid": "USR-12345678-1234-1234-1234-123456789012",
            "user_id": "user_123"
        },
        {
            "account_number": "0987654321",
            "apr": null,
            "apy": 0.50,
            "available_balance": 5000.00,
            "available_credit": null,
            "balance": 5000.00,
            "cash_balance": null,
            "cash_surrender_value": null,
            "created_at": "2024-01-15T10:00:00Z",
            "credit_limit": null,
            "currency_code": "USD",
            "day_payment_is_due": null,
            "death_benefit": null,
            "guid": "ACT-87654321-4321-4321-4321-210987654321",
            "holdings_value": null,
            "id": "account_456",
            "imported_at": "2024-01-15T10:00:00Z",
            "institution_code": "chase",
            "insured_name": null,
            "interest_rate": null,
            "is_closed": false,
            "is_hidden": false,
            "last_payment": null,
            "last_payment_at": null,
            "loan_amount": null,
            "matures_on": null,
            "member_guid": "MBR-12345678-1234-1234-1234-123456789012",
            "member_id": "member_123",
            "member_is_managed_by_user": true,
            "metadata": null,
            "minimum_balance": null,
            "minimum_payment": null,
            "name": "Chase Savings",
            "nickname": null,
            "original_balance": null,
            "pay_out_amount": null,
            "payment_due_at": null,
            "payoff_balance": null,
            "premium_amount": null,
            "routing_number": "021000021",
            "started_on": null,
            "subtype": "SAVINGS",
            "total_account_value": null,
            "type": "SAVINGS",
            "updated_at": "2024-01-15T12:00:00Z",
            "user_guid": "USR-12345678-1234-1234-1234-123456789012",
            "user_id": "user_123"
        }
    ],
    "pagination": {
        "current_page": 1,
        "per_page": 25,
        "total_entries": 2,
        "total_pages": 1
    }
}"#;

pub const ACCOUNT_RESPONSE: &str = r#"{
    "account": {
        "account_number": "1234567890",
        "apr": null,
        "apy": 0.01,
        "available_balance": 1500.00,
        "available_credit": null,
        "balance": 1600.00,
        "cash_balance": null,
        "cash_surrender_value": null,
        "created_at": "2024-01-15T10:00:00Z",
        "credit_limit": null,
        "currency_code": "USD",
        "day_payment_is_due": null,
        "death_benefit": null,
        "guid": "ACT-12345678-1234-1234-1234-123456789012",
        "holdings_value": null,
        "id": "account_123",
        "imported_at": "2024-01-15T10:00:00Z",
        "institution_code": "chase",
        "insured_name": null,
        "interest_rate": null,
        "is_closed": false,
        "is_hidden": false,
        "last_payment": null,
        "last_payment_at": null,
        "loan_amount": null,
        "matures_on": null,
        "member_guid": "MBR-12345678-1234-1234-1234-123456789012",
        "member_id": "member_123",
        "member_is_managed_by_user": true,
        "metadata": null,
        "minimum_balance": null,
        "minimum_payment": null,
        "name": "Chase Checking",
        "nickname": null,
        "original_balance": null,
        "pay_out_amount": null,
        "payment_due_at": null,
        "payoff_balance": null,
        "premium_amount": null,
        "routing_number": "021000021",
        "started_on": null,
        "subtype": "CHECKING",
        "total_account_value": null,
        "type": "CHECKING",
        "updated_at": "2024-01-15T12:00:00Z",
        "user_guid": "USR-12345678-1234-1234-1234-123456789012",
        "user_id": "user_123"
    }
}"#;

pub const TRANSACTIONS_RESPONSE: &str = r#"{
    "transactions": [
        {
            "account_guid": "ACT-12345678-1234-1234-1234-123456789012",
            "account_id": "account_123",
            "amount": 25.00,
            "category": "Food & Dining",
            "category_guid": "CAT-12345678-1234-1234-1234-123456789012",
            "check_number_string": null,
            "created_at": "2024-01-15T14:30:00Z",
            "currency_code": "USD",
            "date": "2024-01-15",
            "description": "Starbucks",
            "extended_transaction_type": "fee",
            "guid": "TRN-12345678-1234-1234-1234-123456789012",
            "id": "transaction_001",
            "is_bill_pay": false,
            "is_direct_deposit": false,
            "is_expense": true,
            "is_fee": false,
            "is_income": false,
            "is_international": false,
            "is_overdraft_fee": false,
            "is_payroll_advance": false,
            "is_recurring": false,
            "is_subscription": false,
            "latitude": 37.7749,
            "longitude": -122.4194,
            "member_guid": "MBR-12345678-1234-1234-1234-123456789012",
            "member_is_managed_by_user": true,
            "memo": null,
            "merchant_category_code": 5814,
            "merchant_guid": "MCH-12345678-1234-1234-1234-123456789012",
            "merchant_location_guid": null,
            "metadata": null,
            "original_description": "STARBUCKS STORE #12345",
            "posted_at": "2024-01-15T14:30:00Z",
            "status": "POSTED",
            "top_level_category": "Food & Dining",
            "transacted_at": "2024-01-15T14:30:00Z",
            "type": "DEBIT",
            "updated_at": "2024-01-15T14:30:00Z",
            "user_guid": "USR-12345678-1234-1234-1234-123456789012",
            "user_id": "user_123"
        },
        {
            "account_guid": "ACT-12345678-1234-1234-1234-123456789012",
            "account_id": "account_123",
            "amount": -3000.00,
            "category": "Income",
            "category_guid": "CAT-87654321-4321-4321-4321-210987654321",
            "check_number_string": null,
            "created_at": "2024-01-14T09:00:00Z",
            "currency_code": "USD",
            "date": "2024-01-14",
            "description": "Direct Deposit - ACME Corp",
            "extended_transaction_type": null,
            "guid": "TRN-87654321-4321-4321-4321-210987654321",
            "id": "transaction_002",
            "is_bill_pay": false,
            "is_direct_deposit": true,
            "is_expense": false,
            "is_fee": false,
            "is_income": true,
            "is_international": false,
            "is_overdraft_fee": false,
            "is_payroll_advance": false,
            "is_recurring": true,
            "is_subscription": false,
            "latitude": null,
            "longitude": null,
            "member_guid": "MBR-12345678-1234-1234-1234-123456789012",
            "member_is_managed_by_user": true,
            "memo": "January Salary",
            "merchant_category_code": null,
            "merchant_guid": null,
            "merchant_location_guid": null,
            "metadata": null,
            "original_description": "ACME CORP PAYROLL",
            "posted_at": "2024-01-14T09:00:00Z",
            "status": "POSTED",
            "top_level_category": "Income",
            "transacted_at": "2024-01-14T09:00:00Z",
            "type": "CREDIT",
            "updated_at": "2024-01-14T09:00:00Z",
            "user_guid": "USR-12345678-1234-1234-1234-123456789012",
            "user_id": "user_123"
        }
    ],
    "pagination": {
        "current_page": 1,
        "per_page": 25,
        "total_entries": 2,
        "total_pages": 1
    }
}"#;

pub const ALL_TRANSACTIONS_RESPONSE: &str = r#"{
    "transactions": [
        {
            "account_guid": "ACT-12345678-1234-1234-1234-123456789012",
            "account_id": "account_123",
            "amount": 25.00,
            "category": "Food & Dining",
            "date": "2024-01-15",
            "description": "Starbucks",
            "guid": "TRN-12345678-1234-1234-1234-123456789012",
            "id": "transaction_001",
            "status": "POSTED",
            "type": "DEBIT"
        }
    ],
    "pagination": {
        "current_page": 1,
        "per_page": 25,
        "total_entries": 1,
        "total_pages": 1
    }
}"#;

pub const INSTITUTIONS_RESPONSE: &str = r#"{
    "institutions": [
        {
            "code": "chase",
            "medium_logo_url": "https://content.moneydesktop.com/storage/MD_Assets/Iphone%20Icons/100x100/chase.png",
            "name": "Chase",
            "small_logo_url": "https://content.moneydesktop.com/storage/MD_Assets/Iphone%20Icons/50x50/chase.png",
            "supports_account_identification": true,
            "supports_account_statement": true,
            "supports_account_verification": true,
            "supports_oauth": false,
            "supports_transaction_history": true,
            "url": "https://www.chase.com"
        },
        {
            "code": "wells_fargo",
            "medium_logo_url": "https://content.moneydesktop.com/storage/MD_Assets/Iphone%20Icons/100x100/wells_fargo.png",
            "name": "Wells Fargo",
            "small_logo_url": "https://content.moneydesktop.com/storage/MD_Assets/Iphone%20Icons/50x50/wells_fargo.png",
            "supports_account_identification": true,
            "supports_account_statement": true,
            "supports_account_verification": true,
            "supports_oauth": false,
            "supports_transaction_history": true,
            "url": "https://www.wellsfargo.com"
        }
    ],
    "pagination": {
        "current_page": 1,
        "per_page": 25,
        "total_entries": 2,
        "total_pages": 1
    }
}"#;

pub const INSTITUTION_RESPONSE: &str = r#"{
    "institution": {
        "code": "chase",
        "medium_logo_url": "https://content.moneydesktop.com/storage/MD_Assets/Iphone%20Icons/100x100/chase.png",
        "name": "Chase",
        "small_logo_url": "https://content.moneydesktop.com/storage/MD_Assets/Iphone%20Icons/50x50/chase.png",
        "supports_account_identification": true,
        "supports_account_statement": true,
        "supports_account_verification": true,
        "supports_oauth": false,
        "supports_transaction_history": true,
        "url": "https://www.chase.com"
    }
}"#;

pub const CONNECT_WIDGET_URL_RESPONSE: &str = r#"{
    "user": {
        "connect_widget_url": "https://int-widgets.moneydesktop.com/md/connect/12345abcde",
        "guid": "USR-12345678-1234-1234-1234-123456789012"
    }
}"#;

pub const ACCOUNT_OWNERS_RESPONSE: &str = r#"{
    "account_owners": [
        {
            "account_guid": "ACT-12345678-1234-1234-1234-123456789012",
            "address": "123 Main St, San Francisco, CA 94102",
            "city": "San Francisco",
            "country": "US",
            "email": "john.doe@example.com",
            "first_name": "John",
            "guid": "AOW-12345678-1234-1234-1234-123456789012",
            "last_name": "Doe",
            "member_guid": "MBR-12345678-1234-1234-1234-123456789012",
            "owner_name": "John Doe",
            "phone": "+1 415-555-0123",
            "postal_code": "94102",
            "state": "CA"
        }
    ],
    "pagination": {
        "current_page": 1,
        "per_page": 25,
        "total_entries": 1,
        "total_pages": 1
    }
}"#;
