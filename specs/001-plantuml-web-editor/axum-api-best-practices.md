# Axum フレームワークでの PlantUML 変換 API サーバー設計: ベストプラクティス

調査日: 2025年12月15日

## エグゼクティブサマリー

本文書では、Axum フレームワークを使用した PlantUML 変換 API サーバーの設計におけるベストプラクティスを、公式ドキュメントおよびエコシステムのドキュメントに基づいて調査し、決定事項としてまとめる。

---

## 1. APIエンドポイント設計

### Decision (決定事項)

1. **RESTful API 設計パターン**
   - エンドポイント: `POST /api/v1/convert`
   - リクエスト形式: JSON (`application/json`)
   - レスポンス形式: JSON with structured error handling
   
2. **リクエスト/レスポンス形式**
   ```rust
   // リクエスト
   {
     "plantuml": "string", // PlantUML source code
     "format": "svg" | "png" // 出力フォーマット
   }
   
   // 成功レスポンス
   {
     "status": "success",
     "data": "base64-encoded-image-data"
   }
   
   // エラーレスポンス
   {
     "status": "error",
     "error": {
       "code": "VALIDATION_ERROR" | "CONVERSION_ERROR" | "INTERNAL_ERROR",
       "message": "Human readable error message",
       "details": {} // Optional additional context
     }
   }
   ```

3. **エラーレスポンス設計**
   - カスタム extractor を実装し、統一されたエラーレスポンスフォーマットを提供
   - HTTP ステータスコードの適切な使用:
     - 200: 成功
     - 400: バリデーションエラー
     - 413: リクエストボディが大きすぎる
     - 500: 内部サーバーエラー
     - 503: サービス過負荷 (load shedding)

### Rationale (根拠)

1. **Axum の設計思想との整合性**
   - Axum は `Json<T>` extractor を提供し、serde によるシリアライズ/デシリアライズを自動的に処理
   - `FromRequest` trait を実装することで、バリデーションロジックをハンドラから分離可能
   - 公式ドキュメントで推奨される型安全なアプローチ

2. **エラーハンドリングの予測可能性**
   - Axum の公式ドキュメントで "Simple and predictable error handling model" が強調されている
   - `Result<T, E>` パターンを使用することで、エラーを明示的に処理
   - カスタム rejection type を定義することで、一貫したエラーレスポンスを実現

3. **JSON vs Multipart の選択**
   - PlantUML のソースコードはテキストデータであり、JSON で十分
   - Multipart は複雑性が増し、必要性が低い
   - Axum の `DefaultBodyLimit` でサイズ制限を簡単に設定可能

### Alternatives Considered (検討した代替案)

1. **Multipart フォームデータ**
   - メリット: ファイルアップロードに適している
   - デメリット: 複雑性が増す、この用途では過剰
   - 却下理由: テキストデータの送信に JSON で十分

2. **GraphQL**
   - メリット: 柔軟なクエリが可能
   - デメリット: シンプルな API には複雑すぎる
   - 却下理由: 単一エンドポイントの単純な変換 API には不要

3. **カスタムバイナリプロトコル**
   - メリット: 効率的
   - デメリット: 実装とデバッグが困難
   - 却下理由: 社内ツールには HTTP/JSON で十分

---

## 2. ログ出力

### Decision (決定事項)

1. **tracing クレートの使用**
   ```rust
   use tracing::{info, warn, error, instrument};
   use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
   
   #[instrument(skip(state))]
   async fn convert_handler(
       State(state): State<AppState>,
       Json(req): Json<ConvertRequest>,
   ) -> Result<Json<ConvertResponse>, ApiError> {
       info!(
           plantuml_length = req.plantuml.len(),
           format = ?req.format,
           "Received conversion request"
       );
       // 処理...
   }
   ```

2. **ログフォーマット設計**
   - 構造化ログ (Structured Logging) を使用
   - フィールド:
     ```
     {
       "level": "INFO" | "WARN" | "ERROR",
       "timestamp": "2025-12-15T10:30:00.123Z",
       "target": "api::convert",
       "span": "convert_handler",
       "fields": {
         "method": "POST",
         "path": "/api/v1/convert",
         "status": 200,
         "duration_ms": 45,
         "request_size": 1024,
         "response_size": 4096
       },
       "message": "Request completed successfully"
     }
     ```

3. **ログローテーション戦略**
   ```rust
   use tracing_appender::rolling;
   use tracing_subscriber::fmt;
   
   let file_appender = rolling::daily("./logs", "plantuml-api.log");
   let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
   
   tracing_subscriber::registry()
       .with(fmt::layer().with_writer(non_blocking).json())
       .with(tracing_subscriber::EnvFilter::from_default_env())
       .init();
   ```
   - 日次ローテーション (`rolling::daily`)
   - ファイル名パターン: `plantuml-api.log.YYYY-MM-DD`
   - 非ブロッキング書き込み (`non_blocking`)

