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

/// Error codes for processing results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    // 正常完了
    ConversionOk,
    
    // バリデーションエラー (WARNING)
    ValidationEmpty,
    ValidationTextLimit,
    
    // 処理エラー (ERROR)
    SizeLimit,
    EncodingError,
    ParseError,
    
    // サーバー・ネットワークエラー (ERROR)
    ServerError,
    TimeoutError,
    NetworkError,
    
    // エクスポートエラー (ERROR)
    ExportError,
    
    // ストレージエラー (WARNING/ERROR)
    StorageInputLimit,
    StorageSlotLimit,
    StorageWriteError,
    StorageReadError,
    StorageDeleteError,
    
    // 成功メッセージ (INFO)
    ExportOk,
    SaveSuccess,
    LoadSuccess,
    DeleteSuccess,
}

/// Processing result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Status level (INFO/WARNING/ERROR)
    pub level: StatusLevel,
    
    /// Error code
    pub code: ErrorCode,
    
    /// Optional additional context (e.g., slot number, max length)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

/// Generation result status
#[derive(Debug, Clone)]
pub enum GenerationResult {
    /// Success (valid diagram)
    Success,
    
    /// Syntax error (error image generated)
    SyntaxError { message: String },
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
    
    /// Generation result
    pub result: GenerationResult,
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
        if slot_number < 1 || slot_number > Self::MAX_SLOTS {
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
            result: ProcessResult {
                level: StatusLevel::Info,
                code: ErrorCode::ConversionOk,
                context: None,
            },
            image_data: Some(image_data),
        }
    }
    
    /// Create error response without image data
    pub fn error(level: StatusLevel, code: ErrorCode, context: Option<serde_json::Value>) -> Self {
        Self {
            result: ProcessResult {
                level,
                code,
                context,
            },
            image_data: None,
        }
    }
}

/// API Error Response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message (Constitution UX principle: what/why/how)
    pub error: String,
    
    /// Optional error details (for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    /// System error
    pub fn system_error() -> Self {
        Self {
            error: "システムエラーが発生しました。PlantUMLサーバーの状態を確認してください。".to_string(),
            details: None,
        }
    }
    
    /// Network error
    pub fn network_error() -> Self {
        Self {
            error: "ネットワークエラーが発生しました。接続を確認して再試行してください。".to_string(),
            details: None,
        }
    }
    
    /// Validation error
    pub fn validation_error(message: String) -> Self {
        Self {
            error: format!("入力エラー: {}。PlantUML構文を確認してください。", message),
            details: None,
        }
    }
}

