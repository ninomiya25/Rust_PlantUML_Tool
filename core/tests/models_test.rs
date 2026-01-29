// Unit tests for models module

use plantuml_editor_core::*;

#[test]
fn test_document_creation() {
    let content = "@startuml\nAlice -> Bob: Hello\n@enduml".to_string();
    let doc = PlantUMLDocument::new(content.clone());
    
    assert_eq!(doc.content, content);
    assert!(doc.title.is_none());
    assert!(doc.created_at > 0);
    assert_eq!(doc.created_at, doc.updated_at);
}

#[test]
fn test_document_validation_valid() {
    let content = "@startuml\nAlice -> Bob: Hello\n@enduml".to_string();
    let doc = PlantUMLDocument::new(content);
    
    assert!(doc.validate().is_ok());
}

#[test]
fn test_document_validation_empty() {
    let doc = PlantUMLDocument::new("   ".to_string());
    assert!(doc.validate().is_err());
}

#[test]
fn test_document_validation_missing_start_tag() {
    let doc = PlantUMLDocument::new("Alice -> Bob\n@enduml".to_string());
    assert!(doc.validate().is_err());
}

#[test]
fn test_document_validation_missing_end_tag() {
    let doc = PlantUMLDocument::new("@startuml\nAlice -> Bob".to_string());
    assert!(doc.validate().is_err());
}

#[test]
fn test_document_validation_too_large() {
    let large_content = format!("@startuml\n{}\n@enduml", "x".repeat(25000));
    let doc = PlantUMLDocument::new(large_content);
    assert!(doc.validate().is_err());
}

#[test]
fn test_document_id_generation() {
    let doc1 = PlantUMLDocument::new("@startuml\nA\n@enduml".to_string());
    let doc2 = PlantUMLDocument::new("@startuml\nB\n@enduml".to_string());
    
    // IDs should be different
    assert_ne!(doc1.id, doc2.id);
}

#[test]
fn test_diagram_image_png_validation_valid() {
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    let image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Png,
        data: png_data,
        dimensions: (800, 600),
        generated_at: chrono::Utc::now().timestamp(),
        result: GenerationResult::Success,
    };
    
    assert!(image.validate_png().is_ok());
}

#[test]
fn test_diagram_image_png_validation_invalid_header() {
    let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
    let image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Png,
        data: invalid_data,
        dimensions: (800, 600),
        generated_at: chrono::Utc::now().timestamp(),
        result: GenerationResult::Success,
    };
    
    assert!(image.validate_png().is_err());
}

#[test]
fn test_diagram_image_png_validation_wrong_format() {
    let svg_image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Svg,
        data: vec![0x89, 0x50, 0x4E, 0x47],
        dimensions: (800, 600),
        generated_at: chrono::Utc::now().timestamp(),
        result: GenerationResult::Success,
    };
    
    assert!(svg_image.validate_png().is_err());
}

#[test]
fn test_diagram_image_to_data_url_png() {
    let png_data = vec![0x89, 0x50, 0x4E, 0x47];
    let image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Png,
        data: png_data,
        dimensions: (800, 600),
        generated_at: chrono::Utc::now().timestamp(),
        result: GenerationResult::Success,
    };
    
    let data_url = image.to_data_url();
    assert!(data_url.starts_with("data:image/png;base64,"));
}

#[test]
fn test_diagram_image_to_data_url_svg() {
    let svg_data = b"<svg></svg>".to_vec();
    let image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Svg,
        data: svg_data,
        dimensions: (800, 600),
        generated_at: chrono::Utc::now().timestamp(),
        result: GenerationResult::Success,
    };
    
    let data_url = image.to_data_url();
    assert!(data_url.starts_with("data:image/svg+xml;base64,"));
}

#[test]
fn test_storage_slot_validation_valid() {
    assert!(StorageSlot::validate_slot_number(1).is_ok());
    assert!(StorageSlot::validate_slot_number(5).is_ok());
    assert!(StorageSlot::validate_slot_number(10).is_ok());
}

#[test]
fn test_storage_slot_validation_invalid() {
    assert!(StorageSlot::validate_slot_number(0).is_err());
    assert!(StorageSlot::validate_slot_number(11).is_err());
    assert!(StorageSlot::validate_slot_number(255).is_err());
}

#[test]
fn test_storage_slot_key() {
    assert_eq!(StorageSlot::storage_key(1), "plantuml_slot_1");
    assert_eq!(StorageSlot::storage_key(10), "plantuml_slot_10");
}

#[test]
fn test_convert_request_validation() {
    let valid_request = ConvertRequest {
        plantuml_text: "@startuml\nAlice -> Bob\n@enduml".to_string(),
        format: ImageFormat::Png,
    };
    assert!(valid_request.validate().is_ok());
    
    let invalid_request = ConvertRequest {
        plantuml_text: "invalid".to_string(),
        format: ImageFormat::Png,
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_error_response_messages() {
    let system_err = ErrorResponse::system_error();
    assert!(system_err.error.contains("システムエラー"));
    
    let network_err = ErrorResponse::network_error();
    assert!(network_err.error.contains("ネットワークエラー"));
    
    let validation_err = ErrorResponse::validation_error("test".to_string());
    assert!(validation_err.error.contains("入力エラー"));
}
