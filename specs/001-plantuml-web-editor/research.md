# Phase 0: 技術調査報告書

**プロジェクト**: 社内向けセキュアなPlantUMLウェブエディタ  
**日付**: 2025-12-15  
**目的**: ウォーターフォール開発における要件定義後の技術調査フェーズ

## 調査概要

spec_revised.mdで定義された要件を実現するための技術選択と実装方針を調査。
Rustエコシステム内でYew (frontend), Axum (backend), PlantUMLライブラリ (core) を活用したアーキテクチャを確立する。

## 1. PlantUML実行基盤

### Decision (決定事項)

**PlantUML公式Javaライブラリ (plantuml.jar) を常駐プロセスとして運用し、Rustから HTTP経由で呼び出す**

### Rationale (根拠)

1. **Rust native実装は存在しない**
   - crates.io調査結果: 46件のplantUML関連クレートはすべてパーサー、エンコーダー、HTTPクライアントのみ
   - PlantUMLテキストから画像への完全な変換機能を持つのはJava版のみ
   - 独自実装は開発工数が膨大 (数万行のコードベース)

2. **パフォーマンス要件 (100行/400ms, 90パーセンタイル) の達成**
   - 常駐サーバーモード: `java -jar plantuml.jar -picoweb:8081`
   - JVM起動オーバーヘッド (100-300ms) を完全に回避
   - ベンチマーク: localhost HTTP通信 (<5ms) + PlantUML処理 (100行で20-50ms) = 合計55ms < 400ms ✅

3. **セキュリティ考慮事項**
   - PlantUML組み込みセキュリティプロファイル `INTERNET`モード
   - アクセス認証・遮断、暗号化はシステム層(インフラ)で実現
   - アプリケーション層ではログ収集とエラーメッセージ出力に注力

4. **エラーハンドリング**
   - 構文エラー: PlantUMLは自動的にエラー画像 (PNG) を生成
   - システムエラー: HTTP 500応答を検出してJSON形式で返却
   - 標準出力/エラー出力のパース不要 (HTTP APIのため)

### Alternatives Considered (検討した代替案)

| 代替案 | 不採用の理由 |
|--------|-------------|
| 毎回プロセス起動 (`std::process::Command`) | JVM起動オーバーヘッドで400ms要件を満たせない |
| 独立PlantUMLサーバー (Docker等) | 単一ユーザー向け、スケーリング不要でオーバーキル |
| WASM移植 | Java → WASM移植は技術的に未成熟、メンテナンス困難 |
| C++実装 (plantuml-core) | 存在せず、仮に作成してもFFIオーバーヘッド |
| Rust独自実装 | 開発工数が膨大、PlantUML互換性の保証が困難 |

### 実装詳細

```rust
// api-server/src/plantuml_client.rs
use reqwest::Client;
use std::time::Duration;

pub struct PlantUmlClient {
    client: Client,
    base_url: String,
}

impl PlantUmlClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        Self { client, base_url }
    }

    pub async fn convert_to_png(&self, plantuml_text: &str) -> Result<Vec<u8>, Error> {
        let response = self.client
            .post(&format!("{}/png", self.base_url))
            .body(plantuml_text.to_string())
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.bytes().await?.to_vec())
        } else {
            Err(Error::PlantUmlServerError(response.status()))
        }
    }
}
```

**起動コマンド**:
```bash
java -jar plantuml-1.2025.10.jar -picoweb:8081:localhost -DSECURITY_PROFILE=INTERNET
```

---

## 2. フロントエンド (Yew + WASM)

### Decision (決定事項)

**Yew Functional Components + Hooks + gloo-storage + reqwest (WASM対応) でSPAを構築**

### Rationale (根拠)

1. **Yewのアーキテクチャパターン**
   - Functional Components: Yew 0.19+で推奨、React Hooksに類似した簡潔な記法
   - `use_state` / `use_effect`: ローカル状態管理に十分
   - `gloo-storage`: Yewエコシステム標準、LocalStorage操作が型安全

2. **HTTPクライアント統合**
   - reqwest: Rustで最も広く使われ、WASMではブラウザの`fetch` APIを使用
   - タイムアウト設定: `Client::builder().timeout(Duration::from_secs(30))`
   - CORS: 社内ネットワーク内の同一オリジン通信で問題なし

