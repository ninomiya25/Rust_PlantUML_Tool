// API client errors

use plantuml_editor_core::{ProcessResult, ErrorCode, StatusLevel};

/// API client error types
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Network communication error
    NetworkError(String),
    /// Server returned an error response
    ServerError(String),
    /// Validation error (4xx status codes)
    ValidationError(String),
    /// Processing error with code
    ProcessError { code: ErrorCode, level: StatusLevel, context: Option<serde_json::Value> },
}

impl ApiError {
    /// Convert from ProcessResult
    pub fn from_process_result(result: ProcessResult) -> Self {
        ApiError::ProcessError {
            code: result.code,
            level: result.level,
            context: result.context,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "ネットワークエラー: {}", msg),
            ApiError::ServerError(msg) => write!(f, "サーバーエラー: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "入力エラー: {}", msg),
            ApiError::ProcessError { code, .. } => write!(f, "処理エラー: {:?}", code),
        }
    }
}

impl std::error::Error for ApiError {}
