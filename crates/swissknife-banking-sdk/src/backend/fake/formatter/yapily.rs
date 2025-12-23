use super::super::domain::{account, institution, transaction, user};
use super::super::generator::{LinkTokenData, TokenData};
use super::ProviderFormatter;
use serde_json::json;

pub struct YapilyFormatter;

impl ProviderFormatter for YapilyFormatter {
    fn format_token(&self, token: &TokenData) -> String {
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": {
                "consentToken": token.access_token,
                "status": "AUTHORIZED"
            }
        }).to_string()
    }

    fn format_link_token(&self, token: &LinkTokenData) -> String {
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": {
                "id": format!("auth_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase()),
                "userUuid": uuid::Uuid::new_v4().to_string(),
                "applicationUserId": token.request_id,
                "institutionId": "modelo-sandbox",
                "status": "AWAITING_AUTHORIZATION",
                "createdAt": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                "featureScope": ["ACCOUNTS", "TRANSACTIONS"],
                "authorisationUrl": format!("https://ob.yapily.com/authorize?token={}", token.link_token),
                "qrCodeUrl": null
            }
        }).to_string()
    }

    fn format_accounts(&self, accounts: &[account::Model]) -> String {
        let formatted: Vec<_> = accounts.iter().map(|a| self.account_to_json(a)).collect();
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": formatted
        }).to_string()
    }

    fn format_account(&self, account: &account::Model) -> String {
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": self.account_to_json(account)
        }).to_string()
    }

    fn format_balances(&self, account: &account::Model) -> String {
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": {
                "accountBalances": [
                    {
                        "type": "CLOSING_AVAILABLE",
                        "dateTime": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        "balanceAmount": {
                            "amount": account.balance_available,
                            "currency": account.currency
                        },
                        "creditLineIncluded": false
                    },
                    {
                        "type": "INTERIM_BOOKED",
                        "dateTime": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        "balanceAmount": {
                            "amount": account.balance_current,
                            "currency": account.currency
                        },
                        "creditLineIncluded": false
                    }
                ]
            }
        }).to_string()
    }

    fn format_transactions(&self, transactions: &[transaction::Model]) -> String {
        let formatted: Vec<_> = transactions.iter().map(|t| self.transaction_to_json(t)).collect();
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": formatted
        }).to_string()
    }

    fn format_transaction(&self, transaction: &transaction::Model) -> String {
        self.transaction_to_json(transaction).to_string()
    }

    fn format_institutions(&self, institutions: &[institution::Model]) -> String {
        let formatted: Vec<_> = institutions.iter().map(|i| self.institution_to_json(i)).collect();
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": formatted
        }).to_string()
    }

    fn format_institution(&self, institution: &institution::Model) -> String {
        json!({
            "meta": {
                "tracingId": uuid::Uuid::new_v4().to_string()
            },
            "data": self.institution_to_json(institution)
        }).to_string()
    }

    fn format_identity(&self, user: Option<&user::Model>) -> String {
        match user {
            Some(u) => json!({
                "meta": {
                    "tracingId": uuid::Uuid::new_v4().to_string()
                },
                "data": {
                    "id": u.external_id,
                    "fullName": u.name,
                    "firstName": u.name.split_whitespace().next().unwrap_or(&u.name),
                    "lastName": u.name.split_whitespace().last().unwrap_or(""),
                    "dateOfBirth": u.date_of_birth,
                    "addresses": [{
                        "addressLines": [u.address_street.clone()],
                        "city": u.address_city,
                        "postCode": u.address_postal_code,
                        "country": u.address_country,
                        "addressType": "HOME"
                    }],
                    "emails": [u.email],
                    "phones": [u.phone]
                }
            }).to_string(),
            None => json!({
                "meta": {
                    "tracingId": uuid::Uuid::new_v4().to_string()
                },
                "data": null
            }).to_string(),
        }
    }
}

impl YapilyFormatter {
    fn account_to_json(&self, account: &account::Model) -> serde_json::Value {
        json!({
            "id": account.external_id,
            "type": "PERSONAL",
            "description": account.name,
            "balance": account.balance_current,
            "currency": account.currency,
            "usageType": "PERSONAL",
            "accountType": account.account_type.to_uppercase(),
            "nickname": account.name,
            "accountNames": [{
                "name": "Account Holder"
            }],
            "accountIdentifications": [
                {"type": "ACCOUNT_NUMBER", "identification": account.account_number},
                {"type": "IBAN", "identification": account.iban}
            ],
            "accountBalances": [{
                "type": "CLOSING_AVAILABLE",
                "dateTime": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                "balanceAmount": {
                    "amount": account.balance_available,
                    "currency": account.currency
                },
                "creditLineIncluded": false
            }]
        })
    }

    fn transaction_to_json(&self, txn: &transaction::Model) -> serde_json::Value {
        json!({
            "id": txn.external_id,
            "date": txn.date,
            "bookingDateTime": txn.posted_at.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            "valueDateTime": txn.posted_at.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            "status": "BOOKED",
            "amount": txn.amount,
            "currency": txn.currency,
            "transactionAmount": {
                "amount": txn.amount,
                "currency": txn.currency
            },
            "reference": txn.description,
            "description": txn.merchant_name.clone().unwrap_or_else(|| txn.description.clone()),
            "transactionInformation": [txn.description.clone()],
            "proprietaryBankTransactionCode": {
                "code": if txn.amount < 0.0 { "CARD" } else { "TRANSFER" },
                "issuer": "BANK"
            }
        })
    }

    fn institution_to_json(&self, inst: &institution::Model) -> serde_json::Value {
        json!({
            "id": inst.external_id,
            "name": inst.name,
            "fullName": format!("{} Bank", inst.name),
            "countries": [{
                "displayName": inst.country.clone(),
                "countryCode2": inst.country
            }],
            "environmentType": "SANDBOX",
            "credentialsType": "OPEN_BANKING_UK_AUTO",
            "media": [{
                "source": inst.logo_url,
                "type": "icon"
            }],
            "features": ["ACCOUNT_STATEMENT", "ACCOUNTS", "IDENTITY", "TRANSACTIONS"]
        })
    }
}
