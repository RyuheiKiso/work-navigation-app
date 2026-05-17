# 02 コーディング規約_Rust

## 1. Rust Edition 2024 採用根拠と unsafe 禁止

本プロジェクトのバックエンドは **Rust Edition 2024** を採用する。Edition 2024 を選択した根拠は以下のとおり。

- **`use<>` キャプチャ構文**: より明示的なライフタイムキャプチャが可能となり、非同期コード（tokio）における曖昧なライフタイム境界を排除できる。
- **生の文字列リテラル変更**: SQL クエリ文字列の可読性が向上する。
- **`gen` キーワード予約**: 将来のジェネレータ構文への円滑な移行を保証する。

### `#![forbid(unsafe_code)]` の全クレート適用

**すべてのクレートルート**（`src/main.rs` または `src/lib.rs`）に以下を必ず明記する。

```rust
#![forbid(unsafe_code)]
```

これは宣言ではなく強制である。このアトリビュートが存在しないファイルは PR でマージを拒否する。FFI や ネイティブライブラリとの連携が真に必要な場合は `§13 unsafe 例外申請手順` に従い、専用の隔離クレートを作成する。

**本節で確定した方針**
- **Rust Edition 2024 を全クレートで採用し、Edition を `Cargo.toml` に明示する。**
- **`#![forbid(unsafe_code)]` を全クレートルートに必須とし、CI で存在チェックを行う。**
- **`unsafe` の使用は隔離クレートに限定し、ADR-IMPL-NNN の記録を必須とする。**

---

## 2. ツールチェイン

### rustup と rust-toolchain.toml

プロジェクトルートに `rust-toolchain.toml` を配置し、toolchain バージョンを固定する。

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy", "rust-src"]
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc"
]
```

### MSRV（Minimum Supported Rust Version）方針

`Cargo.toml` に MSRV を明記し、CI でその最小バージョンでのビルドを確認する。

```toml
[package]
name = "wnav-api"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"  # Rust Edition 2024 の最初の安定版
```

### cargo install 必須ツール

開発環境のセットアップ時に必須のツールを列挙する（詳細は `06_開発環境構築手順.md §3` を参照）。

```bash
cargo install cargo-watch       # ファイル変更監視・自動ビルド
cargo install cargo-nextest     # 並列テストランナー
cargo install sqlx-cli          # SQLx マイグレーション管理
cargo install cargo-audit       # 依存クレートの脆弱性チェック
cargo install cargo-sbom        # SBOM 生成
```

**本節で確定した方針**
- **`rust-toolchain.toml` でツールチェインを固定し、開発者間のバージョン差異を排除する。**
- **`Cargo.toml` に `rust-version` を明記し、MSRV 未満でのビルドを CI で検出する。**
- **`cargo audit` を CI に組み込み、脆弱な依存クレートを自動検出する。**

---

## 3. rustfmt 設定

プロジェクトルートの `rustfmt.toml` で統一フォーマットを強制する。

```toml
# rustfmt.toml
edition = "2024"
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true
use_small_heuristics = "Default"
comment_width = 100
wrap_comments = true
format_code_in_doc_comments = true
normalize_comments = true
```

### 主要設定の解説

| 設定名 | 値 | 理由 |
|---|---|---|
| `edition` | `"2024"` | Edition 2024 の構文を正しくフォーマットするため |
| `max_width` | `100` | 縦長の一般的なモニタで横スクロールなしに読める幅 |
| `imports_granularity` | `"Crate"` | クレート単位でインポートをグループ化し、可読性を向上する |
| `group_imports` | `"StdExternalCrate"` | 標準ライブラリ・外部クレート・ローカルの 3 グループに自動分離 |

CI では `cargo fmt -- --check` を実行し、フォーマット差分があればビルドを失敗させる。

**本節で確定した方針**
- **`rustfmt.toml` を `edition = "2024"`・`max_width = 100`・`imports_granularity = "Crate"` で統一する。**
- **CI で `cargo fmt -- --check` を実行し、未フォーマットコードのマージを禁止する。**
- **`format_code_in_doc_comments = true` を設定し、ドキュメントコメント内のコード例も整形する。**

---

## 4. clippy ルール

### deny レベルの設定

クレートルートに以下を設定し、clippy の警告をエラーとして扱う。

```rust
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
```

CI では `cargo clippy -- -D warnings` を実行し、警告を 1 件でもエラーとして処理する。

### 許容する warn（個別 allow の記録）

以下の lint は ADR-IMPL-NNN に理由を記録したうえで `#[allow]` を付与することを許容する。

