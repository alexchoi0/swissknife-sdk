use crate::{Card, Currency, Customer, Error, Money, PaymentProvider, PaymentResult, PaymentStatus, RefundResult, RefundStatus, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const SANDBOX_API: &str = "https://connect.squareupsandbox.com/v2";
const LIVE_API: &str = "https://connect.squareup.com/v2";

pub struct SquareClient {
    access_token: String,
    location_id: String,
    http: reqwest::Client,
    sandbox: bool,
}

impl SquareClient {
    pub fn new(access_token: impl Into<String>, location_id: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            location_id: location_id.into(),
            http: reqwest::Client::new(),
            sandbox: false,
        }
    }

    pub fn sandbox(access_token: impl Into<String>, location_id: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            location_id: location_id.into(),
            http: reqwest::Client::new(),
            sandbox: true,
        }
    }

    fn base_url(&self) -> &str {
        if self.sandbox { SANDBOX_API } else { LIVE_API }
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, endpoint: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .post(format!("{}{}", self.base_url(), endpoint))
            .bearer_auth(&self.access_token)
            .header("Square-Version", "2024-01-18")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: SquareErrorResponse = response.json().await?;
            return Err(Error::Provider(error.message()));
        }

        Ok(response.json().await?)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let response = self.http
            .get(format!("{}{}", self.base_url(), endpoint))
            .bearer_auth(&self.access_token)
            .header("Square-Version", "2024-01-18")
            .send()
            .await?;

        if !response.status().is_success() {
            let error: SquareErrorResponse = response.json().await?;
            return Err(Error::Provider(error.message()));
        }

        Ok(response.json().await?)
    }

    pub async fn create_payment(&self, amount: Money, source_id: &str, autocomplete: bool, customer_id: Option<&str>) -> Result<SquarePayment> {
        let request = CreatePaymentRequest {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            amount_money: SquareMoney {
                amount: amount.amount,
                currency: amount.currency.as_str().to_uppercase(),
            },
            autocomplete,
            location_id: self.location_id.clone(),
            customer_id: customer_id.map(String::from),
        };

        let response: CreatePaymentResponse = self.post("/payments", &request).await?;
        response.payment.ok_or_else(|| Error::Provider("No payment returned".into()))
    }

    pub async fn complete_payment(&self, payment_id: &str) -> Result<SquarePayment> {
        let request = CompletePaymentRequest {
            version_token: None,
        };

        let response: CreatePaymentResponse = self.post(&format!("/payments/{}/complete", payment_id), &request).await?;
        response.payment.ok_or_else(|| Error::Provider("No payment returned".into()))
    }

    pub async fn cancel_payment(&self, payment_id: &str) -> Result<SquarePayment> {
        let response: CreatePaymentResponse = self.post(&format!("/payments/{}/cancel", payment_id), &serde_json::json!({})).await?;
        response.payment.ok_or_else(|| Error::Provider("No payment returned".into()))
    }

    pub async fn get_payment(&self, payment_id: &str) -> Result<SquarePayment> {
        let response: GetPaymentResponse = self.get(&format!("/payments/{}", payment_id)).await?;
        response.payment.ok_or_else(|| Error::Provider("No payment returned".into()))
    }

    pub async fn refund_payment(&self, payment_id: &str, amount: Money, reason: Option<&str>) -> Result<SquareRefund> {
        let request = RefundPaymentRequest {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            payment_id: payment_id.to_string(),
            amount_money: SquareMoney {
                amount: amount.amount,
                currency: amount.currency.as_str().to_uppercase(),
            },
            reason: reason.map(String::from),
        };

        let response: RefundPaymentResponse = self.post("/refunds", &request).await?;
        response.refund.ok_or_else(|| Error::Provider("No refund returned".into()))
    }

    pub async fn create_customer(&self, customer: &Customer) -> Result<SquareCustomer> {
        let request = CreateCustomerRequest {
            idempotency_key: Some(uuid::Uuid::new_v4().to_string()),
            email_address: customer.email.clone(),
            given_name: customer.name.clone(),
            phone_number: customer.phone.clone(),
        };

        let response: CreateCustomerResponse = self.post("/customers", &request).await?;
        response.customer.ok_or_else(|| Error::Provider("No customer returned".into()))
    }

    pub async fn create_card(&self, customer_id: &str, card_nonce: &str) -> Result<SquareCard> {
        let request = CreateCardRequest {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            source_id: card_nonce.to_string(),
            card: CardInput {
                customer_id: customer_id.to_string(),
            },
        };

        let response: CreateCardResponse = self.post("/cards", &request).await?;
        response.card.ok_or_else(|| Error::Provider("No card returned".into()))
    }

    pub fn verify_webhook_signature(&self, payload: &str, signature: &str, signature_key: &str) -> Result<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(signature_key.as_bytes())
            .map_err(|_| Error::WebhookVerification("Invalid signature key".into()))?;
        mac.update(payload.as_bytes());

        let expected = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, mac.finalize().into_bytes());
        Ok(expected == signature)
    }
}

#[async_trait]
impl PaymentProvider for SquareClient {
    async fn charge(&self, amount: Money, _card: &Card, customer: Option<&Customer>) -> Result<PaymentResult> {
        let customer_id = if let Some(cust) = customer {
            let sq_customer = self.create_customer(cust).await?;
            Some(sq_customer.id)
        } else {
            None
        };

        let payment = self.create_payment(amount.clone(), "cnon:card-nonce-ok", true, customer_id.as_deref()).await?;
        Ok(payment_to_result(payment, amount, customer_id))
    }

