// Client errors

/// Client errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// タイムアウト・ネットワークエラーなどの通信障害、PlantUML サーバーが HTTP エラーを返した場合を含む
    #[error("ネットワークエラー: {0}")]
    Network(#[from] reqwest::Error),
    
    /// エンコード処理で発生したエラー
    #[error("エンコードエラー: {0}")]
    EncodingError(String),
}
