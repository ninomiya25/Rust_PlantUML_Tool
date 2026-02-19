// Core data models for PlantUML Editor

use serde::{Deserialize, Serialize};

/// Document ID (UUID v4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub uuid::Uuid);

impl DocumentId {
    /// Generate a new random document ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

/// PlantUML document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantUMLDocument {
    /// Unique document identifier
    pub id: DocumentId,
    
    /// PlantUML text content
    pub content: String,
    
    /// Creation timestamp (Unix timestamp)
    pub created_at: i64,
    
    /// Last update timestamp (Unix timestamp)
    pub updated_at: i64,
    
    /// Optional title (user input)
    pub title: Option<String>,
}

impl PlantUMLDocument {
    /// Create a new document with given content
    pub fn new(content: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: DocumentId::new(),
            content,
            created_at: now,
            updated_at: now,
            title: None,
        }
    }
    
    /// Validate document content
    pub fn validate(&self) -> Result<(), crate::validation::ValidationError> {
        crate::validation::validate_plantuml_content(&self.content)
    }
}

/// Image format for diagram output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Svg,
}

/// Status level for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StatusLevel {
    /// 処理が正常に完了
    Info,
    /// 問題が発生したが処理は続行可能
    Warning,
    /// エラーが発生し処理が失敗
    Error,
}

/// Error codes for processing results (Algebraic Data Type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")] 
pub enum ErrorCode {
    // 正常完了 (INFO)
    ConversionOk,
    ExportOk,
    
    // データ付き成功メッセージ (INFO)
    SaveSuccess { 
        slot_number: u8 
    },
    LoadSuccess { 
        slot_number: u8 
    },
    DeleteSuccess { 
        slot_number: u8 
    },
    
    // バリデーションエラー (WARNING)
    ValidationEmpty,
    ValidationTextLimit { 
        actual: usize, 
        max: usize 
    },
    
    // ストレージエラー (WARNING/ERROR)
    StorageInputLimit { 
        actual: usize, 
        max: usize 
    },
    StorageSlotLimit { 
        max_slots: usize 
    },
    StorageWriteError { 
        reason: String 
    },
    StorageReadError { 
        reason: String 
    },
    StorageDeleteError { 
        reason: String 
    },
    
    // 処理エラー (ERROR)
    SizeLimit { 
        actual_bytes: usize, 
        max_bytes: usize 
    },
    EncodingError { 
        encoding: String 
    },
    ParseError { 
        line: Option<usize> 
    },
    ExportError { 
        format: String 
    },
    
    // サーバー・ネットワークエラー (ERROR)
    ServerError { 
        message: String 
    },
    TimeoutError { 
        duration_ms: u64 
    },
    NetworkError { 
        endpoint: String 
    },
}

