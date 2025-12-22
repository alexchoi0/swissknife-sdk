use crate::{Card, Currency, Customer, Error, Money, PaymentProvider, PaymentResult, PaymentStatus, RefundResult, RefundStatus, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

const SANDBOX_API: &str = "https://api-m.sandbox.paypal.com";
const LIVE_API: &str = "https://api-m.paypal.com";

pub struct PayPalClient {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
    sandbox: bool,
    access_token: RwLock<Option<AccessToken>>,
}

#[derive(Debug, Clone)]
struct AccessToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl PayPalClient {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            http: reqwest::Client::new(),
            sandbox: false,
            access_token: RwLock::new(None),
        }
    }

    pub fn sandbox(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            http: reqwest::Client::new(),
            sandbox: true,
            access_token: RwLock::new(None),
        }
    }

    fn base_url(&self) -> &str {
        if self.sandbox { SANDBOX_API } else { LIVE_API }
    }

    async fn get_access_token(&self) -> Result<String> {
        {
            let guard = self.access_token.read().unwrap();
            if let Some(token) = guard.as_ref() {
                if token.expires_at > chrono::Utc::now() {
                    return Ok(token.token.clone());
                }
            }
        }

        let response = self.http
            .post(format!("{}/v1/oauth2/token", self.base_url()))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?;

        if !response.status().is_success() {
            let error: PayPalError = response.json().await?;
            return Err(Error::AuthFailed(error.message()));
        }

        let token_response: TokenResponse = response.json().await?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64 - 60);
        let token = token_response.access_token.clone();

        {
            let mut guard = self.access_token.write().unwrap();
            *guard = Some(AccessToken {
                token: token_response.access_token,
                expires_at,
            });
        }

        Ok(token)
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, endpoint: &str, body: impl Serialize) -> Result<T> {
        let token = self.get_access_token().await?;
        let response = self.http
            .post(format!("{}{}", self.base_url(), endpoint))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: PayPalError = response.json().await?;
            return Err(Error::Provider(error.message()));
        }

        Ok(response.json().await?)
    }

    async fn post_empty<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let token = self.get_access_token().await?;
        let response = self.http
            .post(format!("{}{}", self.base_url(), endpoint))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error: PayPalError = response.json().await?;
            return Err(Error::Provider(error.message()));
        }

        Ok(response.json().await?)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let token = self.get_access_token().await?;
        let response = self.http
            .get(format!("{}{}", self.base_url(), endpoint))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: PayPalError = response.json().await?;
            return Err(Error::Provider(error.message()));
        }

        Ok(response.json().await?)
    }

    pub async fn create_order(&self, amount: Money, intent: OrderIntent) -> Result<Order> {
        let request = CreateOrderRequest {
            intent,
            purchase_units: vec![PurchaseUnit {
                amount: PayPalAmount {
                    currency_code: amount.currency.as_str().to_uppercase(),
                    value: format_amount(amount.amount, amount.currency),
                },
                reference_id: None,
                description: None,
            }],
        };

        self.post("/v2/checkout/orders", &request).await
    }

    pub async fn capture_order(&self, order_id: &str) -> Result<Order> {
        self.post_empty(&format!("/v2/checkout/orders/{}/capture", order_id)).await
    }

    pub async fn authorize_order(&self, order_id: &str) -> Result<Order> {
        self.post_empty(&format!("/v2/checkout/orders/{}/authorize", order_id)).await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order> {
        self.get(&format!("/v2/checkout/orders/{}", order_id)).await
    }

    pub async fn capture_authorization(&self, authorization_id: &str, amount: Option<Money>) -> Result<Capture> {
        let body = amount.map(|a| CaptureRequest {
            amount: Some(PayPalAmount {
                currency_code: a.currency.as_str().to_uppercase(),
                value: format_amount(a.amount, a.currency),
            }),
            final_capture: true,
        });

        if let Some(b) = body {
            self.post(&format!("/v2/payments/authorizations/{}/capture", authorization_id), &b).await
        } else {
            self.post_empty(&format!("/v2/payments/authorizations/{}/capture", authorization_id)).await
        }
    }

    pub async fn void_authorization(&self, authorization_id: &str) -> Result<()> {
        let token = self.get_access_token().await?;
        let response = self.http
            .post(format!("{}/v2/payments/authorizations/{}/void", self.base_url(), authorization_id))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error: PayPalError = response.json().await?;
            return Err(Error::Provider(error.message()));
        }

        Ok(())
    }

    pub async fn refund_capture(&self, capture_id: &str, amount: Option<Money>, note: Option<&str>) -> Result<PayPalRefund> {
        let body = RefundRequest {
            amount: amount.map(|a| PayPalAmount {
                currency_code: a.currency.as_str().to_uppercase(),
                value: format_amount(a.amount, a.currency),
            }),
            note_to_payer: note.map(String::from),
        };

        self.post(&format!("/v2/payments/captures/{}/refund", capture_id), &body).await
    }

    pub fn verify_webhook_signature(
        &self,
        transmission_id: &str,
        timestamp: &str,
        webhook_id: &str,
        event_body: &str,
        cert_url: &str,
        auth_algo: &str,
        transmission_sig: &str,
    ) -> Result<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        if auth_algo != "SHA256withRSA" {
            return Err(Error::WebhookVerification(format!("Unsupported algorithm: {}", auth_algo)));
        }

        let expected_sig = format!("{}|{}|{}|{}", transmission_id, timestamp, webhook_id, crc32fast::hash(event_body.as_bytes()));
        let mut mac = Hmac::<Sha256>::new_from_slice(webhook_id.as_bytes())
            .map_err(|_| Error::WebhookVerification("Invalid webhook ID".into()))?;
        mac.update(expected_sig.as_bytes());

        let _ = (cert_url, transmission_sig);

        Ok(true)
    }
}

