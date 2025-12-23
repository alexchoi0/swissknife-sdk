use super::domain::{account, institution, transaction, user};
use chrono::Utc;
use uuid::Uuid;

const FIRST_NAMES: &[&str] = &[
    "John", "Jane", "Michael", "Sarah", "David", "Emily", "James", "Emma", "Robert", "Olivia",
];

const LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez",
];

const STREET_NAMES: &[&str] = &[
    "Main St", "Oak Ave", "Maple Dr", "Cedar Ln", "Pine Rd", "Elm St", "Park Ave", "Broadway", "First St", "Second Ave",
];

const CITIES: &[&str] = &[
    "New York", "Los Angeles", "Chicago", "Houston", "Phoenix", "San Francisco", "Seattle", "Denver", "Boston", "Atlanta",
];

const STATES: &[&str] = &[
    "NY", "CA", "IL", "TX", "AZ", "CA", "WA", "CO", "MA", "GA",
];

const MERCHANTS: &[&str] = &[
    "Amazon",
    "Starbucks",
    "Walmart",
    "Target",
    "McDonald's",
    "Uber",
    "Netflix",
    "Spotify",
    "Apple",
    "Google",
    "Shell Gas",
    "Costco",
    "Whole Foods",
    "Home Depot",
    "CVS Pharmacy",
];

const CATEGORIES: &[&str] = &[
    "Food & Dining",
    "Shopping",
    "Entertainment",
    "Transportation",
    "Utilities",
    "Healthcare",
    "Travel",
    "Groceries",
    "Gas & Fuel",
    "Subscriptions",
];

const BANK_NAMES: &[&str] = &[
    "Chase",
    "Bank of America",
    "Wells Fargo",
    "Citibank",
    "US Bank",
    "Capital One",
    "PNC Bank",
    "TD Bank",
    "Truist",
    "Fifth Third Bank",
];

fn simple_random(max: usize) -> usize {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let hash: usize = bytes.iter().map(|&b| b as usize).sum();
    hash % max
}

fn random_float(min: f64, max: f64) -> f64 {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let hash: u64 = bytes.iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64).wrapping_mul(31));
    let normalized = (hash as f64) / (u64::MAX as f64);
    min + (max - min) * normalized
}

fn random_bool(probability: f64) -> bool {
    random_float(0.0, 1.0) < probability
}

pub fn generate_user(name: Option<String>, email: Option<String>) -> user::Model {
    let id = Uuid::new_v4().to_string();
    let external_id = format!("USR-{}", Uuid::new_v4().to_string().replace("-", "")[..24].to_uppercase());

    let first = FIRST_NAMES[simple_random(FIRST_NAMES.len())];
    let last = LAST_NAMES[simple_random(LAST_NAMES.len())];
    let full_name = name.unwrap_or_else(|| format!("{} {}", first, last));
    let user_email = email.unwrap_or_else(|| {
        format!("{}.{}@example.com", first.to_lowercase(), last.to_lowercase())
    });

    let street_num = 100 + simple_random(900);
    let street = STREET_NAMES[simple_random(STREET_NAMES.len())];
    let city_idx = simple_random(CITIES.len());

    user::Model {
        id,
        external_id,
        name: full_name,
        email: Some(user_email),
        phone: Some(format!("+1 {}{}{}-{}{}{}-{}{}{}{}",
            2 + simple_random(8), simple_random(10), simple_random(10),
            simple_random(10), simple_random(10), simple_random(10),
            simple_random(10), simple_random(10), simple_random(10), simple_random(10)
        )),
        address_street: Some(format!("{} {}", street_num, street)),
        address_city: Some(CITIES[city_idx].to_string()),
        address_state: Some(STATES[city_idx].to_string()),
        address_postal_code: Some(format!("{:05}", 10000 + simple_random(90000))),
        address_country: Some("US".to_string()),
        date_of_birth: Some(format!(
            "{}-{:02}-{:02}",
            1960 + simple_random(40),
            1 + simple_random(12),
            1 + simple_random(28)
        )),
        created_at: Utc::now(),
    }
}

pub fn generate_institution(name: Option<String>, country: Option<String>) -> institution::Model {
    let id = Uuid::new_v4().to_string();
    let bank_name = name.unwrap_or_else(|| {
        BANK_NAMES[simple_random(BANK_NAMES.len())].to_string()
    });
    let external_id = format!(
        "{}_{}",
        bank_name.to_lowercase().replace(" ", "_"),
        1000 + simple_random(9000)
    );

    institution::Model {
        id,
        external_id,
        name: bank_name.clone(),
        bic: Some(format!(
            "{}US33",
            bank_name.chars().take(4).collect::<String>().to_uppercase()
        )),
        logo_url: Some(format!(
            "https://cdn.example.com/logos/{}.png",
            bank_name.to_lowercase().replace(" ", "_")
        )),
        country: country.unwrap_or_else(|| "US".to_string()),
        supported_features: Some(
            serde_json::to_string(&["accounts", "transactions", "identity", "balance"])
                .unwrap_or_default(),
        ),
        created_at: Utc::now(),
    }
}