impl ErrorCode {
    /// Get user-friendly message from ErrorCode
    pub fn to_message(&self) -> String {
        match self {
            // 成功系 (INFO)
            Self::ConversionOk => "図が正常に生成されました".to_string(),
            Self::ExportOk => "図が正常にエクスポートされました".to_string(),
            
            Self::SaveSuccess { slot_number } => {
                format!("PlantUMLソースをスロット{}に保存しました", slot_number)
            }
            Self::LoadSuccess { slot_number } => {
                format!("スロット{}からPlantUMLソースを読み込みました", slot_number)
            }
            Self::DeleteSuccess { slot_number } => {
                format!("スロット{}のデータを削除しました", slot_number)
            }
            
            // バリデーションエラー (WARNING)
            Self::ValidationEmpty => "PlantUMLソースを入力してください".to_string(),
            Self::ValidationTextLimit { actual, max } => {
                format!(
                    "PlantUMLソースが長すぎます。文字数を{}文字以内に減らしてください（現在: {}文字）",
                    max, actual
                )
            }
            
            // ストレージエラー (WARNING/ERROR)
            Self::StorageInputLimit { actual, max } => {
                format!(
                    "保存する内容の文字数が上限({}文字)を超えています。内容を短縮してください（現在: {}文字）",
                    max, actual
                )
            }
            Self::StorageSlotLimit { max_slots } => {
                format!(
                    "一時保存上限に達しています（最大{}個）。既存のスロットを削除してから保存してください",
                    max_slots
                )
            }
            Self::StorageWriteError { reason } => {
                format!("ローカルストレージへの保存に失敗しました。{}", reason)
            }
            Self::StorageReadError { reason } => {
                format!("ローカルストレージからの読み込みに失敗しました。{}", reason)
            }
            Self::StorageDeleteError { reason } => {
                format!("ローカルストレージのデータ削除に失敗しました。{}", reason)
            }
            
            // 処理エラー (ERROR)
            Self::SizeLimit { actual_bytes, max_bytes } => {
                format!(
                    "画像サイズが上限を超えています（現在: {} bytes、上限: {} bytes）。'scale'でサイズを縮小するか、図を分割してください",
                    actual_bytes, max_bytes
                )
            }
            Self::EncodingError { encoding } => {
                format!(
                    "PlantUMLソースの変換に失敗しました（エンコーディング: {}）。文字コードや特殊文字が含まれていないかご確認ください",
                    encoding
                )
            }
            Self::ParseError { line } => {
                if let Some(line_num) = line {
                    format!("PlantUMLの処理中にエラーが発生しました（行: {}）。管理者へお問い合わせください", line_num)
                } else {
                    "PlantUMLの処理中にエラーが発生しました。管理者へお問い合わせください".to_string()
                }
            }
            Self::ExportError { format } => {
                format!("ファイルのエクスポートに失敗しました（形式: {}）。再度お試しください", format)
            }
            
            // サーバー・ネットワークエラー (ERROR)
            Self::ServerError { message } => {
                format!("サーバーエラー: {}。時間をおいて再度接続を試すか管理者に問い合わせてください", message)
            }
            Self::TimeoutError { duration_ms } => {
                format!(
                    "通信がタイムアウトしました（{}ms）。ネットワーク状況をご確認のうえ、再度お試しください",
                    duration_ms
                )
            }
            Self::NetworkError { endpoint } => {
                format!("ネットワーク接続に失敗しました（エンドポイント: {}）。インターネット接続をご確認ください", endpoint)
            }
        }
    }
    
    /// Get status level for this error code
    pub fn status_level(&self) -> StatusLevel {
        match self {
            // INFO
            Self::ConversionOk 
            | Self::ExportOk 
            | Self::SaveSuccess { .. } 
            | Self::LoadSuccess { .. } 
            | Self::DeleteSuccess { .. } => StatusLevel::Info,
            
            // WARNING
            Self::ValidationEmpty 
            | Self::ValidationTextLimit { .. } 
            | Self::StorageInputLimit { .. } 
            | Self::StorageSlotLimit { .. } 
            | Self::SizeLimit { .. } => StatusLevel::Warning,
            
            // ERROR
            _ => StatusLevel::Error,
        }
    }
}

/// Processing result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Status level (INFO/WARNING/ERROR)
    pub level: StatusLevel,
    
    /// Error code (contains all necessary data)
    pub code: ErrorCode,
}

impl ProcessResult {
    /// Create a ProcessResult with automatic status level determination
    pub fn new(code: ErrorCode) -> Self {
        let level = code.status_level();
        Self { level, code }
    }
    
    /// Get user-friendly message
    pub fn message(&self) -> String {
        self.code.to_message()
    }
}

/// Diagram image data with metadata
#[derive(Debug, Clone)]
pub struct DiagramImage {
    /// Source PlantUML document ID
    pub document_id: DocumentId,
    
    /// Image format
    pub format: ImageFormat,
    
    /// Binary image data
    pub data: Vec<u8>,
    
    /// Image dimensions (width, height)
    pub dimensions: (u32, u32),
    
    /// Generation timestamp (Unix timestamp)
    pub generated_at: i64,
}

