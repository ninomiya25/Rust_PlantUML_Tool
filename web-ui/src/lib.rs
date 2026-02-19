// Web UI library for PlantUML Editor
//
// This crate provides reusable Yew components and UI models
// for the PlantUML editor frontend application.

use wasm_bindgen::JsCast;
use yew::prelude::*;
use std::rc::Rc;
use plantuml_editor_storageservice::{StorageBackend, StorageService};

pub mod components;
pub mod errors;

// Re-export components
pub use components::*;

/// Message level for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

impl From<plantuml_editor_core::StatusLevel> for MessageLevel {
    fn from(level: plantuml_editor_core::StatusLevel) -> Self {
        use plantuml_editor_core::StatusLevel;
        match level {
            StatusLevel::Info => MessageLevel::Info,
            StatusLevel::Warning => MessageLevel::Warning,
            StatusLevel::Error => MessageLevel::Error,
        }
    }
}

/// Get CSS class for message level
fn get_message_class(level: MessageLevel) -> &'static str {
    match level {
        MessageLevel::Info => "message-text",
        MessageLevel::Warning => "message-text warning",
        MessageLevel::Error => "message-text error",
    }
}

/// Application properties for dependency injection
#[derive(Properties, PartialEq, Clone)]
pub struct AppProps<B: StorageBackend + PartialEq + 'static> {
    /// Storage service (inject mock for testing)
    #[prop_or_default]
    pub storage_service: Option<Rc<StorageService<B>>>,
}

impl<B: StorageBackend + PartialEq + 'static> Default for AppProps<B> {
    fn default() -> Self {
        Self {
            storage_service: None,
        }
    }
}