```rust
// clippy::module_name_repetitions: モジュール名と型名の重複
// 例: `auth::AuthError` の重複は可読性のために許容する
#[allow(clippy::module_name_repetitions)]
pub struct AuthError { ... }

// clippy::must_use_candidate: Result 型の明示的 use 強制
// テストヘルパー関数で返り値を意図的に無視する場合
#[allow(clippy::must_use_candidate)]
fn setup_test_db() -> PgPool { ... }
```

### カスタム clippy 設定

`Clippy.toml`（または `clippy.toml`）で閾値を設定する。

```toml
# .clippy.toml
too-many-arguments-threshold = 5
too-many-lines-threshold = 100
cognitive-complexity-threshold = 15
```

**本節で確定した方針**
- **`#![deny(clippy::all, clippy::pedantic)]` を全クレートに設定し、CI でエラー扱いとする。**
- **`#[allow(clippy::...)]` は ADR-IMPL-NNN への根拠記録を必須とし、サイレントな抑制を禁止する。**
- **関数の引数は 5 個以内、行数は 100 行以内を目安とし、超過時はリファクタリングを検討する。**

---

## 5. 命名規約

Rust の標準命名規則に加え、本プロジェクト固有のルールを定める。

| 要素 | 規則 | 例 |
|---|---|---|
| モジュール | `snake_case` | `hash_chain`, `event_store` |
| 型・トレイト・列挙型 | `PascalCase` | `WorkEvent`, `EventActivity` |
| 関数・メソッド | `snake_case` | `persist_event`, `verify_hash_chain` |
| 定数 | `SCREAMING_SNAKE_CASE` | `MAX_RETRY_COUNT`, `HASH_CHAIN_GENESIS` |
| 静的変数 | `SCREAMING_SNAKE_CASE` | `IDEMPOTENCY_CACHE_TTL_SECS` |
| マクロ | `snake_case!` | `ensure_permission!` |

### ライフタイム名の規則

単文字ライフタイム（`'a`, `'b`）は意味が不明瞭になるため、説明的な名前を使用する。

```rust
// 禁止: 意味のないライフタイム名
fn get_operator<'a>(pool: &'a PgPool) -> &'a Operator { ... }

// 推奨: 何のライフタイムかを示す名前
fn get_operator<'pool>(pool: &'pool PgPool) -> &'pool Operator { ... }

// 例外: 標準ライブラリが使用する 'static, 'self は許容
impl EventProcessor<'static> { ... }
```

ただし、`Iterator`・`Future` 等の標準トレイト実装において慣例的に使用される `'a` は許容する。

**本節で確定した方針**
- **ライフタイム名は単文字（`'a`）ではなく説明的な名前（`'pool`, `'req`）を使用する。**
- **`SCREAMING_SNAKE_CASE` を定数・静的変数の両方に適用し、大文字フォーマットで区別する。**
- **型名に `Impl`, `Base`, `Abstract` などの実装詳細を示す接尾辞を禁止する。**

---

## 6. エラー型

### thiserror 専用

エラー型は `thiserror` クレートのみを使用して定義する。`anyhow` は `main()` 関数の `?` 伝播のみに使用を限定する。

```rust
use thiserror::Error;

/// アプリケーション全体のエラー型。
/// HTTP レスポンスへの変換は `IntoResponse` 実装を参照。
#[derive(Debug, Error)]
pub enum AppError {
    // データベース操作エラー
    #[error("データベース操作に失敗しました: {0}")]
    Database(#[from] sqlx::Error),

    // ハッシュチェーンの整合性エラー
    #[error("ハッシュチェーンが破断しています: expected={expected}, actual={actual}")]
    HashChainBroken {
        expected: String,
        actual: String,
    },

    // 冪等性キーが重複している（キャッシュヒット時は返さない）
    #[error("冪等性キーが重複しています: key={key}")]
    DuplicateIdempotencyKey { key: String },

    // 認可エラー
    #[error("権限が不足しています: required_role={required_role}")]
    InsufficientPermission { required_role: String },

    // 環境変数不足
    #[error("必須環境変数が設定されていません: {0}")]
    MissingEnvVar(&'static str),
}
```

