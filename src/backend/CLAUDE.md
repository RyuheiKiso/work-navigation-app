# src/backend — Rust バックエンド実装規約

Rust Edition 2024 + tokio + axum + sqlx で構築するバックエンド API サーバーの実装規約。
横断共通原則は `src/CLAUDE.md` が権威。本ファイルはバックエンド固有の規約を記す。

権威ドキュメント: `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` 第 4 節

---

## 技術スタック

| 要素 | 採用 |
|---|---|
| 言語 | Rust Edition 2024 |
| 非同期ランタイム | tokio |
| Web フレームワーク | axum |
| DB ドライバ | sqlx（コンパイル時クエリ検証） |
| ミドルウェア合成 | Tower |
| OpenAPI 生成 | utoipa |
| エラー型定義 | thiserror |
| ロギング | tracing + tracing-subscriber |
| 負荷ベンチマーク | criterion |

---

## `unsafe` 禁止

クレートルートに以下を必ず明記する:

```rust
#![forbid(unsafe_code)]
```

例外は認めない。ネイティブライブラリとの FFI が必要な場合は ADR に根拠を記録し、`unsafe` を隔離した別クレートとして分離する。

---

## バイナリ 2 分割

バックエンドは `wnav_terminal_api` と `wnav_master_api` の **2 バイナリ**に分割する。

**分割の理由**:
1. **DB ロール物理保証**: `app_event_insert` 接続プールは `wnav_terminal_api` プロセスのみが保有し、`wnav_master_api` プロセスは一切保有しない。`app_write` 接続プールは `wnav_master_api` プロセスのみが保有する。単一バイナリでは OS プロセス境界による分離が不可能なため、2 バイナリが必須。
2. **独立可用性**: 現場作業記録（ハンディ端末 → `wnav_terminal_api`）と管理操作（管理 PC → `wnav_master_api`）が独立してクラッシュ・再起動できる。管理側の障害が現場作業を止めない。
3. **セキュリティ境界**: ファイアウォールで端末（工場 LAN → `:8080`）と管理 PC（管理 LAN → `:8081`）をネットワーク分離できる。

### crate 構成

| バイナリ | ファイルパス | ポート | 役割 |
|---|---|---|---|
| `wnav_terminal_api` | `crates/wnav_terminal_api/` | 8080 | ハンディ端末向け axum ルータ・Idempotency 検証・レート制限・作業ログ受信・Outbox Consumer |
| `wnav_master_api` | `crates/wnav_master_api/` | 8081 | マスタメンテ・管理コンソール向け axum ルータ・SOP 編集・承認・監査・ユーザー管理・HashChainVerifier |

---

## 設定読み込み（wnav_config クレート）

権威ドキュメント: `docs/05_詳細設計/02_バックエンド詳細設計/10_wnav_config詳細設計.md`、ADR-IMPL-001

すべてのバイナリは **`wnav_config` クレート経由で設定を読み込む**。環境変数から直接読む `envy::from_env()` やハードコードは禁止。

```rust
// wnav_terminal_api/src/main.rs
let config: TerminalApiConfig = wnav_config::load_terminal_api()?;

// wnav_master_api/src/main.rs
let config: MasterApiConfig = wnav_config::load_master_api()?;
```

### 設定の取得元

| 設定種別 | 格納場所 | 例 |
|---|---|---|
| 非機密（接続先・ポート等） | `src/infra/config/config.{profile}.yml` | `database.host`, `server.terminal_api.port` |
| 機密（パスワード・鍵等） | `secret_ref:` で参照・実体は env / DPAPI / Docker secrets | `secret_ref: "env:WNAV_DB_PASSWORD_WRITE"` |
| プロファイル選択 | `WNAV_PROFILE` 環境変数 | `WNAV_PROFILE=prod` |

### バイナリ別型分離

`TerminalApiConfig` と `MasterApiConfig` は別々の構造体であり、誤ったフィールドへのアクセスはコンパイルエラーになる。DB ロール物理保証をコンパイル時に強制する。

