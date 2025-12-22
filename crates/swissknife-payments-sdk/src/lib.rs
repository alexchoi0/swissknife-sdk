mod error;

pub use error::{Error, Result};

#[cfg(feature = "stripe")]
pub mod stripe;

#[cfg(feature = "paypal")]
pub mod paypal;

#[cfg(feature = "square")]
pub mod square;

#[cfg(feature = "braintree")]
pub mod braintree;

#[cfg(feature = "adyen")]
pub mod adyen;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    USD,
    EUR,
    GBP,
    CAD,
    AUD,
    JPY,
    CNY,
    INR,
    BRL,
    MXN,
    CHF,
    SEK,
    NOK,
    DKK,
    NZD,
    SGD,
    HKD,
    KRW,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::USD => "usd",
            Currency::EUR => "eur",
            Currency::GBP => "gbp",
            Currency::CAD => "cad",
            Currency::AUD => "aud",
            Currency::JPY => "jpy",
            Currency::CNY => "cny",
            Currency::INR => "inr",
            Currency::BRL => "brl",
            Currency::MXN => "mxn",
            Currency::CHF => "chf",
            Currency::SEK => "sek",
            Currency::NOK => "nok",
            Currency::DKK => "dkk",
            Currency::NZD => "nzd",
            Currency::SGD => "sgd",
            Currency::HKD => "hkd",
            Currency::KRW => "krw",
        }
    }

    pub fn zero_decimal(&self) -> bool {
        matches!(self, Currency::JPY | Currency::KRW)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: i64,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn usd(cents: i64) -> Self {
        Self::new(cents, Currency::USD)
    }

    pub fn eur(cents: i64) -> Self {
        Self::new(cents, Currency::EUR)
    }

    pub fn from_decimal(amount: f64, currency: Currency) -> Self {
        let multiplier = if currency.zero_decimal() { 1.0 } else { 100.0 };
        Self::new((amount * multiplier) as i64, currency)
    }

    pub fn to_decimal(&self) -> f64 {
        let divisor = if self.currency.zero_decimal() { 1.0 } else { 100.0 };
        self.amount as f64 / divisor
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub number: String,
    pub exp_month: u8,
    pub exp_year: u16,
    pub cvc: String,
    pub holder_name: Option<String>,
}

impl Card {
    pub fn new(number: impl Into<String>, exp_month: u8, exp_year: u16, cvc: impl Into<String>) -> Self {
        Self {
            number: number.into(),
            exp_month,
            exp_year,
            cvc: cvc.into(),
            holder_name: None,
        }
    }

    pub fn with_holder_name(mut self, name: impl Into<String>) -> Self {
        self.holder_name = Some(name.into());
        self
    }

    pub fn last_four(&self) -> &str {
        if self.number.len() >= 4 {
            &self.number[self.number.len() - 4..]
        } else {
            &self.number
        }
    }

    pub fn brand(&self) -> CardBrand {
        let number = self.number.replace([' ', '-'], "");
        if number.starts_with('4') {
            CardBrand::Visa
        } else if number.starts_with("34") || number.starts_with("37") {
            CardBrand::Amex
        } else if number.starts_with("51") || number.starts_with("52") || number.starts_with("53") || number.starts_with("54") || number.starts_with("55") {
            CardBrand::Mastercard
        } else if number.starts_with("6011") || number.starts_with("65") {
            CardBrand::Discover
        } else if number.starts_with("35") {
            CardBrand::JCB
        } else if number.starts_with("30") || number.starts_with("36") || number.starts_with("38") {
            CardBrand::DinersClub
        } else if number.starts_with("62") {
            CardBrand::UnionPay
        } else {
            CardBrand::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardBrand {
    Visa,
    Mastercard,
    Amex,
    Discover,
    JCB,
    DinersClub,
    UnionPay,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

impl Address {
    pub fn new(line1: impl Into<String>, city: impl Into<String>, postal_code: impl Into<String>, country: impl Into<String>) -> Self {
        Self {
            line1: line1.into(),
            line2: None,
            city: city.into(),
            state: None,
            postal_code: postal_code.into(),
            country: country.into(),
        }
    }

    pub fn with_line2(mut self, line2: impl Into<String>) -> Self {
        self.line2 = Some(line2.into());
        self
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<Address>,
}

impl Customer {
    pub fn new() -> Self {
        Self {
            id: None,
            email: None,
            name: None,
            phone: None,
            address: None,
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }
}

impl Default for Customer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Canceled,
    Refunded,
    PartiallyRefunded,
    RequiresAction,
    RequiresCapture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub id: String,
    pub status: PaymentStatus,
    pub amount: Money,
    pub customer_id: Option<String>,
    pub payment_method_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResult {
    pub id: String,
    pub payment_id: String,
    pub amount: Money,
    pub status: RefundStatus,
    pub reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundStatus {
    Pending,
    Succeeded,
    Failed,
    Canceled,
}

use async_trait::async_trait;

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn charge(&self, amount: Money, card: &Card, customer: Option<&Customer>) -> Result<PaymentResult>;
    async fn authorize(&self, amount: Money, card: &Card, customer: Option<&Customer>) -> Result<PaymentResult>;
    async fn capture(&self, payment_id: &str, amount: Option<Money>) -> Result<PaymentResult>;
    async fn void(&self, payment_id: &str) -> Result<PaymentResult>;
    async fn refund(&self, payment_id: &str, amount: Option<Money>, reason: Option<&str>) -> Result<RefundResult>;
    async fn get_payment(&self, payment_id: &str) -> Result<PaymentResult>;
}
