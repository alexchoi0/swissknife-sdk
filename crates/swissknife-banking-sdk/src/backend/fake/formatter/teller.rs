use super::super::domain::{account, institution, transaction, user};
use super::super::generator::{LinkTokenData, TokenData};
use super::ProviderFormatter;
use serde_json::json;

pub struct TellerFormatter;

impl ProviderFormatter for TellerFormatter {
    fn format_token(&self, token: &TokenData) -> String {
        json!({
            "access_token": token.access_token,
            "token_type": token.token_type
        }).to_string()
    }

    fn format_link_token(&self, token: &LinkTokenData) -> String {
        json!({
            "enrollment_id": format!("enr_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase()),
            "access_token": token.link_token
        }).to_string()
    }

    fn format_accounts(&self, accounts: &[account::Model]) -> String {
        let formatted: Vec<_> = accounts.iter().map(|a| self.account_to_json(a)).collect();
        serde_json::to_string(&formatted).unwrap_or_else(|_| "[]".to_string())
    }

    fn format_account(&self, account: &account::Model) -> String {
        self.account_to_json(account).to_string()
    }

    fn format_balances(&self, account: &account::Model) -> String {
        json!({
            "account_id": account.external_id,
            "available": account.balance_available.map(|b| format!("{:.2}", b)),
            "ledger": account.balance_current.map(|b| format!("{:.2}", b)),
            "links": {
                "account": format!("https://api.teller.io/accounts/{}", account.external_id),
                "self": format!("https://api.teller.io/accounts/{}/balances", account.external_id)
            }
        }).to_string()
    }

    fn format_transactions(&self, transactions: &[transaction::Model]) -> String {
        let formatted: Vec<_> = transactions.iter().map(|t| self.transaction_to_json(t)).collect();
        serde_json::to_string(&formatted).unwrap_or_else(|_| "[]".to_string())
    }

    fn format_transaction(&self, transaction: &transaction::Model) -> String {
        self.transaction_to_json(transaction).to_string()
    }

    fn format_institutions(&self, institutions: &[institution::Model]) -> String {
        let formatted: Vec<_> = institutions.iter().map(|i| self.institution_to_json(i)).collect();
        serde_json::to_string(&formatted).unwrap_or_else(|_| "[]".to_string())
    }

    fn format_institution(&self, institution: &institution::Model) -> String {
        self.institution_to_json(institution).to_string()
    }

    fn format_identity(&self, user: Option<&user::Model>) -> String {
        match user {
            Some(u) => json!({
                "emails": [{
                    "data": u.email,
                    "type": "primary"
                }],
                "names": [{
                    "data": u.name
                }],
                "phone_numbers": [{
                    "data": u.phone,
                    "type": "mobile"
                }],
                "addresses": [{
                    "data": {
                        "street": u.address_street,
                        "city": u.address_city,
                        "state": u.address_state,
                        "postal_code": u.address_postal_code,
                        "country": u.address_country
                    },
                    "type": "primary"
                }]
            }).to_string(),
            None => json!({
                "emails": [],
                "names": [],
                "phone_numbers": [],
                "addresses": []
            }).to_string(),
        }
    }
}

impl TellerFormatter {
    fn account_to_json(&self, account: &account::Model) -> serde_json::Value {
        json!({
            "id": account.external_id,
            "name": account.name,
            "type": "depository",
            "subtype": account.subtype,
            "currency": account.currency,
            "enrollment_id": format!("enr_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase()),
            "institution": {
                "id": account.institution_id,
                "name": "Institution"
            },
            "last_four": account.mask,
            "status": account.status,
            "links": {
                "balances": format!("https://api.teller.io/accounts/{}/balances", account.external_id),
                "transactions": format!("https://api.teller.io/accounts/{}/transactions", account.external_id),
                "details": format!("https://api.teller.io/accounts/{}/details", account.external_id),
                "self": format!("https://api.teller.io/accounts/{}", account.external_id)
            }
        })
    }

    fn transaction_to_json(&self, txn: &transaction::Model) -> serde_json::Value {
        json!({
            "id": txn.external_id,
            "account_id": txn.account_id,
            "date": txn.date,
            "description": txn.description,
            "details": {
                "processing_status": "complete",
                "category": txn.category.clone().unwrap_or_else(|| "other".to_string()).to_lowercase(),
                "counterparty": {
                    "name": txn.merchant_name,
                    "type": if txn.merchant_name.is_some() { "merchant" } else { "organization" }
                }
            },
            "status": txn.status,
            "amount": format!("{:.2}", txn.amount),
            "running_balance": null,
            "type": if txn.amount < 0.0 { "card_payment" } else { "ach" },
            "links": {
                "account": format!("https://api.teller.io/accounts/{}", txn.account_id),
                "self": format!("https://api.teller.io/accounts/{}/transactions/{}", txn.account_id, txn.external_id)
            }
        })
    }

    fn institution_to_json(&self, inst: &institution::Model) -> serde_json::Value {
        json!({
            "id": inst.external_id,
            "name": inst.name
        })
    }
}
