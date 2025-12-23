use crate::{
    AccessToken, Account, AccountBalance, AccountIdentity, BankingProvider, CreateLinkTokenRequest,
    CreatePaymentRequest, Institution, InstitutionOptions, Item, LinkToken, Payment, Transaction,
    TransactionList, TransactionOptions, Error, Result,
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct YapilyClient {
    client: Client,
    application_uuid: String,
    application_secret: String,
    base_url: String,
}

impl YapilyClient {
    pub fn new(application_uuid: impl Into<String>, application_secret: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            application_uuid: application_uuid.into(),
            application_secret: application_secret.into(),
            base_url: "https://api.yapily.com".to_string(),
        }
    }

    fn basic_auth(&self) -> String {
        let credentials = format!("{}:{}", self.application_uuid, self.application_secret);
        format!("Basic {}", BASE64.encode(credentials.as_bytes()))
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str, consent: Option<&str>) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut req = self.client.get(&url).header("Authorization", self.basic_auth());

        if let Some(consent_token) = consent {
            req = req.header("Consent", consent_token);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: YapilyError = response.json().await?;
            Err(Error::Api {
                code: error.error.as_ref().map(|e| e.code.clone()).unwrap_or_else(|| "unknown".to_string()),
                message: error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
        consent: Option<&str>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut req = self
            .client
            .post(&url)
            .header("Authorization", self.basic_auth())
            .json(body);

        if let Some(consent_token) = consent {
            req = req.header("Consent", consent_token);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: YapilyError = response.json().await?;
            Err(Error::Api {
                code: error.error.as_ref().map(|e| e.code.clone()).unwrap_or_else(|| "unknown".to_string()),
                message: error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }

    async fn delete(&self, endpoint: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.basic_auth())
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            let error: YapilyError = response.json().await?;
            Err(Error::Api {
                code: error.error.as_ref().map(|e| e.code.clone()).unwrap_or_else(|| "unknown".to_string()),
                message: error.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
            })
        }
    }
}

#[derive(Deserialize)]
struct YapilyError {
    error: Option<YapilyErrorDetail>,
}

#[derive(Deserialize)]
struct YapilyErrorDetail {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct YapilyData<T> {
    data: T,
}

#[derive(Deserialize)]
struct YapilyMeta {
    count: Option<u32>,
}

#[derive(Deserialize)]
struct YapilyPaginatedData<T> {
    data: Vec<T>,
    meta: Option<YapilyMeta>,
}

#[derive(Serialize)]
struct AccountAuthorisationRequest {
    application_user_id: String,
    institution_id: String,
    callback: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    one_time_token: Option<bool>,
}

#[derive(Deserialize)]
struct AuthorisationResponse {
    data: AuthorisationData,
}

#[derive(Deserialize)]
struct AuthorisationData {
    id: Option<String>,
    user_uuid: Option<String>,
    application_user_id: Option<String>,
    institution_id: Option<String>,
    status: Option<String>,
    created_at: Option<String>,
    feature_scope: Option<Vec<String>>,
    consent_token: Option<String>,
    state: Option<String>,
    authorisation_url: Option<String>,
    qr_code_url: Option<String>,
}

#[derive(Deserialize)]
struct YapilyInstitution {
    id: String,
    name: String,
    full_name: Option<String>,
    countries: Vec<YapilyCountry>,
    media: Option<Vec<YapilyMedia>>,
    features: Vec<String>,
}

#[derive(Deserialize)]
struct YapilyCountry {
    display_name: String,
    country_code2: String,
}

#[derive(Deserialize)]
struct YapilyMedia {
    #[serde(rename = "type")]
    media_type: String,
    source: String,
}

#[derive(Deserialize)]
struct YapilyAccount {
    id: String,
    #[serde(rename = "type")]
    account_type: Option<String>,
    description: Option<String>,
    balance: Option<f64>,
    currency: Option<String>,
    usage_type: Option<String>,
    account_type_description: Option<String>,
    account_names: Option<Vec<YapilyAccountName>>,
    account_identifications: Option<Vec<YapilyAccountIdentification>>,
    account_balances: Option<Vec<YapilyAccountBalance>>,
}

#[derive(Deserialize)]
struct YapilyAccountName {
    name: Option<String>,
}

#[derive(Deserialize)]
struct YapilyAccountIdentification {
    #[serde(rename = "type")]
    id_type: String,
    identification: String,
}

#[derive(Deserialize)]
struct YapilyAccountBalance {
    #[serde(rename = "type")]
    balance_type: String,
    balance_amount: YapilyAmount,
    credit_line_included: Option<bool>,
    date_time: Option<String>,
}

#[derive(Deserialize)]
struct YapilyAmount {
    amount: f64,
    currency: String,
}

#[derive(Deserialize)]
struct YapilyTransaction {
    id: Option<String>,
    date: Option<String>,
    booking_date_time: Option<String>,
    value_date_time: Option<String>,
    status: Option<String>,
    amount: f64,
    currency: Option<String>,
    transaction_amount: Option<YapilyAmount>,
    gross_amount: Option<YapilyAmount>,
    charge_details: Option<YapilyChargeDetails>,
    reference: Option<String>,
    description: Option<String>,
    transaction_information: Option<Vec<String>>,
    address_details: Option<YapilyAddressDetails>,
    iso_bank_transaction_code: Option<YapilyBankTransactionCode>,
    proprietary_bank_transaction_code: Option<YapilyProprietaryCode>,
    payee_details: Option<YapilyPartyDetails>,
    payer_details: Option<YapilyPartyDetails>,
    balance: Option<YapilyBalanceAfterTransaction>,
    enrichment: Option<YapilyEnrichment>,
}

#[derive(Deserialize)]
struct YapilyChargeDetails {
    charge_amount: Option<YapilyAmount>,
}

#[derive(Deserialize)]
struct YapilyAddressDetails {
    address_line: Option<String>,
}

#[derive(Deserialize)]
struct YapilyBankTransactionCode {
    domain_code: Option<String>,
    family_code: Option<String>,
    sub_family_code: Option<String>,
}

#[derive(Deserialize)]
struct YapilyProprietaryCode {
    code: Option<String>,
    issuer: Option<String>,
}

#[derive(Deserialize)]
struct YapilyPartyDetails {
    name: Option<String>,
    account_identifications: Option<Vec<YapilyAccountIdentification>>,
}

#[derive(Deserialize)]
struct YapilyBalanceAfterTransaction {
    balance_amount: Option<YapilyAmount>,
}

#[derive(Deserialize)]
struct YapilyEnrichment {
    categorisation: Option<YapilyCategorisation>,
    merchant: Option<YapilyMerchant>,
}

#[derive(Deserialize)]
struct YapilyCategorisation {
    categories: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct YapilyMerchant {
    merchant_name: Option<String>,
    merchant_category_code: Option<String>,
}

#[derive(Deserialize)]
struct YapilyIdentity {
    id: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    full_name: Option<String>,
    gender: Option<String>,
    birthdate: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    addresses: Option<Vec<YapilyAddress>>,
}

#[derive(Deserialize)]
struct YapilyAddress {
    address_lines: Option<Vec<String>>,
    street_name: Option<String>,
    building_number: Option<String>,
    post_code: Option<String>,
    town_name: Option<String>,
    county: Option<Vec<String>>,
    country: Option<String>,
    address_type: Option<String>,
}

#[derive(Deserialize)]
struct YapilyConsent {
    id: String,
    user_uuid: Option<String>,
    application_user_id: Option<String>,
    institution_id: Option<String>,
    status: String,
    created_at: Option<String>,
    expires_at: Option<String>,
    feature_scope: Option<Vec<String>>,
    consent_token: String,
}

impl YapilyClient {
    fn convert_account(&self, account: YapilyAccount) -> Account {
        let currency = account.currency.clone().unwrap_or_else(|| "GBP".to_string());

        let balance = account.account_balances
            .as_ref()
            .and_then(|balances| {
                balances.iter().find(|b| b.balance_type == "EXPECTED" || b.balance_type == "CLOSING_AVAILABLE")
            })
            .or_else(|| account.account_balances.as_ref().and_then(|b| b.first()));

        let current = balance.map(|b| b.balance_amount.amount).or(account.balance).unwrap_or(0.0);

        let iban = account.account_identifications
            .as_ref()
            .and_then(|ids| ids.iter().find(|i| i.id_type == "IBAN").map(|i| i.identification.clone()));

        let sort_code = account.account_identifications
            .as_ref()
            .and_then(|ids| ids.iter().find(|i| i.id_type == "SORT_CODE").map(|i| i.identification.clone()));

        let account_number = account.account_identifications
            .as_ref()
            .and_then(|ids| ids.iter().find(|i| i.id_type == "ACCOUNT_NUMBER").map(|i| i.identification.clone()));

        Account {
            id: account.id.clone(),
            name: account.description.clone()
                .or(account.account_names.as_ref().and_then(|n| n.first()).and_then(|n| n.name.clone()))
                .unwrap_or_else(|| "Account".to_string()),
            official_name: account.account_type_description,
            account_type: match account.account_type.as_deref() {
                Some("CURRENT") | Some("PERSONAL") => crate::AccountType::Depository,
                Some("SAVINGS") => crate::AccountType::Depository,
                Some("CREDIT_CARD") => crate::AccountType::Credit,
                Some("LOAN") | Some("MORTGAGE") => crate::AccountType::Loan,
                Some("INVESTMENT") => crate::AccountType::Investment,
                _ => crate::AccountType::Other,
            },
            subtype: account.account_type,
            mask: iban.as_ref().map(|i| i.chars().rev().take(4).collect::<String>().chars().rev().collect()),
            currency: currency.clone(),
            institution_id: None,
            balances: AccountBalance {
                account_id: account.id,
                current,
                available: Some(current),
                limit: None,
                currency,
                last_updated: balance.and_then(|b| {
                    b.date_time.as_ref().and_then(|dt| {
                        chrono::DateTime::parse_from_rfc3339(dt).ok().map(|d| d.with_timezone(&chrono::Utc))
                    })
                }),
            },
            account_number: Some(crate::AccountNumber {
                account_number,
                routing_number: None,
                sort_code,
                iban,
                bic: None,
            }),
            extra: HashMap::new(),
        }
    }

    fn convert_transaction(&self, tx: YapilyTransaction, account_id: &str) -> Transaction {
        let amount = tx.transaction_amount.as_ref().map(|a| a.amount).unwrap_or(tx.amount);
        let currency = tx.transaction_amount.as_ref().map(|a| a.currency.clone())
            .or(tx.currency.clone())
            .unwrap_or_else(|| "GBP".to_string());

        let is_credit = amount > 0.0;

        let description = tx.description.clone()
            .or(tx.reference.clone())
            .or(tx.transaction_information.as_ref().and_then(|i| i.first().cloned()))
            .unwrap_or_else(|| "Transaction".to_string());

        let counterparty_name = if is_credit {
            tx.payer_details.as_ref().and_then(|p| p.name.clone())
        } else {
            tx.payee_details.as_ref().and_then(|p| p.name.clone())
        };

        let merchant_name = tx.enrichment.as_ref()
            .and_then(|e| e.merchant.as_ref())
            .and_then(|m| m.merchant_name.clone())
            .or(counterparty_name.clone());

        Transaction {
            id: tx.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            account_id: account_id.to_string(),
            amount: amount.abs(),
            currency,
            date: tx.date
                .or(tx.booking_date_time.as_ref().map(|d| d.split('T').next().unwrap_or(d).to_string()))
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                .unwrap_or_else(|| chrono::Utc::now().date_naive()),
            datetime: tx.booking_date_time.as_ref()
                .and_then(|dt| chrono::DateTime::parse_from_rfc3339(dt).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            name: description,
            merchant_name,
            merchant_id: tx.enrichment.as_ref()
                .and_then(|e| e.merchant.as_ref())
                .and_then(|m| m.merchant_category_code.clone()),
            category: tx.enrichment.as_ref()
                .and_then(|e| e.categorisation.as_ref())
                .and_then(|c| c.categories.clone()),
            category_id: tx.iso_bank_transaction_code.as_ref()
                .and_then(|c| c.sub_family_code.clone()),
            pending: tx.status.as_deref() == Some("PENDING"),
            transaction_type: if is_credit {
                crate::TransactionType::Credit
            } else {
                crate::TransactionType::Debit
            },
            payment_channel: None,
            location: tx.address_details.map(|a| crate::TransactionLocation {
                address: a.address_line,
                city: None,
                region: None,
                postal_code: None,
                country: None,
                lat: None,
                lon: None,
                store_number: None,
            }),
            counterparty: counterparty_name.map(|name| crate::Counterparty {
                name: Some(name),
                entity_id: None,
                counterparty_type: None,
                logo_url: None,
                website: None,
            }),
            extra: HashMap::new(),
        }
    }

    fn convert_institution(&self, inst: YapilyInstitution) -> Institution {
        let logo = inst.media.as_ref()
            .and_then(|m| m.iter().find(|media| media.media_type == "icon" || media.media_type == "logo"))
            .map(|m| m.source.clone());

        Institution {
            id: inst.id,
            name: inst.name,
            url: None,
            logo,
            primary_color: None,
            country_codes: inst.countries.into_iter().map(|c| c.country_code2).collect(),
            products: inst.features.iter().filter_map(|f| {
                match f.as_str() {
                    "ACCOUNT_TRANSACTIONS" => Some(crate::Product::Transactions),
                    "ACCOUNT" | "ACCOUNTS" => Some(crate::Product::Auth),
                    "IDENTITY" => Some(crate::Product::Identity),
                    "INITIATE_DOMESTIC_SINGLE_PAYMENT" | "CREATE_DOMESTIC_SINGLE_PAYMENT" => Some(crate::Product::PaymentInitiation),
                    _ => None,
                }
            }).collect(),
            routing_numbers: None,
            oauth: true,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl BankingProvider for YapilyClient {
    async fn create_link_token(&self, request: CreateLinkTokenRequest) -> Result<LinkToken> {
        let redirect_uri = request.redirect_uri.ok_or_else(|| {
            Error::InvalidRequest("redirect_uri (callback) is required".to_string())
        })?;

        let institution_id = request.extra.get("institution_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("institution_id is required".to_string()))?;

        let auth_request = AccountAuthorisationRequest {
            application_user_id: request.user_id,
            institution_id: institution_id.to_string(),
            callback: redirect_uri,
            one_time_token: Some(false),
        };

        let response: AuthorisationResponse = self
            .post("/account-auth-requests", &auth_request, None)
            .await?;

        let auth_url = response.data.authorisation_url.ok_or_else(|| {
            Error::Provider("No authorisation URL returned".to_string())
        })?;

        Ok(LinkToken {
            link_token: auth_url,
            expiration: chrono::Utc::now() + chrono::Duration::hours(1),
            request_id: response.data.id,
        })
    }

    async fn exchange_public_token(&self, consent_token: &str) -> Result<AccessToken> {
        Ok(AccessToken {
            access_token: consent_token.to_string(),
            item_id: None,
            request_id: None,
        })
    }

    async fn list_accounts(&self, consent_token: &str) -> Result<Vec<Account>> {
        let response: YapilyPaginatedData<YapilyAccount> = self
            .get("/accounts", Some(consent_token))
            .await?;

        Ok(response.data.into_iter().map(|a| self.convert_account(a)).collect())
    }

    async fn get_account(&self, consent_token: &str, account_id: &str) -> Result<Account> {
        let response: YapilyData<YapilyAccount> = self
            .get(&format!("/accounts/{}", account_id), Some(consent_token))
            .await?;

        Ok(self.convert_account(response.data))
    }

    async fn get_balances(&self, consent_token: &str) -> Result<Vec<AccountBalance>> {
        let accounts = self.list_accounts(consent_token).await?;
        Ok(accounts.into_iter().map(|a| a.balances).collect())
    }

    async fn list_transactions(
        &self,
        consent_token: &str,
        options: TransactionOptions,
    ) -> Result<TransactionList> {
        let accounts = self.list_accounts(consent_token).await?;

        let mut all_transactions = Vec::new();

        for account in accounts {
            if let Some(ref account_ids) = options.account_ids {
                if !account_ids.contains(&account.id) {
                    continue;
                }
            }

            let today = chrono::Utc::now().date_naive();
            let from = options.start_date.unwrap_or_else(|| today - chrono::Duration::days(90));
            let to = options.end_date.unwrap_or(today);

            let endpoint = format!(
                "/accounts/{}/transactions?from={}&before={}",
                account.id,
                from.format("%Y-%m-%d"),
                to.format("%Y-%m-%d")
            );

            if let Ok(response) = self.get::<YapilyPaginatedData<YapilyTransaction>>(&endpoint, Some(consent_token)).await {
                for tx in response.data {
                    all_transactions.push(self.convert_transaction(tx, &account.id));
                }
            }
        }

        let total = all_transactions.len() as u32;

        Ok(TransactionList {
            transactions: all_transactions,
            total_transactions: total,
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_transaction(
        &self,
        consent_token: &str,
        transaction_id: &str,
    ) -> Result<Transaction> {
        let transactions = self
            .list_transactions(consent_token, TransactionOptions::default())
            .await?;

        transactions
            .transactions
            .into_iter()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| Error::Api {
                code: "TRANSACTION_NOT_FOUND".to_string(),
                message: format!("Transaction {} not found", transaction_id),
            })
    }

    async fn list_institutions(&self, options: InstitutionOptions) -> Result<Vec<Institution>> {
        let country = options.country_codes
            .as_ref()
            .and_then(|c| c.first())
            .cloned();

        let endpoint = if let Some(ref country) = country {
            format!("/institutions?country={}", country)
        } else {
            "/institutions".to_string()
        };

        let response: YapilyPaginatedData<YapilyInstitution> = self
            .get(&endpoint, None)
            .await?;

        Ok(response.data.into_iter().map(|i| self.convert_institution(i)).collect())
    }

    async fn get_institution(&self, institution_id: &str) -> Result<Institution> {
        let response: YapilyData<YapilyInstitution> = self
            .get(&format!("/institutions/{}", institution_id), None)
            .await?;

        Ok(self.convert_institution(response.data))
    }

    async fn get_identity(&self, consent_token: &str) -> Result<Vec<AccountIdentity>> {
        let accounts = self.list_accounts(consent_token).await?;

        let mut identities = Vec::new();

        for account in accounts {
            let endpoint = format!("/accounts/{}/identity", account.id);
            if let Ok(response) = self.get::<YapilyData<YapilyIdentity>>(&endpoint, Some(consent_token)).await {
                let identity = response.data;
                identities.push(AccountIdentity {
                    account_id: account.id,
                    owners: vec![crate::Owner {
                        names: identity.full_name.or_else(|| {
                            match (&identity.first_name, &identity.last_name) {
                                (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                                (Some(first), None) => Some(first.clone()),
                                (None, Some(last)) => Some(last.clone()),
                                _ => None,
                            }
                        }).into_iter().collect(),
                        phone_numbers: identity.phone.map(|p| vec![crate::PhoneNumber {
                            data: p,
                            primary: true,
                            phone_type: None,
                        }]).unwrap_or_default(),
                        emails: identity.email.map(|e| vec![crate::Email {
                            data: e,
                            primary: true,
                            email_type: None,
                        }]).unwrap_or_default(),
                        addresses: identity.addresses.unwrap_or_default().into_iter().enumerate().map(|(i, a)| {
                            crate::Address {
                                street: a.address_lines.and_then(|l| l.into_iter().next())
                                    .or(a.street_name.map(|s| {
                                        if let Some(bn) = a.building_number.as_ref() {
                                            format!("{} {}", bn, s)
                                        } else {
                                            s
                                        }
                                    })),
                                city: a.town_name,
                                region: a.county.and_then(|c| c.into_iter().next()),
                                postal_code: a.post_code,
                                country: a.country,
                                primary: i == 0,
                            }
                        }).collect(),
                    }],
                });
            }
        }

        Ok(identities)
    }

    async fn create_payment(
        &self,
        _consent_token: &str,
        _request: CreatePaymentRequest,
    ) -> Result<Payment> {
        Err(Error::Provider(
            "Yapily payment initiation requires separate consent flow".to_string(),
        ))
    }

    async fn get_payment(&self, _consent_token: &str, _payment_id: &str) -> Result<Payment> {
        Err(Error::Provider(
            "Yapily payment initiation requires separate consent flow".to_string(),
        ))
    }

    async fn remove_item(&self, consent_id: &str) -> Result<()> {
        self.delete(&format!("/consents/{}", consent_id)).await
    }

    async fn get_item(&self, consent_token: &str) -> Result<Item> {
        let response: YapilyData<YapilyConsent> = self
            .get(&format!("/consents/{}", consent_token), None)
            .await?;

        let consent = response.data;

        Ok(Item {
            id: consent.id,
            institution_id: consent.institution_id,
            webhook: None,
            error: None,
            available_products: consent.feature_scope.unwrap_or_default().iter().filter_map(|f| {
                match f.as_str() {
                    "ACCOUNT_TRANSACTIONS" => Some(crate::Product::Transactions),
                    "ACCOUNTS" => Some(crate::Product::Auth),
                    "IDENTITY" => Some(crate::Product::Identity),
                    _ => None,
                }
            }).collect(),
            billed_products: vec![],
            consent_expiration_time: consent.expires_at.and_then(|e| {
                chrono::DateTime::parse_from_rfc3339(&e).ok().map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            update_type: Some(consent.status),
            extra: HashMap::new(),
        })
    }
}
