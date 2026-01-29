# 実装計画: 社内向けセキュアなPlantUMLウェブエディタ

**Branch**: `001-plantuml-web-editor` | **Date**: 2025-12-15 | **Spec**: [spec_revised.md](./spec_revised.md)
**Input**: `/specs/001-plantuml-web-editor/spec_revised.md` の機能仕様

**注記**: このテンプレートは `/speckit.plan` コマンドにより入力されます。実行ワークフローについては `.specify/templates/commands/plan.md` を参照してください。

## 概要

環境構築なしで全社員がPlantUMLによるUML図を作成できるウェブエディタを開発する。
Rustエコシステムを活用し、クライアント・サーバーアーキテクチャで実装する。
フロントエンドはYewフレームワークによるWASM、バックエンドはAxumによるWebサーバー、
コアロジックにPlantUMLライブラリを統合する。社内ネットワーク限定でセキュアに運用する。

## 技術的コンテキスト

**Language/Version**: Rust 1.75+  
**Primary Dependencies**:
- **core**: PlantUMLライブラリ (PlantUMLテキストから図への変換処理)
- **api-server**: Axum (Webアプリケーションサーバフレームワーク), tracing (ログ出力)
- **api-client**: reqwest (HTTPクライアント、フロントエンド→バックエンド通信)
- **web-ui**: Yew (Rustで記述するフロントエンドWebアプリケーションフレームワーク、WASM出力)

**Storage**: LocalStorage (ブラウザ側、最大10個のPlantUMLテキスト一時保存)  
**Testing**: cargo test (ユニット・統合・契約テスト)  
**Target Platform**: 
- Frontend: WASM (Chrome, Edge, Firefox最新版)
- Backend: Linux server (社内ネットワーク内)

**Project Type**: Web application (frontend + backend)  
**Performance Goals**: 
- 100行のPlantUMLソースを90パーセンタイルで400ms以内で処理
- メモリ使用量はファイルサイズの3倍以内
- テキスト編集から図更新まで2秒以内

**Constraints**: 
- データベース不使用
- ユーザー認証不使用
- オフライン対応不要
- セキュリティ(認証・暗号化)はシステム層で実現

**Scale/Scope**: 
- 対象ユーザー: 全社員
- 対応PlantUMLファイル: 3000行まで (推奨1000行以内)
- 一時保存: 最大10スロット (LocalStorage)

## Constitution Check

*ゲート: Phase 0 調査前に合格する必要があります。Phase 1 設計後に再チェックしてください。*

この機能が以下の原則に準拠していることを確認してください:

- **シンプルさ優先**: ✅ 新規依存関係は最小限
  - Yew (WASM frontend), Axum (backend), reqwest (HTTP client), tracing (logging), PlantUMLライブラリ (core)
  - 各クレートは明確な役割を持ち、標準ライブラリで代替不可能な機能を提供
  - データベース不使用、ユーザー認証不使用でシンプルさを維持
  
- **テスト優先**: ✅ テストを先に記述する計画
  - 契約テスト: API入出力の境界検証 (POST /api/v1/convert)
  - ユニットテスト: PlantUML変換ロジック、バリデーション、エラーハンドリング
  - 統合テスト: フロントエンド↔バックエンド連携、LocalStorage永続化
  
- **コード品質**: ✅ clippy、fmt、ドキュメントの基準を満たす計画
  - すべての公開APIにRustdocコメント記述
  - CI/CDでclippy警告ゼロ、fmtチェック実施
  - 複雑な関数は分割 (cyclomatic complexity < 10)
  
- **パフォーマンス**: ✅ 基準を満たす設計
  - 100行/400ms以内 (90パーセンタイル) - 実用的な範囲で十分な性能
  - メモリ使用量はファイルサイズの3倍以内
  - WASMによるクライアント側最適化とAxumによる効率的なサーバー処理
  
- **UX一貫性**: ✅ エラーメッセージ形式統一
  - 構文エラー: PlantUMLが生成するエラー画像で表示 (何が/なぜ/どう修正)
  - システムエラー: JSON形式 {"error": "システムエラーが発生しました"}
  - ネットワークエラー: JSON形式 {"error": "ネットワークエラーが発生しました"}
  - ログ形式: {ログレベル, 時間, アクセス対象, 処理内容, 処理結果, メッセージ}
  
