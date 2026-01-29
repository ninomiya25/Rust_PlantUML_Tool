// Unit tests for PlantUML client module

use plantuml_editor_core::*;

#[tokio::test]
async fn test_client_creation() {
    let client = PlantUmlClient::new("http://localhost:8081".to_string());
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_client_creation_with_invalid_url() {
    // Client creation should succeed even with invalid URL
    // Actual connection test happens during convert
    let client = PlantUmlClient::new("invalid_url".to_string());
    assert!(client.is_ok());
}

// Integration tests with mock server will be added in Phase 3
// when we implement the full API contract tests
