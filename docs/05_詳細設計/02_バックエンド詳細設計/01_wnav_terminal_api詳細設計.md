# 01 wnav_terminal_api 詳細設計（MOD-BE-001）

本章は `crates/wnav_terminal_api/` の axum ルータ・ミドルウェアチェーン・AppState・main.rs 構造の詳細設計を確定する。本バイナリはハンディ端末向け Presentation 層の実装であり、ポート 8080 でリッスンする。BAT-002（outbox_dispatcher）・BAT-003（master_sync_puller）・BAT-008（webhook_retry_scheduler）を tokio task として内包する。

---

## 1. AppState 構造体

AppState はすべてのハンドラと共有される依存注入コンテナである。Arc でラップして axum の Extension として全ハンドラに注入する。

```rust
// crates/wnav_terminal_api/src/state.rs

use std::sync::Arc;
use wnav_auth::AuthService;
use wnav_domain::service::{
    WorkExecutionService, EvidenceService,
};
use wnav_outbox::OutboxConsumer;
use wnav_outbox::MasterSyncPuller;
use wnav_webhook::WebhookRetryScheduler;

/// wnav_terminal_api の依存コンテナ。
/// axum::Router に `.with_state(state)` で渡す。
#[derive(Clone)]
pub struct AppState {
    /// 作業実行ユースケース（FNC-BE-001〜004 を含む）
    pub work_execution_svc: Arc<dyn WorkExecutionService>,
    /// 証拠記録ユースケース
    pub evidence_svc: Arc<dyn EvidenceService>,
    /// JWT 検証・LDAP 認証
    pub auth_svc: Arc<dyn AuthService>,
    /// Outbox Consumer（BAT-002）
    pub outbox_consumer: Arc<OutboxConsumer>,
    /// Master Sync Puller（BAT-003）
    pub master_sync_puller: Arc<MasterSyncPuller>,
    /// Webhook Retry Scheduler（BAT-008）
    pub webhook_retry_scheduler: Arc<WebhookRetryScheduler>,
    /// 書き込み専用コネクションプール（DBロール: app_event_insert）
    pub app_event_insert: Arc<sqlx::PgPool>,
    /// 読み取り専用コネクションプール（DBロール: app_read）
    pub app_read: Arc<sqlx::PgPool>,
    /// アプリケーション設定
    pub config: Arc<wnav_config::TerminalApiConfig>,
}
```

---

## 2. ミドルウェアチェーン

ミドルウェアは tower::ServiceBuilder で順番に積み上げる。リクエストは上から下の順に通過し、レスポンスは逆順に通過する。

適用順序:

1. **TracingMiddleware** — X-Trace-Id 付与・構造化ログ出力
2. **AuthMiddleware** — Authorization ヘッダの JWT 検証 → CurrentUser extension
3. **RateLimitMiddleware** — トークンバケット（CFG-002 rpm）
4. **IdempotencyMiddleware** — Idempotency-Key ヘッダ → TBL-035 照合（端末書き込み専用）
5. **CaseLockMiddleware** — 作業ケースの排他制御（Issue 4 対応として Phase 3 で追加予定）

```rust
// crates/wnav_terminal_api/src/middleware/mod.rs

pub fn apply_middleware(router: Router<AppState>, config: &TerminalApiConfig) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http()
                .make_span_with(make_trace_span)
                .on_request(on_request)
                .on_response(on_response))
            .layer(axum::middleware::from_fn_with_state(config.clone(), auth_middleware))
            .layer(axum::middleware::from_fn_with_state(config.clone(), rate_limit_middleware))
            .layer(axum::middleware::from_fn_with_state(config.clone(), idempotency_middleware)),
        // CaseLockMiddleware は Issue 4 対応として Phase 3 で追加予定
    )
}
```

---

## 3. ルータ担当エンドポイント

本バイナリが担当するエンドポイント一覧を示す。すべて `/api/v1` プレフィックス配下に配置する。

