use super::super::domain::{account, institution, transaction, user};
use super::super::generator::{LinkTokenData, TokenData};
use super::ProviderFormatter;
use serde_json::json;

pub struct TrueLayerFormatter;

impl ProviderFormatter for TrueLayerFormatter {
    fn format_token(&self, token: &TokenData) -> String {
        json!({
            "access_token": token.access_token,
            "token_type": token.token_type,
            "expires_in": token.expires_in
        }).to_string()
    }

    fn format_link_token(&self, token: &LinkTokenData) -> String {
        json!({
            "authorization_url": format!("https://auth.truelayer.com/?token={}", token.link_token),
            "token": token.link_token
        }).to_string()
    }

    fn format_accounts(&self, accounts: &[account::Model]) -> String {
        let formatted: Vec<_> = accounts.iter().map(|a| self.account_to_json(a)).collect();
        json!({
            "results": formatted,
            "status": "Succeeded"
        }).to_string()
    }

    fn format_account(&self, account: &account::Model) -> String {
        json!({
            "results": [self.account_to_json(account)],
            "status": "Succeeded"
        }).to_string()
    }

    fn format_balances(&self, account: &account::Model) -> String {
        json!({
            "results": [{
                "currency": account.currency,
                "available": account.balance_available,
                "current": account.balance_current,
                "overdraft": 0.0,
                "update_timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
            }],
            "status": "Succeeded"
        }).to_string()
    }

    fn format_transactions(&self, transactions: &[transaction::Model]) -> String {
        let formatted: Vec<_> = transactions.iter().map(|t| self.transaction_to_json(t)).collect();
        json!({
            "results": formatted,
            "status": "Succeeded"
        }).to_string()
    }

    fn format_transaction(&self, transaction: &transaction::Model) -> String {
        self.transaction_to_json(transaction).to_string()
    }

    fn format_institutions(&self, institutions: &[institution::Model]) -> String {
        let formatted: Vec<_> = institutions.iter().map(|i| self.institution_to_json(i)).collect();
        json!({
            "results": formatted,
            "status": "Succeeded"
        }).to_string()
    }

    fn format_institution(&self, institution: &institution::Model) -> String {
        json!({
            "results": [self.institution_to_json(institution)],
            "status": "Succeeded"
        }).to_string()
    }

    fn format_identity(&self, user: Option<&user::Model>) -> String {
        match user {
            Some(u) => json!({
                "results": [{
                    "full_name": u.name,
                    "emails": [u.email],
                    "phones": [u.phone],
                    "addresses": [{
                        "address": u.address_street,
                        "city": u.address_city,
                        "state": u.address_state,
                        "zip": u.address_postal_code,
                        "country": u.address_country
                    }],
                    "date_of_birth": u.date_of_birth
                }],
                "status": "Succeeded"
            }).to_string(),
            None => json!({
                "results": [],
                "status": "Succeeded"
            }).to_string(),
        }
    }
}

impl TrueLayerFormatter {
    fn account_to_json(&self, account: &account::Model) -> serde_json::Value {
        json!({
            "account_id": account.external_id,
            "account_type": account.account_type.to_uppercase(),
            "display_name": account.name,
            "currency": account.currency,
            "account_number": {
                "iban": account.iban,
                "swift_bic": null,
                "number": account.account_number,
                "sort_code": account.sort_code
            },
            "provider": {
                "provider_id": account.institution_id,
                "display_name": "Provider",
                "logo_uri": null
            }
        })
    }

    fn transaction_to_json(&self, txn: &transaction::Model) -> serde_json::Value {
        json!({
            "transaction_id": txn.external_id,
            "timestamp": txn.posted_at.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            "description": txn.description,
            "amount": txn.amount,
            "currency": txn.currency,
            "transaction_type": if txn.amount < 0.0 { "DEBIT" } else { "CREDIT" },
            "transaction_category": txn.category.clone().unwrap_or_else(|| "PURCHASE".to_string()).to_uppercase(),
            "merchant_name": txn.merchant_name,
            "running_balance": null
        })
    }

    fn institution_to_json(&self, inst: &institution::Model) -> serde_json::Value {
        json!({
            "provider_id": inst.external_id,
            "display_name": inst.name,
            "logo_uri": inst.logo_url,
            "country": inst.country
        })
    }
}
