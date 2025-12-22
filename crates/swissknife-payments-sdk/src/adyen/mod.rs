use crate::{Card, Currency, Customer, Error, Money, PaymentProvider, PaymentResult, PaymentStatus, RefundResult, RefundStatus, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const TEST_API: &str = "https://checkout-test.adyen.com";
const LIVE_API: &str = "https://checkout-live.adyen.com";

pub struct AdyenClient {
    api_key: String,
    merchant_account: String,
    http: reqwest::Client,
    live: bool,
    live_prefix: Option<String>,
}

impl AdyenClient {
    pub fn new(api_key: impl Into<String>, merchant_account: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            merchant_account: merchant_account.into(),
            http: reqwest::Client::new(),
            live: false,
            live_prefix: None,
        }
    }

    pub fn live(api_key: impl Into<String>, merchant_account: impl Into<String>, live_prefix: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            merchant_account: merchant_account.into(),
            http: reqwest::Client::new(),
            live: true,
            live_prefix: Some(live_prefix.into()),
        }
    }

    fn base_url(&self) -> String {
        if self.live {
            if let Some(prefix) = &self.live_prefix {
                format!("https://{}-checkout-live.adyenpayments.com/checkout", prefix)
            } else {
                LIVE_API.to_string()
            }
        } else {
            TEST_API.to_string()
        }
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, endpoint: &str, body: impl Serialize) -> Result<T> {
        let response = self.http
            .post(format!("{}{}", self.base_url(), endpoint))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: AdyenError = response.json().await?;
            return Err(Error::Provider(error.message()));
        }

        Ok(response.json().await?)
    }

    pub async fn create_payment(&self, amount: Money, card: &Card, reference: &str, capture: bool) -> Result<AdyenPaymentResponse> {
        let request = PaymentRequest {
            merchant_account: self.merchant_account.clone(),
            amount: AdyenAmount {
                value: amount.amount,
                currency: amount.currency.as_str().to_uppercase(),
            },
            reference: reference.to_string(),
            payment_method: CardPaymentMethod {
                payment_type: "scheme".to_string(),
                encrypted_card_number: None,
                encrypted_expiry_month: None,
                encrypted_expiry_year: None,
                encrypted_security_code: None,
                number: Some(card.number.clone()),
                expiry_month: Some(format!("{:02}", card.exp_month)),
                expiry_year: Some(card.exp_year.to_string()),
                cvc: Some(card.cvc.clone()),
                holder_name: card.holder_name.clone(),
            },
            capture_delay_hours: if capture { Some(0) } else { None },
            return_url: Some("https://example.com/return".to_string()),
        };

        self.post("/v71/payments", &request).await
    }

    pub async fn capture_payment(&self, psp_reference: &str, amount: Money) -> Result<AdyenModificationResponse> {
        let request = CaptureRequest {
            merchant_account: self.merchant_account.clone(),
            amount: AdyenAmount {
                value: amount.amount,
                currency: amount.currency.as_str().to_uppercase(),
            },
            reference: uuid::Uuid::new_v4().to_string(),
        };

        self.post(&format!("/v71/payments/{}/captures", psp_reference), &request).await
    }

    pub async fn cancel_payment(&self, psp_reference: &str) -> Result<AdyenModificationResponse> {
        let request = CancelRequest {
            merchant_account: self.merchant_account.clone(),
            reference: uuid::Uuid::new_v4().to_string(),
        };

        self.post(&format!("/v71/payments/{}/cancels", psp_reference), &request).await
    }

    pub async fn refund_payment(&self, psp_reference: &str, amount: Money) -> Result<AdyenModificationResponse> {
        let request = RefundRequest {
            merchant_account: self.merchant_account.clone(),
            amount: AdyenAmount {
                value: amount.amount,
                currency: amount.currency.as_str().to_uppercase(),
            },
            reference: uuid::Uuid::new_v4().to_string(),
        };

        self.post(&format!("/v71/payments/{}/refunds", psp_reference), &request).await
    }

    pub async fn get_payment_details(&self, psp_reference: &str) -> Result<AdyenPaymentResponse> {
        let request = serde_json::json!({
            "merchantAccount": self.merchant_account
        });

        self.post(&format!("/v71/payments/{}/details", psp_reference), &request).await
    }

    pub fn verify_webhook_signature(&self, payload: &str, hmac_signature: &str, hmac_key: &str) -> Result<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let key_bytes = hex::decode(hmac_key)
            .map_err(|_| Error::WebhookVerification("Invalid HMAC key".into()))?;

        let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes)
            .map_err(|_| Error::WebhookVerification("Invalid HMAC key".into()))?;
        mac.update(payload.as_bytes());

        let expected = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, mac.finalize().into_bytes());
        Ok(expected == hmac_signature)
    }
}