| 構造体 | 含むフィールド | 含まないフィールド |
|---|---|---|
| `TerminalApiConfig` | `database.event_insert` / `database.read` | **`database.write`** / **`jwt.private_key`** |
| `MasterApiConfig` | `database.write` / `database.read` / `jwt` | `database.event_insert` / `idempotency` / `outbox` |

### 起動時 fail-fast

`WNAV_PROFILE` 未設定・YAML 欠損・`secret_ref` 解決失敗時はバイナリが exit code 78 で即座に終了する。起動後に設定読み込みエラーが発生しない設計を保証する。

---

## Append-only の物理保証

権威ドキュメント: `docs/02_企画/システム化計画/05_アーキテクチャ原則.md` 第 2 節

### DB ロール 3 分離とバイナリへの割り当て

| ロール | 権限 | 接続プールの用途 | 保有バイナリ |
|---|---|---|---|
| `app_write` | SELECT / INSERT / UPDATE（マスタテーブルのみ） | マスタ CRUD | `wnav_master_api` のみ |
| `app_event_insert` | INSERT のみ（作業ログテーブル） | 作業ログ記録 | `wnav_terminal_api` のみ |
| `app_read` | SELECT のみ（全テーブル） | Audit Trail 照会・ダッシュボード | 両バイナリ |

- `wnav_terminal_api` は `app_event_insert` + `app_read` の 2 プールのみを起動時に初期化する。`app_write` プールを保有してはならない。
- `wnav_master_api` は `app_write` + `app_read` の 2 プールのみを起動時に初期化する。`app_event_insert` プールを保有してはならない。
- `app_event_insert` プールから SELECT を発行してはならない。
- 同一クレデンシャルを複数用途で共有しない。

**例外制御テーブル**: `case_locks`（TBL-051）と `idempotency_keys`（TBL-035）は Append-only 原則の例外として `app_event_insert` ロールに INSERT/UPDATE/DELETE を許可する。heartbeat 更新と解放 DELETE が必要なため。詳細: `docs/05_詳細設計/07_アルゴリズム詳細設計/08_Case端末占有アルゴリズム.md`

### sqlx コンパイル時クエリ検証

```rust
// 正しい: コンパイル時に SQL 構文・型・カラム名を検証する
let events = sqlx::query!(
    "SELECT id, activity FROM work_events WHERE case_id = $1",
    case_id
)
.fetch_all(&event_insert_pool)
.await?;

// 禁止: 実行時まで SQL エラーを検知できない
let events = sqlx::query("SELECT id, activity FROM work_events WHERE case_id = $1")
    .bind(case_id)
    .fetch_all(&pool)
    .await?;
```

`sqlx::query!` マクロを必ず使用する。`sqlx::query()` は禁止。

---

## Idempotent API

権威ドキュメント: `docs/02_企画/システム化計画/05_アーキテクチャ原則.md` 第 3 節

Tower ミドルウェアで `Idempotency-Key`（UUID v4）を全作業ログ受信エンドポイントで検証する。

```
リクエスト受信
  → Idempotency-Key 検証ミドルウェア
  → キャッシュヒット: 保存済みレスポンスを返却（DB 操作なし）
  → キャッシュミス: ハンドラ実行 → レスポンスをキャッシュ保存（TTL 24h）
```

同一 Key の再送は冪等であり、重複イベントを DB に書き込まない。

> **適用スコープ**: `Idempotency-Key` 検証は **`wnav_terminal_api` のみ**に適用する。`wnav_master_api` の SOP 編集・マスタ更新系エンドポイントには適用しない（マスタ操作はべき等設計の必要性が異なるため）。

---

## 権威タイムスタンプ

権威ドキュメント: `docs/02_企画/システム化計画/05_アーキテクチャ原則.md` 第 1 節

```rust
// ハンドラ内でサーバー受信時刻を付与する
let server_received_at = Utc::now().timestamp_millis();
```

- `server_received_at`: サーバーがリクエストを受信した時刻（UTC ms）。必ずサーバー側で付与し、クライアントによる上書きを禁止する
- `client_recorded_at`: クライアントが記録を入力した時刻（申告値）。保持するがタイムスタンプ権威ではない