- **バージョニング**: ✅ 初回リリース、破壊的変更なし
  - セマンティックバージョニング適用 (初回は0.1.0)
  - 今後の破壊的変更時は2バージョン前に非推奨警告を表示

**違反**: なし

## プロジェクト構造

### ドキュメント (この機能)

```text
specs/[###-feature]/
├── plan.md              # このファイル (/speckit.plan コマンド出力)
├── research.md          # Phase 0 出力 (/speckit.plan コマンド)
├── data-model.md        # Phase 1 出力 (/speckit.plan コマンド)
├── quickstart.md        # Phase 1 出力 (/speckit.plan コマンド)
├── contracts/           # Phase 1 出力 (/speckit.plan コマンド)
└── tasks.md             # Phase 2 出力 (/speckit.tasks コマンド - /speckit.plan では作成されません)
```

### ソースコード (リポジトリルート)

```text
rust_PlantUMLtool/
├── Cargo.toml              # Workspace設定
├── core/                   # PlantUML通信ロジック (共通)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── client.rs       # PlantUMLサーバークライアント
│       ├── models.rs       # PlantUMLDocument, DiagramImage
│       └── validation.rs   # バリデーションロジック
├── api-server/             # Axumバックエンド
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── handlers.rs     # /api/v1/convert, /health
│       ├── middleware.rs   # ログ、エラーハンドリング
│       └── config.rs       # 環境変数、設定
├── web-ui/                 # Yewフロントエンド (WASM)
│   ├── Cargo.toml
│   ├── index.html
│   ├── Trunk.toml
│   └── src/
│       ├── main.rs
│       ├── app.rs          # ルートコンポーネント
│       ├── components/
│       │   ├── editor.rs   # PlantUMLテキストエディタ
│       │   ├── preview.rs  # 図プレビュー表示
│       │   └── storage.rs  # LocalStorage操作
│       └── services/
│           └── api_client.rs # バックエンドAPI通信
└── tests/
    ├── contract/           # API契約テスト
    │   └── api_contract_test.rs
    ├── integration/        # E2Eテスト
    │   └── full_flow_test.rs
    └── unit/               # ユニットテスト (各クレート内)
```

**構造の決定**: Cargo Workspaceによるモノレポ構成。research.mdで定義した通り、core (共通ロジック), api-server (バックエンド), web-ui (フロントエンド) の3クレート構成。

## 複雑さの追跡

**Constitution Check に違反なし、このセクションは該当なし**

---

## ウォーターフォール開発工程

### 工程概要 (V字モデル)

```
要件定義 ────────────────────┐
    ↓                        │
外部設計 ──────────────────┐ │
    ↓                      │ │
内部設計 ────────────────┐ │ │
    ↓                    │ │ │
実装 ──────────────────┐ │ │ │
    ↓                  │ │ │ │
単体テスト ←───────────┘ │ │ │
    ↓                    │ │ │
結合テスト ←─────────────┘ │ │
    ↓                      │ │
システムテスト ←───────────┘ │
    ↓                        │
受入テスト ←─────────────────┘
    ↓
本番デプロイ
```

### Phase 0: 要件定義 (✅完了)

| 項目 | 成果物 | 状態 | 完了日 |
|------|--------|------|--------|
| 機能仕様書 | spec_revised.md | ✅完了 | 2025-12-15 |
| ユーザーストーリー定義 | spec_revised.md §ユーザーストーリー | ✅完了 | 2025-12-15 |
| 非機能要件定義 | spec_revised.md §技術・品質要件 | ✅完了 | 2025-12-15 |
| 技術調査 | research.md | ✅完了 | 2025-12-16 |

**レビューポイント**:
- [x] Constitution原則に準拠しているか
- [x] パフォーマンス要件は測定可能か (1000行/400ms)
- [x] セキュリティ要件は明確か (社内ネットワーク限定)

---

### Phase 1: 外部設計 (✅完了)

| 項目 | 成果物 | 状態 | 完了日 |
|------|--------|------|--------|
| データモデル定義 | data-model.md | ✅完了 | 2025-12-16 |
| API仕様書 | contracts/api.yaml (OpenAPI 3.0) | ✅完了 | 2025-12-16 |
| 画面設計 | spec_revised.md §画面遷移 | ✅完了 | 2025-12-15 |
| クイックスタート | quickstart.md | ✅完了 | 2025-12-16 |