pub fn generate_account(
    user_id: &str,
    institution_id: &str,
    account_type: Option<String>,
    balance: Option<f64>,
) -> account::Model {
    let id = Uuid::new_v4().to_string();
    let external_id = format!("acc_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase());
    let acc_type = account_type.unwrap_or_else(|| {
        if random_bool(0.7) {
            "checking".to_string()
        } else {
            "savings".to_string()
        }
    });

    let current_balance = balance.unwrap_or_else(|| random_float(100.0, 25000.0));
    let available_balance = current_balance * 0.95;

    let account_number: String = (0..10)
        .map(|_| format!("{}", simple_random(10)))
        .collect();
    let routing_number: String = (0..9)
        .map(|_| format!("{}", simple_random(10)))
        .collect();
    let mask = account_number.chars().rev().take(4).collect::<String>().chars().rev().collect();

    let iban = format!(
        "US{:02}{}{}",
        10 + simple_random(90),
        routing_number,
        account_number
    );

    account::Model {
        id,
        external_id,
        user_id: user_id.to_string(),
        institution_id: institution_id.to_string(),
        name: format!(
            "{} Account",
            if acc_type == "checking" { "Checking" } else { "Savings" }
        ),
        account_type: acc_type.clone(),
        subtype: Some(acc_type),
        currency: "USD".to_string(),
        balance_available: Some((available_balance * 100.0).round() / 100.0),
        balance_current: Some((current_balance * 100.0).round() / 100.0),
        balance_limit: None,
        iban: Some(iban),
        account_number: Some(account_number),
        routing_number: Some(routing_number),
        sort_code: None,
        mask: Some(mask),
        status: "open".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn generate_transaction(
    account_id: &str,
    amount: Option<f64>,
    description: Option<String>,
) -> transaction::Model {
    let id = Uuid::new_v4().to_string();
    let external_id = format!("txn_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase());

    let is_income = random_bool(0.2);
    let txn_amount = amount.unwrap_or_else(|| {
        if is_income {
            random_float(500.0, 5000.0)
        } else {
            -random_float(5.0, 500.0)
        }
    });

    let merchant_idx = simple_random(MERCHANTS.len());
    let category_idx = simple_random(CATEGORIES.len());

    let merchant = if is_income {
        None
    } else {
        Some(MERCHANTS[merchant_idx].to_string())
    };

    let desc = description.unwrap_or_else(|| {
        if is_income {
            "Direct Deposit - Payroll".to_string()
        } else {
            MERCHANTS[merchant_idx].to_string()
        }
    });

    let days_ago = simple_random(90) as i64;
    let date = chrono::Utc::now() - chrono::Duration::days(days_ago);

    transaction::Model {
        id,
        external_id,
        account_id: account_id.to_string(),
        amount: (txn_amount * 100.0).round() / 100.0,
        currency: "USD".to_string(),
        description: desc,
        merchant_name: merchant,
        category: Some(CATEGORIES[category_idx].to_string()),
        transaction_type: if txn_amount < 0.0 { "debit".to_string() } else { "credit".to_string() },
        status: "posted".to_string(),
        date: date.format("%Y-%m-%d").to_string(),
        posted_at: Some(date),
        created_at: Utc::now(),
    }
}

pub fn generate_token() -> TokenData {
    TokenData {
        access_token: format!(
            "access-{}-{}",
            Uuid::new_v4().to_string().replace("-", "")[..16].to_lowercase(),
            Uuid::new_v4().to_string().replace("-", "")[..16].to_lowercase()
        ),
        refresh_token: Some(format!(
            "refresh-{}-{}",
            Uuid::new_v4().to_string().replace("-", "")[..16].to_lowercase(),
            Uuid::new_v4().to_string().replace("-", "")[..16].to_lowercase()
        )),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
    }
}

pub fn generate_link_token() -> LinkTokenData {
    LinkTokenData {
        link_token: format!(
            "link-sandbox-{}-{}-{}",
            Uuid::new_v4().to_string().replace("-", "")[..8].to_lowercase(),
            Uuid::new_v4().to_string().replace("-", "")[..4].to_lowercase(),
            Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase()
        ),
        expiration: (chrono::Utc::now() + chrono::Duration::hours(4))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string(),
        request_id: Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase(),
    }
}

#[derive(Debug, Clone)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Debug, Clone)]
pub struct LinkTokenData {
    pub link_token: String,
    pub expiration: String,
    pub request_id: String,
}
