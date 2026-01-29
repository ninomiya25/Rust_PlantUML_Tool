# 実装タスク: 社内向けセキュアなPlantUMLウェブエディタ

**Branch**: `001-plantuml-web-editor` | **Date**: 2025-12-26  
**Input**: [spec_revised.md](./spec_revised.md), [plan.md](./plan.md), [data-model.md](./data-model.md), [contracts/api.yaml](./contracts/api.yaml)

---

## タスク概要

本ドキュメントは、spec_revised.mdで定義された3つのユーザーストーリーを実装するための具体的なタスクを、ユーザーストーリー単位でフェーズ分けして記載する。各フェーズは独立してテスト可能で、段階的にデリバリーできるよう設計されている。

**開発アプローチ**: テスト駆動開発 (TDD) + ウォーターフォールV字モデル

---

## Phase 1: セットアップ (プロジェクト初期化)

**目標**: Cargo Workspaceとビルド環境を構築し、開発の基盤を整える。

**独立テスト基準**: `cargo build`が成功し、各クレートが正しくリンクされること。

### タスク

- [X] T001 Cargo Workspaceの初期化 in `Cargo.toml`
- [X] T002 [P] coreクレート作成 in `core/Cargo.toml` + `core/src/lib.rs`
- [X] T003 [P] api-serverクレート作成 in `api-server/Cargo.toml` + `api-server/src/main.rs`
- [X] T004 [P] web-uiクレート作成 in `web-ui/Cargo.toml` + `web-ui/src/main.rs`
- [X] T005 [P] testsディレクトリ構造作成 in `tests/contract/`, `tests/integration/`, `tests/unit/`
- [X] T006 CI/CD設定 (.github/workflows/ci.yml): clippy, fmt, test
- [X] T007 依存関係定義: Axum, Yew, reqwest, tracing, serde, thiserror

---

## Phase 2: 基盤実装 (全ストーリー共通の前提条件)

**目標**: すべてのユーザーストーリーで必要となる共通ロジック (データモデル、バリデーション、PlantUML通信) を実装。

**独立テスト基準**: ユニットテストが80%以上のカバレッジで合格すること。

**ブロッキング**: このフェーズが完了しないと、後続のユーザーストーリー実装は開始できない。

### タスク

#### core/models.rs - データモデル

- [X] T008 PlantUMLDocument構造体定義 in `core/src/models.rs`
- [X] T009 PlantUMLDocument::new()メソッド実装 in `core/src/models.rs`
- [X] T010 PlantUMLDocument::validate()メソッド実装 in `core/src/models.rs`
- [X] T011 [P] DiagramImage構造体定義 in `core/src/models.rs`
- [X] T012 [P] DiagramImage::validate_png()メソッド実装 in `core/src/models.rs`
- [X] T013 [P] DiagramImage::to_data_url()メソッド実装 in `core/src/models.rs`
- [X] T014 [P] StorageSlot構造体定義 in `core/src/models.rs`

#### core/validation.rs - バリデーション

- [X] T015 ValidationError enum定義 in `core/src/validation.rs`
- [X] T016 入力バリデーション関数実装 (UTF-8, 24,000文字上限, @startuml/@enduml) in `core/src/validation.rs`

#### core/client.rs - PlantUML通信

- [X] T017 PlantUmlClient構造体定義 in `core/src/client.rs`
- [X] T018 PlantUmlClient::new()メソッド実装 (reqwest Clientビルダー) in `core/src/client.rs`
- [X] T019 PlantUmlClient::convert_to_png()メソッド実装 in `core/src/client.rs`
- [X] T020 [P] PlantUmlClient::convert_to_svg()メソッド実装 in `core/src/client.rs`
- [X] T021 エラーハンドリング (タイムアウト30秒, 接続エラー) in `core/src/client.rs`

#### 単体テスト

- [X] T022 PlantUMLDocument::validate()テスト (正常系3ケース, 異常系4ケース) in `core/tests/models_test.rs`
- [X] T023 DiagramImage::validate_png()テスト (PNGヘッダー検証) in `core/tests/models_test.rs`
- [X] T024 PlantUmlClient::convert_to_png()テスト (モックサーバー使用) in `core/tests/client_test.rs`

---

## Phase 3: ユーザーストーリー1 - リアルタイム図生成 (P1/MVP)

**目標**: PlantUMLテキスト入力→即座に図表示の基本フローを実装。

**独立テスト基準**: ブラウザでテキスト入力後、500ms debounce経過で自動的に図が表示されること。

**受入条件** (spec_revised.md参照):
1. PlantUMLテキスト入力→シーケンス図がリアルタイム表示
2. テキスト編集→図が自動更新
3. 無効な構文→エラー画像表示

