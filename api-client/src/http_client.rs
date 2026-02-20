// HTTP client for PlantUML API

use crate::errors::ApiError;
use plantuml_editor_core::{ConvertRequest, ConvertResponse, ImageFormat, ProcessResult};
use std::env;

fn get_api_base_url() -> String {
    env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
}


/// Convert PlantUML text to image via API server
///
/// # Arguments
/// * `plantuml_text` - PlantUML source code
/// * `format` - Output image format (PNG or SVG)
///
/// # Returns
/// Binary image data and processing result on success
pub async fn convert_plantuml(
    plantuml_text: String,
    format: ImageFormat,
) -> Result<(Vec<u8>, ProcessResult), ApiError> {
    let request = ConvertRequest {
        plantuml_text,
        format,
    };
    
    let client = reqwest::Client::new();
    let api_base_url = get_api_base_url();
    let response = client
        .post(format!("{}/api/v1/convert", api_base_url))
        .json(&request)
        .send()
        .await
        .map_err(|_| ApiError::NetworkError("サーバーが応答していません。時間をおいて再度接続を試すか管理者に問い合わせてください。".to_string()))?;
    
    if response.status().is_success() {
        let convert_response: ConvertResponse = response
            .json()
            .await
            .map_err(|_| ApiError::NetworkError("レスポンスの解析に失敗しました。".to_string()))?;
        
        // Check if conversion succeeded
        if let Some(image_data) = convert_response.image_data {
            Ok((image_data, convert_response.result))
        } else {
            // Server returned an error result
            Err(ApiError::from_process_result(convert_response.result))
        }
    } else {
        // HTTP error (should not happen with new API design, but keep for safety)
        Err(ApiError::ServerError(
            format!("HTTPエラー: {}", response.status())
        ))
    }
}

/// Export PlantUML diagram via API server
///
/// # Arguments
/// * `plantuml_text` - PlantUML source code
/// * `format` - Output image format (PNG or SVG)
///
/// # Returns
/// Binary image data and processing result on success
pub async fn export_plantuml(
    plantuml_text: String,
    format: ImageFormat,
) -> Result<(Vec<u8>, ProcessResult), ApiError> {
    let request = ConvertRequest {
        plantuml_text,
        format,
    };
    
    let client = reqwest::Client::new();
    let api_base_url = get_api_base_url();
    let response = client
        .post(format!("{}/api/v1/export", api_base_url))
        .json(&request)
        .send()
        .await
        .map_err(|_| ApiError::NetworkError("サーバーが応答していません。時間をおいて再度接続を試すか管理者に問い合わせてください。".to_string()))?;
    
    if response.status().is_success() {
        let convert_response: ConvertResponse = response
            .json()
            .await
            .map_err(|_| ApiError::NetworkError("レスポンスの解析に失敗しました。".to_string()))?;
        
        if let Some(image_data) = convert_response.image_data {
            Ok((image_data, convert_response.result))
        } else {
            Err(ApiError::from_process_result(convert_response.result))
        }
    } else {
        Err(ApiError::ServerError(
            format!("HTTPエラー: {}", response.status())
        ))
    }
}
