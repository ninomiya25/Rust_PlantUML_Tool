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


#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_editor_core::ErrorCode;

    // ========================================
    // ApiError::Display トレイトのテスト
    // ========================================

    #[test]
    fn test_api_error_display_network_error() {
        // NetworkError のDisplay実装をテスト
        // 期待される出力: "ネットワークエラー: {メッセージ}"
        let error = ApiError::NetworkError("接続タイムアウト".to_string());
        let display_string = format!("{}", error);
        
        assert_eq!(display_string, "ネットワークエラー: 接続タイムアウト");
    }

    #[test]
    fn test_api_error_display_server_error() {
        // ServerError のDisplay実装をテスト
        // 期待される出力: "サーバーエラー: {メッセージ}"
        let error = ApiError::ServerError("HTTPエラー: 500".to_string());
        let display_string = format!("{}", error);
        
        assert_eq!(display_string, "サーバーエラー: HTTPエラー: 500");
    }

    #[test]
    fn test_api_error_display_process_error_validation_empty() {
        // ProcessError (ValidationEmpty) のDisplay実装をテスト
        // ErrorCode::to_message() が正しく呼ばれることを確認
        let error = ApiError::ProcessError(ErrorCode::ValidationEmpty);
        let display_string = format!("{}", error);
        
        assert_eq!(
            display_string,
            "処理エラー: PlantUMLソースを入力してください"
        );
    }
}