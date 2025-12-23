use super::super::domain::{account, institution, transaction, user};
use super::super::generator::{LinkTokenData, TokenData};
use super::ProviderFormatter;
use serde_json::json;

pub struct PlaidFormatter;

impl ProviderFormatter for PlaidFormatter {
    fn format_token(&self, token: &TokenData) -> String {
        json!({
            "access_token": token.access_token,
            "item_id": format!("item_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..24].to_lowercase()),
            "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
        }).to_string()
    }

    fn format_link_token(&self, token: &LinkTokenData) -> String {
        json!({
            "link_token": token.link_token,
            "expiration": token.expiration,
            "request_id": token.request_id
        }).to_string()
    }

    fn format_accounts(&self, accounts: &[account::Model]) -> String {
        let formatted: Vec<_> = accounts.iter().map(|a| self.account_to_json(a)).collect();
        json!({
            "accounts": formatted,
            "item": {
                "available_products": ["balance", "identity"],
                "billed_products": ["transactions"],
                "consent_expiration_time": null,
                "error": null,
                "institution_id": accounts.first().map(|a| &a.institution_id).unwrap_or(&String::new()),
                "item_id": format!("item_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..24].to_lowercase()),
                "update_type": "background",
                "webhook": ""
            },
            "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
        }).to_string()
    }

    fn format_account(&self, account: &account::Model) -> String {
        json!({
            "account": self.account_to_json(account),
            "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
        }).to_string()
    }

    fn format_balances(&self, account: &account::Model) -> String {
        json!({
            "accounts": [self.account_to_json(account)],
            "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
        }).to_string()
    }

    fn format_transactions(&self, transactions: &[transaction::Model]) -> String {
        let formatted: Vec<_> = transactions.iter().map(|t| self.transaction_to_json(t)).collect();
        json!({
            "accounts": [],
            "transactions": formatted,
            "total_transactions": transactions.len(),
            "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
        }).to_string()
    }

    fn format_transaction(&self, transaction: &transaction::Model) -> String {
        self.transaction_to_json(transaction).to_string()
    }

    fn format_institutions(&self, institutions: &[institution::Model]) -> String {
        let formatted: Vec<_> = institutions.iter().map(|i| self.institution_to_json(i)).collect();
        json!({
            "institutions": formatted,
            "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
        }).to_string()
    }

    fn format_institution(&self, institution: &institution::Model) -> String {
        json!({
            "institution": self.institution_to_json(institution),
            "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
        }).to_string()
    }

    fn format_identity(&self, user: Option<&user::Model>) -> String {
        match user {
            Some(u) => json!({
                "accounts": [{
                    "account_id": format!("acc_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_lowercase()),
                    "owners": [{
                        "addresses": [{
                            "data": {
                                "city": u.address_city,
                                "country": u.address_country,
                                "postal_code": u.address_postal_code,
                                "region": u.address_state,
                                "street": u.address_street
                            },
                            "primary": true
                        }],
                        "emails": [{
                            "data": u.email,
                            "primary": true,
                            "type": "primary"
                        }],
                        "names": [u.name.clone()],
                        "phone_numbers": [{
                            "data": u.phone,
                            "primary": true,
                            "type": "mobile"
                        }]
                    }]
                }],
                "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
            }).to_string(),
            None => json!({
                "accounts": [],
                "request_id": uuid::Uuid::new_v4().to_string().replace("-", "")[..15].to_uppercase()
            }).to_string(),
        }
    }
}

impl PlaidFormatter {
    fn account_to_json(&self, account: &account::Model) -> serde_json::Value {
        json!({
            "account_id": account.external_id,
            "balances": {
                "available": account.balance_available,
                "current": account.balance_current,
                "iso_currency_code": account.currency,
                "limit": account.balance_limit,
                "unofficial_currency_code": null
            },
            "mask": account.mask,
            "name": account.name,
            "official_name": format!("{} Official", account.name),
            "subtype": account.subtype,
            "type": "depository"
        })
    }

    fn transaction_to_json(&self, txn: &transaction::Model) -> serde_json::Value {
        json!({
            "account_id": txn.account_id,
            "amount": txn.amount.abs(),
            "iso_currency_code": txn.currency,
            "unofficial_currency_code": null,
            "category": [txn.category],
            "category_id": "13005000",
            "check_number": null,
            "date": txn.date,
            "datetime": txn.posted_at,
            "location": {},
            "merchant_name": txn.merchant_name,
            "merchant_entity_id": null,
            "name": txn.description,
            "payment_channel": if txn.transaction_type == "debit" { "in store" } else { "other" },
            "pending": txn.status == "pending",
            "pending_transaction_id": null,
            "transaction_id": txn.external_id,
            "transaction_type": if txn.amount < 0.0 { "place" } else { "special" },
            "counterparties": []
        })
    }

    fn institution_to_json(&self, inst: &institution::Model) -> serde_json::Value {
        json!({
            "country_codes": [inst.country],
            "institution_id": inst.external_id,
            "name": inst.name,
            "oauth": false,
            "products": ["transactions", "auth", "balance", "identity"],
            "routing_numbers": [],
            "url": format!("https://www.{}.com", inst.name.to_lowercase().replace(" ", "")),
            "primary_color": "#117ACA",
            "logo": inst.logo_url
        })
    }
}