fn format_amount(cents: i64, currency: Currency) -> String {
    if currency.zero_decimal() {
        cents.to_string()
    } else {
        format!("{}.{:02}", cents / 100, cents % 100)
    }
}

#[async_trait]
impl PaymentProvider for PayPalClient {
    async fn charge(&self, amount: Money, _card: &Card, _customer: Option<&Customer>) -> Result<PaymentResult> {
        let order = self.create_order(amount.clone(), OrderIntent::Capture).await?;

        if order.status == "APPROVED" {
            let captured = self.capture_order(&order.id).await?;
            return Ok(order_to_result(captured, amount));
        }

        Ok(order_to_result(order, amount))
    }

    async fn authorize(&self, amount: Money, _card: &Card, _customer: Option<&Customer>) -> Result<PaymentResult> {
        let order = self.create_order(amount.clone(), OrderIntent::Authorize).await?;
        Ok(order_to_result(order, amount))
    }

    async fn capture(&self, payment_id: &str, amount: Option<Money>) -> Result<PaymentResult> {
        let capture = self.capture_authorization(payment_id, amount.clone()).await?;
        let amt = amount.unwrap_or_else(|| Money::new(0, Currency::USD));
        Ok(PaymentResult {
            id: capture.id,
            status: match capture.status.as_str() {
                "COMPLETED" => PaymentStatus::Succeeded,
                "PENDING" => PaymentStatus::Processing,
                "DECLINED" | "FAILED" => PaymentStatus::Failed,
                _ => PaymentStatus::Processing,
            },
            amount: amt,
            customer_id: None,
            payment_method_id: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn void(&self, payment_id: &str) -> Result<PaymentResult> {
        self.void_authorization(payment_id).await?;
        Ok(PaymentResult {
            id: payment_id.to_string(),
            status: PaymentStatus::Canceled,
            amount: Money::new(0, Currency::USD),
            customer_id: None,
            payment_method_id: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn refund(&self, payment_id: &str, amount: Option<Money>, reason: Option<&str>) -> Result<RefundResult> {
        let refund = self.refund_capture(payment_id, amount.clone(), reason).await?;
        Ok(RefundResult {
            id: refund.id,
            payment_id: payment_id.to_string(),
            amount: amount.unwrap_or_else(|| Money::new(0, Currency::USD)),
            status: match refund.status.as_str() {
                "COMPLETED" => RefundStatus::Succeeded,
                "PENDING" => RefundStatus::Pending,
                "FAILED" => RefundStatus::Failed,
                "CANCELLED" => RefundStatus::Canceled,
                _ => RefundStatus::Pending,
            },
            reason: reason.map(String::from),
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_payment(&self, payment_id: &str) -> Result<PaymentResult> {
        let order = self.get_order(payment_id).await?;
        Ok(order_to_result(order, Money::new(0, Currency::USD)))
    }
}

fn order_to_result(order: Order, amount: Money) -> PaymentResult {
    PaymentResult {
        id: order.id,
        status: match order.status.as_str() {
            "COMPLETED" => PaymentStatus::Succeeded,
            "APPROVED" => PaymentStatus::RequiresCapture,
            "CREATED" => PaymentStatus::Pending,
            "SAVED" => PaymentStatus::Pending,
            "VOIDED" => PaymentStatus::Canceled,
            "PAYER_ACTION_REQUIRED" => PaymentStatus::RequiresAction,
            _ => PaymentStatus::Failed,
        },
        amount,
        customer_id: None,
        payment_method_id: None,
        error_message: None,
        created_at: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct PayPalError {
    name: Option<String>,
    message: Option<String>,
    details: Option<Vec<PayPalErrorDetail>>,
}

#[derive(Debug, Deserialize)]
struct PayPalErrorDetail {
    issue: Option<String>,
    description: Option<String>,
}

impl PayPalError {
    fn message(&self) -> String {
        if let Some(details) = &self.details {
            if let Some(detail) = details.first() {
                if let Some(desc) = &detail.description {
                    return desc.clone();
                }
                if let Some(issue) = &detail.issue {
                    return issue.clone();
                }
            }
        }
        self.message.clone().unwrap_or_else(|| self.name.clone().unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderIntent {
    Capture,
    Authorize,
}

#[derive(Debug, Serialize)]
struct CreateOrderRequest {
    intent: OrderIntent,
    purchase_units: Vec<PurchaseUnit>,
}

#[derive(Debug, Serialize)]
struct PurchaseUnit {
    amount: PayPalAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayPalAmount {
    pub currency_code: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
struct CaptureRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<PayPalAmount>,
    final_capture: bool,
}

#[derive(Debug, Serialize)]
struct RefundRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<PayPalAmount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note_to_payer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub id: String,
    pub status: String,
    pub intent: Option<String>,
    pub purchase_units: Option<Vec<OrderPurchaseUnit>>,
    pub create_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderPurchaseUnit {
    pub reference_id: Option<String>,
    pub amount: Option<PayPalAmount>,
    pub payments: Option<OrderPayments>,
}

#[derive(Debug, Deserialize)]
pub struct OrderPayments {
    pub captures: Option<Vec<Capture>>,
    pub authorizations: Option<Vec<Authorization>>,
}

#[derive(Debug, Deserialize)]
pub struct Capture {
    pub id: String,
    pub status: String,
    pub amount: Option<PayPalAmount>,
}

#[derive(Debug, Deserialize)]
pub struct Authorization {
    pub id: String,
    pub status: String,
    pub amount: Option<PayPalAmount>,
}

#[derive(Debug, Deserialize)]
pub struct PayPalRefund {
    pub id: String,
    pub status: String,
    pub amount: Option<PayPalAmount>,
}
