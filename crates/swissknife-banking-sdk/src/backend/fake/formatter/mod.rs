mod plaid;
mod truelayer;
mod teller;
mod gocardless;
mod yapily;
mod mx;

pub use plaid::PlaidFormatter;
pub use truelayer::TrueLayerFormatter;
pub use teller::TellerFormatter;
pub use gocardless::GoCardlessFormatter;
pub use yapily::YapilyFormatter;
pub use mx::MxFormatter;

use super::domain::{account, institution, transaction, user};
use super::generator::{LinkTokenData, TokenData};

pub trait ProviderFormatter: Send + Sync {
    fn format_token(&self, token: &TokenData) -> String;
    fn format_link_token(&self, token: &LinkTokenData) -> String;
    fn format_accounts(&self, accounts: &[account::Model]) -> String;
    fn format_account(&self, account: &account::Model) -> String;
    fn format_balances(&self, account: &account::Model) -> String;
    fn format_transactions(&self, transactions: &[transaction::Model]) -> String;
    fn format_transaction(&self, transaction: &transaction::Model) -> String;
    fn format_institutions(&self, institutions: &[institution::Model]) -> String;
    fn format_institution(&self, institution: &institution::Model) -> String;
    fn format_identity(&self, user: Option<&user::Model>) -> String;
}
