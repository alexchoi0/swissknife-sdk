use crate::{Card, Currency, Customer, Error, Money, PaymentProvider, PaymentResult, PaymentStatus, RefundResult, RefundStatus, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.stripe.com/v1";

pub struct StripeClient {
    secret_key: String,
    http: reqwest::Client,
}

impl StripeClient {
    pub fn new(secret_key: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn post<T: for<'de> Deserialize<'de>>(&self, endpoint: &str, form: &[(&str, String)]) -> Result<T> {
        let response = self.http
            .post(format!("{}{}", API_BASE, endpoint))
            .basic_auth(&self.secret_key, None::<&str>)
            .form(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: StripeError = response.json().await?;
            return Err(Error::Provider(error.error.message));
        }

        Ok(response.json().await?)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let response = self.http
            .get(format!("{}{}", API_BASE, endpoint))
            .basic_auth(&self.secret_key, None::<&str>)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: StripeError = response.json().await?;
            return Err(Error::Provider(error.error.message));
        }

        Ok(response.json().await?)
    }

    pub async fn create_payment_method(&self, card: &Card) -> Result<String> {
        let form = vec![
            ("type", "card".to_string()),
            ("card[number]", card.number.clone()),
            ("card[exp_month]", card.exp_month.to_string()),
            ("card[exp_year]", card.exp_year.to_string()),
            ("card[cvc]", card.cvc.clone()),
        ];

        let pm: PaymentMethod = self.post("/payment_methods", &form).await?;
        Ok(pm.id)
    }

    pub async fn create_customer(&self, customer: &Customer) -> Result<String> {
        let mut form = Vec::new();
        if let Some(email) = &customer.email {
            form.push(("email", email.clone()));
        }
        if let Some(name) = &customer.name {
            form.push(("name", name.clone()));
        }
        if let Some(phone) = &customer.phone {
            form.push(("phone", phone.clone()));
        }

        let cust: StripeCustomer = self.post("/customers", &form).await?;
        Ok(cust.id)
    }

    pub async fn attach_payment_method(&self, payment_method_id: &str, customer_id: &str) -> Result<()> {
        let form = vec![("customer", customer_id.to_string())];
        let _: PaymentMethod = self.post(&format!("/payment_methods/{}/attach", payment_method_id), &form).await?;
        Ok(())
    }

    pub async fn create_payment_intent(&self, amount: Money, payment_method_id: &str, customer_id: Option<&str>, capture: bool) -> Result<PaymentIntent> {
        let mut form = vec![
            ("amount", amount.amount.to_string()),
            ("currency", amount.currency.as_str().to_string()),
            ("payment_method", payment_method_id.to_string()),
            ("confirm", "true".to_string()),
            ("capture_method", if capture { "automatic" } else { "manual" }.to_string()),
        ];

        if let Some(cid) = customer_id {
            form.push(("customer", cid.to_string()));
        }

        self.post("/payment_intents", &form).await
    }

    pub async fn capture_payment_intent(&self, payment_intent_id: &str, amount: Option<Money>) -> Result<PaymentIntent> {
        let mut form = Vec::new();
        if let Some(amt) = amount {
            form.push(("amount_to_capture", amt.amount.to_string()));
        }

        self.post(&format!("/payment_intents/{}/capture", payment_intent_id), &form).await
    }

    pub async fn cancel_payment_intent(&self, payment_intent_id: &str) -> Result<PaymentIntent> {
        self.post(&format!("/payment_intents/{}/cancel", payment_intent_id), &[]).await
    }

    pub async fn retrieve_payment_intent(&self, payment_intent_id: &str) -> Result<PaymentIntent> {
        self.get(&format!("/payment_intents/{}", payment_intent_id)).await
    }

    pub async fn create_refund(&self, payment_intent_id: &str, amount: Option<Money>, reason: Option<&str>) -> Result<Refund> {
        let mut form = vec![("payment_intent", payment_intent_id.to_string())];

        if let Some(amt) = amount {
            form.push(("amount", amt.amount.to_string()));
        }
        if let Some(r) = reason {
            form.push(("reason", r.to_string()));
        }

        self.post("/refunds", &form).await
    }

    pub fn verify_webhook_signature(&self, payload: &str, signature: &str, webhook_secret: &str) -> Result<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let parts: Vec<&str> = signature.split(',').collect();
        let mut timestamp = None;
        let mut sig = None;

        for part in parts {
            if let Some(t) = part.strip_prefix("t=") {
                timestamp = Some(t);
            } else if let Some(s) = part.strip_prefix("v1=") {
                sig = Some(s);
            }
        }

        let timestamp = timestamp.ok_or_else(|| Error::WebhookVerification("Missing timestamp".into()))?;
        let sig = sig.ok_or_else(|| Error::WebhookVerification("Missing signature".into()))?;

        let signed_payload = format!("{}.{}", timestamp, payload);
        let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes())
            .map_err(|_| Error::WebhookVerification("Invalid secret".into()))?;
        mac.update(signed_payload.as_bytes());

        let expected = hex::encode(mac.finalize().into_bytes());
        Ok(expected == sig)
    }
}

