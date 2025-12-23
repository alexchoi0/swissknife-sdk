use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "payments")]
use swissknife_payments_sdk as payments;

#[derive(Clone)]
pub struct PaymentsTools {
    #[cfg(feature = "stripe")]
    pub stripe: Option<payments::stripe::StripeClient>,
    #[cfg(feature = "paypal")]
    pub paypal: Option<payments::paypal::PayPalClient>,
    #[cfg(feature = "square")]
    pub square: Option<payments::square::SquareClient>,
}

impl PaymentsTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "stripe")]
            stripe: None,
            #[cfg(feature = "paypal")]
            paypal: None,
            #[cfg(feature = "square")]
            square: None,
        }
    }

    #[cfg(feature = "stripe")]
    pub fn with_stripe(mut self, client: payments::stripe::StripeClient) -> Self {
        self.stripe = Some(client);
        self
    }

    #[cfg(feature = "paypal")]
    pub fn with_paypal(mut self, client: payments::paypal::PayPalClient) -> Self {
        self.paypal = Some(client);
        self
    }

    #[cfg(feature = "square")]
    pub fn with_square(mut self, client: payments::square::SquareClient) -> Self {
        self.square = Some(client);
        self
    }
}

impl Default for PaymentsTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StripeCreateCustomerRequest {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StripeCreatePaymentIntentRequest {
    pub amount: i64,
    pub currency: String,
    pub payment_method_id: String,
    #[serde(default)]
    pub customer_id: Option<String>,
    #[serde(default = "default_true")]
    pub capture: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StripeCapturePaymentRequest {
    pub payment_intent_id: String,
    #[serde(default)]
    pub amount: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StripeRefundRequest {
    pub payment_intent_id: String,
    #[serde(default)]
    pub amount: Option<i64>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StripeGetPaymentIntentRequest {
    pub payment_intent_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PayPalCreateOrderRequest {
    pub amount: String,
    pub currency: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PayPalCaptureOrderRequest {
    pub order_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PayPalRefundRequest {
    pub capture_id: String,
    #[serde(default)]
    pub amount: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SquareCreatePaymentRequest {
    pub amount: i64,
    pub currency: String,
    pub source_id: String,
    #[serde(default)]
    pub customer_id: Option<String>,
}

#[tool_box]
impl PaymentsTools {
    #[cfg(feature = "stripe")]
    #[rmcp::tool(description = "Create a new Stripe customer")]
    pub async fn stripe_create_customer(
        &self,
        #[rmcp::tool(aggr)] req: StripeCreateCustomerRequest,
    ) -> Result<String, String> {
        let client = self.stripe.as_ref()
            .ok_or_else(|| "Stripe client not configured".to_string())?;

        let customer = payments::Customer {
            email: req.email,
            name: req.name,
            phone: req.phone,
        };

        let customer_id = client.create_customer(&customer).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "customer_id": customer_id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "stripe")]
    #[rmcp::tool(description = "Create a Stripe payment intent to charge a customer")]
    pub async fn stripe_create_payment_intent(
        &self,
        #[rmcp::tool(aggr)] req: StripeCreatePaymentIntentRequest,
    ) -> Result<String, String> {
        let client = self.stripe.as_ref()
            .ok_or_else(|| "Stripe client not configured".to_string())?;

        let currency = req.currency.parse()
            .map_err(|_| format!("Invalid currency: {}", req.currency))?;

        let money = payments::Money {
            amount: req.amount,
            currency,
        };

        let intent = client.create_payment_intent(
            money,
            &req.payment_method_id,
            req.customer_id.as_deref(),
            req.capture,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": intent.id,
            "status": intent.status,
            "amount": intent.amount
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "stripe")]
    #[rmcp::tool(description = "Capture a previously authorized Stripe payment")]
    pub async fn stripe_capture_payment(
        &self,
        #[rmcp::tool(aggr)] req: StripeCapturePaymentRequest,
    ) -> Result<String, String> {
        let client = self.stripe.as_ref()
            .ok_or_else(|| "Stripe client not configured".to_string())?;

        let amount = req.amount.map(|a| payments::Money {
            amount: a,
            currency: payments::Currency::Usd,
        });

        let intent = client.capture_payment_intent(&req.payment_intent_id, amount).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": intent.id,
            "status": intent.status
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "stripe")]
    #[rmcp::tool(description = "Create a refund for a Stripe payment")]
    pub async fn stripe_refund(
        &self,
        #[rmcp::tool(aggr)] req: StripeRefundRequest,
    ) -> Result<String, String> {
        let client = self.stripe.as_ref()
            .ok_or_else(|| "Stripe client not configured".to_string())?;

        let amount = req.amount.map(|a| payments::Money {
            amount: a,
            currency: payments::Currency::Usd,
        });

        let refund = client.create_refund(
            &req.payment_intent_id,
            amount,
            req.reason.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": refund.id,
            "status": refund.status,
            "amount": refund.amount
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "stripe")]
    #[rmcp::tool(description = "Get details of a Stripe payment intent")]
    pub async fn stripe_get_payment_intent(
        &self,
        #[rmcp::tool(aggr)] req: StripeGetPaymentIntentRequest,
    ) -> Result<String, String> {
        let client = self.stripe.as_ref()
            .ok_or_else(|| "Stripe client not configured".to_string())?;

        let intent = client.retrieve_payment_intent(&req.payment_intent_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": intent.id,
            "status": intent.status,
            "amount": intent.amount,
            "currency": intent.currency
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "paypal")]
    #[rmcp::tool(description = "Create a PayPal order for payment")]
    pub async fn paypal_create_order(
        &self,
        #[rmcp::tool(aggr)] req: PayPalCreateOrderRequest,
    ) -> Result<String, String> {
        let client = self.paypal.as_ref()
            .ok_or_else(|| "PayPal client not configured".to_string())?;

        let order = client.create_order(&req.amount, &req.currency, req.description.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": order.id,
            "status": order.status
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "paypal")]
    #[rmcp::tool(description = "Capture an approved PayPal order")]
    pub async fn paypal_capture_order(
        &self,
        #[rmcp::tool(aggr)] req: PayPalCaptureOrderRequest,
    ) -> Result<String, String> {
        let client = self.paypal.as_ref()
            .ok_or_else(|| "PayPal client not configured".to_string())?;

        let order = client.capture_order(&req.order_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": order.id,
            "status": order.status
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "paypal")]
    #[rmcp::tool(description = "Refund a PayPal capture")]
    pub async fn paypal_refund(
        &self,
        #[rmcp::tool(aggr)] req: PayPalRefundRequest,
    ) -> Result<String, String> {
        let client = self.paypal.as_ref()
            .ok_or_else(|| "PayPal client not configured".to_string())?;

        let refund = client.refund_capture(&req.capture_id, req.amount.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": refund.id,
            "status": refund.status
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "square")]
    #[rmcp::tool(description = "Create a Square payment")]
    pub async fn square_create_payment(
        &self,
        #[rmcp::tool(aggr)] req: SquareCreatePaymentRequest,
    ) -> Result<String, String> {
        let client = self.square.as_ref()
            .ok_or_else(|| "Square client not configured".to_string())?;

        let payment = client.create_payment(
            req.amount,
            &req.currency,
            &req.source_id,
            req.customer_id.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": payment.id,
            "status": payment.status
        })).map_err(|e| e.to_string())
    }
}