### 契約テスト (テスト先行)

- [X] T025 API契約テスト: POST /api/v1/convert (正常系: PNG生成) in `tests/contract/api_contract_test.rs`
- [X] T026 API契約テスト: POST /api/v1/convert (構文エラー: エラー画像) in `tests/contract/api_contract_test.rs`
- [X] T027 API契約テスト: POST /api/v1/convert (バリデーションエラー) in `tests/contract/api_contract_test.rs`

### バックエンド実装

- [X] T028 [US1] ConvertRequest/ConvertResponse構造体定義 in `api-server/src/models.rs`
- [X] T029 [US1] ErrorResponse構造体定義 in `api-server/src/models.rs`
- [X] T030 [US1] POST /api/v1/convert ハンドラー実装 in `api-server/src/handlers.rs`
- [X] T031 [US1] 入力バリデーションミドルウェア in `api-server/src/middleware.rs`
- [X] T032 [US1] エラーハンドリングミドルウェア in `api-server/src/middleware.rs`
- [X] T033 [US1] tracing構造化ログ設定 in `api-server/src/main.rs`
- [X] T034 [US1] リクエストボディ上限設定 (1MB) in `api-server/src/main.rs`

### フロントエンド実装

- [X] T035 [P] [US1] Appコンポーネント (ルート) 実装 in `web-ui/src/app.rs`
- [X] T036 [US1] Editorコンポーネント (textarea) 実装 in `web-ui/src/components/editor.rs`
- [X] T037 [US1] Debounceロジック実装 (500ms) in `web-ui/src/components/editor.rs`
- [X] T038 [US1] Previewコンポーネント (img要素) 実装 in `web-ui/src/components/preview.rs`
- [X] T039 [US1] ApiClient::convert()メソッド実装 in `web-ui/src/services/api_client.rs`
- [X] T040 [US1] Data URL生成とimg.src設定 in `web-ui/src/components/preview.rs`
- [X] T041 [US1] エラー表示UI (構文エラー画像) in `web-ui/src/components/preview.rs`

### 統合テスト

- [ ] T042 [US1] E2Eテスト: テキスト入力→図表示 in `tests/integration/us1_realtime_test.rs`
- [ ] T043 [US1] E2Eテスト: テキスト編集→図更新 in `tests/integration/us1_realtime_test.rs`
- [ ] T044 [US1] E2Eテスト: 構文エラー→エラー画像表示 in `tests/integration/us1_realtime_test.rs`

---

## Phase 4: ユーザーストーリー2 - エクスポート機能 (P2)

**目標**: 表示中の図をPNG/SVG形式でダウンロードできる。

**独立テスト基準**: エクスポートボタンをクリックして、ブラウザのダウンロードフォルダに画像ファイルが保存されること。

**依存**: US1 (Phase 3) の完了 - 図が表示されていることが前提

**受入条件** (spec_revised.md参照):
1. PNG形式でエクスポートボタン→PNG画像ダウンロード
2. SVG形式でエクスポートボタン→SVG画像ダウンロード

### 契約テスト (テスト先行)

- [X] T045 [P] [US2] API契約テスト: POST /api/v1/convert (SVG生成) in `tests/contract/api_contract_test.rs`

### バックエンド実装

- [X] T046 [P] [US2] format=svg対応 in `api-server/src/handlers.rs`
- [X] T047 [P] [US2] Content-Type切り替え (image/png, image/svg+xml) in `api-server/src/handlers.rs`

### フロントエンド実装

- [X] T048 [P] [US2] ExportButtonsコンポーネント実装 in `web-ui/src/components/export_buttons.rs`
- [X] T049 [US2] PNG/SVGエクスポート関数実装 (Blob + URL.createObjectURL) in `web-ui/src/components/export_buttons.rs`
- [X] T050 [US2] ファイル名生成 (diagram_YYYYMMDD_HHMMSS.png) in `web-ui/src/components/export_buttons.rs`
- [X] T051 [US2] ダウンロードトリガー (a要素のdownload属性) in `web-ui/src/components/export_buttons.rs`

### 統合テスト

- [ ] T052 [US2] E2Eテスト: PNGエクスポート in `tests/integration/us2_export_test.rs`
- [ ] T053 [US2] E2Eテスト: SVGエクスポート in `tests/integration/us2_export_test.rs`

---

## Phase 5: ユーザーストーリー3 - 一時保存・再読込 (P3)

**目標**: PlantUMLテキストをLocalStorageに最大10個保存し、再読込できる。

