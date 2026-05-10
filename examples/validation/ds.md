# 設計仕様（DS）テンプレート

> 対応規格: GMP Annex 11 §4.4

FS を **実装に直接マッピングできる粒度** に展開する。

## 1. 設計一覧（DS-XXX-NN）

| ID | 設計 | 対応 FS | 実装箇所 |
| --- | --- | --- | --- |
| DS-AUTH-01 | Argon2id + HMAC-SHA256 セッション | FS-AUTH-01 | `services/backend/crates/adapter/src/{argon2_hasher,hs256_session_factory}.rs` |
| DS-NAV-01 | HSM + ナビ 6 基本機能 UI | FS-NAV-01 | `apps/terminal/src/presentation/components/navigation-shell.tsx` ／ `services/backend/crates/domain/src/task.rs` |
| DS-COMP-01 | `CompletionCriteria` Manual/Photo | FS-COMP-01 | `services/backend/crates/domain/src/value_object.rs` |
| DS-AUDIT-01 | DB トリガ＋追記不変ストア | FS-AUDIT-01 | `services/backend/migrations/0001_init.sql` |
| DS-PERF-01 | Rust／tokio／axum + Vite ビルド | FS-PERF-01 | §11.7.2 |
| DS-OFFLINE-01 | SQLCipher＋OS Keystore＋ローカルキャッシュ | FS-OFFLINE-01 | `apps/terminal/src-tauri/src/secure_storage.rs` |
| DS-A11Y-01 | WCAG 2.2 AA／axe-core 回帰／タッチターゲット 9mm | FS-A11Y-01 | §11.2 §9.5 |

## 2. アーキテクチャ図（drawio）

`docs/01_企画/依存関係マップ.drawio` を参照。

## 3. 承認

| 役割 | 氏名 | 署名 | 日付 |
| --- | --- | --- | --- |
| 設計責任者 | | | |
| QA | | | |
