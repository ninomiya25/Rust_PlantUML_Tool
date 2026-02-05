// Web UI library for PlantUML Editor
//
// This crate provides reusable Yew components and UI models
// for the PlantUML editor frontend application.

use wasm_bindgen::JsCast;
use yew::prelude::*;

pub mod components;
pub mod errors;

// Re-export components
pub use components::*;

/// Main application component（状態管理とイベントハンドリング）
#[function_component(App)]
pub fn app() -> Html {
    use plantuml_editor_api_client::{convert_plantuml, export_plantuml};
    use plantuml_editor_core::{ImageFormat, ProcessResult, StatusLevel};
    use wasm_bindgen_futures::spawn_local;

    /// Message level for UI display
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MessageLevel {
        Info,
        Warning,
        Error,
    }

    impl From<StatusLevel> for MessageLevel {
        fn from(level: StatusLevel) -> Self {
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
                                let result = ProcessResult::error(code);
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
                                let result = ProcessResult::error(code);
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
        let plantuml_text = plantuml_text.clone();
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |slot: usize| {
            use plantuml_editor_core::ErrorCode;
            use plantuml_editor_storageservice::{
                storage_error_to_result, storage_success_result, LocalStorageBackend,
                StorageService,
            };

            let service = StorageService::new(LocalStorageBackend::new());
            let result = match service.save_to_slot(slot, &plantuml_text) {
                Ok(_) => storage_success_result(ErrorCode::SaveSuccess { slot_number: slot as u8 }, slot as u8),
                Err(e) => storage_error_to_result(&e, Some(slot as u8)),
            };

            message.set(result.message());
            message_level.set(result.level.into());
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
                    ProcessResult::error(ErrorCode::ValidationEmpty)
                }
                SaveValidationError::ContentTooLarge(actual_length) => {
                    ProcessResult::error(ErrorCode::StorageInputLimit {
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
        let plantuml_text = plantuml_text.clone();
        let editor_key = editor_key.clone();
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |slot: usize| {
            use plantuml_editor_core::ErrorCode;
            use plantuml_editor_storageservice::{
                storage_error_to_result, storage_success_result, LocalStorageBackend,
                StorageService,
            };

            let service = StorageService::new(LocalStorageBackend::new());
            let result = match service.load_from_slot(slot) {
                Ok(Some(text)) => {
                    plantuml_text.set(text);
                    editor_key.set(*editor_key + 1);
                    storage_success_result(ErrorCode::LoadSuccess { slot_number: slot as u8 }, slot as u8)
                }
                Ok(None) => {
                    ProcessResult::error(ErrorCode::StorageReadError {
                        reason: "スロットにデータがありません".to_string(),
                    })
                }
                Err(e) => storage_error_to_result(&e, Some(slot as u8)),
            };

            message.set(result.message());
            message_level.set(result.level.into());
        })
    };

    let on_delete = {
        let message = message.clone();
        let message_level = message_level.clone();

        Callback::from(move |slot: usize| {
            use plantuml_editor_core::ErrorCode;
            use plantuml_editor_storageservice::{
                storage_error_to_result, storage_success_result, LocalStorageBackend,
                StorageService,
            };

            let service = StorageService::new(LocalStorageBackend::new());
            let result = match service.delete_slot(slot) {
                Ok(_) => storage_success_result(ErrorCode::DeleteSuccess { slot_number: slot as u8 }, slot as u8),
                Err(e) => storage_error_to_result(&e, Some(slot as u8)),
            };

            message.set(result.message());
            message_level.set(result.level.into());
            // Note: SlotList will automatically refresh via its internal state
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
