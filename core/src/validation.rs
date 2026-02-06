// Validation logic for PlantUML content

use crate::models::{ErrorCode, StatusLevel};

/// Validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("コンテンツが空です")]
    EmptyContent,

    #[error("コンテンツが大きすぎます: {0}文字 (上限: {1}文字)")]
    ContentTooLarge(usize, usize),
}

impl ValidationError {
    /// Convert to ErrorCode with embedded data
    pub fn to_error_code(&self) -> ErrorCode {
        match self {
            ValidationError::EmptyContent => ErrorCode::ValidationEmpty,
            ValidationError::ContentTooLarge(actual, max) => ErrorCode::ValidationTextLimit {
                actual: *actual,
                max: *max,
            },
        }
    }

    /// Get status level for this validation error
    pub fn status_level(&self) -> StatusLevel {
        StatusLevel::Warning
    }
}

/// Validate PlantUML content
///
/// # Rules
/// - Content must not be empty
/// - Content must be within 24,000 character limit (300 lines × 80 chars/line)
///
/// Note: @startuml/@enduml tags are NOT validated here.
/// PlantUML.jar will generate an error image if tags are missing.
pub fn validate_plantuml_content(content: &str) -> Result<(), ValidationError> {
    // Empty check
    if content.trim().is_empty() {
        return Err(ValidationError::EmptyContent);
    }

    // Character limit check (300 lines × 80 chars/line = 24,000 chars)
    const MAX_CHARS: usize = 24_000;
    if content.len() > MAX_CHARS {
        return Err(ValidationError::ContentTooLarge(content.len(), MAX_CHARS));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_conversions() {
        // EmptyContent
        let error = ValidationError::EmptyContent;
        assert!(matches!(error.to_error_code(), ErrorCode::ValidationEmpty));
        assert_eq!(error.status_level(), StatusLevel::Warning);

        // ContentTooLarge
        let error = ValidationError::ContentTooLarge(25000, 24000);
        match error.to_error_code() {
            ErrorCode::ValidationTextLimit { actual, max } => {
                assert_eq!(actual, 25000);
                assert_eq!(max, 24000);
            }
            _ => panic!("Expected ValidationTextLimit"),
        }
        assert_eq!(error.status_level(), StatusLevel::Warning);
    }

    #[test]
    fn test_valid_plantuml() {
        let content = "@startuml\nAlice -> Bob: Hello\n@enduml";
        assert!(validate_plantuml_content(content).is_ok());
    }

    #[test]
    fn test_empty_content() {
        let content = "   ";
        assert!(matches!(
            validate_plantuml_content(content),
            Err(ValidationError::EmptyContent)
        ));
    }

    #[test]
    fn test_missing_tags_allowed() {
        // Tags are not validated - PlantUML.jar will handle this
        let content = "Alice -> Bob: Hello";
        assert!(validate_plantuml_content(content).is_ok());
    }

    #[test]
    fn test_content_too_large() {
        let content = format!("@startuml\n{}\n@enduml", "x".repeat(25000));
        assert!(matches!(
            validate_plantuml_content(&content),
            Err(ValidationError::ContentTooLarge(_, _))
        ));
    }
}
