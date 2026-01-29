// Save button component

use yew::prelude::*;
use plantuml_editor_core::StorageError;

/// Validation result for save operation
pub enum SaveValidationError {
    EmptyContent,
    ContentTooLarge(usize),
    StorageError(StorageError),
}

#[derive(Properties, PartialEq)]
pub struct SaveButtonProps {
    pub plantuml_text: String,
    pub on_save: Callback<usize>,
    pub on_error: Callback<SaveValidationError>,
}

#[function_component(SaveButton)]
pub fn save_button(props: &SaveButtonProps) -> Html {
    let on_click = {
        let plantuml_text = props.plantuml_text.clone();
        let on_save = props.on_save.clone();
        let on_error = props.on_error.clone();
        
        Callback::from(move |_| {
           use crate::model::storage::StorageService;
        
           // Validate PlantUML text before saving
           // Rule 1: Not empty or whitespace only
           if plantuml_text.trim().is_empty() {
               on_error.emit(SaveValidationError::EmptyContent);
               return;
           }
           
           // Rule 2: Max 24,000 characters
           const MAX_CHARS: usize = 24_000;
           if plantuml_text.len() > MAX_CHARS {
               on_error.emit(SaveValidationError::ContentTooLarge(plantuml_text.len()));
               return;
           }
           
           let service = StorageService::new();
        
            // 空きスロットを探す
        for slot_num in 1..=10 {
            if let Ok(None) = service.load_from_slot(slot_num) {
                // このスロットは空いている
                on_save.emit(slot_num);
                return;
            }
        }
        
        // 全スロット埋まっている場合 - エラーを通知
        on_error.emit(SaveValidationError::StorageError(StorageError::SlotsFull));
        })
    };
    
    html! {
        <button 
            class="save-btn"
            onclick={on_click}
        >
            {"一時保存"}
        </button>
    }
}