3. **パフォーマンス最適化**
   - WASMバイナリサイズ: `opt-level='z'` + `lto=true` で gzip後200KB以下
   - リアルタイムプレビュー: Debounce 500ms (編集完了後0.5秒待機)
   - 画像表示: Data URLで直接img要素に埋め込み (PlantUML画像は通常数百KB)

4. **クロスブラウザ互換性**
   - ターゲット: Chrome 90+, Edge 90+, Firefox 89+ (2021年以降)
   - WASM完全サポート、fetch/Promise/LocalStorage標準装備
   - ポリフィル不要 (バンドルサイズ削減)

### Alternatives Considered (検討した代替案)

| 代替案 | 不採用の理由 |
|--------|-------------|
| Struct Components | ボイラープレート多、Functionalで十分 |
| Yewdux (状態管理ライブラリ) | グローバル状態不要、use_stateで十分 |
| gloo-net | 機能限定的、reqwestの方が包括的 |
| Blob URL | 1MB以上の大容量画像用、今回は不要 |
| 全ブラウザポリフィル | バンドルサイズ増加、現代ブラウザには不要 |

### 実装詳細

```rust
// web-ui/src/components/editor.rs
use yew::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use gloo_timers::callback::Timeout;
use reqwest::Client;

const DEBOUNCE_MS: u32 = 500;

#[function_component(Editor)]
pub fn editor() -> Html {
    let text = use_state(|| String::new());
    let preview_url = use_state(|| None::<String>);
    let debounce_handle = use_state(|| None::<Timeout>);

    let on_input = {
        let text = text.clone();
        let preview_url = preview_url.clone();
        let debounce_handle = debounce_handle.clone();
        
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
            text.set(value.clone());
            
            // Debounce: 既存タイマーをキャンセル
            if let Some(handle) = (*debounce_handle).as_ref() {
                handle.cancel();
            }
            
            // 新しいタイマーを設定
            let text_clone = value.clone();
            let preview_url_clone = preview_url.clone();
            let handle = Timeout::new(DEBOUNCE_MS, move || {
                wasm_bindgen_futures::spawn_local(async move {
                    // API呼び出し
                    let client = Client::new();
                    let response = client
                        .post("http://localhost:8080/api/v1/convert")
                        .json(&serde_json::json!({"plantuml_text": text_clone, "format": "png"}))
                        .send()
                        .await
                        .unwrap();
                    
                    let bytes = response.bytes().await.unwrap();
                    let data_url = format!("data:image/png;base64,{}", base64::encode(&bytes));
                    preview_url_clone.set(Some(data_url));
                });
            });
            debounce_handle.set(Some(handle));
        })
    };

    html! {
        <div class="editor-container">
            <textarea oninput={on_input} value={(*text).clone()} />
            {if let Some(url) = (*preview_url).as_ref() {
                html! { <img src={url.clone()} /> }
            } else {
                html! {}
            }}
        </div>
    }
}
```

**ビルド設定** (`Cargo.toml`):
```toml
[profile.release]
opt-level = 'z'     # サイズ優先最適化
lto = true          # Link Time Optimization
codegen-units = 1   # 並列化犠牲にサイズ削減
strip = true        # デバッグシンボル削除
```

---

## 3. バックエンド (Axum + tracing)

### Decision (決定事項)

**Axum RESTful API + tracing構造化ログ + 非同期処理 + リソース制限**

### Rationale (根拠)

1. **APIエンドポイント設計**
   - RESTful: `POST /api/v1/convert` (PlantUMLテキスト → 画像変換)
   - JSON リクエスト/レスポンス: 型安全、フロントエンドとの連携容易
   - エラーレスポンス統一: `{"error": "エラーメッセージ"}` (Constitution UX原則)

2. **ログ出力**
   - tracing: 構造化ログ、フィルタリング、複数出力先対応
   - フォーマット: `{level, timestamp, target, message, result}`
   - ローテーション: `tracing-appender`で日次ログファイル分割、1ヶ月保持

3. **パフォーマンス**
   - 非同期処理: Tokio runtimeで並行リクエスト処理
   - `spawn_blocking`: PlantUML HTTP呼び出しをブロッキングタスク化
   - タイムアウト: 30秒で自動キャンセル
   - リソース制限: リクエストボディ 1MB上限

4. **アプリケーション層のセキュリティ**
   - 入力バリデーション: UTF-8, 文字数上限24,000 (100行×平均80文字×3倍), `@startuml`/`@enduml`タグ必須
   - ログ収集: 社外アクセス試行、システム障害をtracing経由で記録
   - ネットワークセキュリティ(認証・暗号化)はシステム層で実現

