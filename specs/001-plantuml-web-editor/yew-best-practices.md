# Yew WASM フロントエンド開発ベストプラクティス調査結果

## 調査日: 2025年12月15日

---

## 1. Yewのアーキテクチャパターン

### Decision (決定事項)

**Functional Components**を主要なコンポーネント実装方式として採用し、状態管理には**use_state/use_context**によるReact風のHooksパターンを使用する。LocalStorage連携には**gloo-storage**クレートを利用する。

### Rationale (根拠)

#### 1.1 コンポーネント構成

**Functional Components の優位性:**
- **現代的なパターン**: Yew 0.19以降、Functional Componentsが推奨されている
- **簡潔性**: ボイラープレートコードが少なく、可読性が高い
- **Hooks統合**: `use_state`, `use_effect`, `use_context`などのHooksが自然に使える
- **コンポーネント合成**: 小さな関数として扱えるため、テストと再利用が容易

```rust
#[function_component(MyComponent)]
fn my_component() -> Html {
    let state = use_state(|| 0);
    
    html! {
        <div>{ *state }</div>
    }
}
```

**Struct Components との比較:**
- Struct Componentsは複雑なライフサイクル制御が必要な場合に有用
- しかし、多くのケースでFunctional Componentsで十分
- メンテナンス性と学習曲線を考慮すると、Functionalが優位

#### 1.2 状態管理

**推奨アプローチ:**

1. **ローカル状態**: `use_state` / `use_reducer`
   - コンポーネント内完結の状態
   - フォーム入力、UI状態（開閉状態など）
   
```rust
let (text, set_text) = use_state(|| String::new());
```

2. **グローバル状態**: `use_context` + Context Provider
   - アプリケーション全体で共有する状態
   - ユーザー認証情報、テーマ設定など
   - PlantUMLエディタの場合: PlantUMLコード、設定、履歴

```rust
#[derive(Clone, PartialEq)]
struct AppState {
    plantuml_code: String,
    settings: EditorSettings,
}

// Provider
html! {
    <ContextProvider<AppState> context={state}>
        <App />
    </ContextProvider<AppState>>
}

// Consumer
let state = use_context::<AppState>().expect("no context found");
```

**代替案: Yewdux**
- Reduxライクなグローバル状態管理
- より複雑なアプリケーションに適している
- 今回のPlantUMLエディタには過剰な可能性

#### 1.3 LocalStorage統合

**推奨クレート: gloo-storage**

```rust
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EditorState {
    code: String,
    theme: String,
}

// 保存
LocalStorage::set("editor_state", &state).unwrap();

// 読み込み
let state: EditorState = LocalStorage::get("editor_state")
    .unwrap_or_default();
```

**根拠:**
- `gloo`はYewエコシステムで標準的に使われるWASM用ユーティリティ
- 型安全なserdeベースのシリアライゼーション
- エラーハンドリングが明確

**カスタムHookの作成:**
```rust
#[hook]
fn use_local_storage<T>(key: &str, initial: T) -> UseStateHandle<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    let state = use_state(|| {
        LocalStorage::get::<T>(key).unwrap_or(initial.clone())
    });
    
    let state_clone = state.clone();
    use_effect_with_deps(
        move |state| {
            LocalStorage::set(key, &**state).ok();
            || ()
        },
        (*state).clone(),
    );
    
    state
}
```

### Alternatives Considered (検討した代替案)

1. **Struct Components**: 古いパターン、ボイラープレートが多い
2. **Yewdux/Bounce**: 複雑さが今回のユースケースに不要
3. **web-sys直接呼び出し**: 型安全性とエルゴノミクスでglooに劣る

---

## 2. HTTPクライアント統合

### Decision (決定事項)

**reqwest**クレートの`wasm32`フィーチャーを有効にして使用する。タイムアウト処理には`tokio::time::timeout`を使用し、CORS対応のため適切なヘッダー設定を行う。

### Rationale (根拠)

#### 2.1 reqwest + WASM

**Cargo.toml設定:**
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen-futures = "0.4"
```

**実装例:**
```rust
use reqwest::Client;
use wasm_bindgen_futures::spawn_local;

