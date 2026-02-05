// API client errors

use plantuml_editor_core::{ProcessResult, ErrorCode};

/// API client error types
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Network communication error
    NetworkError(String),
    /// Server returned an error response
    ServerError(String),
    /// Processing error with code
    ProcessError(ErrorCode),
}

impl ApiError {
    /// Convert from ProcessResult
    pub fn from_process_result(result: ProcessResult) -> Self {
        ApiError::ProcessError(result.code)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "ネットワークエラー: {}", msg),
            ApiError::ServerError(msg) => write!(f, "サーバーエラー: {}", msg),
            ApiError::ProcessError(code) => write!(f, "処理エラー: {}", code.to_message()),
        }
    }
}

impl std::error::Error for ApiError {}