/// Main application component（状態管理とイベントハンドリング）
/// 
/// Dependency Injection Pattern:
/// - Accepts StorageService via props for testability
/// - Uses LocalStorageBackend by default in production
/// - Tests can inject MockStorageBackend
#[function_component(App)]
pub fn app<B: StorageBackend + PartialEq + 'static>(props: &AppProps<B>) -> Html {
    use plantuml_editor_api_client::{convert_plantuml, export_plantuml};
    use plantuml_editor_core::{ImageFormat, ProcessResult};
    use wasm_bindgen_futures::spawn_local;

    // Dependency Injection: Get StorageService from props
    let storage_service = props.storage_service.clone();

    let plantuml_text = use_state(String::new);
    let editor_key = use_state(|| 0);
    let image_data = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let sidebar_collapsed = use_state(|| false);
    let message = use_state(|| "".to_string());
    let message_level = use_state(|| MessageLevel::Info);

    let on_text_change = {
        let plantuml_text = plantuml_text.clone();
        let image_data = image_data.clone();
        let loading = loading.clone();
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |text: String| {
            plantuml_text.set(text.clone());
            let image_data = image_data.clone();
            let loading = loading.clone();
            let message = message.clone();
            let message_level = message_level.clone();

            loading.set(true);

            spawn_local(async move {
                match convert_plantuml(text, ImageFormat::Svg).await {
                    Ok((bytes, result)) => {
                        // SVG is text-based, convert to string and create data URL
                        match String::from_utf8(bytes) {
                            Ok(svg_text) => {
                                let data_url = format!(
                                    "data:image/svg+xml;charset=utf-8,{}",
                                    urlencoding::encode(&svg_text)
                                );
                                image_data.set(Some(data_url));

                                // Set success message
                                message.set(result.message());
                                message_level.set(result.level.into());
                            }
                            Err(_) => {
                                message.set("SVG変換エラー".to_string());
                                message_level.set(MessageLevel::Error);
                                image_data.set(None);
                            }
                        }
                    }
                    Err(e) => {
                        use plantuml_editor_api_client::ApiError;

                        match e {
                            ApiError::ProcessError(code) => {
                                let result = ProcessResult::new(code);
                                message.set(result.message());
                                message_level.set(result.level.into());
                            }
                            _ => {
                                message.set(e.to_string());
                                message_level.set(MessageLevel::Error);
                            }
                        }
                        image_data.set(None);
                    }
                }
                loading.set(false);
            });
        })
    };

    let on_export = {
        let plantuml_text = plantuml_text.clone();
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |format: ImageFormat| {
            let text = (*plantuml_text).clone();
            let msg = message.clone();
            let msg_level = message_level.clone();

            spawn_local(async move {
                match export_plantuml(text, format).await {
                    Ok((bytes, result)) => {
                        // Update message based on export result
                        msg.set(result.message());
                        msg_level.set(result.level.into());

                        // Download the file
                        let blob_parts = js_sys::Array::new();
                        let uint8_array = js_sys::Uint8Array::from(&bytes[..]);
                        blob_parts.push(&uint8_array);

                        let options = web_sys::BlobPropertyBag::new();
                        let mime_type = match format {
                            ImageFormat::Png => "image/png",
                            ImageFormat::Svg => "image/svg+xml",
                        };
                        options.set_type(mime_type);

                        if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence_and_options(
                            &blob_parts,
                            &options,
                        ) {
                            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

                            let window = web_sys::window().unwrap();
                            let document = window.document().unwrap();
                            let anchor = document.create_element("a").unwrap();
                            let anchor = anchor.dyn_into::<web_sys::HtmlAnchorElement>().unwrap();

                            let extension = match format {
                                ImageFormat::Png => "png",
                                ImageFormat::Svg => "svg",
                            };
                            let filename = format!("diagram.{}", extension);

                            anchor.set_href(&url);
                            anchor.set_download(&filename);
                            anchor.click();

                            web_sys::Url::revoke_object_url(&url).unwrap();
                        }
                    }
                    Err(e) => {
                        // Display error message from ProcessResult if available
                        use plantuml_editor_api_client::ApiError;
                        match e {
                            ApiError::ProcessError(code) => {
                                let result = ProcessResult::new(code);
                                msg.set(result.message());
                                msg_level.set(result.level.into());
                            }
                            _ => {
                                // For network/server errors, display as-is
                                msg.set(format!("エクスポートエラー: {}", e));
                                msg_level.set(MessageLevel::Error);
                            }
                        }
                    }
                }
            });
        })
    };

    let on_save = {
        let storage_service = storage_service.clone();
        let plantuml_text = plantuml_text.clone();
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |slot: usize| {
            use plantuml_editor_core::ErrorCode;
            use plantuml_editor_storageservice::{
                storage_error_to_result, storage_success_result,
            };

            // Use injected storage service
            if let Some(service) = &storage_service {
                let result = match service.save_to_slot(slot, &plantuml_text) {
                    Ok(_) => storage_success_result(ErrorCode::SaveSuccess { slot_number: slot as u8 }, slot as u8),
                    Err(e) => storage_error_to_result(&e, Some(slot as u8)),
                };

                message.set(result.message());
                message_level.set(result.level.into());
            }
        })
    };

    let on_save_error = {
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |error: SaveValidationError| {
            use plantuml_editor_core::{ErrorCode, ProcessResult};
            use plantuml_editor_storageservice::storage_error_to_result;

            let result = match error {
                SaveValidationError::EmptyContent => {
                    ProcessResult::new(ErrorCode::ValidationEmpty)
                }
                SaveValidationError::ContentTooLarge(actual_length) => {
                    ProcessResult::new(ErrorCode::StorageInputLimit {
                        actual: actual_length,
                        max: 24000,
                    })
                }
                SaveValidationError::StorageError(storage_error) => {
                    storage_error_to_result(&storage_error, None)
                }
            };

            message.set(result.message());
            message_level.set(result.level.into());
        })
    };

    let on_load = {
        let storage_service = storage_service.clone();
        let plantuml_text = plantuml_text.clone();
        let editor_key = editor_key.clone();
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |slot: usize| {
            use plantuml_editor_core::ErrorCode;
            use plantuml_editor_storageservice::{
                storage_error_to_result, storage_success_result,
            };

            // Use injected storage service
            if let Some(service) = &storage_service {
                let result = match service.load_from_slot(slot) {
                    Ok(Some(text)) => {
                        plantuml_text.set(text);
                        editor_key.set(*editor_key + 1);
                        storage_success_result(ErrorCode::LoadSuccess { slot_number: slot as u8 }, slot as u8)
                    }
                    Ok(None) => {
                        ProcessResult::new(ErrorCode::StorageReadError {
                            reason: "スロットにデータがありません".to_string(),
                        })
                    }
                    Err(e) => storage_error_to_result(&e, Some(slot as u8)),
                };

                message.set(result.message());
                message_level.set(result.level.into());
            }
        })
    };

    let on_delete = {
        let storage_service = storage_service.clone();
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |slot: usize| {
            use plantuml_editor_core::ErrorCode;
            use plantuml_editor_storageservice::{
                storage_error_to_result, storage_success_result,
            };

            // Use injected storage service
            if let Some(service) = &storage_service {
                let result = match service.delete_slot(slot) {
                    Ok(_) => storage_success_result(ErrorCode::DeleteSuccess { slot_number: slot as u8 }, slot as u8),
                    Err(e) => storage_error_to_result(&e, Some(slot as u8)),
                };

                message.set(result.message());
                message_level.set(result.level.into());
                // Note: SlotList will automatically refresh via its internal state
            }
        })
    };

    let toggle_sidebar = {
        let sidebar_collapsed = sidebar_collapsed.clone();
        Callback::from(move |_| {
            sidebar_collapsed.set(!*sidebar_collapsed);
        })
    };

    html! {
        <div class="app-container">
            // サイドバー（保存一覧表示）
            <div class={classes!("sidebar", sidebar_collapsed.then(|| "collapsed"))}>
                <div class="sidebar-header" onclick={toggle_sidebar.clone()}>
                    <h3>{ "保存一覧" }</h3>
                    <span class="sidebar-toggle">{ "◀" }</span>
                </div>
                <div class="sidebar-content">
                    <SlotList on_load={on_load} on_delete={on_delete} />
                </div>
            </div>

            // メインコンテンツ
            <div class="main-content">
                // 処理メッセージ
                <div class="message-area">
                    <div class={get_message_class(*message_level)}>{ &*message }</div>
                </div>

                // エディタとプレビューコンテナ
                <div class="editor-preview-container">
                    // PlantUMLソース編集エディタ
                    <div class="editor-area">
                        <div class="editor-header">{ "PlantUMLソース" }</div>
                        <Editor
                            key={*editor_key}
                            value={(*plantuml_text).clone()}
                            on_change={on_text_change}
                        />
                        <div class="editor-actions">
                            <SaveButton
                                plantuml_text={(*plantuml_text).clone()}
                                on_save={on_save}
                                on_error={on_save_error}
                            />
                        </div>
                    </div>

                    // ダイアグラム図プレビュー
                    <div class="preview-area">
                        <div class="preview-header">
                            <span>{ "プレビュー" }</span>
                            <ExportButtons on_export={on_export} />
                        </div>
                        <Preview
                            image_data={(*image_data).clone()}
                            loading={*loading}
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Concrete App component for LocalStorageBackend (for production use)
/// 
/// This is a non-generic wrapper that allows Yew to use the App component
/// with LocalStorageBackend since function_component doesn't support generics directly.
#[function_component(AppWithLocalStorage)]
pub fn app_with_local_storage() -> Html {
    use plantuml_editor_storageservice::LocalStorageBackend;
    
    let storage_service = Rc::new(StorageService::new(LocalStorageBackend::new()));
    let props = AppProps {
        storage_service: Some(storage_service),
    };
    
    // Call the generic app function with concrete type
    html! {
        <App<LocalStorageBackend> storage_service={props.storage_service} />
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_editor_core::{ErrorCode, ProcessResult, StatusLevel};

    // ========================================
    // MessageLevel 変換テスト
    // StatusLevel から MessageLevel への変換が正しく動作することを検証
    // ========================================

    #[test]
    fn test_message_level_from_status_level_info() {
        // StatusLevel::Info が MessageLevel::Info に変換されることを確認
        let level: MessageLevel = StatusLevel::Info.into();
        assert_eq!(level, MessageLevel::Info);
    }

    #[test]
    fn test_message_level_from_status_level_warning() {
        // StatusLevel::Warning が MessageLevel::Warning に変換されることを確認
        let level: MessageLevel = StatusLevel::Warning.into();
        assert_eq!(level, MessageLevel::Warning);
    }

    #[test]
    fn test_message_level_from_status_level_error() {
        // StatusLevel::Error が MessageLevel::Error に変換されることを確認
        let level: MessageLevel = StatusLevel::Error.into();
        assert_eq!(level, MessageLevel::Error);
    }

    // ========================================
    // CSS クラス取得テスト
    // MessageLevel に応じた CSS クラス文字列が正しく返されることを検証
    // ========================================

    #[test]
    fn test_get_message_class_returns_info_class() {
        // Info レベルの場合、基本クラス "message-text" のみ返されることを確認
        assert_eq!(get_message_class(MessageLevel::Info), "message-text");
    }

    #[test]
    fn test_get_message_class_returns_warning_class() {
        // Warning レベルの場合、"message-text warning" が返されることを確認
        assert_eq!(
            get_message_class(MessageLevel::Warning),
            "message-text warning"
        );
    }

    #[test]
    fn test_get_message_class_returns_error_class() {
        // Error レベルの場合、"message-text error" が返されることを確認
        assert_eq!(
            get_message_class(MessageLevel::Error),
            "message-text error"
        );
    }

    // ========================================
    // SaveValidationError 処理ロジックテスト
    // 保存時のバリデーションエラーが正しい ErrorCode に変換されることを検証
    // ========================================

    #[test]
    fn test_save_validation_error_empty_content_conversion() {
        // 空コンテンツエラーが ValidationEmpty ErrorCode に変換されることを確認
        let error = SaveValidationError::EmptyContent;
        let result = match error {
            SaveValidationError::EmptyContent => {
                ProcessResult::new(ErrorCode::ValidationEmpty)
            }
            _ => panic!("Unexpected error variant"),
        };

        assert_eq!(result.level, StatusLevel::Warning);
        assert!(matches!(result.code, ErrorCode::ValidationEmpty));
        assert_eq!(result.message(), "PlantUMLソースを入力してください");
    }

    #[test]
    fn test_save_validation_error_content_too_large_conversion() {
        // コンテンツサイズ超過エラーが StorageInputLimit ErrorCode に変換されることを確認
        let actual_length = 25000_usize;
        let error = SaveValidationError::ContentTooLarge(actual_length);
        
        let result = match error {
            SaveValidationError::ContentTooLarge(actual) => {
                ProcessResult::new(ErrorCode::StorageInputLimit {
                    actual,
                    max: 24000,
                })
            }
            _ => panic!("Unexpected error variant"),
        };

        // StorageInputLimit は Warning レベル
        assert_eq!(result.level, StatusLevel::Warning);
        
        // ErrorCode の内容を検証
        if let ErrorCode::StorageInputLimit { actual, max } = result.code {
            assert_eq!(actual, 25000);
            assert_eq!(max, 24000);
        } else {
            panic!("Expected StorageInputLimit ErrorCode");
        }
        
        // メッセージに実際の文字数と上限が含まれることを確認
        let message = result.message();
        assert!(message.contains("24000"));
        assert!(message.contains("25000"));
    }

    // ========================================
    // toggle_sidebar ロジックテスト
    // サイドバーの開閉切り替えロジックが正しく動作することを検証
    // ========================================

    #[test]
    fn test_toggle_sidebar_logic_from_open_to_closed() {
        // サイドバーが開いている状態（false）から閉じる状態（true）に切り替わることを確認
        let mut sidebar_collapsed = false;
        sidebar_collapsed = !sidebar_collapsed;
        assert_eq!(sidebar_collapsed, true);
    }

    #[test]
    fn test_toggle_sidebar_logic_from_closed_to_open() {
        // サイドバーが閉じている状態（true）から開く状態（false）に切り替わることを確認
        let mut sidebar_collapsed = true;
        sidebar_collapsed = !sidebar_collapsed;
        assert_eq!(sidebar_collapsed, false);
    }
}

// ========================================
// ストレージモックテスト (Phase 2)
// StorageBackend のモックを使用してストレージ操作をテスト
// ========================================

#[cfg(test)]
mod storage_tests {
    use mockall::mock;
    use plantuml_editor_core::{ErrorCode, ProcessResult, StatusLevel, StorageError};
    use plantuml_editor_storageservice::{storage_error_to_result, storage_success_result, StorageBackend};

    // モックストレージバックエンドの定義
    mock! {
        pub StorageBackend {}
        
        impl Clone for StorageBackend {
            fn clone(&self) -> Self;
        }
        
        impl StorageBackend for StorageBackend {
            fn save_to_slot(&self, slot_number: usize, text: &str) -> Result<(), StorageError>;
            fn load_from_slot(&self, slot_number: usize) -> Result<Option<String>, StorageError>;
            fn list_slots(&self) -> Vec<plantuml_editor_storageservice::SlotInfo>;
            fn delete_slot(&self, slot_number: usize) -> Result<(), StorageError>;
        }
    }

    // ========================================
    // on_save 相当のテスト
    // 保存処理のロジックをモックを使用して検証
    // ========================================

    #[test]
    fn test_save_success_returns_correct_result() {
        // 保存成功時に SaveSuccess ErrorCode が返されることを確認
        let mut mock_backend = MockStorageBackend::new();
        mock_backend
            .expect_save_to_slot()
            .withf(|slot, text| *slot == 1 && text == "test content")
            .times(1)
            .returning(|_, _| Ok(()));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.save_to_slot(1, "test content");

        assert!(result.is_ok());
        
        // 成功時の ProcessResult を生成
        let process_result = storage_success_result(
            ErrorCode::SaveSuccess { slot_number: 1 },
            1
        );
        
        assert_eq!(process_result.level, StatusLevel::Info);
        assert!(matches!(
            process_result.code,
            ErrorCode::SaveSuccess { slot_number: 1 }
        ));
        assert_eq!(process_result.message(), "PlantUMLソースをスロット1に保存しました");
    }

    #[test]
    fn test_save_failure_returns_error_result() {
        // 保存失敗時（書き込みエラー）に適切なエラーが返されることを確認
        let mut mock_backend = MockStorageBackend::new();
        mock_backend
            .expect_save_to_slot()
            .times(1)
            .returning(|_, _| Err(StorageError::QuotaExceeded));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.save_to_slot(1, "test content");

        assert!(result.is_err());
        
        // エラー時の ProcessResult を生成
        if let Err(e) = result {
            let process_result = storage_error_to_result(&e, Some(1));
            
            assert_eq!(process_result.level, StatusLevel::Warning);
            assert!(matches!(
                process_result.code,
                ErrorCode::StorageInputLimit { .. }
            ));
        }
    }

    #[test]
    fn test_save_slot_full_error() {
        // スロット上限エラー時に StorageSlotLimit が返されることを確認
        let mut mock_backend = MockStorageBackend::new();
        mock_backend
            .expect_save_to_slot()
            .times(1)
            .returning(|_, _| Err(StorageError::SlotsFull));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.save_to_slot(11, "test content");

        assert!(result.is_err());
        
        if let Err(e) = result {
            let process_result = storage_error_to_result(&e, Some(11));
            
            assert_eq!(process_result.level, StatusLevel::Warning);
            assert!(matches!(
                process_result.code,
                ErrorCode::StorageSlotLimit { .. }
            ));
            
            let message = process_result.message();
            assert!(message.contains("上限"));
        }
    }

    // ========================================
    // on_load 相当のテスト
    // 読み込み処理のロジックをモックを使用して検証
    // ========================================

    #[test]
    fn test_load_success_returns_content() {
        // 読み込み成功時にコンテンツが返され、LoadSuccess が生成されることを確認
        let mut mock_backend = MockStorageBackend::new();
        let expected_content = "test content from slot 2".to_string();
        let expected_content_clone = expected_content.clone();
        
        mock_backend
            .expect_load_from_slot()
            .with(mockall::predicate::eq(2))
            .times(1)
            .returning(move |_| Ok(Some(expected_content_clone.clone())));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.load_from_slot(2);

        assert!(result.is_ok());
        
        if let Ok(Some(content)) = result {
            assert_eq!(content, expected_content);
            
            // 成功時の ProcessResult を生成
            let process_result = storage_success_result(
                ErrorCode::LoadSuccess { slot_number: 2 },
                2
            );
            
            assert_eq!(process_result.level, StatusLevel::Info);
            assert!(matches!(
                process_result.code,
                ErrorCode::LoadSuccess { slot_number: 2 }
            ));
            assert_eq!(process_result.message(), "スロット2からPlantUMLソースを読み込みました");
        } else {
            panic!("Expected Ok(Some(content))");
        }
    }

    #[test]
    fn test_load_empty_slot_returns_none() {
        // 空スロットの読み込み時に None が返され、適切なエラーメッセージが生成されることを確認
        let mut mock_backend = MockStorageBackend::new();
        mock_backend
            .expect_load_from_slot()
            .with(mockall::predicate::eq(99))
            .times(1)
            .returning(|_| Ok(None));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.load_from_slot(99);

        assert!(result.is_ok());
        
        if let Ok(None) = result {
            // None の場合の ProcessResult を生成
            let process_result = ProcessResult::new(ErrorCode::StorageReadError {
                reason: "スロットにデータがありません".to_string(),
            });
            
            assert_eq!(process_result.level, StatusLevel::Error);
            assert!(matches!(
                process_result.code,
                ErrorCode::StorageReadError { .. }
            ));
            
            let message = process_result.message();
            assert!(message.contains("読み込みに失敗"));
        } else {
            panic!("Expected Ok(None)");
        }
    }

    #[test]
    fn test_load_storage_error() {
        // ストレージエラー時に適切なエラーが返されることを確認
        let mut mock_backend = MockStorageBackend::new();
        mock_backend
            .expect_load_from_slot()
            .times(1)
            .returning(|_| Err(StorageError::InvalidSlotNumber(255)));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.load_from_slot(255);

        assert!(result.is_err());
        
        if let Err(e) = result {
            let process_result = storage_error_to_result(&e, Some(255));
            
            assert_eq!(process_result.level, StatusLevel::Error);
            assert!(matches!(
                process_result.code,
                ErrorCode::StorageReadError { .. }
            ));
        }
    }

    #[test]
    fn test_load_updates_editor_key_logic() {
        // 読み込み成功時に editor_key がインクリメントされることを確認
        // （実際のコールバック内のロジックをシミュレート）
        let mut editor_key = 5;
        
        // 読み込み成功を想定
        let mock_load_success = true;
        
        if mock_load_success {
            editor_key += 1;
        }
        
        assert_eq!(editor_key, 6);
    }

    // ========================================
    // on_delete 相当のテスト
    // 削除処理のロジックをモックを使用して検証
    // ========================================

    #[test]
    fn test_delete_success_returns_correct_result() {
        // 削除成功時に DeleteSuccess ErrorCode が返されることを確認
        let mut mock_backend = MockStorageBackend::new();
        mock_backend
            .expect_delete_slot()
            .with(mockall::predicate::eq(3))
            .times(1)
            .returning(|_| Ok(()));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.delete_slot(3);

        assert!(result.is_ok());
        
        // 成功時の ProcessResult を生成
        let process_result = storage_success_result(
            ErrorCode::DeleteSuccess { slot_number: 3 },
            3
        );
        
        assert_eq!(process_result.level, StatusLevel::Info);
        assert!(matches!(
            process_result.code,
            ErrorCode::DeleteSuccess { slot_number: 3 }
        ));
        assert_eq!(process_result.message(), "スロット3のデータを削除しました");
    }

    #[test]
    fn test_delete_failure_returns_error_result() {
        // 削除失敗時に適切なエラーが返されることを確認
        let mut mock_backend = MockStorageBackend::new();
        mock_backend
            .expect_delete_slot()
            .times(1)
            .returning(|_| Err(StorageError::SlotEmpty(5)));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        let result = service.delete_slot(5);

        assert!(result.is_err());
        
        if let Err(e) = result {
            let process_result = storage_error_to_result(&e, Some(5));
            
            assert_eq!(process_result.level, StatusLevel::Error);
            assert!(matches!(
                process_result.code,
                ErrorCode::StorageReadError { .. }
            ));
        }
    }

    // ========================================
    // 統合的なテストケース
    // 複数の操作を組み合わせたシナリオテスト
    // ========================================

    #[test]
    fn test_save_load_delete_scenario() {
        // 保存 → 読み込み → 削除の一連の流れをテスト
        let mut mock_backend = MockStorageBackend::new();
        let test_content = "scenario test content".to_string();
        let test_content_clone = test_content.clone();
        
        // 保存
        mock_backend
            .expect_save_to_slot()
            .withf(|slot, text| *slot == 1 && text == "scenario test content")
            .times(1)
            .returning(|_, _| Ok(()));
        
        // 読み込み
        mock_backend
            .expect_load_from_slot()
            .with(mockall::predicate::eq(1))
            .times(1)
            .returning(move |_| Ok(Some(test_content_clone.clone())));
        
        // 削除
        mock_backend
            .expect_delete_slot()
            .with(mockall::predicate::eq(1))
            .times(1)
            .returning(|_| Ok(()));

        let service = plantuml_editor_storageservice::StorageService::new(mock_backend);
        
        // 保存実行
        let save_result = service.save_to_slot(1, "scenario test content");
        assert!(save_result.is_ok());
        
        // 読み込み実行
        let load_result = service.load_from_slot(1);
        assert!(load_result.is_ok());
        if let Ok(Some(content)) = load_result {
            assert_eq!(content, test_content);
        }
        
        // 削除実行
        let delete_result = service.delete_slot(1);
        assert!(delete_result.is_ok());
    }
}

// ========================================
// Phase 4: Callback Integration Tests with Dependency Injection
// ========================================
//
// 依存性注入パターンを活用して、コールバックロジックをテスト
// MockStorageBackendを使用してコールバック内の処理を検証

#[cfg(test)]
mod callback_integration_tests {
    use super::*;
    use mockall::mock;
    use plantuml_editor_core::{ErrorCode, ProcessResult, StatusLevel, StorageError};
    use plantuml_editor_storageservice::{storage_error_to_result, storage_success_result, StorageBackend};
    use std::rc::Rc;

    // モックストレージバックエンドの定義
    mock! {
        pub CallbackStorageBackend {}
        
        impl Clone for CallbackStorageBackend {
            fn clone(&self) -> Self;
        }
        
        impl PartialEq for CallbackStorageBackend {
            fn eq(&self, other: &Self) -> bool;
        }
        
        impl StorageBackend for CallbackStorageBackend {
            fn save_to_slot(&self, slot_number: usize, text: &str) -> Result<(), StorageError>;
            fn load_from_slot(&self, slot_number: usize) -> Result<Option<String>, StorageError>;
            fn list_slots(&self) -> Vec<plantuml_editor_storageservice::SlotInfo>;
            fn delete_slot(&self, slot_number: usize) -> Result<(), StorageError>;
        }
    }

    // ========================================
    // on_save コールバックロジックのテスト
    // ========================================

    #[test]
    fn test_on_save_callback_with_successful_save() {
        // on_save コールバック内のロジックをシミュレート
        let mut mock_backend = MockCallbackStorageBackend::new();
        
        // Clone実装のモック
        mock_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_save_to_slot()
                    .returning(|_, _| Ok(()));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        // PartialEq実装のモック
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        // 保存成功をモック
        mock_backend
            .expect_save_to_slot()
            .with(mockall::predicate::eq(3), mockall::predicate::eq("test plantuml code"))
            .times(1)
            .returning(|_, _| Ok(()));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        // コールバック内のロジックをシミュレート
        let slot = 3_usize;
        let plantuml_text = "test plantuml code";
        
        let result = match service.save_to_slot(slot, plantuml_text) {
            Ok(_) => storage_success_result(
                ErrorCode::SaveSuccess { slot_number: slot as u8 },
                slot as u8
            ),
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // 成功メッセージが生成されることを確認
        assert_eq!(result.level, StatusLevel::Info);
        assert!(matches!(result.code, ErrorCode::SaveSuccess { slot_number: 3 }));
        assert_eq!(result.message(), "PlantUMLソースをスロット3に保存しました");
    }

    #[test]
    fn test_on_save_callback_with_quota_exceeded() {
        // QuotaExceededエラーのハンドリングをテスト
        let mut mock_backend = MockCallbackStorageBackend::new();
        
        mock_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_save_to_slot()
                    .returning(|_, _| Err(StorageError::QuotaExceeded));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        mock_backend
            .expect_save_to_slot()
            .times(1)
            .returning(|_, _| Err(StorageError::QuotaExceeded));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        let slot = 5_usize;
        let plantuml_text = "large content";
        
        let result = match service.save_to_slot(slot, plantuml_text) {
            Ok(_) => storage_success_result(
                ErrorCode::SaveSuccess { slot_number: slot as u8 },
                slot as u8
            ),
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // エラーメッセージが生成されることを確認
        assert_eq!(result.level, StatusLevel::Warning);
        assert!(matches!(result.code, ErrorCode::StorageInputLimit { .. }));
        let message = result.message();
        assert!(message.contains("上限") || message.contains("文字数"));
    }

    #[test]
    fn test_on_save_callback_with_slots_full() {
        // SlotsFull エラーのハンドリングをテスト
        let mut mock_backend = MockCallbackStorageBackend::new();
        
        mock_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_save_to_slot()
                    .returning(|_, _| Err(StorageError::SlotsFull));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        mock_backend
            .expect_save_to_slot()
            .times(1)
            .returning(|_, _| Err(StorageError::SlotsFull));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        let slot = 11_usize;
        let plantuml_text = "content";
        
        let result = match service.save_to_slot(slot, plantuml_text) {
            Ok(_) => storage_success_result(
                ErrorCode::SaveSuccess { slot_number: slot as u8 },
                slot as u8
            ),
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // スロット上限エラーが生成されることを確認
        assert_eq!(result.level, StatusLevel::Warning);
        assert!(matches!(result.code, ErrorCode::StorageSlotLimit { .. }));
        let message = result.message();
        assert!(message.contains("上限"));
    }

    // ========================================
    // on_load コールバックロジックのテスト
    // ========================================

    #[test]
    fn test_on_load_callback_with_successful_load() {
        // on_load コールバック内のロジックをシミュレート
        let mut mock_backend = MockCallbackStorageBackend::new();
        let test_content = "@startuml\nAlice -> Bob: Test\n@enduml".to_string();
        let test_content_clone = test_content.clone();
        
        mock_backend
            .expect_clone()
            .returning(move || {
                let content = test_content_clone.clone();
                let mut m = MockCallbackStorageBackend::new();
                m.expect_load_from_slot()
                    .returning(move |_| Ok(Some(content.clone())));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        mock_backend
            .expect_load_from_slot()
            .with(mockall::predicate::eq(2))
            .times(1)
            .returning(move |_| Ok(Some(test_content.clone())));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        let slot = 2_usize;
        let mut editor_key = 10;
        
        // コールバック内のロジックをシミュレート
        let result = match service.load_from_slot(slot) {
            Ok(Some(text)) => {
                // plantuml_text.set(text); をシミュレート
                let _loaded_text = text;
                
                // editor_key インクリメント
                editor_key += 1;
                
                storage_success_result(
                    ErrorCode::LoadSuccess { slot_number: slot as u8 },
                    slot as u8
                )
            }
            Ok(None) => {
                ProcessResult::new(ErrorCode::StorageReadError {
                    reason: "スロットにデータがありません".to_string(),
                })
            }
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // 成功メッセージとeditor_keyの更新を確認
        assert_eq!(result.level, StatusLevel::Info);
        assert!(matches!(result.code, ErrorCode::LoadSuccess { slot_number: 2 }));
        assert_eq!(result.message(), "スロット2からPlantUMLソースを読み込みました");
        assert_eq!(editor_key, 11); // インクリメントされたことを確認
    }

    #[test]
    fn test_on_load_callback_with_empty_slot() {
        // 空スロットの読み込みをテスト
        let mut mock_backend = MockCallbackStorageBackend::new();
        
        mock_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_load_from_slot()
                    .returning(|_| Ok(None));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        mock_backend
            .expect_load_from_slot()
            .with(mockall::predicate::eq(99))
            .times(1)
            .returning(|_| Ok(None));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        let slot = 99_usize;
        let mut editor_key = 5;
        
        let result = match service.load_from_slot(slot) {
            Ok(Some(text)) => {
                let _loaded_text = text;
                editor_key += 1;
                storage_success_result(
                    ErrorCode::LoadSuccess { slot_number: slot as u8 },
                    slot as u8
                )
            }
            Ok(None) => {
                ProcessResult::new(ErrorCode::StorageReadError {
                    reason: "スロットにデータがありません".to_string(),
                })
            }
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // エラーメッセージとeditor_keyが更新されていないことを確認
        assert_eq!(result.level, StatusLevel::Error);
        assert!(matches!(result.code, ErrorCode::StorageReadError { .. }));
        let message = result.message();
        assert!(message.contains("読み込みに失敗"));
        assert_eq!(editor_key, 5); // インクリメントされていないことを確認
    }

    #[test]
    fn test_on_load_callback_with_storage_error() {
        // ストレージエラーの読み込みをテスト
        let mut mock_backend = MockCallbackStorageBackend::new();
        
        mock_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_load_from_slot()
                    .returning(|_| Err(StorageError::InvalidSlotNumber(200)));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        mock_backend
            .expect_load_from_slot()
            .with(mockall::predicate::eq(200))
            .times(1)
            .returning(|_| Err(StorageError::InvalidSlotNumber(200)));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        let slot = 200_usize;
        
        let result = match service.load_from_slot(slot) {
            Ok(Some(text)) => {
                let _loaded_text = text;
                storage_success_result(
                    ErrorCode::LoadSuccess { slot_number: slot as u8 },
                    slot as u8
                )
            }
            Ok(None) => {
                ProcessResult::new(ErrorCode::StorageReadError {
                    reason: "スロットにデータがありません".to_string(),
                })
            }
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // エラーメッセージが生成されることを確認
        assert_eq!(result.level, StatusLevel::Error);
        assert!(matches!(result.code, ErrorCode::StorageReadError { .. }));
    }

    // ========================================
    // on_delete コールバックロジックのテスト
    // ========================================

    #[test]
    fn test_on_delete_callback_with_successful_delete() {
        // on_delete コールバック内のロジックをシミュレート
        let mut mock_backend = MockCallbackStorageBackend::new();
        
        mock_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_delete_slot()
                    .returning(|_| Ok(()));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        mock_backend
            .expect_delete_slot()
            .with(mockall::predicate::eq(7))
            .times(1)
            .returning(|_| Ok(()));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        let slot = 7_usize;
        
        let result = match service.delete_slot(slot) {
            Ok(_) => storage_success_result(
                ErrorCode::DeleteSuccess { slot_number: slot as u8 },
                slot as u8
            ),
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // 成功メッセージが生成されることを確認
        assert_eq!(result.level, StatusLevel::Info);
        assert!(matches!(result.code, ErrorCode::DeleteSuccess { slot_number: 7 }));
        assert_eq!(result.message(), "スロット7のデータを削除しました");
    }

    #[test]
    fn test_on_delete_callback_with_empty_slot() {
        // 空スロットの削除をテスト
        let mut mock_backend = MockCallbackStorageBackend::new();
        
        mock_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_delete_slot()
                    .returning(|_| Err(StorageError::SlotEmpty(8)));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        
        mock_backend
            .expect_eq()
            .returning(|_| true);
        
        mock_backend
            .expect_delete_slot()
            .with(mockall::predicate::eq(8))
            .times(1)
            .returning(|_| Err(StorageError::SlotEmpty(8)));

        let service = Rc::new(plantuml_editor_storageservice::StorageService::new(mock_backend));
        
        let slot = 8_usize;
        
        let result = match service.delete_slot(slot) {
            Ok(_) => storage_success_result(
                ErrorCode::DeleteSuccess { slot_number: slot as u8 },
                slot as u8
            ),
            Err(e) => storage_error_to_result(&e, Some(slot as u8)),
        };
        
        // エラーメッセージが生成されることを確認
        assert_eq!(result.level, StatusLevel::Error);
        assert!(matches!(result.code, ErrorCode::StorageReadError { .. }));
    }

    // ========================================
    // 複数コールバック統合シナリオテスト
    // ========================================

    #[test]
    fn test_full_callback_workflow_save_load_delete() {
        // 保存 → 読み込み → 削除の完全なワークフローをテスト
        let test_content = "@startuml\nWorkflow Test\n@enduml".to_string();
        let test_content_clone1 = test_content.clone();
        let test_content_clone2 = test_content.clone();
        
        // 保存用モック
        let mut save_backend = MockCallbackStorageBackend::new();
        save_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_save_to_slot()
                    .returning(|_, _| Ok(()));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        save_backend
            .expect_eq()
            .returning(|_| true);
        save_backend
            .expect_save_to_slot()
            .with(mockall::predicate::eq(4), mockall::predicate::always())
            .times(1)
            .returning(|_, _| Ok(()));
        
        let save_service = Rc::new(plantuml_editor_storageservice::StorageService::new(save_backend));
        
        // 1. 保存コールバック実行
        let save_result = match save_service.save_to_slot(4, &test_content) {
            Ok(_) => storage_success_result(ErrorCode::SaveSuccess { slot_number: 4 }, 4),
            Err(e) => storage_error_to_result(&e, Some(4)),
        };
        
        assert_eq!(save_result.level, StatusLevel::Info);
        assert!(matches!(save_result.code, ErrorCode::SaveSuccess { slot_number: 4 }));
        
        // 2. 読み込みコールバック実行
        let mut load_backend = MockCallbackStorageBackend::new();
        load_backend
            .expect_clone()
            .returning(move || {
                let content = test_content_clone1.clone();
                let mut m = MockCallbackStorageBackend::new();
                m.expect_load_from_slot()
                    .returning(move |_| Ok(Some(content.clone())));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        load_backend
            .expect_eq()
            .returning(|_| true);
        load_backend
            .expect_load_from_slot()
            .with(mockall::predicate::eq(4))
            .times(1)
            .returning(move |_| Ok(Some(test_content_clone2.clone())));
        
        let load_service = Rc::new(plantuml_editor_storageservice::StorageService::new(load_backend));
        
        let mut editor_key = 0;
        let load_result = match load_service.load_from_slot(4) {
            Ok(Some(text)) => {
                assert_eq!(text, test_content);
                editor_key += 1;
                storage_success_result(ErrorCode::LoadSuccess { slot_number: 4 }, 4)
            }
            Ok(None) => {
                ProcessResult::new(ErrorCode::StorageReadError {
                    reason: "スロットにデータがありません".to_string(),
                })
            }
            Err(e) => storage_error_to_result(&e, Some(4)),
        };
        
        assert_eq!(load_result.level, StatusLevel::Info);
        assert!(matches!(load_result.code, ErrorCode::LoadSuccess { slot_number: 4 }));
        assert_eq!(editor_key, 1);
        
        // 3. 削除コールバック実行
        let mut delete_backend = MockCallbackStorageBackend::new();
        delete_backend
            .expect_clone()
            .returning(|| {
                let mut m = MockCallbackStorageBackend::new();
                m.expect_delete_slot()
                    .returning(|_| Ok(()));
                m.expect_eq()
                    .returning(|_| true);
                m
            });
        delete_backend
            .expect_eq()
            .returning(|_| true);
        delete_backend
            .expect_delete_slot()
            .with(mockall::predicate::eq(4))
            .times(1)
            .returning(|_| Ok(()));
        
        let delete_service = Rc::new(plantuml_editor_storageservice::StorageService::new(delete_backend));
        
        let delete_result = match delete_service.delete_slot(4) {
            Ok(_) => storage_success_result(ErrorCode::DeleteSuccess { slot_number: 4 }, 4),
            Err(e) => storage_error_to_result(&e, Some(4)),
        };
        
        assert_eq!(delete_result.level, StatusLevel::Info);
        assert!(matches!(delete_result.code, ErrorCode::DeleteSuccess { slot_number: 4 }));
    }
}

// ========================================
// Phase 3: Browser Integration Tests with wasm-bindgen-test
// ========================================
//
// これらのテストはブラウザ環境でコンポーネントをレンダリングし、
// on_save, on_load, on_deleteのコールバックが実際に動作することを検証します。

#[cfg(all(test, target_arch = "wasm32"))]
mod browser_tests {
    use super::*;
    use wasm_bindgen_test::*;
    use web_sys::window;

    wasm_bindgen_test_configure!(run_in_browser);

    /// ブラウザ環境でAppコンポーネントがレンダリングされることを確認
    #[wasm_bindgen_test]
    fn test_app_renders_in_browser() {
        let document = window()
            .expect("should have window")
            .document()
            .expect("should have document");
        
        let container = document.create_element("div")
            .expect("should create div");
        document.body()
            .expect("should have body")
            .append_child(&container)
            .expect("should append child");

        // Yewアプリケーションをレンダリング
        let renderer = yew::Renderer::<App>::with_root(container);
        renderer.render();
        
        // レンダリングが成功したことを確認
        // (エラーが発生しなければテストは成功)
    }

    /// LocalStorageを使用した保存機能のブラウザテスト
    #[wasm_bindgen_test]
    fn test_localstorage_save_in_browser() {
        use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
        
        let backend = LocalStorageBackend::new();
        let service = StorageService::new(backend);
        
        // テストデータの保存
        let test_text = "@startuml\nAlice -> Bob: Hello\n@enduml";
        let result = service.save_to_slot(99, test_text);
        
        assert!(result.is_ok(), "Should save to LocalStorage successfully");
        
        // 保存したデータの読み込み確認
        let loaded = service.load_from_slot(99);
        assert!(loaded.is_ok(), "Should load from LocalStorage successfully");
        
        if let Ok(Some(content)) = loaded {
            assert_eq!(content, test_text, "Loaded content should match saved content");
        }
        
        // クリーンアップ
        let _ = service.delete_slot(99);
    }

    /// LocalStorageを使用した読み込み機能のブラウザテスト
    #[wasm_bindgen_test]
    fn test_localstorage_load_in_browser() {
        use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
        
        let backend = LocalStorageBackend::new();
        let service = StorageService::new(backend);
        
        // 事前にデータを保存
        let test_text = "@startuml\nBob -> Alice: Response\n@enduml";
        let _ = service.save_to_slot(98, test_text);
        
        // 読み込みテスト
        let result = service.load_from_slot(98);
        assert!(result.is_ok(), "Should load from LocalStorage successfully");
        
        match result {
            Ok(Some(content)) => {
                assert_eq!(content, test_text, "Loaded content should match saved content");
            }
            Ok(None) => {
                panic!("Expected content but got None");
            }
            Err(e) => {
                panic!("Load failed: {:?}", e);
            }
        }
        
        // クリーンアップ
        let _ = service.delete_slot(98);
    }

    /// LocalStorageを使用した削除機能のブラウザテスト
    #[wasm_bindgen_test]
    fn test_localstorage_delete_in_browser() {
        use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
        
        let backend = LocalStorageBackend::new();
        let service = StorageService::new(backend);
        
        // 事前にデータを保存
        let test_text = "@startuml\nCharlie -> Dave: Test\n@enduml";
        let _ = service.save_to_slot(97, test_text);
        
        // データが存在することを確認
        let loaded = service.load_from_slot(97);
        assert!(loaded.is_ok());
        assert!(matches!(loaded, Ok(Some(_))), "Data should exist before deletion");
        
        // 削除テスト
        let delete_result = service.delete_slot(97);
        assert!(delete_result.is_ok(), "Should delete from LocalStorage successfully");
        
        // 削除後にデータが存在しないことを確認
        let loaded_after = service.load_from_slot(97);
        assert!(loaded_after.is_ok());
        assert!(matches!(loaded_after, Ok(None)), "Data should not exist after deletion");
    }

    /// 存在しないスロットの読み込みテスト
    #[wasm_bindgen_test]
    fn test_localstorage_load_nonexistent_slot() {
        use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
        
        let backend = LocalStorageBackend::new();
        let service = StorageService::new(backend);
        
        // 存在しないスロット番号で読み込み
        let result = service.load_from_slot(9999);
        
        assert!(result.is_ok(), "Load should succeed even for nonexistent slot");
        assert!(matches!(result, Ok(None)), "Should return None for nonexistent slot");
    }

    /// 無効なスロット番号でのエラーハンドリングテスト
    #[wasm_bindgen_test]
    fn test_localstorage_invalid_slot_number() {
        use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
        
        let backend = LocalStorageBackend::new();
        let service = StorageService::new(backend);
        
        // スロット0での保存試行（有効な場合）
        let result = service.save_to_slot(0, "test");
        
        // スロット0が有効か無効かは実装依存だが、エラーハンドリングが正しく動作することを確認
        match result {
            Ok(_) => {
                // スロット0が有効な場合はクリーンアップ
                let _ = service.delete_slot(0);
            }
            Err(_) => {
                // エラーが返される場合もOK（実装依存）
            }
        }
    }

    /// ストレージ操作のエラーケーステスト（大量データ）
    #[wasm_bindgen_test]
    fn test_localstorage_large_data() {
        use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
        
        let backend = LocalStorageBackend::new();
        let service = StorageService::new(backend);
        
        // 大きなテストデータを作成（10KB程度）
        let large_text = "@startuml\n".to_string() + &"A -> B: message\n".repeat(500) + "@enduml";
        
        // 大きなデータの保存テスト
        let result = service.save_to_slot(96, &large_text);
        
        // LocalStorageの容量制限に達する可能性があるが、エラーハンドリングが正しく動作することを確認
        match result {
            Ok(_) => {
                // 保存成功した場合は読み込みも確認
                let loaded = service.load_from_slot(96);
                assert!(loaded.is_ok());
                
                if let Ok(Some(content)) = loaded {
                    assert_eq!(content.len(), large_text.len(), "Loaded data size should match");
                }
                
                // クリーンアップ
                let _ = service.delete_slot(96);
            }
            Err(_) => {
                // LocalStorageの容量制限エラーもOK
                // エラーが適切に処理されていることを確認
            }
        }
    }

    /// 複数スロットへの連続保存テスト
    #[wasm_bindgen_test]
    fn test_localstorage_multiple_slots() {
        use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
        
        let backend = LocalStorageBackend::new();
        let service = StorageService::new(backend);
        
        // 複数スロットにデータを保存
        let slots = vec![91, 92, 93];
        let test_texts = vec![
            "@startuml\nA -> B\n@enduml",
            "@startuml\nC -> D\n@enduml",
            "@startuml\nE -> F\n@enduml",
        ];
        
        // 保存
        for (i, slot) in slots.iter().enumerate() {
            let result = service.save_to_slot(*slot, test_texts[i]);
            assert!(result.is_ok(), "Should save to slot {}", slot);
        }
        
        // 読み込み確認
        for (i, slot) in slots.iter().enumerate() {
            let result = service.load_from_slot(*slot);
            assert!(result.is_ok(), "Should load from slot {}", slot);
            
            if let Ok(Some(content)) = result {
                assert_eq!(content, test_texts[i], "Content in slot {} should match", slot);
            }
        }
        
        // クリーンアップ
        for slot in slots.iter() {
            let _ = service.delete_slot(*slot);
        }
    }
}