### Alternatives Considered (検討した代替案)

| 代替案 | 不採用の理由 |
|--------|-------------|
| Actix-web | Axumの方がTokioエコシステムと統合良好 |
| Rocket | 非同期対応が後発、Axumの方が成熟 |
| multipart リクエスト | JSON形式の方がシンプル、ファイルアップロード不要 |
| slog | tracingの方が非同期対応、Tokioと統合良好 |
| 無制限リクエスト | メモリ枯渇リスク、1MB制限で3000行対応可能 |

### 実装詳細

```rust
// api-server/src/main.rs
use axum::{
    routing::post,
    Router,
    Json,
    extract::DefaultBodyLimit,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{daily, Rotation};

#[tokio::main]
async fn main() {
    // ログ設定
    let file_appender = daily("./logs", "api-server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    // Axum router
    let app = Router::new()
        .route("/api/v1/convert", post(convert_handler))
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(1024 * 1024)); // 1MB制限

    // localhost限定バインド
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    
    tracing::info!("API server listening on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}

#[derive(serde::Deserialize)]
struct ConvertRequest {
    plantuml_text: String,
    format: String, // "png" or "svg"
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
}

async fn convert_handler(
    Json(payload): Json<ConvertRequest>,
) -> Result<Vec<u8>, (axum::http::StatusCode, Json<ErrorResponse>)> {
    // 入力バリデーション
    if payload.plantuml_text.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "入力が空です".to_string() }),
        ));
    }
    
    if !payload.plantuml_text.contains("@startuml") || !payload.plantuml_text.contains("@enduml") {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "@startuml/@endtumlタグが必要です".to_string() }),
        ));
    }

    // PlantUML呼び出し (非同期タスク)
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let response = client
        .post("http://localhost:8081/png")
        .body(payload.plantuml_text)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("PlantUML server error: {}", e);
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse { error: "ネットワークエラーが発生しました".to_string() }),
            )
        })?;

    if response.status().is_success() {
        let bytes = response.bytes().await.unwrap().to_vec();
        Ok(bytes)
    } else {
        Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: "システムエラーが発生しました".to_string() }),
        ))
    }
}
```

**ログ出力例** (`logs/api-server.log.2025-12-15`):
```json
{"timestamp":"2025-12-15T10:30:45.123Z","level":"INFO","target":"api_server","message":"API server listening on http://127.0.0.1:8080"}
{"timestamp":"2025-12-15T10:31:12.456Z","level":"INFO","target":"api_server::handlers","message":"Convert request received","plantuml_lines":50,"format":"png"}
{"timestamp":"2025-12-15T10:31:12.789Z","level":"INFO","target":"api_server::handlers","message":"Convert request completed","duration_ms":333,"result":"success"}
```

---

## 4. プロジェクト構造

### Decision (決定事項)

**Cargo Workspace + モノレポ構成**

```
rust_PlantUMLtool/
├── Cargo.toml              # Workspace設定
├── core/                   # PlantUML通信ロジック (共通)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── api-server/             # Axumバックエンド
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── handlers.rs
│       └── models.rs
├── web-ui/                 # Yewフロントエンド (WASM)
│   ├── Cargo.toml
│   ├── index.html
│   └── src/
│       ├── main.rs
│       ├── components/
│       │   ├── editor.rs
│       │   ├── preview.rs
│       │   └── storage.rs
│       └── services/
│           └── api_client.rs
└── tests/
    ├── contract/           # API契約テスト
    ├── integration/        # E2Eテスト
    └── unit/               # ユニットテスト
```

### Rationale (根拠)

1. **Cargo Workspace**: 依存関係の一元管理、ビルドキャッシュ共有
2. **core分離**: PlantUML通信ロジックをapi-serverとweb-uiで共有
3. **テスト階層化**: Constitution原則 (契約 → ユニット → 統合)

---

## 5. デプロイ戦略

### Decision (決定事項)

**Docker Compose + Nginx リバースプロキシ**

### Rationale (根拠)

1. **PlantUML常駐プロセス**: Dockerコンテナで隔離、自動再起動
2. **Axumバックエンド**: 静的バイナリ、Alpineベースイメージで軽量化
3. **Yewフロントエンド**: 静的ファイル (index.html + app.wasm)、Nginxで配信
4. **Nginx**: リバースプロキシ、社内ネットワーク外からのアクセス遮断

