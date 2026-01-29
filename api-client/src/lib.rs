// API client library for PlantUML Editor
//
// This crate provides HTTP client functionality for communicating
// with the PlantUML API server from the browser-based frontend.

pub mod errors;
pub mod http_client;

// Re-export commonly used items
pub use errors::ApiError;
pub use http_client::{convert_plantuml, export_plantuml};
