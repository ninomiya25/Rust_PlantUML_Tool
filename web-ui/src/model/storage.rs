// LocalStorage service for temporary save/load

use plantuml_editor_core::{
    PlantUMLDocument, StorageSlot, StorageError,
    ProcessResult, ErrorCode, StatusLevel,
};
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

/// Convert StorageError to ProcessResult
pub fn storage_error_to_result(error: &StorageError, _slot_number: Option<u8>) -> ProcessResult {
    let (level, code, context) = match error {
        StorageError::InvalidSlotNumber(_) | StorageError::SlotEmpty(_) => {
            (StatusLevel::Warning, ErrorCode::StorageReadError, None)
        }
        StorageError::SlotsFull => {
            (StatusLevel::Warning, ErrorCode::StorageSlotLimit, None)
        }
        StorageError::QuotaExceeded => {
            (StatusLevel::Warning, ErrorCode::StorageInputLimit, Some(serde_json::json!({
                "maxChars": 24000
            })))
        }
    };
    
    ProcessResult { level, code, context }
}

/// Create success ProcessResult for storage operations
pub fn storage_success_result(code: ErrorCode, slot_number: u8) -> ProcessResult {
    ProcessResult {
        level: StatusLevel::Info,
        code,
        context: Some(serde_json::json!({
            "slotNumber": slot_number
        })),
    }
}

/// Storage service for managing saved PlantUML documents
pub struct StorageService;

impl StorageService {
    pub fn new() -> Self {
        Self
    }
    
    /// Save text to slot
    pub fn save_to_slot(&self, slot_number: usize, text: &str) -> Result<(), StorageError> {
        let slot_number = slot_number as u8;
        StorageSlot::validate_slot_number(slot_number)?;
        
        let now = chrono::Utc::now().timestamp();
        let document = PlantUMLDocument {
            id: plantuml_editor_core::DocumentId::new(),
            content: text.to_string(),
            created_at: now,
            updated_at: now,
            title: None,
        };
        
        let slot = StorageSlot {
            slot_number,
            document,
            saved_at: chrono::Utc::now().timestamp(),
        };
        
        let key = StorageSlot::storage_key(slot_number);
        LocalStorage::set(&key, &slot)
            .map_err(|_| StorageError::QuotaExceeded)?;
        
        Ok(())
    }
    
    /// Load text from slot
    pub fn load_from_slot(&self, slot_number: usize) -> Result<Option<String>, StorageError> {
        let slot_number = slot_number as u8;
        StorageSlot::validate_slot_number(slot_number)?;
        
        let key = StorageSlot::storage_key(slot_number);
        match LocalStorage::get::<StorageSlot>(&key) {
            Ok(slot) => Ok(Some(slot.document.content)),
            Err(_) => Ok(None),
        }
    }
    
    /// List all saved slots
    pub fn list_slots(&self) -> Vec<SlotInfo> {
        let mut slots = Vec::new();
        
        for slot_number in 1..=StorageSlot::MAX_SLOTS {
            let key = StorageSlot::storage_key(slot_number);
            if let Ok(slot) = LocalStorage::get::<StorageSlot>(&key) {
                slots.push(SlotInfo {
                    slot_number,
                    title: slot.document.title.clone().unwrap_or_else(|| "無題".to_string()),
                    saved_at: slot.saved_at,
                    preview: get_preview(&slot.document.content),
                });
            }
        }
        
        slots
    }
    
    /// Delete slot
    pub fn delete_slot(&self, slot_number: usize) -> Result<(), StorageError> {
        let slot_number = slot_number as u8;
        StorageSlot::validate_slot_number(slot_number)?;
        
        let key = StorageSlot::storage_key(slot_number);
        LocalStorage::delete(&key);
        
        Ok(())
    }
}

/// Slot information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub slot_number: u8,
    pub title: String,
    pub saved_at: i64,
    pub preview: String,
}

fn get_preview(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let preview_lines = lines.iter().take(3).map(|s| *s).collect::<Vec<_>>();
    let preview = preview_lines.join("\n");
    
    if preview.len() > 100 {
        format!("{}...", &preview[..100])
    } else {
        preview
    }
}
