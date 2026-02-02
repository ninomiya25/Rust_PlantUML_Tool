// Client errors

/// Client errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("ネットワークエラー: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("タイムアウト: PlantUMLサーバーが応答しません")]
    Timeout,
    
    #[error("PlantUMLサーバーエラー: {0}")]
    ServerError(String),
    
    #[error("無効なレスポンス形式")]
    InvalidResponse,
    
    #[error("エンコードエラー: {0}")]
    EncodingError(String),
}