impl DiagramImage {
    /// Validate PNG image
    pub fn validate_png(&self) -> Result<(), ImageError> {
        if self.format != ImageFormat::Png {
            return Err(ImageError::WrongFormat);
        }
        
        // Check PNG header (89 50 4E 47)
        const PNG_HEADER: &[u8] = &[0x89, 0x50, 0x4E, 0x47];
        if !self.data.starts_with(PNG_HEADER) {
            return Err(ImageError::InvalidPngHeader);
        }
        
        // Check data size
        if self.data.is_empty() {
            return Err(ImageError::EmptyData);
        }
        
        // Check max dimensions (8192 x 8192)
        const MAX_DIMENSION: u32 = 8192;
        if self.dimensions.0 > MAX_DIMENSION || self.dimensions.1 > MAX_DIMENSION {
            return Err(ImageError::DimensionsTooLarge(self.dimensions));
        }
        
        Ok(())
    }
    
    /// Convert to Data URL format (for img src attribute)
    pub fn to_data_url(&self) -> String {
        let mime_type = match self.format {
            ImageFormat::Png => "image/png",
            ImageFormat::Svg => "image/svg+xml",
        };
        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&self.data);
        format!("data:{};base64,{}", mime_type, base64_data)
    }
}

/// Image-related errors
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("画像形式が正しくありません")]
    WrongFormat,
    
    #[error("無効なPNGヘッダーです")]
    InvalidPngHeader,
    
    #[error("画像データが空です")]
    EmptyData,
    
    #[error("画像サイズが大きすぎます: {0:?} (上限: 8192x8192)")]
    DimensionsTooLarge((u32, u32)),
}

/// LocalStorage temporary save slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSlot {
    /// Slot number (1-10)
    pub slot_number: u8,
    
    /// Saved document
    pub document: PlantUMLDocument,
    
    /// Save timestamp (Unix timestamp)
    pub saved_at: i64,
}

impl StorageSlot {
    pub const MAX_SLOTS: u8 = 10;
    
    /// Validate slot number
    pub fn validate_slot_number(slot_number: u8) -> Result<(), StorageError> {
        if !(1..=Self::MAX_SLOTS).contains(&slot_number) {
            return Err(StorageError::InvalidSlotNumber(slot_number));
        }
        Ok(())
    }
    
    /// Generate LocalStorage key
    pub fn storage_key(slot_number: u8) -> String {
        format!("plantuml_slot_{}", slot_number)
    }
}

/// Storage-related errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("無効なスロット番号です: {0} (有効範囲: 1-10)")]
    InvalidSlotNumber(u8),
    
    #[error("スロットが満杯です (最大: 10)")]
    SlotsFull,
    
    #[error("LocalStorage容量超過 (上限: 5MB)")]
    QuotaExceeded,
    
    #[error("スロット{0}は空です")]
    SlotEmpty(u8),
}

/// API Request: POST /api/v1/convert
#[derive(Debug, Serialize, Deserialize)]
pub struct ConvertRequest {
    /// PlantUML text content
    pub plantuml_text: String,
    
    /// Output image format
    pub format: ImageFormat,
}

impl ConvertRequest {
    /// Validate request
    pub fn validate(&self) -> Result<(), crate::validation::ValidationError> {
        let doc = PlantUMLDocument::new(self.plantuml_text.clone());
        doc.validate()
    }
}

/// API Response: POST /api/v1/convert
#[derive(Debug, Serialize, Deserialize)]
pub struct ConvertResponse {
    /// Processing result information
    pub result: ProcessResult,
    
    /// Binary image data (optional, only present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<Vec<u8>>,
}

impl ConvertResponse {
    /// Create success response with image data
    pub fn success(image_data: Vec<u8>) -> Self {
        Self {
            result: ProcessResult::new(ErrorCode::ConversionOk),
            image_data: Some(image_data),
        }
    }
    
    /// Create error response without image data
    pub fn error(code: ErrorCode) -> Self {
        Self {
            result: ProcessResult::new(code),
            image_data: None,
        }
    }
}