---

## SHA-256 ハッシュチェーン

権威ドキュメント: `docs/02_企画/システム化計画/05_アーキテクチャ原則.md`

```
event_N.hash = SHA256(event_{N-1}.hash || event_N.canonical_payload)
```

- ハッシュチェーン構成ロジックは `src/domain/hash_chain/` に集約する
- バックグラウンドジョブでチェーンの連続性を定期検証する
- 破断（ハッシュ不一致）を検知した場合、直ちにアラートを発する

---

## OpenAPI 3.1 自動生成

`utoipa` でハンドラ定義から OpenAPI 3.1 スキーマを自動生成する。
生成された `openapi.json` は `src/frontend/master/` の API クライアント生成元として使用する。

- `GET /api/openapi.json` で常に最新スキーマを配信する
- `openapi.json` を手動で編集しない（常に `utoipa` からの自動生成を権威とする）

---

## Webhook 配信

権威ドキュメント: `docs/02_企画/システム化計画/12_外部システム連携アーキテクチャ（子機モード）.md`

- **HMAC-SHA256 署名**: `X-Signature-256: sha256=<hex>` ヘッダを必ず付与する
- **Idempotency-Key の伝播**: ペイロードに元イベントの Idempotency-Key を含め、受信側でリプレイ防止を可能にする
- **再送ポリシー**: 指数バックオフ（最大 5 回・上限 24h）。最終失敗後は dead-letter キューに保存してアラートを発する

---

## 認証・認可

権威ドキュメント: `docs/02_企画/システム化計画/15_セキュリティ深堀り.md`

- **JWT RS256**: TTL 8h。秘密鍵は環境変数またはシークレット管理から取得し、コードにハードコードしない
- **公開鍵ローテーション**: 手順を `docs/09_運用・保守/` に文書化する
- **RBAC 6 ロール**: エンドポイントごとに認可を `axum::extract` の型で強制する

```rust
// 認可の型強制例: AuditorRole 以上でないとコンパイルエラーになる型設計
async fn get_audit_trail(
    AuthenticatedUser { role, .. }: AuthenticatedUser<AuditorRole>,
) -> impl IntoResponse {
    // ...
}
```

---

## エラーハンドリング

- エラー型は `thiserror` で定義する
- HTTP レスポンスへの変換は `axum::response::IntoResponse` を各エラー型に実装する
- クライアント向けエラーボディは **RFC 7807 Problem Details** 準拠:

```json
{
  "type": "https://errors.example.com/insufficient-permission",
  "title": "Insufficient Permission",
  "status": 403,
  "detail": "RBAC role 'operator' cannot access audit trail.",
  "instance": "/api/v1/audit/events/12345"
}
```

- スタックトレース・内部エラー詳細はクライアントに返さない（ログにのみ記録する）
- `unwrap()` / `expect()` は本番コードで禁止する（テストコードでは許容）

---

## レート制御

権威ドキュメント: `docs/02_企画/システム化計画/12_外部システム連携アーキテクチャ（子機モード）.md`

- Tower ミドルウェアとしてトークンバケット方式（`tower-governor` 等）を実装する
- レートはクライアント ID（API キーまたは JWT の `sub`）とエンドポイント単位で設定する
- 超過時は `429 Too Many Requests` と `Retry-After` ヘッダを返す

---

## データベース

### URL バージョニング

すべてのエンドポイントは `/api/v1/` プレフィックスで始める。
破壊的変更時は `/api/v2/` に移行し、旧バージョンを一定期間維持する（廃止スケジュールを Changelog に記録）。

### マイグレーション

```bash
sqlx migrate run          # 適用
sqlx migrate add <name>   # 新規作成（up.sql + down.sql のセット）
```

- `migrations/` ディレクトリをバージョン管理に含める
- ロールバックスクリプト（`down.sql`）を必ずセットで作成する

### PostgreSQL 拡張

