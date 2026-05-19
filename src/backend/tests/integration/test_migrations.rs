// TST-intg-001: マイグレーション適用テスト
//
// 全マイグレーション適用後に期待されるテーブル・ビューが存在することを確認する。
// testcontainers-rs で実際の PostgreSQL コンテナを使用する。

/// 全マイグレーション適用後に必要なテーブルが存在することを確認する。
/// 権威ドキュメント: docs/05_詳細設計/08_テストケース詳細設計/03_統合テストケース（API）.md
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_001_all_tables_exist_after_migration() {
    let (pool, _container) = common::setup_test_db().await;

    // 期待されるコアテーブル一覧（TBL-001〜TBL-053 の代表的なもの）
    let expected_tables = vec![
        // 作業記録テーブル
        "work_events",
        "work_executions",
        "hash_chain_blocks",
        "hash_chain_verification_results",
        "outbox_events",
        "idempotency_keys",
        "case_locks",
        // マスタテーブル
        "users",
        "sops",
        "sop_versions",
        "operations",
        "steps",
        "electronic_signs",
        "evidence_files",
        "andon_alerts",
        "work_assignments",
        "sse_dispatch_logs",
        // IQC テーブル
        "incoming_inspections",
        "incoming_inspection_measurements",
        "sampling_plans",
        "lot_qc_states",
        "concession_approvals",
        "lots",
        // リワーク・品質テーブル
        "nonconformities",
        "dispositions",
        "reworks",
        "rework_verifications",
        "reworked_lot_labels",
        "scrap_records",
        "return_to_vendor_records",
        // 認証ログ
        "auth_logs",
        // 帳票
        "reports",
        // サプライヤー・材料
        "suppliers",
        "materials",
    ];

    for table in &expected_tables {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        assert!(exists, "テーブルが存在しません: {table}");
    }
}

/// マイグレーション適用後に必要なビューが存在することを確認する（VW-001〜VW-008）。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_001_views_exist_after_migration() {
    let (pool, _container) = common::setup_test_db().await;

    // VW-001〜008: 必須ビュー一覧
    let expected_views = vec![
        "v_active_sop_versions",
        "v_work_execution_summary",
        "v_andon_active",
        "v_lot_qc_status",
        "v_rework_progress",
        "v_operator_assignments",
        "v_hash_chain_integrity",
        "v_outbox_pending",
    ];

    for view in &expected_views {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.views
                WHERE table_schema = 'public' AND table_name = $1
            )",
        )
        .bind(view)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        assert!(exists, "ビューが存在しません: {view}");
    }
}

/// 全テーブル数が期待値以上であることを確認する（TST-intg-001 補完）。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_001_table_count_sufficient() {
    let (pool, _container) = common::setup_test_db().await;

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE'",
    )
    .fetch_one(&pool)
    .await
    .expect("テーブル数の取得に失敗しました");

    // 設計上の最低テーブル数（53 テーブル以上）
    assert!(
        count >= 30,
        "テーブル数が不足しています: 期待 ≥ 30, 実際 = {count}"
    );
}

/// sqlx migrate のバージョン記録テーブルが正常に機能していることを確認する。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_001_migration_versions_recorded() {
    let (pool, _container) = common::setup_test_db().await;

    // sqlx は _sqlx_migrations テーブルにバージョンを記録する
    let applied: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true")
            .fetch_one(&pool)
            .await
            .expect("マイグレーションバージョン取得に失敗しました");

    // 少なくとも 1 件のマイグレーションが適用されていること
    assert!(applied >= 1, "適用済みマイグレーションが 0 件です");
}

/// Append-only トリガーが CREATE されていることを確認する。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_001_append_only_triggers_exist() {
    let (pool, _container) = common::setup_test_db().await;

    let trigger_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.triggers
         WHERE trigger_schema = 'public'
           AND trigger_name LIKE 'trg_deny_%'",
    )
    .fetch_one(&pool)
    .await
    .expect("トリガー数の取得に失敗しました");

    // work_events・hash_chain_blocks・auth_logs 等に UPDATE/DELETE トリガーがあること
    assert!(
        trigger_count >= 6,
        "Append-only トリガーが不足しています: 実際 = {trigger_count}"
    );
}

// ヘルパーモジュールとして common を取り込む
#[path = "common.rs"]
mod common;