**レビューポイント**:
- [x] エンティティ間の関係は明確か
- [x] API契約はバリデーションルールを含むか
- [x] エラーレスポンスはConstitution UX原則に準拠しているか

---

### Phase 2: 内部設計 (🔄次工程)

| 項目 | 成果物 | 状態 | 担当 |
|------|--------|------|------|
| 詳細設計書 | design/architecture.md | ⏸未着手 | TBD |
| クラス図 | design/class-diagram.puml | ⏸未着手 | TBD |
| シーケンス図 | design/sequence-diagram.puml | ⏸未着手 | TBD |
| データベース設計 | N/A (LocalStorageのみ) | - | - |

**内容**:
- モジュール構成の詳細 (core, api-server, web-ui)
- 主要クラス/構造体の責務と関係
- API呼び出しフロー (フロントエンド → バックエンド → PlantUML)
- エラーハンドリングフロー (構文エラー、システムエラー、ネットワークエラー)

**レビューポイント**:
- [ ] モジュール間の依存関係は最小限か
- [ ] 循環参照は発生していないか
- [ ] テスト容易性は考慮されているか

---

### Phase 3: 実装 (⏸未着手)

| 項目 | 成果物 | 状態 | 担当 |
|------|--------|------|------|
| core実装 | core/src/ | ⏸未着手 | TBD |
| api-server実装 | api-server/src/ | ⏸未着手 | TBD |
| web-ui実装 | web-ui/src/ | ⏸未着手 | TBD |
| 単体テスト | tests/unit/ | ⏸未着手 | TBD |

**実装順序 (テスト駆動開発)**:
1. **core/models.rs**: PlantUMLDocument, DiagramImage, バリデーション
   - テスト: `tests/unit/models_test.rs`
2. **core/client.rs**: PlantUMLサーバークライアント
   - テスト: `tests/unit/client_test.rs` (モックサーバー使用)
3. **api-server/handlers.rs**: /api/v1/convert, /health
   - テスト: `tests/contract/api_contract_test.rs`
4. **web-ui/components/editor.rs**: テキストエディタ + Debounce
   - テスト: `web-ui/tests/editor_test.rs` (wasm-pack test)
5. **web-ui/components/preview.rs**: 画像プレビュー
   - テスト: `web-ui/tests/preview_test.rs`
6. **web-ui/components/storage.rs**: LocalStorage操作
   - テスト: `web-ui/tests/storage_test.rs`

**コーディング規約**:
- Constitution原則に準拠 (clippy, fmt, Rustdoc)
- `unsafe` コード禁止 (正当化が必要)
- 公開APIには必ず `#[doc]` コメント

---

### Phase 4: 単体テスト (⏸未着手)

| 項目 | 成果物 | 状態 | 担当 |
|------|--------|------|------|
| coreユニットテスト | core/tests/ | ⏸未着手 | TBD |
| api-serverユニットテスト | api-server/tests/ | ⏸未着手 | TBD |
| web-uiユニットテスト | web-ui/tests/ | ⏸未着手 | TBD |
| カバレッジ測定 | coverage/index.html | ⏸未着手 | TBD |

**テストカバレッジ目標**: 80%以上 (Constitution基準)

**テスト項目**:
- [ ] PlantUMLDocument::validate() - 正常系、異常系 (空、タグ欠如、サイズ超過)
- [ ] DiagramImage::validate_png() - PNGヘッダー検証、サイズ検証
- [ ] StorageSlot::validate_slot_number() - 範囲チェック (1-10)
- [ ] ConvertRequest::validate() - 入力バリデーション
- [ ] ErrorResponse::system_error() - エラーメッセージ形式

---

### Phase 5: 結合テスト (⏸未着手)

| 項目 | 成果物 | 状態 | 担当 |
|------|--------|------|------|
| API契約テスト | tests/contract/ | ⏸未着手 | TBD |
| フロント↔バック連携 | tests/integration/ | ⏸未着手 | TBD |
| PlantUML連携テスト | tests/integration/plantuml_test.rs | ⏸未着手 | TBD |

