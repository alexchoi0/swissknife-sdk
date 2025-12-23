mod client;

#[cfg(feature = "google-drive")]
pub mod drive;

#[cfg(feature = "google-docs")]
pub mod docs;

#[cfg(feature = "google-sheets")]
pub mod sheets;

#[cfg(feature = "google-calendar")]
pub mod calendar;

pub use client::GoogleClient;

#[cfg(feature = "google-drive")]
pub use drive::*;

#[cfg(feature = "google-docs")]
pub use docs::{GoogleDocsProvider, GoogleDocument, DocUpdateRequest, BatchUpdateResponse};

#[cfg(feature = "google-sheets")]
pub use sheets::{GoogleSheetsProvider, GoogleSpreadsheet, ValueRange};

#[cfg(feature = "google-calendar")]
pub use calendar::*;
