// API handlers

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use plantuml_editor_core::{
    ConvertRequest, ConvertResponse,
    ErrorCode, StatusLevel,
};
use plantuml_client::PlantUmlClient;
use serde_json::json;

/// GET /api/v1/health - Health check endpoint
pub async fn health() -> Response {
    let health_status = json!({
        "status": "healthy",
        "service": "plantuml-editor-api",
        "version": env!("CARGO_PKG_VERSION"),
    });
    
    (StatusCode::OK, Json(health_status)).into_response()
}

/// POST /api/v1/convert - Convert PlantUML text to image
pub async fn convert(Json(payload): Json<ConvertRequest>) -> Response {
    // Validate request
    if let Err(e) = payload.validate() {
        tracing::warn!("Validation failed: {}", e);
        let response = ConvertResponse::error(
            e.status_level(),
            e.to_error_code(),
            e.context(),
        );
        return (StatusCode::OK, Json(response)).into_response();
    }
    
    // Create PlantUML client
    let client = match PlantUmlClient::new("http://localhost:8081".to_string()) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create PlantUML client: {}", e);
            let response = ConvertResponse::error(
                StatusLevel::Error,
                ErrorCode::ServerError,
                None,
            );
            return (StatusCode::OK, Json(response)).into_response();
        }
    };
    
    // Convert PlantUML text to image
    let document_id = plantuml_editor_core::DocumentId::new();
    let result = match payload.format {
        plantuml_editor_core::ImageFormat::Png => {
            client.convert_to_png(document_id, &payload.plantuml_text).await
        }
        plantuml_editor_core::ImageFormat::Svg => {
            client.convert_to_svg(document_id, &payload.plantuml_text).await
        }
    };
    
    match result {
        Ok(image) => {
            tracing::info!("PlantUML conversion successful: {} bytes", image.data.len());
            let response = ConvertResponse::success(image.data);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("PlantUML conversion failed: {}", e);
            
            // Determine error code based on error type
            // Only distinguish encoding errors; all other errors become ParseError
            let error_code = if e.to_string().contains("エンコードエラー") {
                ErrorCode::EncodingError
            } else {
                // Network, Timeout, ServerError, InvalidResponse → all become ParseError
                ErrorCode::ParseError
            };
            
            let response = ConvertResponse::error(
                StatusLevel::Error,
                error_code,
                None,
            );
            (StatusCode::OK, Json(response)).into_response()
        }
    }
}

/// POST /api/v1/export - Export PlantUML diagram
pub async fn export(Json(payload): Json<ConvertRequest>) -> Response {
    // Validate request
    if let Err(e) = payload.validate() {
        tracing::warn!("Export validation failed: {}", e);
        let response = ConvertResponse::error(
            e.status_level(),
            e.to_error_code(),
            e.context(),
        );
        return (StatusCode::OK, Json(response)).into_response();
    }
    
    // Create PlantUML client
    let client = match PlantUmlClient::new("http://localhost:8081".to_string()) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create PlantUML client for export: {}", e);
            let response = ConvertResponse::error(
                StatusLevel::Error,
                ErrorCode::ServerError,
                None,
            );
            return (StatusCode::OK, Json(response)).into_response();
        }
    };
    
    // Convert PlantUML text to image
    let document_id = plantuml_editor_core::DocumentId::new();
    let result = match payload.format {
        plantuml_editor_core::ImageFormat::Png => {
            client.convert_to_png(document_id, &payload.plantuml_text).await
        }
        plantuml_editor_core::ImageFormat::Svg => {
            client.convert_to_svg(document_id, &payload.plantuml_text).await
        }
    };
    
    match result {
        Ok(image) => {
            tracing::info!("PlantUML export successful: {} bytes", image.data.len());
            // Return ExportOk instead of ConversionOk
            let mut response = ConvertResponse::success(image.data);
            response.result.code = ErrorCode::ExportOk;
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("PlantUML export failed: {}", e);
            
            // Determine error code based on error type
            // Only distinguish encoding errors; all other errors become ExportError
            let error_code = if e.to_string().contains("エンコードエラー") {
                ErrorCode::EncodingError
            } else {
                // Network, Timeout, ServerError, InvalidResponse → all become ExportError
                ErrorCode::ExportError
            };
            
            let response = ConvertResponse::error(
                StatusLevel::Error,
                error_code,
                None,
            );
            (StatusCode::OK, Json(response)).into_response()
        }
    }
}

