// Web UI error types

/// Web UI specific errors
#[derive(Debug, Clone)]
pub enum UiError {
    /// Storage operation failed
    StorageError(String),
    /// Component rendering error
    RenderError(String),
}

impl std::fmt::Display for UiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiError::StorageError(msg) => write!(f, "ストレージエラー: {}", msg),
            UiError::RenderError(msg) => write!(f, "描画エラー: {}", msg),
        }
    }
}

impl std::error::Error for UiError {}