**独立テスト基準**: 一時保存後にブラウザをリフレッシュしても、保存一覧から選択してテキストを復元できること。

**依存**: US1 (Phase 3) の完了 - エディタが存在することが前提

**受入条件** (spec_revised.md参照):
1. 一時保存ボタン→スロット1に保存
2. 別テキストで一時保存→スロット2に保存
3. 保存一覧からスロット1選択→エディタに読み込み
4. ブラウザ閉じる→再アクセス→保存済みテキスト保持

### フロントエンド実装 (テスト先行)

- [X] T054 [P] [US3] StorageSlot構造体 (serde対応) in `web-ui/src/models.rs`
- [X] T055 [US3] LocalStorageService実装 in `web-ui/src/services/storage_service.rs`
- [X] T056 [US3] save_slot()メソッド (最大10チェック) in `web-ui/src/services/storage_service.rs`
- [X] T057 [US3] load_slot()メソッド in `web-ui/src/services/storage_service.rs`
- [X] T058 [US3] list_slots()メソッド in `web-ui/src/services/storage_service.rs`
- [X] T059 [US3] delete_slot()メソッド (オプション) in `web-ui/src/services/storage_service.rs`
- [X] T060 [P] [US3] SaveButtonコンポーネント実装 in `web-ui/src/components/save_button.rs`
- [X] T061 [P] [US3] SlotListコンポーネント実装 in `web-ui/src/components/slot_list.rs`
- [X] T062 [US3] スロット選択→エディタ読み込みロジック in `web-ui/src/components/slot_list.rs`
- [X] T063 [US3] 満杯時の警告表示 (10スロット) in `web-ui/src/components/save_button.rs`

### 単体テスト

- [ ] T064 [US3] LocalStorageService::save_slot()テスト (正常系) in `web-ui/tests/storage_service_test.rs`
- [ ] T065 [US3] LocalStorageService::save_slot()テスト (満杯エラー) in `web-ui/tests/storage_service_test.rs`
- [ ] T066 [US3] LocalStorageService::load_slot()テスト in `web-ui/tests/storage_service_test.rs`

### 統合テスト

- [ ] T067 [US3] E2Eテスト: 一時保存→スロット1確認 in `tests/integration/us3_storage_test.rs`
- [ ] T068 [US3] E2Eテスト: 2つ保存→それぞれ読み込み in `tests/integration/us3_storage_test.rs`
- [ ] T069 [US3] E2Eテスト: ブラウザリフレッシュ→保持確認 in `tests/integration/us3_storage_test.rs`
- [ ] T070 [US3] E2Eテスト: 10スロット満杯→警告表示 in `tests/integration/us3_storage_test.rs`

---

## Phase 6: ポリッシュ & 横断的機能

**目標**: 全体的なUX向上とクロスブラウザ対応。

**独立テスト基準**: Chrome, Edge, Firefoxで全機能が動作すること。

### タスク

- [X] T071 GET /api/v1/health エンドポイント実装 in `api-server/src/handlers.rs`
- [X] T072 CSS/スタイリング (レスポンシブデザイン) in `web-ui/styles.css`
- [X] T073 ローディングインジケーター (変換中) in `web-ui/src/components/preview.rs`
- [X] T074 エラートースト通知 (ネットワークエラー) in `web-ui/src/components/save_button.rs`
- [ ] T075 クロスブラウザテスト (Chrome, Edge, Firefox) in `tests/browser_compat/`
- [X] T076 パフォーマンステスト (100行/400ms, 90パーセンタイル) in `tests/performance/performance_test.rs`
- [ ] T077 セキュリティテスト (CORS設定, localhost限定) in `tests/security/`
- [ ] T078 アクセシビリティ改善 (キーボード操作, ARIA属性) in `web-ui/src/components/`
- [X] T079 Docker Compose設定 (PlantUML + api-server + web-ui) in `docker-compose.yml`
- [X] T080 Nginx設定 (リバースプロキシ) in `nginx.conf`

---

## 依存関係グラフ (ユーザーストーリー完了順序)

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundation) [ブロッキング]
    ↓
    ├─→ Phase 3 (US1: リアルタイム図生成) [P1/MVP] ← 最優先
    │       ↓
    │       ├─→ Phase 4 (US2: エクスポート) [P2] ← US1依存
    │       │
    │       └─→ Phase 5 (US3: 一時保存) [P3] ← US1依存
    │
    └─→ Phase 6 (Polish) ← 全US完了後