#[function_component(PlantUmlRenderer)]
fn plantuml_renderer() -> Html {
    let image_data = use_state(|| None);
    let error = use_state(|| None);
    
    let fetch_diagram = {
        let image_data = image_data.clone();
        let error = error.clone();
        
        Callback::from(move |code: String| {
            let image_data = image_data.clone();
            let error = error.clone();
            
            spawn_local(async move {
                match fetch_plantuml_image(&code).await {
                    Ok(data) => image_data.set(Some(data)),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };
    
    // ... render
}

async fn fetch_plantuml_image(code: &str) -> Result<Vec<u8>, reqwest::Error> {
    let client = Client::new();
    let response = client
        .post("https://www.plantuml.com/plantuml/png")
        .header("Content-Type", "text/plain")
        .body(code.to_string())
        .send()
        .await?;
    
    response.bytes().await.map(|b| b.to_vec())
}
```

**根拠:**
- `reqwest`はRustで最も広く使われるHTTPクライアント
- WASM環境ではブラウザの`fetch` APIを自動的に使用
- 非同期処理が自然に書ける
- エラーハンドリングが包括的

#### 2.2 CORS設定の考慮事項

**重要ポイント:**

1. **PlantUMLサーバーのCORS設定**
   - 公式の`www.plantuml.com`はCORS対応済み
   - セルフホストする場合、以下のヘッダーが必要:
     ```
     Access-Control-Allow-Origin: *
     Access-Control-Allow-Methods: POST, GET, OPTIONS
     Access-Control-Allow-Headers: Content-Type
     ```

2. **プロキシパターン**
   - CORS問題を回避するため、独自バックエンドをプロキシとして使う選択肢
   - しかし、今回は公式サーバーを使うため不要

3. **クレデンシャル付きリクエスト**
   - 認証が必要な場合:
   ```rust
   client
       .post(url)
       .header("Authorization", format!("Bearer {}", token))
       .send()
       .await
   ```

#### 2.3 タイムアウト処理

**実装パターン:**

```rust
use std::time::Duration;

async fn fetch_with_timeout(
    code: &str,
    timeout_secs: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;
    
    let response = client
        .post("https://www.plantuml.com/plantuml/png")
        .body(code.to_string())
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }
    
    Ok(response.bytes().await?.to_vec())
}
```

**エラーハンドリング:**
```rust
match fetch_with_timeout(&code, 30).await {
    Ok(data) => {
        // 成功時の処理
    }
    Err(e) => {
        if e.is_timeout() {
            // タイムアウト専用処理
        } else if e.is_connect() {
            // 接続エラー
        } else {
            // その他のエラー
        }
    }
}
```

### Alternatives Considered (検討した代替案)

1. **gloo-net**: Yew向けの軽量HTTPクライアント
   - シンプルだが、reqwestほど機能豊富ではない
   - タイムアウト制御が限定的
   
2. **web-sys fetch直接使用**: 低レベルすぎる、エルゴノミクスが悪い

3. **surf**: 非同期ランタイムの互換性問題

---

## 3. パフォーマンス最適化

### Decision (決定事項)

WASMバイナリサイズは**wasm-opt**と**リリースビルド最適化**で削減する。リアルタイムプレビューには**debounce**パターン（300-500ms）を実装し、画像レンダリングには**Base64エンコーディング + data URL**を使用する。

### Rationale (根拠)

#### 3.1 WASMバイナリサイズの最適化

**Cargo.toml設定:**
```toml
[profile.release]
opt-level = 'z'     # サイズ最適化
lto = true          # Link Time Optimization
codegen-units = 1   # 単一コード生成ユニット
strip = true        # シンボル削除
panic = 'abort'     # パニック時即座に中止
```

**ビルドコマンド:**
```bash
# trunk経由
trunk build --release

# または wasm-pack
wasm-pack build --target web --release

# wasm-optでさらに最適化
wasm-opt -Oz -o output_optimized.wasm output.wasm
```

**最適化効果の目安:**
- デフォルト: ~2-3MB
- release最適化: ~500KB-1MB
- wasm-opt適用: ~300-500KB
- gzip圧縮後: ~100-200KB

**依存クレートの選択:**
- 重い依存を避ける（例: tokioの全機能）
- 必要なフィーチャーのみ有効化
```toml
serde = { version = "1.0", features = ["derive"], default-features = false }
```

#### 3.2 リアルタイムプレビュー実装

**Debounceパターン:**

```rust
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;

#[function_component(PlantUmlEditor)]
fn plantuml_editor() -> Html {
    let code = use_state(String::new);
    let preview = use_state(String::new);
    let timeout_handle: Rc<RefCell<Option<Timeout>>> = 
        use_mut_ref(|| None);
    
    let on_input = {
        let code = code.clone();
        let preview = preview.clone();
        let timeout_handle = timeout_handle.clone();
        
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            let value = input.value();
            code.set(value.clone());
            
            // 既存のタイムアウトをキャンセル
            if let Some(handle) = timeout_handle.borrow_mut().take() {
                handle.forget();
            }
            
            // 新しいタイムアウトを設定（500ms）
            let preview = preview.clone();
            let timeout = Timeout::new(500, move || {
                // ここでPlantUML APIを呼び出す
                spawn_local(async move {
                    if let Ok(img) = fetch_plantuml_image(&value).await {
                        preview.set(img);
                    }
                });
            });
            
            *timeout_handle.borrow_mut() = Some(timeout);
        })
    };
    
    html! {
        <textarea oninput={on_input} value={(*code).clone()} />
    }
}
```

**Throttleパターン（代替）:**
- 最初のイベントを即座に処理、以降を抑制
- ユーザー体験がdebounceと異なる
- PlantUMLの場合、debounceの方が適切（編集完了を待つ）

**パフォーマンス指標:**
- Debounce時間: 300-500ms（バランスが良い）
- 100ms未満: API呼び出し過多
- 1000ms以上: レスポンスが遅く感じる

#### 3.3 大きな画像データのレンダリング

**推奨アプローチ: Data URL**

```rust
use base64::{Engine as _, engine::general_purpose};

fn render_image(image_bytes: &[u8]) -> Html {
    let base64_image = general_purpose::STANDARD.encode(image_bytes);
    let data_url = format!("data:image/png;base64,{}", base64_image);
    
    html! {
        <img 
            src={data_url} 
            alt="PlantUML Diagram"
            style="max-width: 100%; height: auto;"
        />
    }
}
```

**Blob URL（大容量の場合）:**
```rust
use web_sys::{Blob, BlobPropertyBag, Url};
use wasm_bindgen::JsCast;

fn create_blob_url(image_bytes: &[u8]) -> Result<String, JsValue> {
    let uint8_array = js_sys::Uint8Array::from(image_bytes);
    let array = js_sys::Array::new();
    array.push(&uint8_array);
    
    let mut blob_props = BlobPropertyBag::new();
    blob_props.type_("image/png");
    
    let blob = Blob::new_with_u8_array_sequence_and_options(
        &array, 
        &blob_props
    )?;
    
    Url::create_object_url_with_blob(&blob)
}

// 使用後はメモリリーク防止のため解放
Url::revoke_object_url(&blob_url)?;
```

**推奨事項:**
- 1MB未満: Data URL（シンプル）
- 1MB以上: Blob URL（メモリ効率的）
- PlantUML画像は通常数百KB以下なのでData URLで十分

**遅延ローディング:**
```rust
html! {
    <img 
        src={data_url} 
        loading="lazy"  // ネイティブ遅延ローディング
        alt="PlantUML Diagram"
    />
}
```

### Alternatives Considered (検討した代替案)

1. **連続API呼び出し**: リソース無駄、レート制限のリスク
2. **SVG形式**: PNGより軽量だが、複雑な図では描画コストが高い
3. **Canvas描画**: オーバーヘッドが大きい、Data URLの方がシンプル

---

## 4. クロスブラウザ互換性

### Decision (決定事項)

**Chrome 90+, Edge 90+, Firefox 89+**をターゲットブラウザとし、基本的にポリフィルは不要。ただし、`TextEncoder`/`TextDecoder`とPromiseのサポートを前提とする。Safari対応が必要な場合のみ、追加のテストを実施。

### Rationale (根拠)

#### 4.1 WASM対応ブラウザ

**完全サポート（2025年時点）:**
- Chrome 57+ (2017年リリース)
- Firefox 52+ (2017年リリース)
- Edge 79+ (Chromiumベース, 2020年以降)
- Safari 11+ (2017年リリース)

**現実的なターゲット:**
- Chrome/Edge: 90+ (2021年)
- Firefox: 89+ (2021年)
- 理由: 企業環境でもこのバージョン以降が主流

#### 4.2 動作確認ポイント

**Chrome/Edge（Chromiumベース）:**
```rust
// 特別な対応不要
// WebAssembly, fetch, Promise, async/await すべてネイティブサポート
```

**Firefox:**
- WASMパフォーマンスがChromiumと若干異なる場合がある
- 特に大きなWASMモジュールの初回ロード
- 実測で問題なければ対応不要

**Safari（重要な違い）:**
- LocalStorageのストレージ制限が厳しい（5-10MB）
- IndexedDBの使用を検討する場合も
- プライベートブラウジングモードでLocalStorageが無効化される可能性

**テスト項目:**
```markdown
□ WASMモジュールのロード成功
□ fetch APIでPlantUMLサーバーへの通信
□ LocalStorageへの読み書き
□ 画像のBase64表示
□ テキストエリアの入力パフォーマンス
□ Debounceの動作
□ エラーハンドリング表示
```

#### 4.3 ポリフィルの必要性

**不要なケース（現代ブラウザ）:**
- WebAssembly自体
- fetch API
- Promise
- async/await
- LocalStorage
- TextEncoder/TextDecoder

**必要になる可能性のあるケース:**

1. **古いIE11サポート（非推奨）:**
   - WASMが動作しない
   - 現実的には対応不要

2. **Safari特定機能:**
```rust
// LocalStorageが利用不可の場合のフォールバック
use gloo_storage::{LocalStorage, Storage, errors::StorageError};

fn save_with_fallback(key: &str, value: &str) -> Result<(), String> {
    match LocalStorage::set(key, value) {
        Ok(_) => Ok(()),
        Err(StorageError::QuotaExceeded) => {
            // 古いデータを削除して再試行
            LocalStorage::clear();
            LocalStorage::set(key, value)
                .map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}
```

3. **TextEncoder（極めて稀）:**
```toml
# Cargo.tomlに追加（必要に応じて）
[dependencies]
web-sys = { version = "0.3", features = ["TextEncoder", "TextDecoder"] }
```

#### 4.4 開発時のベストプラクティス

**trunk.toml設定:**
```toml
[build]
target = "web"
public_url = "/"

[watch]
ignore = ["target"]

[serve]
address = "127.0.0.1"
port = 8080
open = true
```

**ブラウザ開発者ツール活用:**
- Chromeの場合: `chrome://inspect/#devices` でWASMデバッグ
- Firefoxの場合: `about:debugging` でWASMインスペクト
- Console.logの代わりに`web_sys::console::log_1()`を使用

```rust
use web_sys::console;

console::log_1(&format!("Debug: {}", value).into());
```

**エラーレポーティング:**
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// main()で呼び出す
fn main() {
    set_panic_hook();
    yew::Renderer::<App>::new().render();
}
```

### Alternatives Considered (検討した代替案)

1. **IE11サポート**: WASMが動作しないため現実的でない
2. **全ポリフィル導入**: バンドルサイズ増加、現代ブラウザには不要
3. **Safari専用ビルド**: メンテナンスコスト高、現時点で不要

---

## 総合的な推奨アーキテクチャ

### プロジェクト構成

```
src/
├── main.rs                 # エントリーポイント
├── app.rs                  # ルートコンポーネント
├── components/
│   ├── mod.rs
│   ├── editor.rs           # PlantUMLエディタ
│   ├── preview.rs          # プレビュー表示
│   └── toolbar.rs          # ツールバー
├── hooks/
│   ├── mod.rs
│   ├── use_local_storage.rs
│   └── use_debounce.rs
├── services/
│   ├── mod.rs
│   └── plantuml_api.rs     # API通信
└── state/
    ├── mod.rs
    └── app_state.rs        # グローバル状態
```

### 技術スタック

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-hooks = "0.3"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["Window", "Document", "HtmlElement"] }
gloo = "0.11"
gloo-storage = "0.3"
gloo-timers = "0.3"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.21"
console_error_panic_hook = "0.1"
```

### パフォーマンスターゲット

- **初回ロード**: < 2秒（3G接続）
- **WASM初期化**: < 500ms
- **API応答**: < 3秒（PlantUMLサーバー依存）
- **Debounce**: 300-500ms
- **バンドルサイズ**: < 500KB (gzip圧縮後 < 200KB)

### セキュリティ考慮事項

1. **XSS対策**: Yewは自動的にエスケープ
2. **CORS**: 公式PlantUMLサーバー使用時は対応済み
3. **LocalStorageデータ**: 機密情報は保存しない
4. **依存クレートの監査**: `cargo audit`を定期実行

---

## まとめ

このベストプラクティスガイドは、Yew + WASMでモダンなPlantUMLエディタを構築するための包括的な指針を提供します。

**重要な決定:**
1. Functional Components + Hooks
2. reqwest for HTTP
3. gloo-storage for LocalStorage
4. Debounce 500ms
5. Data URL for images
6. Chrome 90+, Firefox 89+, Edge 90+ ターゲット

**実装時の注意:**
- パフォーマンスプロファイリングを定期的に実施
- ブラウザごとの動作確認
- エラーハンドリングの徹底
- ユーザーフィードバックの収集と改善

**次のステップ:**
1. プロトタイプ実装
2. パフォーマンステスト
3. クロスブラウザテスト
4. ユーザビリティ評価