#[async_trait]
impl PaymentProvider for AdyenClient {
    async fn charge(&self, amount: Money, card: &Card, _customer: Option<&Customer>) -> Result<PaymentResult> {
        let reference = uuid::Uuid::new_v4().to_string();
        let response = self.create_payment(amount.clone(), card, &reference, true).await?;
        Ok(payment_to_result(response, amount))
    }

    async fn authorize(&self, amount: Money, card: &Card, _customer: Option<&Customer>) -> Result<PaymentResult> {
        let reference = uuid::Uuid::new_v4().to_string();
        let response = self.create_payment(amount.clone(), card, &reference, false).await?;
        Ok(payment_to_result(response, amount))
    }

    async fn capture(&self, payment_id: &str, amount: Option<Money>) -> Result<PaymentResult> {
        let amt = amount.unwrap_or_else(|| Money::new(0, Currency::USD));
        let response = self.capture_payment(payment_id, amt.clone()).await?;
        Ok(PaymentResult {
            id: response.psp_reference,
            status: match response.status.as_str() {
                "received" => PaymentStatus::Processing,
                _ => PaymentStatus::Succeeded,
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
        let response = self.cancel_payment(payment_id).await?;
        Ok(PaymentResult {
            id: response.psp_reference,
            status: match response.status.as_str() {
                "received" => PaymentStatus::Processing,
                _ => PaymentStatus::Canceled,
            },
            amount: Money::new(0, Currency::USD),
            customer_id: None,
            payment_method_id: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn refund(&self, payment_id: &str, amount: Option<Money>, reason: Option<&str>) -> Result<RefundResult> {
        let amt = amount.unwrap_or_else(|| Money::new(0, Currency::USD));
        let response = self.refund_payment(payment_id, amt.clone()).await?;
        Ok(RefundResult {
            id: response.psp_reference,
            payment_id: payment_id.to_string(),
            amount: amt,
            status: match response.status.as_str() {
                "received" => RefundStatus::Pending,
                _ => RefundStatus::Succeeded,
            },
            reason: reason.map(String::from),
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_payment(&self, payment_id: &str) -> Result<PaymentResult> {
        let response = self.get_payment_details(payment_id).await?;
        let amount = Money::new(response.amount.as_ref().map(|a| a.value).unwrap_or(0), Currency::USD);
        Ok(payment_to_result(response, amount))
    }
}

fn payment_to_result(response: AdyenPaymentResponse, amount: Money) -> PaymentResult {
    PaymentResult {
        id: response.psp_reference.unwrap_or_default(),
        status: match response.result_code.as_str() {
            "Authorised" => PaymentStatus::Succeeded,
            "Pending" | "Received" => PaymentStatus::Processing,
            "RedirectShopper" | "IdentifyShopper" | "ChallengeShopper" => PaymentStatus::RequiresAction,
            "Cancelled" => PaymentStatus::Canceled,
            "Refused" | "Error" => PaymentStatus::Failed,
            _ => PaymentStatus::Pending,
        },
        amount,
        customer_id: None,
        payment_method_id: None,
        error_message: response.refusal_reason,
        created_at: chrono::Utc::now(),
        metadata: std::collections::HashMap::new(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct AdyenError {
    status: Option<i32>,
    error_code: Option<String>,
    message: Option<String>,
    error_type: Option<String>,
}

impl AdyenError {
    fn message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            self.error_code.clone().unwrap_or_else(|| "Unknown error".to_string())
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentRequest {
    merchant_account: String,
    amount: AdyenAmount,
    reference: String,
    payment_method: CardPaymentMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    capture_delay_hours: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CardPaymentMethod {
    #[serde(rename = "type")]
    payment_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_card_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_expiry_month: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_expiry_year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_security_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiry_month: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiry_year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cvc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    holder_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureRequest {
    merchant_account: String,
    amount: AdyenAmount,
    reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelRequest {
    merchant_account: String,
    reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RefundRequest {
    merchant_account: String,
    amount: AdyenAmount,
    reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenAmount {
    pub value: i64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenPaymentResponse {
    pub psp_reference: Option<String>,
    pub result_code: String,
    pub amount: Option<AdyenAmount>,
    pub merchant_reference: Option<String>,
    pub refusal_reason: Option<String>,
    pub refusal_reason_code: Option<String>,
    pub action: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenModificationResponse {
    pub psp_reference: String,
    pub status: String,
    pub amount: Option<AdyenAmount>,
    pub merchant_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenWebhookNotification {
    pub live: String,
    pub notification_items: Vec<NotificationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    #[serde(rename = "NotificationRequestItem")]
    pub notification_request_item: NotificationRequestItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRequestItem {
    pub psp_reference: String,
    pub event_code: String,
    pub event_date: String,
    pub merchant_account_code: String,
    pub merchant_reference: String,
    pub amount: AdyenAmount,
    pub success: String,
    pub reason: Option<String>,
}