### Rationale (根拠)

1. **tracing の Axum への統合**
   - Axum の公式ドキュメントで tracing が標準として推奨
   - Tower middleware の `TraceLayer` と自然に統合
   - `#[instrument]` 属性により、スパンの自動作成とコンテキスト伝播が可能

2. **構造化ログの利点**
   - JSON フォーマットによる機械可読性
   - ログ分析ツール (Elasticsearch, Splunk など) との統合が容易
   - フィールドベースのフィルタリングとクエリが可能

3. **非ブロッキング I/O とローテーション**
   - `tracing-appender` の `non_blocking` により、ログ書き込みがアプリケーションのパフォーマンスに影響しない
   - `rolling::daily` による自動的なファイルローテーション
   - `WorkerGuard` により、プロセス終了時にバッファがフラッシュされることを保証

4. **tower-http の TraceLayer**
   - HTTP リクエスト/レスポンスの自動ログ記録
   - リクエスト ID の伝播
   - レイテンシの測定

### Alternatives Considered (検討した代替案)

1. **log クレート**
   - メリット: シンプル、広く使われている
   - デメリット: 構造化ログのサポートが弱い、スパンの概念がない
   - 却下理由: tracing のほうが Axum エコシステムと統合されている

2. **slog クレート**
   - メリット: 構造化ログのサポート
   - デメリット: Axum エコシステムとの統合が弱い
   - 却下理由: tracing のほうが Tokio/Axum で標準

3. **手動ローテーション (cronなど)**
   - メリット: 外部ツールによる柔軟な制御
   - デメリット: アプリケーションの複雑性が増す、リアルタイム性が低い
   - 却下理由: `tracing-appender` で十分

---

## 3. パフォーマンス

### Decision (決定事項)

1. **非同期処理によるスループット向上**
   ```rust
   use tokio::task;
   
   async fn convert_plantuml(source: String) -> Result<Vec<u8>, ConversionError> {
       // CPU バウンドな処理を別スレッドで実行
       task::spawn_blocking(move || {
           // PlantUML 変換処理
           plantuml_processor::convert(&source)
       })
       .await
       .map_err(|_| ConversionError::TaskPanicked)?
   }
   ```

2. **タイムアウト設定 (30秒)**
   ```rust
   use tower::ServiceBuilder;
   use tower_http::timeout::TimeoutLayer;
   use std::time::Duration;
   
   let app = Router::new()
       .route("/api/v1/convert", post(convert_handler))
       .layer(
           ServiceBuilder::new()
               .layer(TimeoutLayer::new(Duration::from_secs(30)))
       );
   ```

3. **リソース制限**
   ```rust
   use axum::extract::DefaultBodyLimit;
   use tower_http::limit::RequestBodyLimitLayer;
   
   let app = Router::new()
       .route("/api/v1/convert", post(convert_handler))
       .layer(DefaultBodyLimit::max(1024 * 1024)) // 1MB
       .layer(
           ServiceBuilder::new()
               .layer(tower::load_shed::LoadShedLayer::new())
               .layer(tower::buffer::BufferLayer::new(1024))
       );
   ```

### Rationale (根拠)

1. **Axum の非同期基盤**
   - Axum は Tokio 上に構築されており、非同期処理がネイティブサポート
   - `async fn` ハンドラにより、I/O 待機時に他のリクエストを処理可能
   - Tower の middleware スタックが非同期的に動作

2. **CPU バウンドタスクの処理**
   - PlantUML 変換は CPU バウンドな処理
   - `tokio::task::spawn_blocking` により、ワーカースレッドプールで実行
   - 非同期ランタイムがブロックされるのを防ぐ

3. **タイムアウトの重要性**
   - `tower-http::timeout::TimeoutLayer` による統一的なタイムアウト管理
   - 長時間実行されるリクエストによるリソース枯渇を防止
   - 30秒は PlantUML 変換には十分な時間

4. **リソース制限によるセキュリティ**
   - `DefaultBodyLimit` による request body サイズの制限 (デフォルト 2MB)
   - `LoadShedLayer` により、過負荷時にリクエストを早期に拒否
   - `BufferLayer` により、バックプレッシャーを管理

### Alternatives Considered (検討した代替案)

1. **同期的な処理**
   - メリット: シンプル
   - デメリット: スループットが大幅に低下
   - 却下理由: Axum の非同期の利点を活かせない

