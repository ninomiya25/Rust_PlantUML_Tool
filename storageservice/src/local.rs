// LocalStorage backend implementation

use super::{StorageBackend, SlotInfo};
use plantuml_editor_core::StorageError;

#[cfg(target_arch = "wasm32")]
use plantuml_editor_core::{PlantUMLDocument, DocumentId, StorageSlot};

/// LocalStorage backend for browser-based storage
#[derive(Default)]
pub struct LocalStorageBackend;

impl LocalStorageBackend {
    pub fn new() -> Self {
        Self
    }
}

// WASM implementation using gloo-storage
#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::*;
    use gloo_storage::{LocalStorage, Storage};

    impl StorageBackend for LocalStorageBackend {
        fn save_to_slot(&self, slot_number: usize, text: &str) -> Result<(), StorageError> {
            let slot_number = slot_number as u8;
            StorageSlot::validate_slot_number(slot_number)?;
            
            let now = chrono::Utc::now().timestamp();
            let document = PlantUMLDocument {
                id: DocumentId::new(),
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
        
        fn load_from_slot(&self, slot_number: usize) -> Result<Option<String>, StorageError> {
            let slot_number = slot_number as u8;
            StorageSlot::validate_slot_number(slot_number)?;
            
            let key = StorageSlot::storage_key(slot_number);
            match LocalStorage::get::<StorageSlot>(&key) {
                Ok(slot) => Ok(Some(slot.document.content)),
                Err(_) => Ok(None),
            }
        }
        
        fn list_slots(&self) -> Vec<SlotInfo> {
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
        
        fn delete_slot(&self, slot_number: usize) -> Result<(), StorageError> {
            let slot_number = slot_number as u8;
            StorageSlot::validate_slot_number(slot_number)?;
            
            let key = StorageSlot::storage_key(slot_number);
            LocalStorage::delete(&key);
            
            Ok(())
        }
    }

    pub(super) fn get_preview(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let preview_lines = lines.iter().take(3).copied().collect::<Vec<_>>();
        let preview = preview_lines.join("\n");
        
        if preview.len() > 100 {
            format!("{}...", &preview[..100])
        } else {
            preview
        }
    }
}

// Stub implementation for non-WASM targets (for compilation purposes)
#[cfg(not(target_arch = "wasm32"))]
impl StorageBackend for LocalStorageBackend {
    fn save_to_slot(&self, _slot_number: usize, _text: &str) -> Result<(), StorageError> {
        panic!("LocalStorageBackend is only available on WASM targets")
    }
    
    fn load_from_slot(&self, _slot_number: usize) -> Result<Option<String>, StorageError> {
        panic!("LocalStorageBackend is only available on WASM targets")
    }
    
    fn list_slots(&self) -> Vec<SlotInfo> {
        panic!("LocalStorageBackend is only available on WASM targets")
    }
    
    fn delete_slot(&self, _slot_number: usize) -> Result<(), StorageError> {
        panic!("LocalStorageBackend is only available on WASM targets")
    }
}
