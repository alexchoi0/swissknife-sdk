pub mod plaid;
pub mod truelayer;
pub mod teller;
pub mod gocardless;
pub mod yapily;
pub mod mx;

use super::{entity::mock_response::CreateMockResponse, MockBackend, MockBuilder};
use super::entity::{mock_request::CreateMockRequest, scenario::CreateScenario};

pub async fn create_plaid_happy_path() -> crate::Result<MockBackend> {
    plaid::happy_path().await
}

pub async fn create_truelayer_happy_path() -> crate::Result<MockBackend> {
    truelayer::happy_path().await
}

pub async fn create_teller_happy_path() -> crate::Result<MockBackend> {
    teller::happy_path().await
}

pub async fn create_gocardless_happy_path() -> crate::Result<MockBackend> {
    gocardless::happy_path().await
}

pub async fn create_yapily_happy_path() -> crate::Result<MockBackend> {
    yapily::happy_path().await
}

pub async fn create_mx_happy_path() -> crate::Result<MockBackend> {
    mx::happy_path().await
}

pub async fn create_all_providers_happy_path() -> crate::Result<MockBackend> {
    let backend = MockBackend::new().await?;

    backend.create_scenario(CreateScenario::new("plaid_happy_path", "plaid")).await?;
    backend.create_scenario(CreateScenario::new("truelayer_happy_path", "truelayer")).await?;
    backend.create_scenario(CreateScenario::new("teller_happy_path", "teller")).await?;
    backend.create_scenario(CreateScenario::new("gocardless_happy_path", "gocardless")).await?;
    backend.create_scenario(CreateScenario::new("yapily_happy_path", "yapily")).await?;
    backend.create_scenario(CreateScenario::new("mx_happy_path", "mx")).await?;

    plaid::add_fixtures(&backend, "plaid_happy_path").await?;
    truelayer::add_fixtures(&backend, "truelayer_happy_path").await?;
    teller::add_fixtures(&backend, "teller_happy_path").await?;
    gocardless::add_fixtures(&backend, "gocardless_happy_path").await?;
    yapily::add_fixtures(&backend, "yapily_happy_path").await?;
    mx::add_fixtures(&backend, "mx_happy_path").await?;

    Ok(backend)
}