2. **無制限のリソース**
   - メリット: 実装が簡単
   - デメリット: DoS 攻撃に脆弱
   - 却下理由: セキュリティリスクが高すぎる

3. **カスタムワーカープール**
   - メリット: 細かい制御が可能
   - デメリット: 実装が複雑、Tokio の機能と重複
   - 却下理由: `spawn_blocking` で十分

---

## 4. セキュリティ

### Decision (決定事項)

1. **社内ネットワーク限定のバインド設定**
   ```rust
   use tokio::net::TcpListener;
   
   #[tokio::main]
   async fn main() {
       let listener = TcpListener::bind("127.0.0.1:3000")
           .await
           .expect("Failed to bind");
       
       axum::serve(listener, app).await.unwrap();
   }
   ```
   - localhost (`127.0.0.1`) または社内ネットワークの IP アドレスにバインド
   - `0.0.0.0` へのバインドは避ける

2. **入力バリデーション**
   ```rust
   use validator::Validate;
   use serde::Deserialize;
   
   #[derive(Debug, Deserialize, Validate)]
   struct ConvertRequest {
       #[validate(length(min = 1, max = 100_000))]
       plantuml: String,
       
       #[validate(custom = "validate_format")]
       format: String,
   }
   
   fn validate_format(format: &str) -> Result<(), validator::ValidationError> {
       match format {
           "svg" | "png" => Ok(()),
           _ => Err(validator::ValidationError::new("invalid_format")),
       }
   }
   ```
   - ファイルサイズ上限: 1MB (body limit)
   - PlantUML ソースの長さ制限: 100,000 文字
   - 文字エンコーディング: UTF-8 (自動的に検証される)
   - 出力フォーマットのホワイトリスト検証

3. **CORS設定** (社内ネットワーク用)
   ```rust
   use tower_http::cors::{CorsLayer, Any};
   use http::Method;
   
   let cors = CorsLayer::new()
       .allow_origin(["http://localhost:8080".parse().unwrap()])
       .allow_methods([Method::POST, Method::OPTIONS])
       .allow_headers(Any);
   
   let app = Router::new()
       .route("/api/v1/convert", post(convert_handler))
       .layer(cors);
   ```
   - 必要な origin のみを許可
   - 必要な HTTP メソッドのみを許可
   - 社内ツールの場合、特定のポートのみを許可

4. **Sensitive Headers**
   ```rust
   use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
   use http::header::AUTHORIZATION;
   
   let app = Router::new()
       .route("/api/v1/convert", post(convert_handler))
       .layer(SetSensitiveRequestHeadersLayer::new([AUTHORIZATION]));
   ```

### Rationale (根拠)

1. **ネットワークレベルのセキュリティ**
   - 社内ネットワーク限定により、攻撃面を最小化
   - localhost バインドはローカルアクセスのみを許可
   - Reverse proxy (nginx など) を使用する場合も、適切なバインド設定が重要

2. **入力バリデーションの重要性**
   - Axum の `Json` extractor は自動的に UTF-8 を検証
   - `DefaultBodyLimit` により、メモリ枯渇攻撃を防止
   - カスタム validation により、不正なフォーマット指定を拒否
   - validator クレートにより、宣言的なバリデーション

3. **CORS の適切な設定**
   - `tower-http::cors::CorsLayer` による統一的な CORS 管理
   - 社内ツールでも、CSRF 攻撃を防ぐために適切な origin 制限が重要
   - Preflight リクエスト (OPTIONS) の自動処理

4. **ログにおけるセンシティブ情報の保護**
   - `SetSensitiveRequestHeadersLayer` により、Authorization ヘッダーがログに記録されない
   - tower-http の TraceLayer と統合

### Alternatives Considered (検討した代替案)

1. **認証/認可機構の追加**
   - メリット: より強固なセキュリティ
   - デメリット: 社内ツールには過剰な複雑性
   - 却下理由: 社内ネットワーク限定で十分 (ただし、将来的に検討の余地あり)

2. **Rate Limiting**
   - メリット: DoS 攻撃の緩和
   - デメリット: 実装の複雑性
   - 却下理由: 社内ツールでは優先度が低い (ただし、tower の rate limit middleware は利用可能)

3. **HTTPS の強制**
   - メリット: 通信の暗号化
   - デメリット: 証明書管理の複雑性
   - 却下理由: 社内ネットワークでは優先度が低い (reverse proxy で対応可能)

---

## 5. 実装例

以下は、上記のベストプラクティスを統合した実装例です:

```rust
use axum::{
    extract::{State, Json, DefaultBodyLimit},
    routing::post,
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    timeout::TimeoutLayer,
    cors::CorsLayer,
    trace::TraceLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
};
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling;
use validator::Validate;

// アプリケーション状態
#[derive(Clone)]
struct AppState {
    // 共有リソース (例: データベース接続プールなど)
}

// リクエスト型
#[derive(Debug, Deserialize, Validate)]
struct ConvertRequest {
    #[validate(length(min = 1, max = 100_000))]
    plantuml: String,
    
    #[validate(custom = "validate_format")]
    format: String,
}

fn validate_format(format: &str) -> Result<(), validator::ValidationError> {
    match format {
        "svg" | "png" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_format")),
    }
}

// レスポンス型
#[derive(Debug, Serialize)]
struct ConvertResponse {
    status: String,
    data: String,
}

// エラー型
#[derive(Debug)]
enum ApiError {
    ValidationError(String),
    ConversionError(String),
    InternalError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::ConversionError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        
        let body = serde_json::json!({
            "status": "error",
            "error": {
                "code": format!("{:?}", self),
                "message": message,
            }
        });
        
        (status, Json(body)).into_response()
    }
}

// ハンドラ
#[instrument(skip(state))]
async fn convert_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
    // バリデーション
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;
    
    info!(
        plantuml_length = req.plantuml.len(),
        format = %req.format,
        "Processing conversion request"
    );
    
    // CPU バウンドな処理を別スレッドで実行
    let source = req.plantuml.clone();
    let result = tokio::task::spawn_blocking(move || {
        // PlantUML 変換処理 (ここは実装依存)
        simulate_conversion(&source)
    })
    .await
    .map_err(|_| ApiError::InternalError)?
    .map_err(|e| ApiError::ConversionError(e))?;
    
    Ok(Json(ConvertResponse {
        status: "success".to_string(),
        data: base64::encode(result),
    }))
}

// PlantUML 変換のシミュレーション
fn simulate_conversion(source: &str) -> Result<Vec<u8>, String> {
    // 実際の実装はここに
    Ok(b"fake-image-data".to_vec())
}

#[tokio::main]
async fn main() {
    // ログ設定
    let file_appender = rolling::daily("./logs", "plantuml-api.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .json()
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // アプリケーション状態
    let state = Arc::new(AppState {});
    
    // CORS 設定
    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:8080".parse().unwrap()])
        .allow_methods([axum::http::Method::POST, axum::http::Method::OPTIONS]);
    
    // ルーター構築
    let app = Router::new()
        .route("/api/v1/convert", post(convert_handler))
        .layer(
            ServiceBuilder::new()
                // タイムアウト
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                // センシティブヘッダー
                .layer(SetSensitiveRequestHeadersLayer::new([
                    axum::http::header::AUTHORIZATION
                ]))
                // トレース
                .layer(TraceLayer::new_for_http())
                // CORS
                .layer(cors)
        )
        // ボディサイズ制限
        .layer(DefaultBodyLimit::max(1024 * 1024)) // 1MB
        .with_state(state);
    
    // サーバー起動
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind");
    
    info!("Server listening on 127.0.0.1:3000");
    
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
```

---

## 6. 依存関係

推奨される `Cargo.toml` の依存関係:

```toml
[dependencies]
axum = { version = "0.8", features = ["json", "tracing"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "timeout", "cors", "sensitive-headers", "limit"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.19", features = ["derive"] }
base64 = "0.22"
```

---

## 7. まとめ

本文書で提示したベストプラクティスは、以下の原則に基づいています:

1. **Axum エコシステムとの統合**: 公式の推奨パターンに従う
2. **型安全性**: Rust の型システムを活用し、コンパイル時にエラーを検出
3. **パフォーマンス**: 非同期処理とリソース制限により、高スループットと安定性を実現
4. **セキュリティ**: Defense in depth の原則に従い、複数の防御層を設ける
5. **運用性**: 構造化ログとメトリクスにより、監視と診断を容易にする

これらの決定は、社内ツールとしての要件と、将来的な拡張性のバランスを取ったものです。

---

## 参考資料

- [Axum 公式ドキュメント](https://docs.rs/axum/latest/axum/)
- [Tower 公式ドキュメント](https://docs.rs/tower/latest/tower/)
- [tower-http 公式ドキュメント](https://docs.rs/tower-http/latest/tower_http/)
- [tracing 公式ドキュメント](https://docs.rs/tracing/latest/tracing/)
- [tracing-subscriber 公式ドキュメント](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/)
- [tracing-appender 公式ドキュメント](https://docs.rs/tracing-appender/latest/tracing_appender/)
- [Axum GitHub リポジトリ](https://github.com/tokio-rs/axum)
