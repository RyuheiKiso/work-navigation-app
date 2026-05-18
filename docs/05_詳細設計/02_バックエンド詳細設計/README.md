# 02 バックエンド詳細設計

本サブは IPA 共通フレーム 2013「**2.5.1 ソフトウェアコンポーネント詳細設計**」に準拠し、Rust Cargo ワークスペース内の 8 クレート（MOD-BE-001〜007・MOD-BE-010）と共通ライブラリ（MOD-SH-001〜004）に対して、コーディングに直接利用可能な struct / trait / fn シグネチャを確定する。バックエンドは `wnav_terminal_api`（ハンディ端末向け, ポート 8080）と `wnav_master_api`（マスタメンテ・管理コンソール向け, ポート 8081）の 2 バイナリ構成とする。

---

## IPA 2.5.1 タスク対応

| IPA 2.5.1 タスク | 本サブでの対応 |
|---|---|
| コンポーネントの責務分割 | 8 クレートへの責務割付（`00_本書の位置づけと識別子規約.md`）|
| インターフェース詳細設計 | 各 trait・fn シグネチャの完全定義（`01_`〜`09_`）|
| データ構造詳細設計 | 各クレートの pub struct・pub enum の全フィールド定義 |
| アルゴリズム詳細設計 | ハッシュチェーン計算・Outbox ディスパッチ・RBAC 評価のロジック |
| エラー処理詳細設計 | ERR-NNN カタログ × RFC 9457 Problem Details（`10_`）|

---

## ファイル構成

| ファイル | 対象 MOD | 内容 |
|---|---|---|
| `00_本書の位置づけと識別子規約.md` | 全 MOD-BE | IPA 対応・FNC-BE 採番規約・カバレッジ表 |
| `01_wnav_terminal_api詳細設計.md` | MOD-BE-001 | ハンディ端末向け axum ルータ・ミドルウェアチェーン（Idempotency 含む）・AppState |
| `02_wnav_master_api詳細設計.md` | MOD-BE-010 | 管理系 axum ルータ・ミドルウェアチェーン・AppState |
| `03_wnav_domain詳細設計.md` | MOD-BE-002 | ドメインモデル・サービス Trait・リポジトリ Trait |
| `04_wnav_db詳細設計.md` | MOD-BE-004 | sqlx リポジトリ実装・コネクションプール設定 |
| `05_wnav_auth詳細設計.md` | MOD-BE-005 | JWT RS256 検証・RBAC ミドルウェア・鍵ローテーション |
| `06_wnav_hash_chain詳細設計.md` | MOD-BE-003 | SHA-256 ハッシュチェーン計算・週次検証アルゴリズム |
| `07_wnav_outbox詳細設計.md` | MOD-BE-006 | Outbox Consumer・指数バックオフ・DLQ 移行 |
| `08_wnav_webhook詳細設計.md` | MOD-BE-007 | Webhook 配信・HMAC-SHA256 署名 |
| `09_共通ライブラリ詳細設計.md` | MOD-SH-001〜004 | LocaleResolver・IdGenerator・ClockService・ApiClient |
| `10_エラーハンドリング詳細設計.md` | 全 MOD-BE | ERR-NNN 全量・AppError enum・IntoResponse 実装 |
| `99_前提制約と本書が約束しないこと.md` | — | 本サブのスコープ外事項 |

---

## 技術スタック（確定済）

| 項目 | 採用技術 |
|---|---|
| 言語 | Rust Edition 2024 |
| 非同期ランタイム | tokio 1.x（multi-thread scheduler）|
| HTTP フレームワーク | axum 0.8.x |
| DB ドライバ | sqlx 0.8.x（PostgreSQL 16）|
| JWT | `jsonwebtoken` crate |
| 暗号 | `sha2`・`hmac`・`hex`（RustCrypto）|
| i18n | `rust-i18n` crate |
| UUID | `uuid` crate（`v7` feature）|
| 時刻 | `chrono` crate |
| エラー | `thiserror` crate |
| OpenAPI 文書 | `utoipa` crate |

---

## バージョン履歴

| 版 | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-17 | RyuheiKiso | 初版 |
| 0.2.0 | 2026-05-18 | RyuheiKiso | `wnav_api` を `wnav_terminal_api`（8080）と `wnav_master_api`（8081）に 2 バイナリ分割。ファイルリナンバ（02〜09 → 03〜10）。MOD-BE-010 追加。FNC-BE-017 採番。|
