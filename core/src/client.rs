// PlantUML HTTP client
// Only compiled when "client" feature is enabled

use crate::models::{DiagramImage, DocumentId, GenerationResult, ImageFormat};
use std::time::Duration;
use plantuml_encoding::encode_plantuml_deflate;

/// PlantUML client for converting text to diagrams
pub struct PlantUmlClient {
    client: reqwest::Client,
    base_url: String,
}

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

impl PlantUmlClient {
    /// Create a new PlantUML client
    /// 
    /// # Arguments
    /// * `base_url` - PlantUML Picoweb server URL (e.g., "http://localhost:8081")
    pub fn new(base_url: String) -> Result<Self, ClientError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .no_proxy() // Disable proxy for localhost connections
            .build()?;
        
        Ok(Self { client, base_url })
    }
    
    /// Convert PlantUML text to PNG image
    /// 
    /// # Arguments
    /// * `document_id` - Document ID for tracking
    /// * `plantuml_text` - PlantUML source text
    /// 
    /// # Returns
    /// DiagramImage with PNG data or syntax error image
    pub async fn convert_to_png(
        &self,
        document_id: DocumentId,
        plantuml_text: &str,
    ) -> Result<DiagramImage, ClientError> {
        self.convert(document_id, plantuml_text, ImageFormat::Png).await
    }
    
    /// Convert PlantUML text to SVG image
    /// 
    /// # Arguments
    /// * `document_id` - Document ID for tracking
    /// * `plantuml_text` - PlantUML source text
    /// 
    /// # Returns
    /// DiagramImage with SVG data or syntax error image
    pub async fn convert_to_svg(
        &self,
        document_id: DocumentId,
        plantuml_text: &str,
    ) -> Result<DiagramImage, ClientError> {
        self.convert(document_id, plantuml_text, ImageFormat::Svg).await
    }
    
    /// Internal conversion method
    async fn convert(
        &self,
        document_id: DocumentId,
        plantuml_text: &str,
        format: ImageFormat,
    ) -> Result<DiagramImage, ClientError> {
        let endpoint = match format {
            ImageFormat::Png => "png",
            ImageFormat::Svg => "svg",
        };
        
        // Encode PlantUML text using deflate compression
        let encoded = encode_plantuml_deflate(plantuml_text)
            .map_err(|e| ClientError::EncodingError(format!("{:?}", e)))?;
        
        // Build URL with encoded text as path parameter
        let url = format!("{}/{}/{}", self.base_url, endpoint, encoded);
        
        // Send GET request (PlantUML Picoweb uses GET with encoded path)
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        // Get binary data
        // Note: PlantUML Picoweb returns HTTP 200 even for syntax errors,
        // with an error image (PNG/SVG containing "Syntax Error" message).
        // We accept all responses and let the client decide how to handle them.
        let data = response.bytes().await?.to_vec();
        
        // TODO: Extract actual dimensions from image data
        // For now, use placeholder values
        let dimensions = (800, 600);
        
        // TODO: Detect syntax error images
        // PlantUML returns PNG with error message for syntax errors
        let result = GenerationResult::Success;
        
        let generated_at = chrono::Utc::now().timestamp();
        
        Ok(DiagramImage {
            document_id,
            format,
            data,
            dimensions,
            generated_at,
            result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = PlantUmlClient::new("http://localhost:8081".to_string());
        assert!(client.is_ok());
    }
    
    // Note: Integration tests with mock server will be in tests/client_test.rs
}

