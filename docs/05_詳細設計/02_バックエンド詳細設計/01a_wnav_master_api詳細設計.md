# 01a wnav_master_api 詳細設計（MOD-BE-010）

本章は `crates/wnav_master_api/` の axum ルータ・ミドルウェアチェーン・AppState・main.rs 構造の詳細設計を確定する。本バイナリはマスタメンテナンス・管理系 Presentation 層の実装であり、ポート 8081 でリッスンする。BAT-001（hash_chain_verifier）および BAT-004〜012 を tokio task として内包する。

---

## 1. AppState 構造体

AppState はすべてのハンドラと共有される依存注入コンテナである。Arc でラップして axum の Extension として全ハンドラに注入する。

```rust
// crates/wnav_master_api/src/state.rs

use std::sync::Arc;
use wnav_auth::AuthService;
use wnav_domain::service::{
    MasterService, AndonService, ReportService,
};
use wnav_hash_chain::HashChainService;

/// wnav_master_api の依存コンテナ。
/// axum::Router に `.with_state(state)` で渡す。
/// Idempotency は適用しない（src/backend/CLAUDE.md の規定による）。
#[derive(Clone)]
pub struct AppState {
    /// マスタ管理ユースケース
    pub master_svc: Arc<dyn MasterService>,
    /// アンドン・不適合ユースケース
    pub andon_svc: Arc<dyn AndonService>,
    /// 帳票生成ユースケース
    pub report_svc: Arc<dyn ReportService>,
    /// JWT 検証・LDAP 認証
    pub auth_svc: Arc<dyn AuthService>,
    /// ハッシュチェーン検証サービス（BAT-001 に使用）
    pub hash_chain_svc: Arc<HashChainService>,
    /// 書き込み用コネクションプール（DBロール: app_write）
    pub app_write: Arc<sqlx::PgPool>,
    /// 読み取り専用コネクションプール（DBロール: app_read）
    pub app_read: Arc<sqlx::PgPool>,
    /// アプリケーション設定
    pub config: Arc<wnav_config::MasterApiConfig>,
}
```

---

## 2. ミドルウェアチェーン

ミドルウェアは tower::ServiceBuilder で順番に積み上げる。リクエストは上から下の順に通過し、レスポンスは逆順に通過する。

適用順序:

1. **TracingMiddleware** — X-Trace-Id 付与・構造化ログ出力
2. **AuthMiddleware** — Authorization ヘッダの JWT 検証 → CurrentUser extension
3. **RateLimitMiddleware** — トークンバケット（CFG-002 rpm）

IdempotencyMiddleware は本バイナリには適用しない（`src/backend/CLAUDE.md` の規定による）。