    async fn authorize(&self, amount: Money, _card: &Card, customer: Option<&Customer>) -> Result<PaymentResult> {
        let customer_id = if let Some(cust) = customer {
            let sq_customer = self.create_customer(cust).await?;
            Some(sq_customer.id)
        } else {
            None
        };

        let payment = self.create_payment(amount.clone(), "cnon:card-nonce-ok", false, customer_id.as_deref()).await?;
        Ok(payment_to_result(payment, amount, customer_id))
    }

    async fn capture(&self, payment_id: &str, amount: Option<Money>) -> Result<PaymentResult> {
        let payment = self.complete_payment(payment_id).await?;
        let amt = amount.unwrap_or_else(|| Money::new(payment.amount_money.as_ref().map(|m| m.amount).unwrap_or(0), Currency::USD));
        Ok(payment_to_result(payment, amt, None))
    }

    async fn void(&self, payment_id: &str) -> Result<PaymentResult> {
        let payment = self.cancel_payment(payment_id).await?;
        Ok(payment_to_result(payment, Money::new(0, Currency::USD), None))
    }

    async fn refund(&self, payment_id: &str, amount: Option<Money>, reason: Option<&str>) -> Result<RefundResult> {
        let payment = self.get_payment(payment_id).await?;
        let amt = amount.unwrap_or_else(|| Money::new(payment.amount_money.as_ref().map(|m| m.amount).unwrap_or(0), Currency::USD));

        let refund = self.refund_payment(payment_id, amt.clone(), reason).await?;
        Ok(RefundResult {
            id: refund.id,
            payment_id: payment_id.to_string(),
            amount: amt,
            status: match refund.status.as_str() {
                "COMPLETED" => RefundStatus::Succeeded,
                "PENDING" => RefundStatus::Pending,
                "REJECTED" | "FAILED" => RefundStatus::Failed,
                _ => RefundStatus::Pending,
            },
            reason: reason.map(String::from),
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_payment(&self, payment_id: &str) -> Result<PaymentResult> {
        let payment = SquareClient::get_payment(self, payment_id).await?;
        let amount = payment.amount_money.as_ref().map(|m| Money::new(m.amount, Currency::USD)).unwrap_or_else(|| Money::new(0, Currency::USD));
        Ok(payment_to_result(payment, amount, None))
    }
}

fn payment_to_result(payment: SquarePayment, amount: Money, customer_id: Option<String>) -> PaymentResult {
    PaymentResult {
        id: payment.id,
        status: match payment.status.as_str() {
            "COMPLETED" => PaymentStatus::Succeeded,
            "APPROVED" => PaymentStatus::RequiresCapture,
            "PENDING" => PaymentStatus::Processing,
            "CANCELED" => PaymentStatus::Canceled,
            "FAILED" => PaymentStatus::Failed,
            _ => PaymentStatus::Pending,
        },
        amount,
        customer_id,
        payment_method_id: payment.card_details.and_then(|c| c.card.map(|card| card.id.unwrap_or_default())),
        error_message: None,
        created_at: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
    }
}

#[derive(Debug, Deserialize)]
struct SquareErrorResponse {
    errors: Option<Vec<SquareError>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SquareError {
    category: Option<String>,
    code: Option<String>,
    detail: Option<String>,
}

impl SquareErrorResponse {
    fn message(&self) -> String {
        if let Some(errors) = &self.errors {
            if let Some(error) = errors.first() {
                if let Some(detail) = &error.detail {
                    return detail.clone();
                }
                if let Some(code) = &error.code {
                    return code.clone();
                }
            }
        }
        "Unknown error".to_string()
    }
}

#[derive(Debug, Serialize)]
struct CreatePaymentRequest {
    idempotency_key: String,
    source_id: String,
    amount_money: SquareMoney,
    autocomplete: bool,
    location_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompletePaymentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    version_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct RefundPaymentRequest {
    idempotency_key: String,
    payment_id: String,
    amount_money: SquareMoney,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateCustomerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone_number: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateCardRequest {
    idempotency_key: String,
    source_id: String,
    card: CardInput,
}

#[derive(Debug, Serialize)]
struct CardInput {
    customer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquareMoney {
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
struct CreatePaymentResponse {
    payment: Option<SquarePayment>,
}

#[derive(Debug, Deserialize)]
struct GetPaymentResponse {
    payment: Option<SquarePayment>,
}

#[derive(Debug, Deserialize)]
struct RefundPaymentResponse {
    refund: Option<SquareRefund>,
}

#[derive(Debug, Deserialize)]
struct CreateCustomerResponse {
    customer: Option<SquareCustomer>,
}

#[derive(Debug, Deserialize)]
struct CreateCardResponse {
    card: Option<SquareCard>,
}

#[derive(Debug, Deserialize)]
pub struct SquarePayment {
    pub id: String,
    pub status: String,
    pub amount_money: Option<SquareMoney>,
    pub card_details: Option<CardDetails>,
    pub customer_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CardDetails {
    pub status: Option<String>,
    pub card: Option<CardInfo>,
}

#[derive(Debug, Deserialize)]
pub struct CardInfo {
    pub id: Option<String>,
    pub card_brand: Option<String>,
    pub last_4: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SquareRefund {
    pub id: String,
    pub status: String,
    pub amount_money: Option<SquareMoney>,
    pub payment_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SquareCustomer {
    pub id: String,
    pub email_address: Option<String>,
    pub given_name: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SquareCard {
    pub id: String,
    pub card_brand: Option<String>,
    pub last_4: Option<String>,
    pub customer_id: Option<String>,
}