### IntoResponse による HTTP 変換

`AppError` を axum の `IntoResponse` に変換し、Problem Details 形式を返す。

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, detail) = match &self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database-error",
                "内部データベースエラーが発生しました。",
            ),
            AppError::HashChainBroken { .. } => (
                StatusCode::CONFLICT,
                "hash-chain-broken",
                "イベントの整合性検証に失敗しました。",
            ),
            AppError::InsufficientPermission { .. } => (
                StatusCode::FORBIDDEN,
                "insufficient-permission",
                "このリソースへのアクセス権限がありません。",
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal-error",
                "予期しないエラーが発生しました。",
            ),
        };

        // スタックトレース・内部詳細はログのみに記録し、クライアントに返さない
        tracing::error!(error = %self, "AppError が発生した");

        let body = Json(json!({
            "type": format!("https://api.wnav.example/errors/{}", error_type),
            "title": status.canonical_reason().unwrap_or("Unknown"),
            "status": status.as_u16(),
            "detail": detail,
        }));

        (status, body).into_response()
    }
}
```

**本節で確定した方針**
- **エラー型は `thiserror` のみで定義し、`anyhow` は `main()` の `?` 伝播に限定する。**
- **`AppError` に `IntoResponse` を実装し、全エラーを Problem Details RFC 9457 形式で返す。**
- **`#[error]` 属性のメッセージは日本語で記述し、エンドポイントの `title` は英語とする（国際化対応）。**

---

## 7. axum ハンドラ規約

### 型抽出パターン

axum のエクストラクタを活用し、型安全なリクエスト処理を行う。

```rust
use axum::extract::{Path, State, Json};
use axum::response::IntoResponse;

/// 作業イベントを受信し、Outbox キューに積む。
///
/// # 冪等性
/// `Idempotency-Key` ヘッダが同一の再送は保存済みレスポンスを返す（DB 操作なし）。
pub async fn post_work_event(
    // 認証・認可: AuditorRole 以外はコンパイルエラー
    AuthenticatedUser { user_id, .. }: AuthenticatedUser<OperatorRole>,
    // 冪等性キーは Tower ミドルウェアで検証済み
    IdempotencyKey(key): IdempotencyKey,
    State(app_state): State<AppState>,
    Json(raw_payload): Json<RawEventPayload>,
) -> Result<impl IntoResponse, AppError> {
    // 入力バリデーション（型変換で検証を強制する）
    let payload = ValidatedEventPayload::try_from(raw_payload)?;

    // サーバー受信時刻を付与する（クライアント上書き禁止）
    let server_received_at = Utc::now().timestamp_millis();

    let event = enqueue_event(
        &app_state.event_pool,
        payload,
        server_received_at,
        &user_id,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(event)))
}
```

### AppState の設計

```rust
/// アプリケーション全体で共有する状態。
/// Arc でラップして axum の State として注入する。
#[derive(Clone)]
pub struct AppState {
    /// マスタ CRUD 用接続プール（app_write ロール）
    pub write_pool: PgPool,
    /// 作業ログ記録用接続プール（app_event_insert ロール）
    pub event_pool: PgPool,
    /// 読み取り専用接続プール（app_read ロール）
    pub read_pool: PgPool,
    /// 冪等性キャッシュ（TTL 24h）
    pub idempotency_cache: Arc<IdempotencyCache>,
    /// JWT 検証用公開鍵
    pub jwt_public_key: Arc<RsaPublicKey>,
}
```

**本節で確定した方針**
- **axum エクストラクタで入力の型変換とバリデーションを一体化し、未検証入力をハンドラ本体に渡さない。**
- **`AppState` に 3 つの接続プールを用途別に持ち、ロール権限分離を実装レベルで強制する。**
- **`server_received_at` はハンドラ冒頭でサーバーが付与し、ペイロードからは受け取らない。**

---

## 8. sqlx 規約

### `sqlx::query!` マクロ必須

