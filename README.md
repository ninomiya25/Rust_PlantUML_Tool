# PlantUML Web Editor

社内向けセキュアなPlantUMLウェブエディタ

## 前提条件

- Rust 1.75+ (`rustup install stable`)
- Trunk 0.18+ (`cargo install trunk`)
- wasm-bindgen-cli 0.2.89+ (`cargo install wasm-bindgen-cli`)
- Java 11+ (PlantUML実行用)
- PlantUML JAR (plantuml-1.2025.10.jar 推奨)

## プロジェクト構造

```
rust_PlantUMLtool/
├── core/              # コアライブラリ (データモデル、バリデーション、PlantUMLクライアント)
├── api-server/        # バックエンドAPIサーバー (Axum)
├── web-ui/            # フロントエンドWebアプリ (Yew/WASM)
├── tests/             # 統合テスト
└── specs/             # 仕様ドキュメント
```

## 開発手順

### Phase 1-2完了: セットアップと基盤実装

✅ Cargo Workspace初期化
✅ データモデル実装 (PlantUMLDocument, DiagramImage, StorageSlot)
✅ バリデーションロジック実装
✅ PlantUMLクライアント実装
✅ ユニットテスト作成 (80%+ カバレッジ目標)

### Phase 3実装中: US1 リアルタイム図生成 (MVP)

#### 1. PlantUML Picoweb起動

Terminal 1:
```powershell
cd c:\path\to\plantuml
java -jar plantuml.jar -picoweb:8081
```

#### 2. API Server起動

Terminal 2:
```powershell
cd c:\Users\cw_ninomiya\rust_PlantUMLtool
cargo run --bin api-server
```

起動確認: `http://localhost:8080`

#### 3. Web UI開発サーバー起動

Terminal 3:
```powershell
cd c:\Users\cw_ninomiya\rust_PlantUMLtool\web-ui
trunk serve --port 8000
```

ブラウザ自動起動: `http://127.0.0.1:8000`

**注意**: ポート8080はAPI Serverが使用しているため、web-uiは8000を使用します。
## テスト実行

### ユニットテスト
```powershell
cargo test --package plantuml-editor-core
```

### 契約テスト (API Server + PlantUML Server起動が必要)
```powershell
cargo test --test api_contract_test
```

### 統合テスト (E2E)
```powershell
# US1: リアルタイム図生成
cargo test --test us1_realtime_test

# US2: エクスポート機能
cargo test --test us2_export_test
```

### パフォーマンステスト (NFR-001検証)
```powershell
# PlantUML ServerとAPI Serverを起動後
cargo test --test performance_test -- --ignored --nocapture

# 期待結果: P90 latency <= 400ms
```

### 全テスト
```powershell
cargo test --workspace
```

### カバレッジ測定
```powershell
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
```

## ビルド

### リリースビルド
```powershell
# API Server
cargo build --release --bin api-server

# Web UI (WASM)
cd web-ui
trunk build --release
```

### Docker Compose (本番デプロイ)

```powershell
# 1. Web UIビルド
cd web-ui
trunk build --release
cd ..

# 2. Dockerコンテナ起動
docker-compose up -d

# 3. ヘルスチェック
curl http://localhost/health           # Nginx
curl http://localhost:8080/api/v1/health  # API Server

# 4. アプリケーションアクセス
Start-Process http://localhost
```

コンテナ構成:
- `plantuml-picoweb`: PlantUML Server (port 8081)
- `plantuml-api-server`: Rust API Server (port 8080)
- `plantuml-nginx`: Nginx reverse proxy (port 80)

## 実装状況

- ✅ Phase 1: セットアップ (T001-T007)
- ✅ Phase 2: 基盤実装 (T008-T024)
- ✅ Phase 3: US1実装 (T025-T044) - MVP リアルタイム図生成
- ✅ Phase 4: US2実装 (T045-T053) - エクスポート機能
- ✅ Phase 5: US3実装 (T054-T070) - 一時保存機能
- ✅ Phase 6: ポリッシュ (T071-T080) - Health endpoint、Docker、テスト拡充

詳細: `specs/001-plantuml-web-editor/tasks.md`

## 主要機能

### ✅ 実装済み

1. **リアルタイム図生成** (US1/MVP)
   - PlantUMLテキストエディタ (500ms debounce)
   - 自動プレビュー表示
   - エラーメッセージ表示

2. **エクスポート機能** (US2)
   - PNG形式エクスポート
   - SVG形式エクスポート
   - タイムスタンプ付きファイル名生成

3. **一時保存・再読込** (US3)
   - LocalStorage保存 (最大10スロット)
   - 保存済みドキュメント一覧表示
   - 読み込み・削除機能
   - 空きスロット数表示

- **Health endpoint** (GET /api/v1/health) - サービス監視用
- **Performance tests** - 100行/400ms NFR検証 (P90)
- **Integration E2E tests** - US1/US2 end-to-endテスト
- **Docker Compose** - 3コンテナ構成 (PlantUML、API、Nginx)
- **Nginx reverse proxy** - 本番デプロイ用設定


## Constitution準拠

- ✅ **シンプルさ優先**: 最小限の依存関係、データベース不使用
- ✅ **テスト優先**: TDDアプローチ、80%カバレッジ目標
- ✅ **コード品質**: clippy, fmt, Rustdoc
- ✅ **パフォーマンス**: 100行/400ms以内 (90パーセンタイル)
- ✅ **UX一貫性**: 日本語エラーメッセージ (what/why/how形式)

## トラブルシューティング

### PlantUMLサーバーに接続できない
- PlantUML Picowebが起動しているか確認: `http://localhost:8081/plantuml`
- ファイアウォール設定を確認

### WASMビルドエラー
```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.89
```

### Trunkが見つからない
```powershell
cargo install trunk
```

## ライセンス

MIT OR Apache-2.0