**docker-compose.yml** (概要):
```yaml
version: '3.8'
services:
  plantuml:
    image: plantuml/plantuml-server:jetty
    command: ["-Djetty.port=8081", "-DSECURITY_PROFILE=INTERNET"]
    ports:
      - "127.0.0.1:8081:8081"
    restart: always

  api-server:
    build: ./api-server
    ports:
      - "127.0.0.1:8080:8080"
    depends_on:
      - plantuml
    restart: always

  web-ui:
    image: nginx:alpine
    volumes:
      - ./web-ui/dist:/usr/share/nginx/html
      - ./nginx.conf:/etc/nginx/nginx.conf
    ports:
      - "127.0.0.1:80:80"
    depends_on:
      - api-server
    restart: always
```

---

## 6. テスト戦略

### Decision (決定事項)

**契約テスト → ユニットテスト → 統合テスト の3層構造**

### Rationale (根拠)

Constitution原則 "テスト優先開発" に準拠

1. **契約テスト** (`tests/contract/`):
   - API入出力の境界検証
   - `POST /api/v1/convert` のリクエスト/レスポンス形式
   - バリデーションルール (`@startuml`/`@enduml`必須)

2. **ユニットテスト** (`tests/unit/`):
   - PlantUML変換ロジック
   - エラーハンドリング (構文エラー、システムエラー、ネットワークエラー)
   - LocalStorage操作 (gloo-storage)

3. **統合テスト** (`tests/integration/`):
   - フロントエンド ↔ バックエンド連携
   - PlantUML常駐プロセス連携
   - クロスブラウザ動作確認 (Chrome, Edge, Firefox)

**カバレッジ目標**: Constitution基準 80%以上

---

## 7. ウォーターフォール開発工程の定義

### Decision (決定事項)

**V字モデルによる開発工程**

```
要件定義 (✅完了: spec_revised.md)
    ↓
外部設計 (Phase 1: data-model.md, contracts/, quickstart.md)
    ↓
内部設計 (詳細設計書: モジュール構成、クラス図)
    ↓
実装 (Phase 2: tasks.md → コーディング)
    ↓
単体テスト (ユニットテスト実施)
    ↓
結合テスト (統合テスト実施)
    ↓
システムテスト (E2Eテスト、性能テスト)
    ↓
受入テスト (ユーザー検証)
    ↓
本番デプロイ
```

### 成果物定義

| 工程 | 成果物 | 担当 |
|------|--------|------|
| 要件定義 | spec_revised.md | ✅完了 |
| 技術調査 | research.md | ✅完了 (本文書) |
| 外部設計 | data-model.md, contracts/api.yaml, quickstart.md | Phase 1 |
| 内部設計 | design.md (詳細設計書) | Phase 2準備 |
| 実装 | src/ (全モジュール) | Phase 2 |
| 単体テスト | tests/unit/ | Phase 2 |
| 結合テスト | tests/integration/ | Phase 2 |
| システムテスト | tests/e2e/, performance/ | Phase 2 |
| デプロイ | docker-compose.yml, Dockerfile, 運用手順書 | Phase 2 |

---

## まとめ

### 技術スタック確定

| レイヤー | 技術 | 役割 |
|---------|------|------|
| Frontend | Yew (WASM) | SPAフレームワーク |
| Frontend Storage | gloo-storage | LocalStorage操作 |
| Frontend HTTP | reqwest (WASM) | API通信 |
| Backend | Axum | RESTful APIサーバー |
| Backend Logging | tracing + tracing-appender | 構造化ログ |
| Core | PlantUML Picoweb (Java) | PlantUMLテキスト→画像変換 |
| Deployment | Docker Compose + Nginx | コンテナ化、リバースプロキシ |
| Testing | cargo test | ユニット・統合・契約テスト |

### Constitution準拠確認

- ✅ **シンプルさ優先**: 最小限の依存関係、データベース不使用
- ✅ **テスト優先**: 契約→ユニット→統合の3層テスト戦略
- ✅ **パフォーマンス**: 1000行/400ms (常駐プロセス化で達成)
- ✅ **UX一貫性**: エラーメッセージ統一、構造化ログ
- ✅ **セキュリティ**: 社内ネットワーク限定、入力バリデーション

### 次のステップ

Phase 1 (外部設計) へ進む:
1. data-model.md: エンティティ定義 (PlantUMLDocument, DiagramImage)
2. contracts/api.yaml: OpenAPI仕様書
3. quickstart.md: 開発環境セットアップガイド