```rust
// 正しい: コンパイル時に SQL 構文・型・カラム名を検証する
let events = sqlx::query!(
    r#"
    SELECT id, case_id, activity, client_recorded_at, server_received_at, hash
    FROM work_events
    WHERE case_id = $1
    ORDER BY server_received_at ASC
    "#,
    case_id
)
.fetch_all(&app_state.read_pool)
.await?;

// 禁止: 実行時まで SQL エラーを検知できない
let events = sqlx::query("SELECT ...")
    .bind(case_id)
    .fetch_all(&pool)
    .await?;
```

### SQLX_OFFLINE モード（CI 用）

CI 環境では `SQLX_OFFLINE=true` を設定し、`.sqlx/` ディレクトリのキャッシュからクエリを検証する。

```bash
# ローカル開発: DB 接続あり
DATABASE_URL=postgres://app_write:...@localhost/wnav_dev sqlx prepare

# CI: DB 接続なし（.sqlx/ キャッシュを使用）
SQLX_OFFLINE=true cargo build
```

`.sqlx/` ディレクトリはバージョン管理に必ず含める。SQL を変更した際は `sqlx prepare` を再実行して `.sqlx/` を更新し、その変更を commit する。

### マイグレーション規則

```bash
# マイグレーションファイルの作成
sqlx migrate add create_work_events_table

# 適用
sqlx migrate run

# ロールバック
sqlx migrate revert
```

すべてのマイグレーションは `up.sql` / `down.sql` のペアで作成し、ロールバックが可能な状態を保つ。

**本節で確定した方針**
- **`sqlx::query!` マクロを全クエリに使用し、`sqlx::query()` を禁止する。**
- **`.sqlx/` ディレクトリをバージョン管理に含め、SQL 変更時は `sqlx prepare` の再実行と commit を必須とする。**
- **マイグレーションは `up.sql` / `down.sql` ペアで作成し、ロールバックテストを統合テストで確認する。**

---

## 9. tracing 規約

### span と instrument

非同期関数には `#[instrument]` アトリビュートを付与し、自動的にスパンを作成する。

```rust
use tracing::{instrument, info, warn, error};

/// 作業イベントを永続化し、Outbox キューに積む。
#[instrument(
    skip(pool, payload),  // 大きなオブジェクトはスキップ
    fields(
        case_id = %payload.case_id,
        activity = %payload.activity,
        operator_id = %payload.operator_id,
    )
)]
pub async fn persist_event(
    pool: &PgPool,
    payload: &ValidatedEventPayload,
    server_received_at: i64,
) -> Result<WorkEvent, AppError> {
    info!("作業イベントの永続化を開始する");
    // ...
    info!(event_id = %event.id, "作業イベントを永続化した");
    Ok(event)
}
```

### JSON フォーマット設定

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .json()                    // JSON 形式で出力する
                .with_current_span(true)
                .with_span_list(true)
                .with_target(true)
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
```

### 機微情報フィールドのマスク

```rust
// PII を含むフィールドは redacted=true でマスクする
tracing::info!(
    operator_id = %operator.id,       // UUID は OK
    operator_name = field::Empty,     // 個人名はフィールド自体を省略
    redacted = true,                  // マスク対象であることを明示
    "作業者認証に成功した"
);
```

**本節で確定した方針**
- **非同期関数には `#[instrument]` を付与し、スパンの自動生成を標準とする。**
- **ログは JSON 形式で出力し、`tracing_subscriber` の JSON レイヤを必ず設定する。**
- **PII フィールド（個人名・メール等）は `field::Empty` で省略するか `[REDACTED]` で置換する。**

---

## 10. 並行制御

### tokio::spawn の使用基準

`tokio::spawn` は独立した非同期タスクの起動に限定する。エラーを握り潰す可能性があるため、スポーンされたタスクは必ず `JoinHandle` を管理するか、パニックを伝播させる仕組みを持つ。

```rust
// Outbox ワーカーの起動（エラーはアラートとして tracing に記録する）
let worker_handle = tokio::spawn(async move {
    if let Err(e) = run_outbox_worker(pool, sender).await {
        // タスクの失敗はプロセスを停止させず、アラートとして記録する
        error!(error = %e, "Outbox ワーカーが異常終了した");
    }
});
```

### Arc<Mutex<_>> vs RwLock の使い分け

