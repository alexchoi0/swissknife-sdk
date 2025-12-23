use super::super::domain::{account, institution, transaction, user};
use super::super::generator::{LinkTokenData, TokenData};
use super::ProviderFormatter;
use serde_json::json;

pub struct MxFormatter;

impl ProviderFormatter for MxFormatter {
    fn format_token(&self, _token: &TokenData) -> String {
        let user_guid = format!("USR-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
        json!({
            "user": {
                "guid": user_guid,
                "id": format!("user_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_lowercase()),
                "is_disabled": false
            }
        }).to_string()
    }

    fn format_link_token(&self, token: &LinkTokenData) -> String {
        let user_guid = format!("USR-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
        json!({
            "user": {
                "connect_widget_url": format!("https://int-widgets.moneydesktop.com/md/connect/{}", token.link_token),
                "guid": user_guid
            }
        }).to_string()
    }

    fn format_accounts(&self, accounts: &[account::Model]) -> String {
        let formatted: Vec<_> = accounts.iter().map(|a| self.account_to_json(a)).collect();
        json!({
            "accounts": formatted,
            "pagination": {
                "current_page": 1,
                "per_page": 25,
                "total_entries": accounts.len(),
                "total_pages": 1
            }
        }).to_string()
    }

    fn format_account(&self, account: &account::Model) -> String {
        json!({
            "account": self.account_to_json(account)
        }).to_string()
    }

    fn format_balances(&self, account: &account::Model) -> String {
        json!({
            "account": self.account_to_json(account)
        }).to_string()
    }

    fn format_transactions(&self, transactions: &[transaction::Model]) -> String {
        let formatted: Vec<_> = transactions.iter().map(|t| self.transaction_to_json(t)).collect();
        json!({
            "transactions": formatted,
            "pagination": {
                "current_page": 1,
                "per_page": 25,
                "total_entries": transactions.len(),
                "total_pages": 1
            }
        }).to_string()
    }

    fn format_transaction(&self, transaction: &transaction::Model) -> String {
        json!({
            "transaction": self.transaction_to_json(transaction)
        }).to_string()
    }

    fn format_institutions(&self, institutions: &[institution::Model]) -> String {
        let formatted: Vec<_> = institutions.iter().map(|i| self.institution_to_json(i)).collect();
        json!({
            "institutions": formatted,
            "pagination": {
                "current_page": 1,
                "per_page": 25,
                "total_entries": institutions.len(),
                "total_pages": 1
            }
        }).to_string()
    }

    fn format_institution(&self, institution: &institution::Model) -> String {
        json!({
            "institution": self.institution_to_json(institution)
        }).to_string()
    }

    fn format_identity(&self, user: Option<&user::Model>) -> String {
        match user {
            Some(u) => {
                let owner_guid = format!("AOW-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
                json!({
                    "account_owners": [{
                        "account_guid": format!("ACT-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string()),
                        "address": format!("{}, {}, {} {}",
                            u.address_street.clone().unwrap_or_default(),
                            u.address_city.clone().unwrap_or_default(),
                            u.address_state.clone().unwrap_or_default(),
                            u.address_postal_code.clone().unwrap_or_default()
                        ),
                        "city": u.address_city,
                        "country": u.address_country,
                        "email": u.email,
                        "first_name": u.name.split_whitespace().next().unwrap_or(&u.name),
                        "guid": owner_guid,
                        "last_name": u.name.split_whitespace().last().unwrap_or(""),
                        "member_guid": format!("MBR-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string()),
                        "owner_name": u.name,
                        "phone": u.phone,
                        "postal_code": u.address_postal_code,
                        "state": u.address_state
                    }],
                    "pagination": {
                        "current_page": 1,
                        "per_page": 25,
                        "total_entries": 1,
                        "total_pages": 1
                    }
                }).to_string()
            },
            None => json!({
                "account_owners": [],
                "pagination": {
                    "current_page": 1,
                    "per_page": 25,
                    "total_entries": 0,
                    "total_pages": 0
                }
            }).to_string(),
        }
    }
}

impl MxFormatter {
    fn account_to_json(&self, account: &account::Model) -> serde_json::Value {
        let guid = format!("ACT-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
        let member_guid = format!("MBR-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
        let user_guid = format!("USR-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());

        json!({
            "account_number": account.account_number,
            "apr": null,
            "apy": 0.01,
            "available_balance": account.balance_available,
            "available_credit": null,
            "balance": account.balance_current,
            "cash_balance": null,
            "cash_surrender_value": null,
            "created_at": account.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "credit_limit": account.balance_limit,
            "currency_code": account.currency,
            "day_payment_is_due": null,
            "death_benefit": null,
            "guid": guid,
            "holdings_value": null,
            "id": account.external_id,
            "imported_at": account.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "institution_code": account.institution_id,
            "insured_name": null,
            "interest_rate": null,
            "is_closed": account.status != "open",
            "is_hidden": false,
            "last_payment": null,
            "last_payment_at": null,
            "loan_amount": null,
            "matures_on": null,
            "member_guid": member_guid,
            "member_id": format!("member_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_lowercase()),
            "member_is_managed_by_user": true,
            "metadata": null,
            "minimum_balance": null,
            "minimum_payment": null,
            "name": account.name,
            "nickname": null,
            "original_balance": null,
            "pay_out_amount": null,
            "payment_due_at": null,
            "payoff_balance": null,
            "premium_amount": null,
            "routing_number": account.routing_number,
            "started_on": null,
            "subtype": account.subtype.clone().map(|s| s.to_uppercase()),
            "total_account_value": null,
            "type": account.account_type.to_uppercase(),
            "updated_at": account.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "user_guid": user_guid,
            "user_id": format!("user_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_lowercase())
        })
    }

    fn transaction_to_json(&self, txn: &transaction::Model) -> serde_json::Value {
        let guid = format!("TRN-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
        let account_guid = format!("ACT-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
        let member_guid = format!("MBR-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());
        let user_guid = format!("USR-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string());

        let is_income = txn.amount > 0.0;

        json!({
            "account_guid": account_guid,
            "account_id": txn.account_id,
            "amount": txn.amount.abs(),
            "category": txn.category,
            "category_guid": format!("CAT-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string()),
            "check_number_string": null,
            "created_at": txn.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "currency_code": txn.currency,
            "date": txn.date,
            "description": txn.description,
            "extended_transaction_type": null,
            "guid": guid,
            "id": txn.external_id,
            "is_bill_pay": false,
            "is_direct_deposit": is_income,
            "is_expense": !is_income,
            "is_fee": false,
            "is_income": is_income,
            "is_international": false,
            "is_overdraft_fee": false,
            "is_payroll_advance": false,
            "is_recurring": false,
            "is_subscription": false,
            "latitude": null,
            "longitude": null,
            "member_guid": member_guid,
            "member_is_managed_by_user": true,
            "memo": null,
            "merchant_category_code": null,
            "merchant_guid": if txn.merchant_name.is_some() {
                Some(format!("MCH-{}", uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..32].to_string()))
            } else {
                None
            },
            "merchant_location_guid": null,
            "metadata": null,
            "original_description": txn.description.to_uppercase(),
            "posted_at": txn.posted_at.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            "status": txn.status.to_uppercase(),
            "top_level_category": txn.category,
            "transacted_at": txn.posted_at.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            "type": if is_income { "CREDIT" } else { "DEBIT" },
            "updated_at": txn.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "user_guid": user_guid,
            "user_id": format!("user_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_lowercase())
        })
    }

    fn institution_to_json(&self, inst: &institution::Model) -> serde_json::Value {
        json!({
            "code": inst.external_id,
            "medium_logo_url": inst.logo_url,
            "name": inst.name,
            "small_logo_url": inst.logo_url,
            "supports_account_identification": true,
            "supports_account_statement": true,
            "supports_account_verification": true,
            "supports_oauth": false,
            "supports_transaction_history": true,
            "url": format!("https://www.{}.com", inst.name.to_lowercase().replace(" ", ""))
        })
    }
}
