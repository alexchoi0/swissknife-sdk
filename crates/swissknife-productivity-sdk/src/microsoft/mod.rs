mod client;

#[cfg(feature = "onedrive")]
mod onedrive;

#[cfg(feature = "sharepoint")]
mod sharepoint;

#[cfg(feature = "excel")]
mod excel;

#[cfg(feature = "planner")]
mod planner;

pub use client::MicrosoftClient;

#[cfg(feature = "onedrive")]
pub use onedrive::*;

#[cfg(feature = "sharepoint")]
pub use sharepoint::*;

#[cfg(feature = "excel")]
pub use excel::*;

#[cfg(feature = "planner")]
pub use planner::*;
