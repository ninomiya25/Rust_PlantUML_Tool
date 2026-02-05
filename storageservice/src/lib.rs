// Storage service library for PlantUML Editor
//
// This crate provides storage abstraction with pluggable backends

use plantuml_editor_core::{StorageError, ProcessResult, ErrorCode};
use serde::{Deserialize, Serialize};

// Re-export local storage backend
pub mod local;
pub use local::LocalStorageBackend;

/// Slot information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub slot_number: u8,
    pub title: String,
    pub saved_at: i64,
    pub preview: String,
}

/// Storage backend trait
pub trait StorageBackend {
    fn save_to_slot(&self, slot_number: usize, text: &str) -> Result<(), StorageError>;
    fn load_from_slot(&self, slot_number: usize) -> Result<Option<String>, StorageError>;
    fn list_slots(&self) -> Vec<SlotInfo>;
    fn delete_slot(&self, slot_number: usize) -> Result<(), StorageError>;
}

/// Storage service with pluggable backend
pub struct StorageService<B: StorageBackend> {
    backend: B,
}

impl<B: StorageBackend> StorageService<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    
    pub fn save_to_slot(&self, slot_number: usize, text: &str) -> Result<(), StorageError> {
        self.backend.save_to_slot(slot_number, text)
    }
    
    pub fn load_from_slot(&self, slot_number: usize) -> Result<Option<String>, StorageError> {
        self.backend.load_from_slot(slot_number)
    }
    
    pub fn list_slots(&self) -> Vec<SlotInfo> {
        self.backend.list_slots()
    }
    
    pub fn delete_slot(&self, slot_number: usize) -> Result<(), StorageError> {
        self.backend.delete_slot(slot_number)
    }
}

/// Convert StorageError to ProcessResult
pub fn storage_error_to_result(error: &StorageError, _slot_number: Option<u8>) -> ProcessResult {
    let code = match error {
        StorageError::InvalidSlotNumber(slot) | StorageError::SlotEmpty(slot) => {
            ErrorCode::StorageReadError {
                reason: format!("スロット{}は無効または空です", slot),
            }
        }
        StorageError::SlotsFull => {
            ErrorCode::StorageSlotLimit {
                max_slots: 10,
            }
        }
        StorageError::QuotaExceeded => {
            ErrorCode::StorageInputLimit {
                actual: 0, // Not available in this context
                max: 24000,
            }
        }
    };
    
    ProcessResult::error(code)
}

/// Create success ProcessResult for storage operations
pub fn storage_success_result(code: ErrorCode, _slot_number: u8) -> ProcessResult {
    ProcessResult::success(code)
}
