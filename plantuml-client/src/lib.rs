// PlantUML Client Library
//
// This crate provides HTTP client functionality for communicating
// with PlantUML Picoweb server.

mod client;
mod errors;

pub use client::PlantUmlClient;
pub use errors::ClientError;