| 状況 | 型 | 理由 |
|---|---|---|
| 読み取りが多い（冪等性キャッシュ等） | `Arc<RwLock<T>>` | 複数スレッドの並列読み取りを許可 |
| 書き込みが頻繁 | `Arc<Mutex<T>>` | 書き込みが多い場合 RwLock のオーバーヘッドが逆効果 |
| DB 接続プール | `PgPool`（sqlx 内蔵） | sqlx の Pool が内部で最適化している |

### cancel safety

`tokio::select!` を使用する場合は各ブランチのキャンセル安全性を確認する。

```rust
// Outbox ワーカーのシャットダウン制御
loop {
    tokio::select! {
        // キャンセル安全: recv() はキャンセルされても次回から再開できる
        event = outbox_receiver.recv() => {
            if let Some(entry) = event {
                process_outbox_entry(entry).await?;
            }
        }
        // シャットダウンシグナルを受信したら安全に終了する
        _ = shutdown_signal.recv() => {
            info!("Outbox ワーカーをシャットダウンする");
            break;
        }
    }
}
```

**本節で確定した方針**
- **`tokio::spawn` のタスクは必ず `JoinHandle` を管理し、エラーを `tracing::error!` で記録する。**
- **読み取り多数のデータには `Arc<RwLock<T>>`、書き込み多数には `Arc<Mutex<T>>` を使用する。**
- **`tokio::select!` ブランチのキャンセル安全性を `#[cancel_safe]` コメントで明示する。**

---

## 11. 依存クレート選定基準

### 選定ルール

| 基準 | 内容 |
|---|---|
| バージョン | 最新のマイナーバージョンを使用する（パッチは自動更新可） |
| unsafe 含有率 | 0% を優先する（`cargo geiger` で確認） |
| ライセンス | MIT または Apache-2.0 のみ許容。GPL 系は禁止 |
| セキュリティ | `cargo audit` で CVE がゼロであること |
| ダウンロード数 | 週次ダウンロード数 10 万以上を目安（エコシステムでの実績） |

### Cargo.toml の記述規則

```toml
[dependencies]
# バージョンは `=` ではなく `^`（互換アップデート許容）を使用する
tokio = { version = "^1.35", features = ["full"] }
axum = { version = "^0.7", features = ["ws", "multipart"] }
sqlx = { version = "^0.8", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono"] }
thiserror = "^1.0"
tracing = "^0.1"
tracing-subscriber = { version = "^0.3", features = ["json", "env-filter"] }
serde = { version = "^1.0", features = ["derive"] }
uuid = { version = "^1.6", features = ["v4", "serde"] }
chrono = { version = "^0.4", features = ["serde"] }
```

**本節で確定した方針**
- **クレートは MIT/Apache-2.0 ライセンスのみ許容し、GPL 系を禁止する。**
- **`cargo audit` を CI に組み込み、CVE が存在する依存を自動検出してビルドを失敗させる。**
- **`Cargo.lock` を必ずバージョン管理に含め、再現可能なビルドを保証する。**

---

## 12. RBAC 型強制パターン

認可をコンパイル時に保証する型設計を採用する。ロールチェックをランタイムの `if` 文で行うことを禁止する。

```rust
use std::marker::PhantomData;

/// 認証済みユーザー。型パラメータ R でロールを制限する。
pub struct AuthenticatedUser<R: Role> {
    pub user_id: Uuid,
    pub operator_id: Uuid,
    _role: PhantomData<R>,
}

/// ロールのマーカートレイト
pub trait Role: Send + Sync {}

/// RBAC 6 ロール
pub struct AdminRole;
pub struct MasterEditorRole;
pub struct AuditorRole;
pub struct SupervisorRole;
pub struct OperatorRole;
pub struct ViewerRole;

impl Role for AdminRole {}
impl Role for MasterEditorRole {}
impl Role for AuditorRole {}
impl Role for SupervisorRole {}
impl Role for OperatorRole {}
impl Role for ViewerRole {}

// axum エクストラクタとして実装する
#[axum::async_trait]
impl<S, R: Role> FromRequestParts<S> for AuthenticatedUser<R>
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // JWT 検証・ロール確認をここで実施する
        // ...
    }
}

// ハンドラ: AuditorRole 以外はコンパイル時にエラーになる
async fn get_audit_trail(
    AuthenticatedUser { user_id, .. }: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // ...
}
```

