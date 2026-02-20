// Unit tests for models module

use plantuml_editor_core::*;

// ==================== DocumentId Tests ====================

#[test]
fn test_document_id_default() {
    let id1 = DocumentId::default();
    let id2 = DocumentId::default();
    assert_ne!(id1, id2);
}

#[test]
fn test_document_id_generation() {
    let doc1 = PlantUMLDocument::new("@startuml\nA\n@enduml".to_string());
    let doc2 = PlantUMLDocument::new("@startuml\nB\n@enduml".to_string());
    
    // IDs should be different
    assert_ne!(doc1.id, doc2.id);
}

// ==================== PlantUMLDocument Tests ====================

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
fn test_document_validation_too_large() {
    let large_content = format!("@startuml\n{}\n@enduml", "x".repeat(25000));
    let doc = PlantUMLDocument::new(large_content);
    assert!(doc.validate().is_err());
}

// ==================== DiagramImage Tests ====================

#[test]
fn test_diagram_image_png_validation_valid() {
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    let image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Png,
        data: png_data,
        dimensions: (800, 600),
        generated_at: chrono::Utc::now().timestamp(),
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
    };
    
    assert!(svg_image.validate_png().is_err());
}

#[test]
fn test_diagram_image_png_validation_empty_data() {
    let image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Png,
        data: vec![],
        dimensions: (800, 600),
        generated_at: chrono::Utc::now().timestamp(),
    };
    
    assert!(image.validate_png().is_err());
}

#[test]
fn test_diagram_image_png_validation_dimensions_too_large() {
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let image = DiagramImage {
        document_id: DocumentId::new(),
        format: ImageFormat::Png,
        data: png_data,
        dimensions: (9000, 9000),
        generated_at: chrono::Utc::now().timestamp(),
    };
    
    assert!(image.validate_png().is_err());
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
    };
    
    let data_url = image.to_data_url();
    assert!(data_url.starts_with("data:image/svg+xml;base64,"));
}

/// ==================== StorageSlot Tests ====================

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
    // Valid request with tags
    let valid_request = ConvertRequest {
        plantuml_text: "@startuml\nAlice -> Bob\n@enduml".to_string(),
        format: ImageFormat::Png,
    };
    assert!(valid_request.validate().is_ok());
    
    // Valid request without tags (tags are not validated - PlantUML.jar handles this)
    let valid_without_tags = ConvertRequest {
        plantuml_text: "Alice -> Bob".to_string(),
        format: ImageFormat::Png,
    };
    assert!(valid_without_tags.validate().is_ok());
    
    // Invalid: Empty content
    let invalid_empty = ConvertRequest {
        plantuml_text: "   ".to_string(),
        format: ImageFormat::Png,
    };
    assert!(invalid_empty.validate().is_err());
    
    // Invalid: Content too large (over 24,000 chars)
    let invalid_too_large = ConvertRequest {
        plantuml_text: "x".repeat(25000),
        format: ImageFormat::Png,
    };
    assert!(invalid_too_large.validate().is_err());
}

// ==================== ErrorCode Tests ====================

#[test]
fn test_error_code_to_message_success() {
    assert_eq!(ErrorCode::ConversionOk.to_message(), "図が正常に生成されました");
    assert_eq!(ErrorCode::ExportOk.to_message(), "図が正常にエクスポートされました");
    assert_eq!(
        ErrorCode::SaveSuccess { slot_number: 3 }.to_message(),
        "PlantUMLソースをスロット3に保存しました"
    );
    assert_eq!(
        ErrorCode::LoadSuccess { slot_number: 5 }.to_message(),
        "スロット5からPlantUMLソースを読み込みました"
    );
    assert_eq!(
        ErrorCode::DeleteSuccess { slot_number: 7 }.to_message(),
        "スロット7のデータを削除しました"
    );
}

