// Message mapping for error codes

use plantuml_editor_core::{ErrorCode, ProcessResult, StatusLevel};

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

/// Get user-friendly message from ProcessResult
pub fn get_message_from_result(result: &ProcessResult) -> String {
    match &result.code {
        // 正常完了 (INFO)
        ErrorCode::ConversionOk => "図が正常に生成されました".to_string(),
        ErrorCode::ExportOk => "図が正常にエクスポートされました".to_string(),
        ErrorCode::SaveSuccess => {
            if let Some(context) = &result.context {
                if let Some(slot) = context.get("slotNumber") {
                    return format!("PlantUMLソースをスロット{}に保存しました", slot);
                }
            }
            "PlantUMLソースを保存しました".to_string()
        }
        ErrorCode::LoadSuccess => {
            if let Some(context) = &result.context {
                if let Some(slot) = context.get("slotNumber") {
                    return format!("スロット{}からPlantUMLソースを読み込みました", slot);
                }
            }
            "PlantUMLソースを読み込みました".to_string()
        }
        ErrorCode::DeleteSuccess => {
            if let Some(context) = &result.context {
                if let Some(slot) = context.get("slotNumber") {
                    return format!("スロット{}のデータを削除しました", slot);
                }
            }
            "データを削除しました".to_string()
        }

        // バリデーションエラー (WARNING)
        ErrorCode::ValidationEmpty => "PlantUMLソースを入力してください".to_string(),
        ErrorCode::ValidationTextLimit => {
            if let Some(context) = &result.context {
                if let Some(max_length) = context.get("maxLength") {
                    return format!("PlantUMLソースが長すぎます。文字数を{}文字以内に減らしてください", max_length);
                }
            }
            "PlantUMLソースが長すぎます".to_string()
        }
        ErrorCode::StorageInputLimit => {
            if let Some(context) = &result.context {
                if let Some(max_chars) = context.get("maxChars") {
                    return format!("保存する内容の文字数が上限({}文字)を超えています。内容を短縮してください", max_chars);
                }
            }
            "保存する内容が長すぎます".to_string()
        }
        ErrorCode::StorageSlotLimit => {
            "一時保存上限に達しています。既存のスロットを削除してから保存してください".to_string()
        }

        // 処理エラー (ERROR)
        ErrorCode::SizeLimit => {
            "画像サイズが上限を超えています。'scale'でサイズを縮小するか、図を分割してください".to_string()
        }
        ErrorCode::EncodingError => {
            "PlantUMLソースの変換に失敗しました。文字コードや特殊文字が含まれていないかご確認ください".to_string()
        }
        ErrorCode::ParseError => {
            "PlantUMLの処理中にエラーが発生しました。管理者へお問い合わせください".to_string()
        }
        ErrorCode::ExportError => {
            "ファイルのエクスポートに失敗しました。再度お試しください".to_string()
        }

        // サーバー・ネットワークエラー (ERROR)
        ErrorCode::ServerError => {
            "サーバーが応答していません。時間をおいて再度接続を試すか管理者に問い合わせてください".to_string()
        }
        ErrorCode::TimeoutError => {
            "通信がタイムアウトしました。ネットワーク状況をご確認のうえ、再度お試しください".to_string()
        }
        ErrorCode::NetworkError => {
            "ネットワーク接続に失敗しました。インターネット接続をご確認ください".to_string()
        }

        // ストレージエラー (ERROR)
        ErrorCode::StorageWriteError => {
            "ローカルストレージへの保存に失敗しました。ブラウザの設定をご確認ください".to_string()
        }
        ErrorCode::StorageReadError => {
            "ローカルストレージからの読み込みに失敗しました。保存されたデータが破損している可能性があります".to_string()
        }
        ErrorCode::StorageDeleteError => {
            "ローカルストレージのデータ削除に失敗しました。ブラウザのキャッシュをクリアしてお試しください".to_string()
        }
    }
}

/// Get CSS class for message level
pub fn get_message_class(level: MessageLevel) -> &'static str {
    match level {
        MessageLevel::Info => "message-text",
        MessageLevel::Warning => "message-text warning",
        MessageLevel::Error => "message-text error",
    }
}