**本節で確定した方針**
- **RBAC を `AuthenticatedUser<R: Role>` の型パラメータで表現し、コンパイル時に認可を保証する。**
- **ランタイムの `if role == "auditor"` チェックを禁止し、型で認可を強制する。**
- **6 ロール（Admin/MasterEditor/Auditor/Supervisor/Operator/Viewer）をマーカー型で定義する。**

---

## 13. unsafe 例外申請手順

`unsafe` の使用が真に必要な場合は以下の手順を踏む。

1. **ADR-IMPL-NNN の作成**: `docs/01_管理/変更管理/ADR-IMPL-NNN.md` に根拠・代替案却下理由・リスク評価を記載する。
2. **隔離クレート化**: `unsafe` コードを独立した `wnav-unsafe-*` クレートに分離する。
3. **`# Safety` コメントの必須記載**: `unsafe` ブロック・関数ごとに安全性の前提条件を記述する。
4. **最小化**: `unsafe` ブロックは可能な限り小さく保ち、安全なラッパー関数で覆う。

```rust
/// SHA-256 ハッシュのバイト比較を定数時間で行う。
///
/// # Safety
/// - `a` と `b` は同じ長さであること（長さが異なる場合は panic する）
/// - サイドチャネル攻撃対策として定数時間比較を使用する
#[inline]
unsafe fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    debug_assert_eq!(a.len(), b.len());
    // ...
}
```

**本節で確定した方針**
- **`unsafe` の使用前に ADR-IMPL-NNN を必ず作成し、事後承認を禁止する。**
- **`unsafe` コードを隔離クレート（`wnav-unsafe-*`）に分離し、メインクレートへの混入を防ぐ。**
- **`# Safety` コメントは `unsafe` ブロック・関数ごとに必須とし、省略を禁止する。**

---

## 14. Outbox Pattern 実装規約

### outbox テーブルの設計

```sql
-- Outbox テーブル: 未送信イベントのキュー
CREATE TABLE outbox (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id      UUID NOT NULL REFERENCES work_events(id),
    idempotency_key UUID NOT NULL UNIQUE,
    payload       JSONB NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    retry_count   INT NOT NULL DEFAULT 0,
    last_attempted_at TIMESTAMPTZ,
    error_message TEXT
);
```

### 送信規則

```rust
/// Outbox エントリを created_at 昇順で順次送信する。
///
/// # 注意
/// 並列送信を禁止する。送信順序の逆転は
/// ハッシュチェーンの整合性を破壊するため。
pub async fn process_outbox(
    pool: &PgPool,
    http_client: &reqwest::Client,
) -> Result<(), AppError> {
    // created_at 昇順で未送信エントリを取得する（FOR UPDATE SKIP LOCKED）
    let entries = sqlx::query!(
        r#"
        SELECT id, event_id, idempotency_key, payload
        FROM outbox
        WHERE retry_count < $1
        ORDER BY created_at ASC
        FOR UPDATE SKIP LOCKED
        LIMIT 10
        "#,
        MAX_RETRY_COUNT
    )
    .fetch_all(pool)
    .await?;

    for entry in entries {
        match send_to_server(http_client, &entry).await {
            Ok(_) => {
                // ACK 後に outbox レコードのみ削除する（work_events は削除禁止）
                sqlx::query!("DELETE FROM outbox WHERE id = $1", entry.id)
                    .execute(pool)
                    .await?;
            }
            Err(e) => {
                // リトライカウントをインクリメントする（指数バックオフ）
                sqlx::query!(
                    "UPDATE outbox SET retry_count = retry_count + 1, error_message = $1 WHERE id = $2",
                    e.to_string(),
                    entry.id
                )
                .execute(pool)
                .await?;
            }
        }
    }
    Ok(())
}
```

**本節で確定した方針**
- **Outbox の送信は `created_at` 昇順で必ず順次実行し、並列送信を禁止する。**
- **ACK 受領後に `outbox` レコードのみ削除し、`work_events` の削除を禁止する（Append-only 原則）。**
- **`FOR UPDATE SKIP LOCKED` でロックを取得し、複数ワーカーによる二重送信を防止する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)
- [`90_業界分析/24_作業者プライバシー・データ倫理と労務監視.md`](../../90_業界分析/24_作業者プライバシー・データ倫理と労務監視.md)
