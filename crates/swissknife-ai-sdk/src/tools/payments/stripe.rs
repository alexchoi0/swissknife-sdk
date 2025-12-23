use crate::error::{Error, Result};
use crate::tool::{get_f64_param, get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_payments_sdk::stripe::StripeClient;
use swissknife_payments_sdk::{Currency, Customer, Money, PaymentProvider};

pub struct StripeCreateCustomerTool;

impl Default for StripeCreateCustomerTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for StripeCreateCustomerTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "stripe_create_customer",
            "Stripe Create Customer",
            "Create a new customer in Stripe",
            "payments",
        )
        .with_param("api_key", ParameterSchema::string("Stripe secret API key").required().user_only())
        .with_param("email", ParameterSchema::string("Customer email address"))
        .with_param("name", ParameterSchema::string("Customer full name"))
        .with_param("phone", ParameterSchema::string("Customer phone number"))
        .with_output("customer_id", OutputSchema::string("The created customer ID"))
        .with_output("success", OutputSchema::boolean("Whether the operation succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let email = get_string_param(&params, "email");
        let name = get_string_param(&params, "name");
        let phone = get_string_param(&params, "phone");

        let client = StripeClient::new(api_key);

        let mut customer = Customer::new();
        if let Some(e) = email {
            customer = customer.with_email(e);
        }
        if let Some(n) = name {
            customer = customer.with_name(n);
        }
        if let Some(p) = phone {
            customer = customer.with_phone(p);
        }

        match client.create_customer(&customer).await {
            Ok(customer_id) => Ok(ToolResponse::success(serde_json::json!({
                "customer_id": customer_id,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to create customer: {}", e))),
        }
    }
}

pub struct StripeGetCustomerTool;

impl Default for StripeGetCustomerTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for StripeGetCustomerTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "stripe_get_customer",
            "Stripe Get Customer",
            "Retrieve a customer from Stripe by ID",
            "payments",
        )
        .with_param("api_key", ParameterSchema::string("Stripe secret API key").required().user_only())
        .with_param("customer_id", ParameterSchema::string("The customer ID to retrieve").required())
        .with_output("customer", OutputSchema::json("The customer object"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let _customer_id = get_required_string_param(&params, "customer_id")?;

        let _client = StripeClient::new(api_key);

        Ok(ToolResponse::error("Get customer not implemented in base SDK - use payment intents"))
    }
}

pub struct StripeChargeTool;

impl Default for StripeChargeTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for StripeChargeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "stripe_charge",
            "Stripe Charge",
            "Create a charge/payment in Stripe",
            "payments",
        )
        .with_param("api_key", ParameterSchema::string("Stripe secret API key").required().user_only())
        .with_param("amount", ParameterSchema::integer("Amount in cents").required())
        .with_param("currency", ParameterSchema::string("Currency code (usd, eur, etc)").with_default(serde_json::json!("usd")))
        .with_param("card_number", ParameterSchema::string("Card number").required())
        .with_param("card_exp_month", ParameterSchema::integer("Card expiration month").required())
        .with_param("card_exp_year", ParameterSchema::integer("Card expiration year").required())
        .with_param("card_cvc", ParameterSchema::string("Card CVC").required())
        .with_param("customer_email", ParameterSchema::string("Customer email"))
        .with_param("customer_name", ParameterSchema::string("Customer name"))
        .with_output("payment_id", OutputSchema::string("The payment/charge ID"))
        .with_output("status", OutputSchema::string("Payment status"))
        .with_output("success", OutputSchema::boolean("Whether the charge succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let amount = get_i64_param(&params, "amount").ok_or_else(|| Error::MissingParameter("amount".into()))?;
        let currency_str = get_string_param(&params, "currency").unwrap_or_else(|| "usd".into());
        let card_number = get_required_string_param(&params, "card_number")?;
        let card_exp_month = get_i64_param(&params, "card_exp_month").ok_or_else(|| Error::MissingParameter("card_exp_month".into()))? as u8;
        let card_exp_year = get_i64_param(&params, "card_exp_year").ok_or_else(|| Error::MissingParameter("card_exp_year".into()))? as u16;
        let card_cvc = get_required_string_param(&params, "card_cvc")?;
        let customer_email = get_string_param(&params, "customer_email");
        let customer_name = get_string_param(&params, "customer_name");

        let currency = match currency_str.to_lowercase().as_str() {
            "usd" => Currency::USD,
            "eur" => Currency::EUR,
            "gbp" => Currency::GBP,
            "cad" => Currency::CAD,
            "aud" => Currency::AUD,
            "jpy" => Currency::JPY,
            _ => Currency::USD,
        };

        let client = StripeClient::new(api_key);
        let money = Money::new(amount, currency);
        let card = swissknife_payments_sdk::Card::new(card_number, card_exp_month, card_exp_year, card_cvc);

        let customer = if customer_email.is_some() || customer_name.is_some() {
            let mut c = Customer::new();
            if let Some(e) = customer_email {
                c = c.with_email(e);
            }
            if let Some(n) = customer_name {
                c = c.with_name(n);
            }
            Some(c)
        } else {
            None
        };

        match client.charge(money, &card, customer.as_ref()).await {
            Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                "payment_id": result.id,
                "status": format!("{:?}", result.status),
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Charge failed: {}", e))),
        }
    }
}

pub struct StripeRefundTool;

impl Default for StripeRefundTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for StripeRefundTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "stripe_refund",
            "Stripe Refund",
            "Refund a Stripe payment",
            "payments",
        )
        .with_param("api_key", ParameterSchema::string("Stripe secret API key").required().user_only())
        .with_param("payment_id", ParameterSchema::string("The payment ID to refund").required())
        .with_param("amount", ParameterSchema::integer("Amount to refund in cents (partial refund). If not specified, full refund."))
        .with_param("reason", ParameterSchema::string("Reason for refund"))
        .with_output("refund_id", OutputSchema::string("The refund ID"))
        .with_output("status", OutputSchema::string("Refund status"))
        .with_output("success", OutputSchema::boolean("Whether the refund succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let payment_id = get_required_string_param(&params, "payment_id")?;
        let amount = get_i64_param(&params, "amount");
        let reason = get_string_param(&params, "reason");

        let client = StripeClient::new(api_key);

        let money = amount.map(|a| Money::usd(a));

        match client.refund(&payment_id, money, reason.as_deref()).await {
            Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                "refund_id": result.id,
                "status": format!("{:?}", result.status),
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Refund failed: {}", e))),
        }
    }
}

pub struct StripeGetPaymentTool;

impl Default for StripeGetPaymentTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for StripeGetPaymentTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "stripe_get_payment",
            "Stripe Get Payment",
            "Retrieve a payment/payment intent from Stripe",
            "payments",
        )
        .with_param("api_key", ParameterSchema::string("Stripe secret API key").required().user_only())
        .with_param("payment_id", ParameterSchema::string("The payment ID to retrieve").required())
        .with_output("payment", OutputSchema::json("The payment object"))
        .with_output("status", OutputSchema::string("Payment status"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let payment_id = get_required_string_param(&params, "payment_id")?;

        let client = StripeClient::new(api_key);

        match client.get_payment(&payment_id).await {
            Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                "payment": {
                    "id": result.id,
                    "status": format!("{:?}", result.status),
                    "amount": result.amount.amount,
                    "currency": result.amount.currency.as_str(),
                    "customer_id": result.customer_id,
                    "created_at": result.created_at.to_rfc3339()
                },
                "status": format!("{:?}", result.status)
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Get payment failed: {}", e))),
        }
    }
}
