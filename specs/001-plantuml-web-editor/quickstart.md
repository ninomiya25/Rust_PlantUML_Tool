# クイックスタートガイド

**プロジェクト**: 社内向けセキュアなPlantUMLウェブエディタ  
**対象読者**: 開発者、テスター、デプロイ担当者  
**最終更新**: 2025-12-16

## 目次

1. [前提条件](#前提条件)
2. [環境構築](#環境構築)
3. [ビルド](#ビルド)
4. [実行](#実行)
5. [テスト](#テスト)
6. [デプロイ](#デプロイ)
7. [トラブルシューティング](#トラブルシューティング)

---

## 前提条件

### 必須ツール

| ツール | バージョン | インストール方法 |
|--------|-----------|-----------------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs/) |
| Trunk | 0.18+ | `cargo install trunk` |
| wasm-bindgen-cli | 0.2.89+ | `cargo install wasm-bindgen-cli` |
| Java | 11+ | [OpenJDK](https://openjdk.org/) |
| PlantUML | 1.2025.10+ | [plantuml.jar](https://plantuml.com/ja/download) |

### オプションツール (本番デプロイ用)

- Docker: 20.10+
- Docker Compose: 2.0+
- Nginx: 1.24+ (静的ファイル配信用)

### 動作確認環境

- OS: Windows 10/11, macOS 12+, Ubuntu 20.04+
- ブラウザ: Chrome 90+, Edge 90+, Firefox 89+

---

## 環境構築

### 1. リポジトリクローン

```powershell
git clone https://github.com/your-company/rust_PlantUMLtool.git
cd rust_PlantUMLtool
git checkout 001-plantuml-web-editor
```

### 2. Rust環境セットアップ

```powershell
# Rustツールチェーンインストール
rustup install stable
rustup default stable

# WASMターゲット追加
rustup target add wasm32-unknown-unknown

# 必須ツールインストール
cargo install trunk
cargo install wasm-bindgen-cli
```

### 3. PlantUMLサーバー準備

```powershell
# PlantUML JARダウンロード
$version = "1.2025.10"
Invoke-WebRequest `
  -Uri "https://github.com/plantuml/plantuml/releases/download/v$version/plantuml-$version.jar" `
  -OutFile "plantuml.jar"

# 動作確認
java -jar plantuml.jar -version
```

### 4. 依存関係インストール

```powershell
# Workspace全体のビルド (依存関係ダウンロード)
cargo build
```

---

## ビルド

### 開発ビルド

```powershell
# バックエンドAPI (デバッグモード)
cd api-server
cargo build

# フロントエンドUI (デバッグモード)
cd ../web-ui
trunk build
```

### リリースビルド

```powershell
# バックエンドAPI (最適化)
cd api-server
cargo build --release

# フロントエンドUI (最適化 + WASM圧縮)
cd ../web-ui
trunk build --release

# WASMバイナリ最適化 (オプション)
wasm-opt -Oz -o dist/app_bg.wasm dist/app_bg.wasm
```

**ビルド成果物**:
- バックエンド: `api-server/target/release/api-server` (実行可能バイナリ)
- フロントエンド: `web-ui/dist/` (index.html + app.wasm + app.js)

---

## 実行

### ローカル開発環境 (3プロセス起動)

#### ターミナル1: PlantUMLサーバー起動

```powershell
# localhost:8081 で起動 (社内ネットワーク限定)
java -jar plantuml.jar -picoweb:8081:127.0.0.1 -DSECURITY_PROFILE=INTERNET
```

**出力例**:
```
PlantUML Picoweb Server is running on http://127.0.0.1:8081
```

#### ターミナル2: バックエンドAPI起動

```powershell
cd api-server

# 環境変数設定 (オプション)
$env:RUST_LOG = "info"
$env:PLANTUML_URL = "http://localhost:8081"

# サーバー起動
cargo run
```

**出力例**:
```
2025-12-15T10:30:00Z INFO api_server: API server listening on http://127.0.0.1:8080
```

#### ターミナル3: フロントエンドUI起動

```powershell
cd web-ui

# Trunk開発サーバー起動 (Hot Reload有効、ポート8000)
trunk serve --port 8000
```

**出力例**:
```
2025-12-15T10:30:05 INFO 📦 building app...
2025-12-15T10:30:10 INFO 📡 serving http://127.0.0.1:8000
```

ブラウザで `http://127.0.0.1:8000` にアクセスしてエディタを使用します。

**注意**: ポート8080はAPI Serverが使用しているため、web-uiは8000を使用します。

### 動作確認

1. **PlantUMLサーバー接続確認**:
   ```powershell
   curl http://localhost:8081/plantuml/png/SyfFKj2rKt3CoKnELR1Io4ZDoSa70000
   ```
   PNG画像が返却されればOK。

2. **バックエンドAPI確認**:
   ```powershell
   curl -X POST http://localhost:8080/api/v1/convert `
     -H "Content-Type: application/json" `
     -d '{"plantuml_text":"@startuml\nAlice->Bob:Hello\n@enduml","format":"png"}' `
     --output test.png
   ```
   `test.png` が生成されればOK。

3. **フロントエンド確認**:
   - ブラウザで `http://127.0.0.1:8000` を開く
   - エディタにPlantUMLテキストを入力
   - 右側にリアルタイムでプレビューが表示されることを確認

---

## テスト

### ユニットテスト

```powershell
# Workspace全体のユニットテスト
cargo test

# 特定クレートのみ
cargo test -p core
cargo test -p api-server
```

### 契約テスト

```powershell
cd tests/contract
cargo test -- --test-threads=1
```

### 統合テスト

```powershell
# PlantUMLサーバーとバックエンドAPIが起動している必要あり
cd tests/integration
cargo test
```

### E2Eテスト (WASM + API連携)

```powershell
# wasm-pack必要
cargo install wasm-pack

cd web-ui
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome
```

### パフォーマンステスト

```powershell
cd tests/performance

# 100行PlantUMLファイルで90パーセンタイル400ms以内を確認
cargo run --release -- --benchmark convert_100_lines --percentile 90
```

### カバレッジ測定

```powershell
# tarpaulinインストール
cargo install cargo-tarpaulin

# カバレッジ計測 (目標: 80%以上)
cargo tarpaulin --out Html --output-dir coverage
```

---

## デプロイ

### Docker Compose デプロイ (推奨)

#### 1. Dockerイメージビルド

```powershell
# フロントエンドビルド (静的ファイル生成)
cd web-ui
trunk build --release

# Dockerイメージビルド
cd ..
docker-compose build
```

#### 2. コンテナ起動

```powershell
docker-compose up -d
```

**構成**:
- `plantuml`: PlantUML Picoweb (localhost:8081)
- `api-server`: Axumバックエンド (localhost:8080)
- `web-ui`: Nginx (localhost:80)

#### 3. 動作確認

```powershell
# ヘルスチェック
curl http://localhost:8080/api/v1/health

# フロントエンドアクセス
Start-Process http://localhost
```

#### 4. ログ確認

```powershell
# 全コンテナのログ
docker-compose logs -f

# 特定コンテナのみ
docker-compose logs -f api-server
```

#### 5. 停止

```powershell
docker-compose down
```

### 手動デプロイ (本番環境)

#### 1. バイナリ配置

```powershell
# リリースビルド
cargo build --release

# バイナリ配置
Copy-Item api-server/target/release/api-server `
  -Destination /opt/plantuml-editor/bin/

# 静的ファイル配置
Copy-Item web-ui/dist/* `
  -Destination /var/www/plantuml-editor/ -Recurse
```

#### 2. Systemdサービス登録 (Linux)

**/etc/systemd/system/plantuml-api.service**:
```ini
[Unit]
Description=PlantUML Web Editor API Server
After=network.target

[Service]
Type=simple
User=plantuml
WorkingDirectory=/opt/plantuml-editor
ExecStart=/opt/plantuml-editor/bin/api-server
Environment="RUST_LOG=info"
Environment="PLANTUML_URL=http://localhost:8081"
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable plantuml-api
sudo systemctl start plantuml-api
```

#### 3. Nginx設定

**/etc/nginx/sites-available/plantuml-editor**:
```nginx
server {
    listen 80;
    server_name plantuml.internal.company.com;

    # 社内ネットワーク限定 (例)
    allow 192.168.0.0/16;
    deny all;

    # フロントエンド静的ファイル
    location / {
        root /var/www/plantuml-editor;
        index index.html;
        try_files $uri $uri/ /index.html;
    }

    # バックエンドAPI (リバースプロキシ)
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 30s;
    }
}
```

```bash
sudo ln -s /etc/nginx/sites-available/plantuml-editor /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## トラブルシューティング

### 問題1: PlantUMLサーバーが起動しない

**症状**: `java -jar plantuml.jar -picoweb` でエラー

**原因**: Javaバージョン不一致、ポート競合

**解決策**:
```powershell
# Javaバージョン確認 (11以上必要)
java -version

# ポート使用確認
netstat -ano | Select-String "8081"

# 別ポートで起動
java -jar plantuml.jar -picoweb:9081:127.0.0.1
```

### 問題2: バックエンドAPIが503エラー

**症状**: `curl http://localhost:8080/api/v1/convert` で503

**原因**: PlantUMLサーバーに接続できない

**解決策**:
```powershell
# PlantUMLサーバーの起動確認
curl http://localhost:8081/plantuml/png/SyfFKj2rKt3CoKnELR1Io4ZDoSa70000

# 環境変数確認
echo $env:PLANTUML_URL

# ログ確認
cd api-server
$env:RUST_LOG = "debug"
cargo run
```

### 問題3: フロントエンドがバックエンドに接続できない

**症状**: ブラウザコンソールで `CORS error` または `net::ERR_CONNECTION_REFUSED`

**原因**: バックエンドAPI未起動、CORS設定不足

**解決策**:
```powershell
# バックエンド起動確認
curl http://localhost:8080/api/v1/health

# ブラウザ開発者ツールでネットワークタブ確認
# → リクエストURLが正しいか確認 (http://localhost:8080/api/v1/convert)
```

### 問題4: WASM ビルドエラー

**症状**: `trunk build` で `wasm-bindgen` エラー

**原因**: wasm-bindgen-cliとCargo.tomlのバージョン不一致

**解決策**:
```powershell
# バージョン確認
wasm-bindgen --version
grep wasm-bindgen web-ui/Cargo.toml

# wasm-bindgen-cli再インストール
cargo install wasm-bindgen-cli --force
```

### 問題5: パフォーマンステストが失敗 (90パーセンタイルで400ms超過)

**症状**: `cargo run --release -- --benchmark` で90パーセンタイルが400ms以上

**原因**: PlantUMLサーバーが常駐していない、デバッグビルド使用

**解決策**:
```powershell
# PlantUML Picowebで常駐プロセス化
java -jar plantuml.jar -picoweb:8081:127.0.0.1

# リリースビルド使用
cargo build --release
cargo run --release -- --benchmark --percentile 90
```

---

## その他のコマンド

### コードフォーマット

```powershell
cargo fmt --all
```

### Lint (Clippy)

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

### ドキュメント生成

```powershell
cargo doc --no-deps --open
```

### 依存関係更新

```powershell
cargo update
cargo audit  # セキュリティ監査
```

---

## リソース

- **仕様書**: [spec_revised.md](./spec_revised.md)
- **技術調査**: [research.md](./research.md)
- **データモデル**: [data-model.md](./data-model.md)
- **API契約**: [contracts/api.yaml](./contracts/api.yaml)
- **Constitution**: [../../.specify/memory/constitution.md](../../.specify/memory/constitution.md)

## サポート

問題が解決しない場合は、開発チームにお問い合わせください:
- Slack: #plantuml-editor-support
- Email: dev-team@company.com