```rust
// crates/wnav_master_api/src/middleware/mod.rs

pub fn apply_middleware(router: Router<AppState>, config: &MasterApiConfig) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http()
                .make_span_with(make_trace_span)
                .on_request(on_request)
                .on_response(on_response))
            .layer(axum::middleware::from_fn_with_state(config.clone(), auth_middleware))
            .layer(axum::middleware::from_fn_with_state(config.clone(), rate_limit_middleware)),
        // IdempotencyMiddleware は非適用
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
| 鍵管理 | GET / POST | `/keys` |
| 鍵管理 | GET / PUT / DELETE | `/keys/:id` |
| マスタ管理 | GET / POST | `/master` |
| マスタ管理 | GET / PUT / DELETE | `/master/:id` |
| マスタバージョン | GET / POST | `/master-versions` |
| マスタバージョン | GET / PUT | `/master-versions/:id` |
| マスタバージョン | POST | `/master-versions/:id/submit-for-review` |
| マスタバージョン | POST | `/master-versions/:id/approve` |
| マスタバージョン | POST | `/master-versions/:id/publish` |
| マスタバージョン | POST | `/master-versions/:id/archive` |
| マスタバージョン | GET | `/master-versions/:id/diff` |
| アンドン | POST | `/andon-002` |
| CAPA | GET / POST | `/capa` |
| CAPA | POST | `/capa/:id/close` |
| 改善提案 | GET / POST | `/kaizen` |
| トレサビ | GET | `/trace` |
| 帳票・レポート | POST | `/reports/work-summary` |
| 帳票・レポート | POST | `/reports/audit-xes` |
| 運用・管理 | GET | `/ops/outbox/dlq` |
| 運用・管理 | POST | `/ops/outbox/:id/requeue` |
| 運用・管理 | POST | `/ops/hash-chain/verify` |
| 運用・管理 | POST | `/ops/master-sync` |
| 受入検査（IQC） | POST | `/iqc-004` |
| 受入検査（IQC） | POST | `/iqc-005` |
| 不適合処置 | GET / POST | `/dispositions` |
| 不適合処置 | PUT | `/dispositions/:id` |
| 手直し | GET / POST | `/reworks` |
| スクラップ記録 | GET / POST | `/scrap-records` |
| 返却記録 | GET / POST | `/return-records` |
| システム | GET | `/healthz` |

---

## 4. main.rs 構造

```rust
// crates/wnav_master_api/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 設定ロード（YAML + 環境変数オーバーレイ・ADR-IMPL-001）
    let config: MasterApiConfig = wnav_config::load_master_api()?;

    // tracing 初期化
    init_tracing(&config.tracing_level);

    // DB コネクションプール
    // app_write: マスタ更新・管理系書き込み専用ロール
    let app_write = Arc::new(
        wnav_db::connect(&config.write_database_url, &config.db_config).await?
    );
    // app_read: 読み取り専用
    let app_read = Arc::new(
        wnav_db::connect(&config.read_database_url, &config.db_config).await?
    );

    // 依存オブジェクト構築
    let state = build_app_state(app_write.clone(), app_read.clone(), &config).await?;

    // ルータ・ミドルウェア構築
    let app = create_router(state.clone());
    let app = apply_middleware(app, &config);
    let app = app.layer(cors_layer(&config.cors_allow_origins));

    // HTTP リスナー（管理 LAN 限定・ポート 8081）
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.port)  // デフォルト: 8081
    ).await?;
    tracing::info!(port = config.port, "wnav_master_api started");

    // tokio::select! で全タスクのライフサイクルを管理
    tokio::select! {
        // 管理 API サーバー
        result = axum::serve(listener, app) => {
            result?;
        }
        // BAT-001: HashChain 検証（週次 cron 月曜 02:00 JST）
        _ = tokio::spawn(wnav_hash_chain::run_weekly_verifier(state.hash_chain_svc.clone())) => {
            tracing::error!("hash_chain_verifier (BAT-001) exited unexpectedly");
        }
        // BAT-004: PII 匿名化（日次 03:00）
        _ = tokio::spawn(pii_anonymizer::run(state.clone())) => {
            tracing::error!("pii_anonymizer (BAT-004) exited unexpectedly");
        }
        // BAT-005: pg_dump バックアップ（日次 01:00）
        _ = tokio::spawn(backup_pg_dump::run(state.clone())) => {
            tracing::error!("backup_pg_dump (BAT-005) exited unexpectedly");
        }
        // BAT-006: WAL アーカイブ（常駐）
        _ = tokio::spawn(backup_wal_archive::run(state.clone())) => {
            tracing::error!("backup_wal_archive (BAT-006) exited unexpectedly");
        }
        // BAT-007: 帳票ハッシュ記録（帳票生成後トリガ）
        _ = tokio::spawn(document_hash_recorder::run(state.clone())) => {
            tracing::error!("document_hash_recorder (BAT-007) exited unexpectedly");
        }
        // BAT-009: 証明書期限通知（日次 09:00）
        _ = tokio::spawn(cert_expiry_notifier::run(state.clone())) => {
            tracing::error!("cert_expiry_notifier (BAT-009) exited unexpectedly");
        }
        // BAT-010: JWT キーローテーション（90 日ごと）
        _ = tokio::spawn(jwt_key_rotator::run(state.clone())) => {
            tracing::error!("jwt_key_rotator (BAT-010) exited unexpectedly");
        }
        // BAT-011: リワークコスト集計（日次）
        _ = tokio::spawn(rework_cost_aggregator::run(state.clone())) => {
            tracing::error!("rework_cost_aggregator (BAT-011) exited unexpectedly");
        }
        // BAT-012: その他管理系バッチ
        _ = tokio::spawn(misc_admin_batch::run(state.clone())) => {
            tracing::error!("misc_admin_batch (BAT-012) exited unexpectedly");
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
| DB ロール | 書き込み系ハンドラは `app_write` プール経由のみ許可。読み取りは `app_read` プールを使用する |
| Idempotency | IdempotencyMiddleware は本バイナリには適用しない（`src/backend/CLAUDE.md` の規定による） |
| ネットワーク | ポート 8081 は管理 LAN 限定とし、ハンディ端末からの直接アクセスを禁止する |
| BAT 範囲 | 本バイナリが起動するバッチは BAT-001・BAT-004〜012 のみ。BAT-002/003/008 は wnav_terminal_api に内包する |
| TLS 終端 | IIS（Windows Server 2022）リバースプロキシに委譲する。本バイナリは HTTP（ポート 8081）でリッスンする |

---

**本節で確定した方針**
- **wnav_master_api はポート 8081 でリッスンし、管理・マスタメンテ向けエンドポイントおよび BAT-001/004〜012 を内包することを確定した。**
- **IdempotencyMiddleware は適用せず、ミドルウェアチェーンを Tracing → Auth → RateLimit の 3 段に限定することを確定した。**
- **app_write ロールで管理系書き込みを集中管理し、端末側バイナリからの意図しないマスタ変更を防止する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`04_概要設計/02_ソフトウェア方式設計/10_バッチ・常駐ジョブ設計.md`](../../../04_概要設計/02_ソフトウェア方式設計/10_バッチ・常駐ジョブ設計.md)