```

**並行実行可能なタスク**:
- Phase 2内: T008-T014 (モデル定義), T015-T016 (バリデーション), T017-T021 (クライアント) は並行可能
- Phase 3内: バックエンド (T028-T034) とフロントエンド (T035-T041) は並行可能
- Phase 4とPhase 5: US2とUS3は独立しており、US1完了後に並行開発可能
- Phase 6内: T071-T074のコンポーネントは並行開発可能

---

## 実装戦略

### MVP (Minimum Viable Product)
**Phase 1-3 (US1のみ)** を完了させることでMVPをリリース可能:
- ユーザーはPlantUMLテキストを入力
- リアルタイムで図が表示される
- 構文エラーも視覚的に確認できる

→ **これだけで実用価値あり** (spec_revised.md §ユーザーストーリー1参照)

### 段階的デリバリー
1. **Week 1-2**: Phase 1-2 (基盤) + Phase 3 (US1) → MVP リリース
2. **Week 3**: Phase 4 (US2: エクスポート) → 機能追加リリース
3. **Week 4**: Phase 5 (US3: 一時保存) → 機能追加リリース
4. **Week 5**: Phase 6 (ポリッシュ) → 正式リリース

---

## テスト戦略

### テストピラミッド

```
        /\
       /E2E\      ← 統合テスト (10-15件)
      /------\
     /契約    \   ← API契約テスト (5-10件)
    /----------\
   /ユニット   \  ← 単体テスト (30-50件)
  /--------------\
```

### カバレッジ目標
- **全体**: 80%以上 (Constitution基準)
- **core/models.rs**: 90%以上 (重要ロジック)
- **api-server/handlers.rs**: 85%以上
- **web-ui/components**: 70%以上 (WASM制約)

### テスト実行順序
1. **ユニットテスト** (`cargo test --lib`)
2. **契約テスト** (`cargo test --test contract`)
3. **統合テスト** (`cargo test --test integration`)
4. **E2Eテスト** (`wasm-pack test --headless --chrome`)

---

## リスク & 対策

| リスク | 影響度 | 対策タスク |
|--------|--------|-----------|
| PlantUML Picoweb起動失敗 | 高 | T017でエラーハンドリング強化、quickstart.mdに詳細手順 |
| WASM互換性問題 | 中 | T075でクロスブラウザテスト自動化 |
| LocalStorage容量超過 | 低 | T056で24KB/スロット制限 (5MB/10スロット = 余裕あり) |
| パフォーマンス未達 | 中 | T076で早期ベンチマーク、必要なら最適化 |

---

## タスク実行ガイドライン

### チェックリストフォーマット
すべてのタスクは以下のフォーマットに従う:
```
- [ ] T001 [P] [US1] 説明 in ファイルパス
```
- `T001`: タスクID (実行順序)
- `[P]`: 並行実行可能 (オプション)
- `[US1]`: ユーザーストーリー (Phase 3-5のみ)
- 説明: 具体的なアクション
- in ファイルパス: 作業対象ファイル

### 作業フロー
1. 契約テスト/単体テストを先に記述 (Red)
2. 実装してテストを合格させる (Green)
3. リファクタリング (Refactor)
4. clippy, fmtチェック
5. タスクを完了としてマーク

---

## 進捗トラッキング

### マイルストーン

| マイルストーン | タスク範囲 | 目標日 | 状態 |
|---------------|-----------|--------|------|
| Setup完了 | T001-T007 | TBD | ⏸未着手 |
| 基盤完了 | T008-T024 | TBD | ⏸未着手 |
| MVP (US1) | T025-T044 | TBD | ⏸未着手 |
| US2完了 | T045-T053 | TBD | ⏸未着手 |
| US3完了 | T054-T070 | TBD | ⏸未着手 |
| 正式リリース | T071-T080 | TBD | ⏸未着手 |

### タスク統計
- **総タスク数**: 80
- **Phase 1 (Setup)**: 7タスク
- **Phase 2 (Foundation)**: 17タスク
- **Phase 3 (US1)**: 20タスク
- **Phase 4 (US2)**: 9タスク
- **Phase 5 (US3)**: 17タスク
- **Phase 6 (Polish)**: 10タスク

### 並行実行機会
- Phase 2内: 3つのサブグループ (models, validation, client)
- Phase 3: バックエンド/フロントエンド並行
- Phase 4-5: US2とUS3並行 (US1完了後)
- Phase 6: 4つの独立コンポーネント

**見積もり効率化**: 最大40%の時間短縮が可能 (並行実行時)

---

## 次のステップ

1. **Phase 1開始**: `cargo init`でWorkspace初期化
2. **Constitution確認**: 各Phaseでclippy/fmt/testゲートを通過
3. **進捗報告**: 各Phase完了時にplan.mdを更新

**準備完了**: 実装開始可能 🚀
