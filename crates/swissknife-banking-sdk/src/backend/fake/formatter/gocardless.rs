use super::super::domain::{account, institution, transaction, user};
use super::super::generator::{LinkTokenData, TokenData};
use super::ProviderFormatter;
use serde_json::json;

pub struct GoCardlessFormatter;

impl ProviderFormatter for GoCardlessFormatter {
    fn format_token(&self, token: &TokenData) -> String {
        json!({
            "access": token.access_token,
            "access_expires": token.expires_in,
            "refresh": token.refresh_token,
            "refresh_expires": 2592000
        }).to_string()
    }

    fn format_link_token(&self, token: &LinkTokenData) -> String {
        let req_id = format!("req_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase());
        json!({
            "id": req_id.clone(),
            "created": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "redirect": "https://example.com/callback",
            "status": "CR",
            "institution_id": "SANDBOXFINANCE_SFIN0000",
            "agreement": null,
            "reference": token.request_id,
            "accounts": [],
            "link": format!("https://ob.gocardless.com/psd2/start/{}", req_id)
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
            "balances": [
                {
                    "balance_amount": {
                        "amount": account.balance_current.map(|b| format!("{:.2}", b)),
                        "currency": account.currency
                    },
                    "balance_type": "closingBooked",
                    "reference_date": chrono::Utc::now().format("%Y-%m-%d").to_string()
                },
                {
                    "balance_amount": {
                        "amount": account.balance_available.map(|b| format!("{:.2}", b)),
                        "currency": account.currency
                    },
                    "balance_type": "interimAvailable",
                    "reference_date": chrono::Utc::now().format("%Y-%m-%d").to_string()
                }
            ]
        }).to_string()
    }

    fn format_transactions(&self, transactions: &[transaction::Model]) -> String {
        let booked: Vec<_> = transactions
            .iter()
            .filter(|t| t.status == "posted")
            .map(|t| self.transaction_to_json(t))
            .collect();
        let pending: Vec<_> = transactions
            .iter()
            .filter(|t| t.status == "pending")
            .map(|t| self.transaction_to_json(t))
            .collect();
        json!({
            "transactions": {
                "booked": booked,
                "pending": pending
            }
        }).to_string()
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
                "account": {
                    "owner_name": u.name
                }
            }).to_string(),
            None => json!({
                "account": {}
            }).to_string(),
        }
    }
}

impl GoCardlessFormatter {
    fn account_to_json(&self, account: &account::Model) -> serde_json::Value {
        json!({
            "id": account.external_id,
            "created": account.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "last_accessed": account.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "iban": account.iban,
            "institution_id": account.institution_id,
            "status": "READY",
            "owner_name": null
        })
    }

    fn transaction_to_json(&self, txn: &transaction::Model) -> serde_json::Value {
        json!({
            "transaction_id": txn.external_id,
            "booking_date": txn.date,
            "value_date": txn.date,
            "transaction_amount": {
                "amount": format!("{:.2}", txn.amount),
                "currency": txn.currency
            },
            "remittance_information_unstructured": txn.description,
            "creditor_name": if txn.amount < 0.0 { txn.merchant_name.clone() } else { None },
            "debtor_name": if txn.amount > 0.0 { Some("Employer".to_string()) } else { None },
            "bank_transaction_code": if txn.amount < 0.0 { "PMNT-ICDT-STDO" } else { "PMNT-RCDT-SALA" }
        })
    }

    fn institution_to_json(&self, inst: &institution::Model) -> serde_json::Value {
        json!({
            "id": inst.external_id,
            "name": inst.name,
            "bic": inst.bic,
            "transaction_total_days": "90",
            "countries": [inst.country],
            "logo": inst.logo_url
        })
    }
}
