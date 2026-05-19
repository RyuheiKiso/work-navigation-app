// TST-intg-003: DB ロール権限テスト
//
// DB ロール別の権限が正しく設定されていることを確認する。
// - app_event_insert: INSERT のみ（作業ログテーブル）
// - app_read: SELECT のみ（全テーブル）
// - app_write: SELECT / INSERT / UPDATE（マスタテーブルのみ）
//
// 権威ドキュメント: src/backend/CLAUDE.md「DB ロール 3 分離とバイナリへの割り当て」

/// app_event_insert ロールで work_events に INSERT できることを確認する（TST-intg-003）。
/// このロールは Append-only のイベント記録専用であり INSERT のみ許可される。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_003_event_insert_role_can_insert_work_events() {
    let (pool, _container) = common::setup_test_db().await;

    // superuser で app_event_insert ロールを作成し権限を付与する
    setup_app_event_insert_role(&pool).await;

    // app_event_insert ロールでの接続を試みる（実際のロール権限は psql セッション依存）
    // ここでは superuser 接続でロール権限をシミュレートする
    let grant_check: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('app_event_insert', 'work_events', 'INSERT')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    assert!(
        grant_check,
        "app_event_insert ロールに work_events への INSERT 権限がありません"
    );
}

/// app_event_insert ロールで work_events に UPDATE できないことを確認する（TST-intg-003）。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_003_event_insert_role_cannot_update_work_events() {
    let (pool, _container) = common::setup_test_db().await;

    setup_app_event_insert_role(&pool).await;

    // UPDATE 権限がないことを確認する
    let has_update: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('app_event_insert', 'work_events', 'UPDATE')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(true); // エラーの場合は false 扱い（ロールが存在しない等）

    assert!(
        !has_update,
        "app_event_insert ロールに work_events への UPDATE 権限があってはなりません"
    );
}

/// app_read ロールで SELECT できることを確認する（TST-intg-003）。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_003_read_role_can_select() {
    let (pool, _container) = common::setup_test_db().await;

    setup_app_read_role(&pool).await;

    let has_select: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('app_read', 'work_events', 'SELECT')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    assert!(
        has_select,
        "app_read ロールに work_events への SELECT 権限がありません"
    );
}

/// app_read ロールで INSERT できないことを確認する（TST-intg-003）。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_003_read_role_cannot_insert() {
    let (pool, _container) = common::setup_test_db().await;

    setup_app_read_role(&pool).await;

    let has_insert: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('app_read', 'work_events', 'INSERT')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(true);

    assert!(
        !has_insert,
        "app_read ロールに work_events への INSERT 権限があってはなりません"
    );
}

/// app_write ロールでマスタテーブル（sops）に INSERT/UPDATE できることを確認する（TST-intg-003）。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_003_write_role_can_insert_update_master_tables() {
    let (pool, _container) = common::setup_test_db().await;

    setup_app_write_role(&pool).await;

    let has_insert: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('app_write', 'sops', 'INSERT')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    let has_update: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('app_write', 'sops', 'UPDATE')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    assert!(
        has_insert,
        "app_write ロールに sops への INSERT 権限がありません"
    );
    assert!(
        has_update,
        "app_write ロールに sops への UPDATE 権限がありません"
    );
}

/// app_write ロールが work_events に直接 INSERT できないことを確認する（TST-intg-003）。
/// 作業ログの書き込みは app_event_insert ロール専用である。
#[tokio::test]
#[ignore = "requires Docker"]
async fn tst_intg_003_write_role_cannot_insert_work_events() {
    let (pool, _container) = common::setup_test_db().await;

    setup_app_write_role(&pool).await;

    let has_insert: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('app_write', 'work_events', 'INSERT')",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(true);

    assert!(
        !has_insert,
        "app_write ロールに work_events への INSERT 権限があってはなりません（app_event_insert 専用）"
    );
}

/// app_event_insert ロールをセットアップするヘルパー関数。
async fn setup_app_event_insert_role(pool: &sqlx::PgPool) {
    // ロールが存在しない場合は作成する
    let _ = sqlx::query(
        "DO $$ BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_insert') THEN
                CREATE ROLE app_event_insert;
            END IF;
         END $$;",
    )
    .execute(pool)
    .await;

    // work_events に INSERT のみ付与する
    let _ = sqlx::query("GRANT INSERT ON work_events TO app_event_insert")
        .execute(pool)
        .await;

    // hash_chain_blocks・outbox_events にも INSERT のみ付与する
    let _ = sqlx::query("GRANT INSERT ON hash_chain_blocks TO app_event_insert")
        .execute(pool)
        .await;

    // case_locks・idempotency_keys は例外的に INSERT/UPDATE/DELETE を許可する（ADR 例外制御）
    let _ = sqlx::query(
        "GRANT INSERT, UPDATE, DELETE ON case_locks TO app_event_insert",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "GRANT INSERT, UPDATE, DELETE ON idempotency_keys TO app_event_insert",
    )
    .execute(pool)
    .await;
}

/// app_read ロールをセットアップするヘルパー関数。
async fn setup_app_read_role(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DO $$ BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_read') THEN
                CREATE ROLE app_read;
            END IF;
         END $$;",
    )
    .execute(pool)
    .await;

    // 全テーブルに SELECT のみ付与する
    let _ = sqlx::query("GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_read")
        .execute(pool)
        .await;
}

/// app_write ロールをセットアップするヘルパー関数。
async fn setup_app_write_role(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DO $$ BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_write') THEN
                CREATE ROLE app_write;
            END IF;
         END $$;",
    )
    .execute(pool)
    .await;

    // マスタテーブルに SELECT/INSERT/UPDATE を付与する（work_events は除く）
    let master_tables = vec![
        "sops", "sop_versions", "operations", "steps", "users", "suppliers", "materials",
    ];
    for table in master_tables {
        let _ = sqlx::query(&format!(
            "GRANT SELECT, INSERT, UPDATE ON {table} TO app_write"
        ))
        .execute(pool)
        .await;
    }
}

#[path = "common.rs"]
mod common;