#[test]
fn test_error_code_to_message_validation() {
    assert_eq!(
        ErrorCode::ValidationEmpty.to_message(),
        "PlantUMLソースを入力してください"
    );
    let msg = ErrorCode::ValidationTextLimit { actual: 25000, max: 24000 }.to_message();
    assert!(msg.contains("24000"));
    assert!(msg.contains("25000"));
}

#[test]
fn test_error_code_to_message_storage() {
    let msg = ErrorCode::StorageInputLimit { actual: 25000, max: 24000 }.to_message();
    assert!(msg.contains("24000"));
    
    let msg = ErrorCode::StorageSlotLimit { max_slots: 10 }.to_message();
    assert!(msg.contains("10"));
    
    let msg = ErrorCode::StorageWriteError { reason: "test".to_string() }.to_message();
    assert!(msg.contains("test"));
    
    let msg = ErrorCode::StorageReadError { reason: "test".to_string() }.to_message();
    assert!(msg.contains("test"));
    
    let msg = ErrorCode::StorageDeleteError { reason: "test".to_string() }.to_message();
    assert!(msg.contains("test"));
}

#[test]
fn test_error_code_to_message_processing() {
    let msg = ErrorCode::SizeLimit { actual_bytes: 5000, max_bytes: 4000 }.to_message();
    assert!(msg.contains("5000"));
    assert!(msg.contains("4000"));
    
    let msg = ErrorCode::EncodingError { encoding: "UTF-8".to_string() }.to_message();
    assert!(msg.contains("UTF-8"));
    
    let msg = ErrorCode::ParseError { line: Some(42) }.to_message();
    assert!(msg.contains("42"));
    
    let msg = ErrorCode::ParseError { line: None }.to_message();
    assert!(!msg.contains("行"));
    
    let msg = ErrorCode::ExportError { format: "PNG".to_string() }.to_message();
    assert!(msg.contains("PNG"));
}

#[test]
fn test_error_code_to_message_network() {
    let msg = ErrorCode::ServerError { message: "500".to_string() }.to_message();
    assert!(msg.contains("500"));
    
    let msg = ErrorCode::TimeoutError { duration_ms: 5000 }.to_message();
    assert!(msg.contains("5000"));
    
    let msg = ErrorCode::NetworkError { endpoint: "/api/v1".to_string() }.to_message();
    assert!(msg.contains("/api/v1"));
}

#[test]
fn test_error_code_status_level_info() {
    assert_eq!(ErrorCode::ConversionOk.status_level(), StatusLevel::Info);
    assert_eq!(ErrorCode::ExportOk.status_level(), StatusLevel::Info);
    assert_eq!(ErrorCode::SaveSuccess { slot_number: 1 }.status_level(), StatusLevel::Info);
    assert_eq!(ErrorCode::LoadSuccess { slot_number: 1 }.status_level(), StatusLevel::Info);
    assert_eq!(ErrorCode::DeleteSuccess { slot_number: 1 }.status_level(), StatusLevel::Info);
}

#[test]
fn test_error_code_status_level_warning() {
    assert_eq!(ErrorCode::ValidationEmpty.status_level(), StatusLevel::Warning);
    assert_eq!(ErrorCode::ValidationTextLimit { actual: 25000, max: 24000 }.status_level(), StatusLevel::Warning);
    assert_eq!(ErrorCode::StorageInputLimit { actual: 25000, max: 24000 }.status_level(), StatusLevel::Warning);
    assert_eq!(ErrorCode::StorageSlotLimit { max_slots: 10 }.status_level(), StatusLevel::Warning);
    assert_eq!(ErrorCode::SizeLimit { actual_bytes: 5000, max_bytes: 4000 }.status_level(), StatusLevel::Warning);
}

