# 実装計画: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: `/specs/[###-feature-name]/spec.md` の機能仕様

**注記**: このテンプレートは `/speckit.plan` コマンドにより入力されます。実行ワークフローについては `.specify/templates/commands/plan.md` を参照してください。

## 概要

[機能仕様から抽出: 主要要件 + 調査からの技術的アプローチ]

## 技術的コンテキスト

<!--
  要アクション: このセクションの内容をプロジェクトの技術的詳細で置き換えてください。
  ここでの構造は、反復プロセスをガイドするためのアドバイザリーとして提示されています。
-->

**Language/Version**: [例: Python 3.11, Swift 5.9, Rust 1.75 または要明確化]  
**Primary Dependencies**: [例: FastAPI, UIKit, LLVM または要明確化]  
**Storage**: [該当する場合、例: PostgreSQL, CoreData, files または N/A]  
**Testing**: [例: pytest, XCTest, cargo test または要明確化]  
**Target Platform**: [例: Linux server, iOS 15+, WASM または要明確化]
**Project Type**: [single/web/mobile - ソース構造を決定]  
**Performance Goals**: [ドメイン固有、例: 1000 req/s, 10k lines/sec, 60 fps または要明確化]  
**Constraints**: [ドメイン固有、例: <200ms p95, <100MB memory, offline-capable または要明確化]  
**Scale/Scope**: [ドメイン固有、例: 10k users, 1M LOC, 50 screens または要明確化]

## Constitution Check

*ゲート: Phase 0 調査前に合格する必要があります。Phase 1 設計後に再チェックしてください。*

この機能が以下の原則に準拠していることを確認してください:

- **シンプルさ優先**: 新規依存関係は最小限か？標準ライブラリで実現できないか？
- **テスト優先**: テストを先に記述する計画があるか？
- **コード品質**: clippy、fmt、ドキュメントの基準を満たせるか？
- **パフォーマンス**: 1000行/100ms、メモリ使用量 3倍以内の基準を満たせるか？
- **UX一貫性**: エラーメッセージ、CLI出力形式は統一されているか？
- **バージョニング**: 破壊的変更がある場合、移行パスは計画されているか？

**違反がある場合**: 複雑さの追跡テーブルで正当化すること

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
<!--
  要アクション: 以下のプレースホルダーツリーをこの機能の具体的なレイアウトで置き換えてください。
  未使用のオプションを削除し、選択した構造を実際のパス(例: apps/admin, packages/something)で展開します。
  提供される計画にはオプションラベルを含めないでください。
-->

```text
# [未使用の場合は削除] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [未使用の場合は削除] Option 2: Web application ("frontend" + "backend" 検出時)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [未使用の場合は削除] Option 3: Mobile + API ("iOS/Android" 検出時)
api/
└── [上記のbackendと同じ]

ios/ or android/
└── [プラットフォーム固有の構造: 機能モジュール, UIフロー, プラットフォームテスト]
```

**構造の決定**: [選択した構造を文書化し、上記でキャプチャした実際のディレクトリを参照]

## 複雑さの追跡

> **Constitution Check に正当化が必要な違反がある場合のみ記入**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