| グループ | メソッド | パス |
|---|---|---|
| 認証 | POST | `/auth/login` |
| 認証 | POST | `/auth/refresh` |
| 認証 | POST | `/auth/logout` |
| 作業指示 | GET | `/work-orders` |
| 作業指示 | GET | `/work-orders/:id` |
| 作業実行 | POST | `/work-execs` |
| 作業実行 | GET | `/work-execs/:id` |
| 作業実行 | POST | `/work-execs/:id/complete` |
| 作業実行 | POST | `/work-execs/:id/suspend` |
| 作業実行 | POST | `/work-execs/:id/resume` |
| ステップイベント | POST | `/step-events` |
| 証拠ファイル | POST | `/evidences` |
| 電子サイン | POST | `/electronic-signs` |
| マスタ同期 | GET | `/sync/masters` |
| 受入検査（IQC） | POST | `/iqc-001` |
| 受入検査（IQC） | POST | `/iqc-002` |
| 受入検査（IQC） | POST | `/iqc-003` |
| 受入検査（IQC） | GET | `/iqc-006` |
| 手直し検証 | POST | `/rework-verifications` |
| システム | GET | `/healthz` |

---

## 4. main.rs 構造

```rust
// crates/wnav_terminal_api/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 設定ロード（YAML + 環境変数オーバーレイ・ADR-IMPL-001）
    let config: TerminalApiConfig = wnav_config::load_terminal_api()?;

    // tracing 初期化
    init_tracing(&config.tracing_level);

    // DB コネクションプール
    // app_event_insert: 書き込み専用（端末イベント INSERT 専用ロール）
    let app_event_insert = Arc::new(
        wnav_db::connect(&config.event_insert_database_url, &config.db_config).await?
    );
    // app_read: 読み取り専用
    let app_read = Arc::new(
        wnav_db::connect(&config.read_database_url, &config.db_config).await?
    );

    // 依存オブジェクト構築
    let state = build_app_state(app_event_insert.clone(), app_read.clone(), &config).await?;

    // ルータ・ミドルウェア構築
    let app = create_router(state.clone());
    let app = apply_middleware(app, &config);
    let app = app.layer(cors_layer(&config.cors_allow_origins));

    // HTTP リスナー（TLS 終端は IIS に委譲）
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.port)  // デフォルト: 8080
    ).await?;
    tracing::info!(port = config.port, "wnav_terminal_api started");

    // tokio::select! で全タスクのライフサイクルを管理
    tokio::select! {
        // 端末 API サーバー
        result = axum::serve(listener, app) => {
            result?;
        }
        // BAT-002: Outbox Dispatcher（常駐 60s ループ）
        _ = tokio::spawn(wnav_outbox::run_consumer(state.outbox_consumer.clone())) => {
            tracing::error!("outbox_dispatcher (BAT-002) exited unexpectedly");
        }
        // BAT-003: Master Sync Puller（毎 60 分 cron）
        _ = tokio::spawn(wnav_outbox::run_master_sync(state.master_sync_puller.clone())) => {
            tracing::error!("master_sync_puller (BAT-003) exited unexpectedly");
        }
        // BAT-008: Webhook Retry Scheduler（毎 1 分 cron）
        _ = tokio::spawn(wnav_webhook::run_retry_scheduler(state.webhook_retry_scheduler.clone())) => {
            tracing::error!("webhook_retry_scheduler (BAT-008) exited unexpectedly");
        }
        // BAT-013: CaseLock Reaper — Issue 4 対応として Phase 3 で追加予定
    }

    Ok(())
}
```

---

## 5. 制約

| 制約項目 | 内容 |
|---|---|
| DB ロール | 書き込み系ハンドラは `app_event_insert` プール経由のみ許可。読み取りは `app_read` プールを使用する |
| Idempotency-Key | 本バイナリのみ IdempotencyMiddleware を適用する。wnav_master_api では適用しない（`src/backend/CLAUDE.md` の規定による） |
| TLS 終端 | IIS（Windows Server 2022）リバースプロキシに委譲する。本バイナリは HTTP（ポート 8080）でリッスンする |
| BAT 範囲 | 本バイナリが起動するバッチは BAT-002・BAT-003・BAT-008 のみ。BAT-001 および BAT-004〜012 は wnav_master_api に内包する |

---

**本節で確定した方針**
- **wnav_terminal_api はポート 8080 でリッスンし、ハンディ端末向けエンドポイントおよび BAT-002/003/008 を内包することを確定した。**
- **Idempotency-Key 検証は本バイナリのみに適用し、管理側 API との責務分離を明確にした。**
- **app_event_insert ロールは INSERT 専用に制限し、マスタ更新等の書き込みが端末 API 経由で実行されることを DB レベルで防止する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`04_概要設計/02_ソフトウェア方式設計/10_バッチ・常駐ジョブ設計.md`](../../../04_概要設計/02_ソフトウェア方式設計/10_バッチ・常駐ジョブ設計.md)