#[async_trait]
impl PaymentProvider for StripeClient {
    async fn charge(&self, amount: Money, card: &Card, customer: Option<&Customer>) -> Result<PaymentResult> {
        let pm_id = self.create_payment_method(card).await?;

        let customer_id = if let Some(cust) = customer {
            let cid = self.create_customer(cust).await?;
            self.attach_payment_method(&pm_id, &cid).await?;
            Some(cid)
        } else {
            None
        };

        let pi = self.create_payment_intent(amount.clone(), &pm_id, customer_id.as_deref(), true).await?;
        Ok(pi.into_result(amount, customer_id))
    }

    async fn authorize(&self, amount: Money, card: &Card, customer: Option<&Customer>) -> Result<PaymentResult> {
        let pm_id = self.create_payment_method(card).await?;

        let customer_id = if let Some(cust) = customer {
            let cid = self.create_customer(cust).await?;
            self.attach_payment_method(&pm_id, &cid).await?;
            Some(cid)
        } else {
            None
        };

        let pi = self.create_payment_intent(amount.clone(), &pm_id, customer_id.as_deref(), false).await?;
        Ok(pi.into_result(amount, customer_id))
    }

    async fn capture(&self, payment_id: &str, amount: Option<Money>) -> Result<PaymentResult> {
        let pi = self.capture_payment_intent(payment_id, amount.clone()).await?;
        let amt = amount.unwrap_or_else(|| Money::new(pi.amount, Currency::USD));
        Ok(pi.into_result(amt, None))
    }

    async fn void(&self, payment_id: &str) -> Result<PaymentResult> {
        let pi = self.cancel_payment_intent(payment_id).await?;
        let amount = Money::new(pi.amount, Currency::USD);
        Ok(pi.into_result(amount, None))
    }

    async fn refund(&self, payment_id: &str, amount: Option<Money>, reason: Option<&str>) -> Result<RefundResult> {
        let refund = self.create_refund(payment_id, amount.clone(), reason).await?;
        Ok(RefundResult {
            id: refund.id,
            payment_id: payment_id.to_string(),
            amount: amount.unwrap_or_else(|| Money::new(refund.amount, Currency::USD)),
            status: match refund.status.as_str() {
                "succeeded" => RefundStatus::Succeeded,
                "pending" => RefundStatus::Pending,
                "failed" => RefundStatus::Failed,
                "canceled" => RefundStatus::Canceled,
                _ => RefundStatus::Pending,
            },
            reason: reason.map(String::from),
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_payment(&self, payment_id: &str) -> Result<PaymentResult> {
        let pi = self.retrieve_payment_intent(payment_id).await?;
        let amount = Money::new(pi.amount, Currency::USD);
        Ok(pi.into_result(amount, None))
    }
}

#[derive(Debug, Deserialize)]
struct StripeError {
    error: StripeErrorDetail,
}

#[derive(Debug, Deserialize)]
struct StripeErrorDetail {
    message: String,
}

#[derive(Debug, Deserialize)]
struct PaymentMethod {
    id: String,
}

#[derive(Debug, Deserialize)]
struct StripeCustomer {
    id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub client_secret: Option<String>,
    pub payment_method: Option<String>,
    pub created: i64,
}

impl PaymentIntent {
    fn into_result(self, amount: Money, customer_id: Option<String>) -> PaymentResult {
        PaymentResult {
            id: self.id,
            status: match self.status.as_str() {
                "succeeded" => PaymentStatus::Succeeded,
                "processing" => PaymentStatus::Processing,
                "requires_payment_method" | "requires_confirmation" => PaymentStatus::Pending,
                "requires_action" => PaymentStatus::RequiresAction,
                "requires_capture" => PaymentStatus::RequiresCapture,
                "canceled" => PaymentStatus::Canceled,
                _ => PaymentStatus::Failed,
            },
            amount,
            customer_id,
            payment_method_id: self.payment_method,
            error_message: None,
            created_at: chrono::DateTime::from_timestamp(self.created, 0).unwrap_or_else(chrono::Utc::now),
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Refund {
    pub id: String,
    pub amount: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
    pub created: i64,
}
