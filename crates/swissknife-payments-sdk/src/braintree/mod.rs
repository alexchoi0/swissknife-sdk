use crate::{Card, Currency, Customer, Error, Money, PaymentProvider, PaymentResult, PaymentStatus, RefundResult, RefundStatus, Result};
use async_trait::async_trait;
use serde::Deserialize;

const SANDBOX_API: &str = "https://payments.sandbox.braintree-api.com/graphql";
const LIVE_API: &str = "https://payments.braintree-api.com/graphql";

pub struct BraintreeClient {
    merchant_id: String,
    public_key: String,
    private_key: String,
    http: reqwest::Client,
    sandbox: bool,
}

impl BraintreeClient {
    pub fn new(merchant_id: impl Into<String>, public_key: impl Into<String>, private_key: impl Into<String>) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            public_key: public_key.into(),
            private_key: private_key.into(),
            http: reqwest::Client::new(),
            sandbox: false,
        }
    }

    pub fn sandbox(merchant_id: impl Into<String>, public_key: impl Into<String>, private_key: impl Into<String>) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            public_key: public_key.into(),
            private_key: private_key.into(),
            http: reqwest::Client::new(),
            sandbox: true,
        }
    }

    fn base_url(&self) -> &str {
        if self.sandbox { SANDBOX_API } else { LIVE_API }
    }

    async fn graphql<T: for<'de> Deserialize<'de>>(&self, query: &str, variables: serde_json::Value) -> Result<T> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });

        let response = self.http
            .post(self.base_url())
            .basic_auth(&self.public_key, Some(&self.private_key))
            .header("Braintree-Version", "2024-01-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("Braintree API error: {}", text)));
        }

        let result: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = result.errors {
            if let Some(error) = errors.first() {
                return Err(Error::Provider(error.message.clone()));
            }
        }

        result.data.ok_or_else(|| Error::Provider("No data returned".into()))
    }

    pub async fn tokenize_card(&self, card: &Card) -> Result<String> {
        let query = r#"
            mutation TokenizeCreditCard($input: TokenizeCreditCardInput!) {
                tokenizeCreditCard(input: $input) {
                    paymentMethod {
                        id
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "input": {
                "creditCard": {
                    "number": card.number,
                    "expirationMonth": format!("{:02}", card.exp_month),
                    "expirationYear": card.exp_year.to_string(),
                    "cvv": card.cvc
                }
            }
        });

        let response: TokenizeCreditCardResponse = self.graphql(query, variables).await?;
        Ok(response.tokenize_credit_card.payment_method.id)
    }

    pub async fn charge_payment_method(&self, payment_method_id: &str, amount: Money, submit_for_settlement: bool) -> Result<BraintreeTransaction> {
        let query = r#"
            mutation ChargePaymentMethod($input: ChargePaymentMethodInput!) {
                chargePaymentMethod(input: $input) {
                    transaction {
                        id
                        status
                        amount {
                            value
                            currencyCode
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "input": {
                "paymentMethodId": payment_method_id,
                "transaction": {
                    "amount": format_amount(amount.amount, amount.currency),
                    "merchantAccountId": self.merchant_id
                },
                "options": {
                    "submitForSettlement": submit_for_settlement
                }
            }
        });

        let response: ChargePaymentMethodResponse = self.graphql(query, variables).await?;
        Ok(response.charge_payment_method.transaction)
    }

    pub async fn capture_transaction(&self, transaction_id: &str, amount: Option<Money>) -> Result<BraintreeTransaction> {
        let query = r#"
            mutation CaptureTransaction($input: CaptureTransactionInput!) {
                captureTransaction(input: $input) {
                    transaction {
                        id
                        status
                        amount {
                            value
                            currencyCode
                        }
                    }
                }
            }
        "#;

        let variables = if let Some(amt) = amount {
            serde_json::json!({
                "input": {
                    "transactionId": transaction_id,
                    "transaction": {
                        "amount": format_amount(amt.amount, amt.currency)
                    }
                }
            })
        } else {
            serde_json::json!({
                "input": {
                    "transactionId": transaction_id
                }
            })
        };

        let response: CaptureTransactionResponse = self.graphql(query, variables).await?;
        Ok(response.capture_transaction.transaction)
    }

    pub async fn void_transaction(&self, transaction_id: &str) -> Result<BraintreeTransaction> {
        let query = r#"
            mutation VoidTransaction($input: VoidTransactionInput!) {
                voidTransaction(input: $input) {
                    transaction {
                        id
                        status
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "input": {
                "transactionId": transaction_id
            }
        });

        let response: VoidTransactionResponse = self.graphql(query, variables).await?;
        Ok(response.void_transaction.transaction)
    }

    pub async fn refund_transaction(&self, transaction_id: &str, amount: Option<Money>) -> Result<BraintreeRefund> {
        let query = r#"
            mutation RefundTransaction($input: RefundTransactionInput!) {
                refundTransaction(input: $input) {
                    refund {
                        id
                        status
                        amount {
                            value
                            currencyCode
                        }
                    }
                }
            }
        "#;

        let variables = if let Some(amt) = amount {
            serde_json::json!({
                "input": {
                    "transactionId": transaction_id,
                    "refund": {
                        "amount": format_amount(amt.amount, amt.currency)
                    }
                }
            })
        } else {
            serde_json::json!({
                "input": {
                    "transactionId": transaction_id
                }
            })
        };

        let response: RefundTransactionResponse = self.graphql(query, variables).await?;
        Ok(response.refund_transaction.refund)
    }

    pub async fn get_transaction(&self, transaction_id: &str) -> Result<BraintreeTransaction> {
        let query = r#"
            query GetTransaction($transactionId: ID!) {
                node(id: $transactionId) {
                    ... on Transaction {
                        id
                        status
                        amount {
                            value
                            currencyCode
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "transactionId": transaction_id
        });

        let response: GetTransactionResponse = self.graphql(query, variables).await?;
        Ok(response.node)
    }

    pub async fn create_customer(&self, customer: &Customer) -> Result<BraintreeCustomer> {
        let query = r#"
            mutation CreateCustomer($input: CreateCustomerInput!) {
                createCustomer(input: $input) {
                    customer {
                        id
                        email
                        firstName
                        lastName
                    }
                }
            }
        "#;

        let (first_name, last_name) = customer.name.as_ref()
            .map(|n| {
                let parts: Vec<&str> = n.splitn(2, ' ').collect();
                (parts.first().map(|s| s.to_string()), parts.get(1).map(|s| s.to_string()))
            })
            .unwrap_or((None, None));

        let variables = serde_json::json!({
            "input": {
                "customer": {
                    "email": customer.email,
                    "firstName": first_name,
                    "lastName": last_name
                }
            }
        });

        let response: CreateCustomerResponse = self.graphql(query, variables).await?;
        Ok(response.create_customer.customer)
    }

    pub fn verify_webhook_signature(&self, payload: &str, signature: &str, public_key: &str) -> Result<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(public_key.as_bytes())
            .map_err(|_| Error::WebhookVerification("Invalid public key".into()))?;
        mac.update(payload.as_bytes());

        let expected = hex::encode(mac.finalize().into_bytes());
        Ok(expected == signature)
    }
}

fn format_amount(cents: i64, currency: Currency) -> String {
    if currency.zero_decimal() {
        cents.to_string()
    } else {
        format!("{}.{:02}", cents / 100, cents.abs() % 100)
    }
}

#[async_trait]
impl PaymentProvider for BraintreeClient {
    async fn charge(&self, amount: Money, card: &Card, _customer: Option<&Customer>) -> Result<PaymentResult> {
        let payment_method_id = self.tokenize_card(card).await?;
        let transaction = self.charge_payment_method(&payment_method_id, amount.clone(), true).await?;
        Ok(transaction_to_result(transaction, amount))
    }

    async fn authorize(&self, amount: Money, card: &Card, _customer: Option<&Customer>) -> Result<PaymentResult> {
        let payment_method_id = self.tokenize_card(card).await?;
        let transaction = self.charge_payment_method(&payment_method_id, amount.clone(), false).await?;
        Ok(transaction_to_result(transaction, amount))
    }

    async fn capture(&self, payment_id: &str, amount: Option<Money>) -> Result<PaymentResult> {
        let transaction = self.capture_transaction(payment_id, amount.clone()).await?;
        let amt = amount.unwrap_or_else(|| {
            transaction.amount.as_ref()
                .map(|a| parse_amount(&a.value, &a.currency_code))
                .unwrap_or_else(|| Money::new(0, Currency::USD))
        });
        Ok(transaction_to_result(transaction, amt))
    }

    async fn void(&self, payment_id: &str) -> Result<PaymentResult> {
        let transaction = self.void_transaction(payment_id).await?;
        Ok(transaction_to_result(transaction, Money::new(0, Currency::USD)))
    }

    async fn refund(&self, payment_id: &str, amount: Option<Money>, reason: Option<&str>) -> Result<RefundResult> {
        let refund = self.refund_transaction(payment_id, amount.clone()).await?;
        let amt = amount.unwrap_or_else(|| {
            refund.amount.as_ref()
                .map(|a| parse_amount(&a.value, &a.currency_code))
                .unwrap_or_else(|| Money::new(0, Currency::USD))
        });

        Ok(RefundResult {
            id: refund.id,
            payment_id: payment_id.to_string(),
            amount: amt,
            status: match refund.status.as_str() {
                "SETTLED" | "SETTLING" => RefundStatus::Succeeded,
                "SUBMITTED_FOR_SETTLEMENT" => RefundStatus::Pending,
                "VOIDED" => RefundStatus::Canceled,
                _ => RefundStatus::Failed,
            },
            reason: reason.map(String::from),
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_payment(&self, payment_id: &str) -> Result<PaymentResult> {
        let transaction = self.get_transaction(payment_id).await?;
        let amount = transaction.amount.as_ref()
            .map(|a| parse_amount(&a.value, &a.currency_code))
            .unwrap_or_else(|| Money::new(0, Currency::USD));
        Ok(transaction_to_result(transaction, amount))
    }
}

fn parse_amount(value: &str, _currency_code: &str) -> Money {
    let cents = value.parse::<f64>().unwrap_or(0.0) * 100.0;
    Money::new(cents as i64, Currency::USD)
}

fn transaction_to_result(transaction: BraintreeTransaction, amount: Money) -> PaymentResult {
    PaymentResult {
        id: transaction.id,
        status: match transaction.status.as_str() {
            "SETTLED" | "SETTLING" | "SUBMITTED_FOR_SETTLEMENT" => PaymentStatus::Succeeded,
            "AUTHORIZED" => PaymentStatus::RequiresCapture,
            "AUTHORIZING" | "SETTLEMENT_PENDING" => PaymentStatus::Processing,
            "VOIDED" => PaymentStatus::Canceled,
            "FAILED" | "GATEWAY_REJECTED" | "PROCESSOR_DECLINED" => PaymentStatus::Failed,
            _ => PaymentStatus::Pending,
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
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenizeCreditCardResponse {
    tokenize_credit_card: TokenizeCreditCardPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenizeCreditCardPayload {
    payment_method: PaymentMethodPayload,
}

#[derive(Debug, Deserialize)]
struct PaymentMethodPayload {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChargePaymentMethodResponse {
    charge_payment_method: ChargePaymentMethodPayload,
}

#[derive(Debug, Deserialize)]
struct ChargePaymentMethodPayload {
    transaction: BraintreeTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptureTransactionResponse {
    capture_transaction: CaptureTransactionPayload,
}

#[derive(Debug, Deserialize)]
struct CaptureTransactionPayload {
    transaction: BraintreeTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VoidTransactionResponse {
    void_transaction: VoidTransactionPayload,
}

#[derive(Debug, Deserialize)]
struct VoidTransactionPayload {
    transaction: BraintreeTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefundTransactionResponse {
    refund_transaction: RefundTransactionPayload,
}

#[derive(Debug, Deserialize)]
struct RefundTransactionPayload {
    refund: BraintreeRefund,
}

#[derive(Debug, Deserialize)]
struct GetTransactionResponse {
    node: BraintreeTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCustomerResponse {
    create_customer: CreateCustomerPayload,
}

#[derive(Debug, Deserialize)]
struct CreateCustomerPayload {
    customer: BraintreeCustomer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BraintreeTransaction {
    pub id: String,
    pub status: String,
    pub amount: Option<BraintreeAmount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BraintreeAmount {
    pub value: String,
    pub currency_code: String,
}

#[derive(Debug, Deserialize)]
pub struct BraintreeRefund {
    pub id: String,
    pub status: String,
    pub amount: Option<BraintreeAmount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BraintreeCustomer {
    pub id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