#[test]
fn test_error_code_status_level_error() {
    assert_eq!(ErrorCode::StorageWriteError { reason: "test".to_string() }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::StorageReadError { reason: "test".to_string() }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::StorageDeleteError { reason: "test".to_string() }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::EncodingError { encoding: "UTF-8".to_string() }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::ParseError { line: Some(42) }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::ExportError { format: "PNG".to_string() }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::ServerError { message: "500".to_string() }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::TimeoutError { duration_ms: 5000 }.status_level(), StatusLevel::Error);
    assert_eq!(ErrorCode::NetworkError { endpoint: "/api".to_string() }.status_level(), StatusLevel::Error);
}

// ==================== ProcessResult Tests ====================

#[test]
fn test_process_result_new() {
    let result = ProcessResult::new(ErrorCode::ConversionOk);
    assert_eq!(result.level, StatusLevel::Info);
    assert!(matches!(result.code, ErrorCode::ConversionOk));
}

#[test]
fn test_process_result_success() {
    let result = ProcessResult::new(ErrorCode::ExportOk);
    assert_eq!(result.level, StatusLevel::Info);
    assert!(matches!(result.code, ErrorCode::ExportOk));
}

#[test]
fn test_process_result_error() {
    let result = ProcessResult::new(ErrorCode::ValidationEmpty);
    assert_eq!(result.level, StatusLevel::Warning);
    assert!(matches!(result.code, ErrorCode::ValidationEmpty));
}

#[test]
fn test_process_result_message() {
    let result = ProcessResult::new(ErrorCode::ConversionOk);
    assert_eq!(result.message(), "図が正常に生成されました");
}

// ==================== ConvertResponse Tests ====================

#[test]
fn test_convert_response_success() {
    let image_data = vec![0x89, 0x50, 0x4E, 0x47];
    let response = ConvertResponse::success(image_data.clone(), ErrorCode::ConversionOk);
    
    assert_eq!(response.result.level, StatusLevel::Info);
    assert!(matches!(response.result.code, ErrorCode::ConversionOk));
    assert_eq!(response.image_data, Some(image_data));
}

#[test]
fn test_convert_response_error() {
    let response = ConvertResponse::error(ErrorCode::ValidationEmpty);
    
    assert_eq!(response.result.level, StatusLevel::Warning);
    assert!(matches!(response.result.code, ErrorCode::ValidationEmpty));
    assert_eq!(response.image_data, None);
}

// ...existing code...

// ==================== ImageError Tests ====================

#[test]
fn test_image_error_wrong_format() {
    let error = ImageError::WrongFormat;
    assert_eq!(error.to_string(), "画像形式が正しくありません");
}

#[test]
fn test_image_error_invalid_png_header() {
    let error = ImageError::InvalidPngHeader;
    assert_eq!(error.to_string(), "無効なPNGヘッダーです");
}

#[test]
fn test_image_error_empty_data() {
    let error = ImageError::EmptyData;
    assert_eq!(error.to_string(), "画像データが空です");
}

#[test]
fn test_image_error_dimensions_too_large() {
    let error = ImageError::DimensionsTooLarge((9000, 9000));
    let error_str = error.to_string();
    assert!(error_str.contains("9000"));
    assert!(error_str.contains("8192"));
}

// ==================== StorageError Tests ====================

#[test]
fn test_storage_error_invalid_slot_number() {
    let error = StorageError::InvalidSlotNumber(15);
    let error_str = error.to_string();
    assert!(error_str.contains("15"));
    assert!(error_str.contains("1-10"));
}

#[test]
fn test_storage_error_slots_full() {
    let error = StorageError::SlotsFull;
    assert_eq!(error.to_string(), "スロットが満杯です (最大: 10)");
}

#[test]
fn test_storage_error_quota_exceeded() {
    let error = StorageError::QuotaExceeded;
    assert_eq!(error.to_string(), "LocalStorage容量超過 (上限: 5MB)");
}

#[test]
fn test_storage_error_slot_empty() {
    let error = StorageError::SlotEmpty(5);
    let error_str = error.to_string();
    assert!(error_str.contains("5"));
    assert!(error_str.contains("空です"));
}


