#[cfg(feature = "payments")]
pub mod payments;

#[cfg(feature = "crm")]
pub mod crm;

#[cfg(feature = "communication")]
pub mod communication;

#[cfg(feature = "social")]
pub mod social;

#[cfg(feature = "hr")]
pub mod hr;

#[cfg(feature = "banking")]
pub mod banking;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "search")]
pub mod search;

#[cfg(feature = "devtools")]
pub mod devtools;

#[cfg(feature = "productivity")]
pub mod productivity;

#[cfg(feature = "pm")]
pub mod pm;

#[cfg(feature = "vectordb")]
pub mod vectordb;

#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "ecommerce")]
pub mod ecommerce;

#[cfg(feature = "observability")]
pub mod observability;

#[cfg(feature = "cloud")]
pub mod cloud;

#[cfg(feature = "llm")]
pub mod llm;