**テスト項目**:
- [ ] POST /api/v1/convert - 正常系 (PNG/SVG生成)
- [ ] POST /api/v1/convert - 構文エラー (エラー画像返却)
- [ ] POST /api/v1/convert - タイムアウト (30秒)
- [ ] GET /api/v1/health - ヘルスチェック
- [ ] LocalStorage永続化 - ブラウザリフレッシュ後も保持
- [ ] Debounce動作 - 500ms待機後にAPIリクエスト

---

### Phase 6: システムテスト (⏸未着手)

| 項目 | 成果物 | 状態 | 担当 |
|------|--------|------|------|
| E2Eテスト | tests/e2e/ | ⏸未着手 | TBD |
| パフォーマンステスト | tests/performance/ | ⏸未着手 | TBD |
| クロスブラウザテスト | tests/browser_compat/ | ⏸未着手 | TBD |
| セキュリティテスト | tests/security/ | ⏸未着手 | TBD |

**テスト項目**:
- [ ] **ユーザーストーリー1**: リアルタイム図生成 (編集→プレビュー更新)
- [ ] **ユーザーストーリー2**: PNG/SVGエクスポート
- [ ] **ユーザーストーリー3**: 一時保存・再読込 (10スロット)
- [ ] **パフォーマンス**: 100行/400ms以内 (90パーセンタイル) (NFR-001)
- [ ] **ブラウザ互換性**: Chrome, Edge, Firefox (NFR-002)
- [ ] **ログ収集**: 社外アクセス・障害ログ (NFR-003)

---

### Phase 7: 受入テスト (⏸未着手)

| 項目 | 成果物 | 状態 | 担当 |
|------|--------|------|------|
| ユーザー受入テスト計画 | uat/test-plan.md | ⏸未着手 | TBD |
| UAT実施報告書 | uat/report.md | ⏸未着手 | TBD |
| 本番デプロイ手順書 | deploy/manual.md | ⏸未着手 | TBD |

**UAT項目**:
- [ ] 設計者による実際のUML作成業務での動作確認
- [ ] 初回利用時の操作理解度 (目標: 90%) - SC-004
- [ ] 1分以内に図作成開始できるか - SC-001
- [ ] 編集→プレビュー更新が2秒以内か - SC-002

---

### Phase 8: 本番デプロイ (⏸未着手)

| 項目 | 成果物 | 状態 | 担当 |
|------|--------|------|------|
| Dockerイメージ | docker-compose.yml, Dockerfile | ⏸未着手 | TBD |
| Nginx設定 | nginx.conf | ⏸未着手 | TBD |
| 運用手順書 | ops/manual.md | ⏸未着手 | TBD |
| 監視設定 | ops/monitoring.md | ⏸未着手 | TBD |

**デプロイ手順**:
1. リリースビルド (`cargo build --release`)
2. Dockerイメージビルド (`docker-compose build`)
3. 社内サーバーへデプロイ (`docker-compose up -d`)
4. ヘルスチェック (`curl http://plantuml.internal.company.com/api/v1/health`)
5. ログ監視開始 (1ヶ月保持 - NFR-005)

---

## 進捗管理

### マイルストーン

| マイルストーン | 予定日 | 状態 |
|---------------|--------|------|
| Phase 0-1完了 (要件定義・外部設計) | 2025-12-16 | ✅完了 |
| Phase 2完了 (内部設計) | TBD | ⏸未着手 |
| Phase 3完了 (実装) | TBD | ⏸未着手 |
| Phase 4-6完了 (テスト) | TBD | ⏸未着手 |
| Phase 7-8完了 (受入・デプロイ) | TBD | ⏸未着手 |

### リスク管理

| リスク | 影響度 | 対策 |
|--------|--------|------|
| PlantUML公式JARの破壊的変更 | 高 | バージョン固定 (1.2025.10)、テスト自動化 |
| WASM互換性問題 (ブラウザ更新) | 中 | 定期的なクロスブラウザテスト |
| パフォーマンス要件未達 (400ms) | 高 | 早期プロトタイプでベンチマーク実施 |
| LocalStorage容量超過 (5MB) | 低 | 仕様上15万文字/スロット対応可能 |

---

## 次のステップ

**現在**: Phase 1完了 (外部設計)

**次**: Phase 2 (内部設計) 着手
- `/speckit.tasks` コマンドでタスク分解
- design/architecture.md 作成
- クラス図・シーケンス図作成
