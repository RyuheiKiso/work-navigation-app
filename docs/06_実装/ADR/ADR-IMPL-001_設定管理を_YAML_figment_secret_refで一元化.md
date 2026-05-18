# ADR-IMPL-001: 設定管理を YAML + figment + secret_ref で一元化

日付: 2026-05-18
状態: 確定
提案者: RyuheiKiso

## 背景

- 詳細設計の記述:
  - `docs/04_概要設計/02_ソフトウェア方式設計/06_共通基盤コンポーネント設計.md` L156: `envy` crate による環境変数読込を明記（`envy::from_env()`）
  - `docs/05_詳細設計/02_バックエンド詳細設計/01_wnav_api詳細設計.md` L47-61, L527: `AppConfig` 構造体定義と `envy::from_env()` の起動コード
  - `docs/05_詳細設計/02_バックエンド詳細設計/03_wnav_db詳細設計.md` L15-53: `DbConfig` も `envy` でデシリアライズ
  - `docs/06_実装/12_環境変数とシークレット一覧.md`: 35 件以上の `WNAV_<SCOPE>_<KEY>` 環境変数台帳。格納先は local=`.env`、CI=GitHub Secrets、本番=Windows DPAPI / Docker secrets
- 実装開始前に発生した問題・制約:
  - 35 件超の環境変数が SCOPE ごとにフラットに並び、接続先トポロジ（ホスト・ポート・バイナリ分割の構成）がコードを見ないと分からない
  - `envy` は環境変数から一方向にデシリアライズするのみで、複数環境（local/dev/staging/prod）の差分管理・マージ・スキーマバリデーションを提供しない
  - 接続先の追加・変更時に環境変数台帳・各環境の `.env` / Secrets / DPAPI 値を個別更新する必要があり、diff が分散する
  - バックエンドが 2 バイナリ分割（`wnav_terminal_api` / `wnav_master_api`）であり、各バイナリが参照すべき設定サブセットをコンパイル時に型で強制する構造にしたい

## 決定

接続先・トポロジ・非機密パラメータは `src/infra/config/config.{base,profile}.yml` に記載し、`figment` crate（YAML + 環境変数オーバーレイ + プロファイル選択）で読み込む。機密（DB パスワード・JWT 秘密鍵・TLS 鍵等）は YAML に書かず `secret_ref: "<scheme>:<id>"` で間接参照する。設定読込専用クレート `wnav_config` を新設し、両バイナリが経由して設定を取得する。

## 理由

### 採用した選択肢の根拠

- **接続先の可視性**: `config.base.yml` に全環境共通の非機密設定が集約されるため、トポロジ変更が 1 箇所の diff に現れる
- **環境差分の宣言的管理**: `config.{profile}.yml` に差分のみ記述し、`WNAV_PROFILE` でプロファイルを選択する。環境ごとに `.env` ファイルを全件コピーする必要がない
- **バイナリ別型分離**: `TerminalApiConfig` 構造体に `database.write` フィールドが存在しないため、terminal_api バイナリが書き込みプールを初期化できないことをコンパイル時に強制できる（`src/backend/CLAUDE.md` L66-72 の DB ロール物理保証原則）
- **機密の完全排除**: `secret_ref:` 参照のみを YAML に書くため、`config.staging.yml` / `config.prod.yml` を Git にコミットしても機密は含まれない。Git 履歴に機密が残らない構造を維持できる
- **起動時 fail-fast**: figment の段階的 extract 後に secret_ref 全件解決・構造体 validate() を実行し、設定ミスを起動時に検出する。運用中に設定不整合が発生しない
- **スキーマバリデーション**: `schema/config.schema.json` による JSON Schema 検証で YAML の構造的誤りを CI で検出できる

### 却下した代替案

| 代替案 | 却下理由 |
|---|---|
| `envy` crate を継続（環境変数のみ） | 35 件超の変数がフラットに並び可視性が低い。複数環境の差分管理・スキーマバリデーションの仕組みがなく、接続先台帳が`.env.example` と コード の両方に分散する |
| `config-rs` crate | YAML / TOML / 環境変数の階層マージに対応しているが、エラーメッセージが不親切で Rust エコシステムでの採用実績・メンテナンス活性が `figment` に劣る |
| `serde_yaml` 直接 + 自前マージ | マージロジック・プロファイル選択・環境変数オーバーレイを全て自作する必要があり複雑性が上がる。`figment` が提供する機能を再発明することになる |
| TOML 一本化 | `docs/04_概要設計/01_システム方式設計/05_アドレッシング・名前解決・時刻同期.md` L127 のみ `config.toml` 記載があるが、他の全ドキュメントは環境変数前提で書かれており、TOML への一本化は整合性改善にならない。YAML は TOML より複数行文字列の扱いが容易でコメントを書きやすい |

## 影響

- **影響範囲**:
  - `src/backend/Cargo.toml`（workspace 新規）
  - `src/backend/crates/wnav_config/`（新規クレート）
  - `src/backend/crates/wnav_terminal_api/`（将来実装時に `wnav_config::load_terminal_api()` を呼ぶ）
  - `src/backend/crates/wnav_master_api/`（将来実装時に `wnav_config::load_master_api()` を呼ぶ）
  - `src/infra/config/`（YAML 配置ディレクトリ新規）
  - `.gitignore`（新規作成・`config.local.yml` 等を除外）
- **追加のテスト**:
  - `wnav_config` クレートのユニットテスト全件（プロファイル選択・マージ規則・secret_ref 解決・fail-fast・型分離・Debug マスキング）
- **ドキュメント更新**:
  - `docs/06_実装/12_環境変数とシークレット一覧.md` — タイトル変更・YAML 階層マッピング追加・`secret_ref:` 文法章追加
  - `docs/04_概要設計/02_ソフトウェア方式設計/06_共通基盤コンポーネント設計.md` L156 — `envy::from_env()` → `figment` 記述
  - `docs/04_概要設計/01_システム方式設計/05_アドレッシング・名前解決・時刻同期.md` L127 — `config.toml` → `config.{profile}.yml`
  - `docs/05_詳細設計/02_バックエンド詳細設計/01_wnav_api詳細設計.md` L47-61, L527 — `AppConfig` 分割・`envy` → `wnav_config`
  - `docs/05_詳細設計/02_バックエンド詳細設計/03_wnav_db詳細設計.md` L15-53 — `DbConfig` の出処変更
  - `docs/05_詳細設計/02_バックエンド詳細設計/10_wnav_config詳細設計.md` — 新規クレート設計書
  - `docs/06_実装/05_コーディング規約_シェルInfra.md` L527-660 — YAML+env ハイブリッド方針への更新
  - `docs/06_実装/10_デプロイ手順.md` L38-48 — `WNAV_PROFILE` + YAML 配置手順追加
  - `docs/06_実装/06_開発環境構築手順.md` — セットアップ手順追加
  - `docs/06_実装/11_CICDパイプライン設定.md` — CI 設定追加
  - `src/backend/CLAUDE.md` — `wnav_config` 経由必須を明記

## 参照

- 関連 ADR: ADR-IMPL-002（マスタ SPA 設定取得方式）、ADR-IMPL-003（ハンディ APP 接続情報管理）
- 関連 FR-NNN: —（非機能要件・運用性に関する判断）
- 上流文書: `docs/04_概要設計/02_ソフトウェア方式設計/06_共通基盤コンポーネント設計.md` §7 CFG-001〜CFG-014（本 ADR により更新対象）