最小限に限定する:
- `pgcrypto` — ハッシュ計算・ランダム UUID 生成

それ以外の拡張追加は ADR に根拠を記録してから実施する。

---

## テスト

| テスト種別 | ツール | 必須シナリオ |
|---|---|---|
| ユニット | `cargo test` | ハッシュチェーン・Idempotency キャッシュ・RBAC ロジック |
| 統合 | testcontainers-rs（実 PostgreSQL） | Append-only 制約・マイグレーション適用・ロール権限検証 |
| 負荷 | criterion | イベント受信スループット・Idempotency キャッシュヒット率 |

**統合テストでのモック禁止**: testcontainers-rs で PostgreSQL コンテナを起動し、実際の DB に対してテストを実行する。
DB ロールの権限制御（`app_event_insert` ロールが UPDATE できないこと）を統合テストで明示的に確認する。

---

## 可観測性

- **ロギング**: `tracing` + `tracing-subscriber`。JSON 形式構造化ログ
- **相関 ID**: 各リクエストに UUID を採番し、`X-Request-Id` ヘッダで返却する。全ログ・エラーに相関 ID を付与する
- **メトリクス**: エンドポイントごとのレイテンシ・スループット・エラー率（Prometheus 互換形式を第一候補）
- **ヘルスチェック**: `GET /healthz`（DB 疎通確認含む）を必ず実装する

---

## ディレクトリ構成案（暫定）

```
src/backend/
  Cargo.toml          # workspace 定義
  Cargo.lock          # 必ずバージョン管理に含める
  crates/
    wnav_terminal_api/   # MOD-BE-001: ハンディ端末向け axum ルータ（ポート 8080）
      src/
        main.rs          # エントリポイント（app_event_insert + app_read プール初期化）
        api/             # axum ハンドラ・ルーティング定義（作業ログ受信・証拠・同期）
        middleware/      # Idempotency-Key 検証・レート制限・相関 ID（terminal 専用）
        error.rs
    wnav_master_api/     # MOD-BE-010: マスタメンテ・管理コンソール向け axum ルータ（ポート 8081）
      src/
        main.rs          # エントリポイント（app_write + app_read プール初期化）
        api/             # axum ハンドラ・ルーティング定義（SOP 編集・承認・監査・ユーザー管理）
        middleware/      # 相関 ID・認可ミドルウェア（master 専用）
        error.rs
    wnav_domain/         # MOD-BE-002: ドメインモデル・サービス・リポジトリ trait
    wnav_hash_chain/     # MOD-BE-003: SHA-256 ハッシュチェーン計算・検証
    wnav_db/             # MOD-BE-004: sqlx クエリ・コネクションプール
    wnav_auth/           # MOD-BE-005: JWT RS256・RBAC ミドルウェア
    wnav_outbox/         # MOD-BE-006: Outbox Consumer（常駐 tokio task）
    wnav_webhook/        # MOD-BE-007: Webhook 配信・HMAC 署名
  migrations/            # sqlx マイグレーション（up.sql + down.sql）
  tests/                 # 統合テスト（testcontainers-rs）
  benches/               # criterion ベンチマーク
```

実装開始時に整合性を確認し、変更する場合は ADR に記録する。

---

## 参照ドキュメント

- `docs/02_企画/システム化計画/05_アーキテクチャ原則.md` — Offline-First / Append-only / Idempotent / ハッシュチェーン
- `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` — Rust 選定根拠・却下された代替案
- `docs/02_企画/システム化計画/08_品質特性と非機能要件方針.md` — RTO/RPO 目標・セキュリティ方針
- `docs/02_企画/システム化計画/12_外部システム連携アーキテクチャ（子機モード）.md` — REST API / Webhook / レート制御
- `docs/02_企画/システム化計画/15_セキュリティ深堀り.md` — JWT RS256 / RBAC 6 ロール / 電子署名
- `src/CLAUDE.md` — 横断共通原則

---

最終更新: 2026-05-18
次回見直しトリガー: Rust 実装開始時、または Cargo.toml にクレート追加を決定した時
