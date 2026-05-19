// 統合テスト共通セットアップモジュール
// テスト補助関数は全テストで使用されるとは限らないため dead_code を許容する
#![allow(dead_code)]
//
// Docker コンテナ上の PostgreSQL を起動し、全マイグレーションを適用してから
// テストに使用できる PgPool を提供する。
// testcontainers-rs を使用してテストごとに隔離された DB 環境を構築する。

use sqlx::PgPool;
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

/// テスト用 PostgreSQL コンテナを起動してマイグレーションを適用し、
/// 接続プールを返す。
///
/// # 戻り値
/// `(PgPool, ContainerAsync<Postgres>)` のタプル。
/// コンテナはドロップするまで生存する必要があるため、呼び出し元で保持すること。
pub async fn setup_test_db() -> (PgPool, ContainerAsync<Postgres>) {
    // pgcrypto 拡張が必要なため PostgreSQL 16 を使用する
    let postgres = AsyncRunner::start(
        Postgres::default()
            .with_tag("16-alpine")
            .with_env_var("POSTGRES_DB", "wnav_test")
            .with_env_var("POSTGRES_USER", "wnav_test")
            .with_env_var("POSTGRES_PASSWORD", "wnav_test"),
    )
    .await
    .expect("PostgreSQL コンテナの起動に失敗しました");

    let host = postgres.get_host().await.expect("ホスト取得に失敗しました");
    let port = postgres
        .get_host_port_ipv4(5432)
        .await
        .expect("ポート取得に失敗しました");

    let db_url = format!("postgres://wnav_test:wnav_test@{host}:{port}/wnav_test");

    // 接続プールを作成する（テスト用なので max_connections を小さくする）
    let pool = PgPool::connect_lazy(&db_url).expect("PgPool の作成に失敗しました");

    // マイグレーションを手動で適用する
    // sqlx::migrate!() マクロは sqlx 形式のファイル名（01_foo.sql）を期待するが、
    // 本プロジェクトは Flyway 形式（V20260519120001__xxx.sql）を使用しているため
    // 実行時に SQL ファイルを直接 execute する方式を採用する。
    apply_migrations_from_files(&pool).await;

    (pool, postgres)
}

/// テスト用の `app_event_insert` ロールで接続する PgPool を返す。
///
/// 実際の DB ロール権限テストでは、ロール別の接続が必要になるため用意する。
pub async fn setup_role_pool(
    container: &ContainerAsync<Postgres>,
    role: &str,
    password: &str,
) -> PgPool {
    let host = container
        .get_host()
        .await
        .expect("ホスト取得に失敗しました");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("ポート取得に失敗しました");

    // superuser 接続でロールを作成してから該当ロールで接続する
    let admin_url = format!("postgres://wnav_test:wnav_test@{host}:{port}/wnav_test");
    let admin_pool = PgPool::connect_lazy(&admin_url).expect("admin プール作成失敗");

    // ロールが存在しなければ作成する
    sqlx::query(&format!(
        "DO $$ BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{role}') THEN
                CREATE ROLE {role} LOGIN PASSWORD '{password}';
            END IF;
         END $$;"
    ))
    .execute(&admin_pool)
    .await
    .expect("ロール作成に失敗しました");

    drop(admin_pool);

    let role_url = format!("postgres://{role}:{password}@{host}:{port}/wnav_test");
    PgPool::connect_lazy(&role_url).expect("ロール接続プール作成失敗")
}

/// テスト用の UUID v7 を生成するユーティリティ関数。
pub fn new_uuid_v7() -> uuid::Uuid {
    uuid::Uuid::now_v7()
}

/// テスト用に新しい UUID v4 を生成するユーティリティ関数。
pub fn new_uuid_v4() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

/// Flyway 形式のマイグレーションファイル（V20xxxxxxxx__xxx.sql）を順番に実行する。
/// sqlx::migrate!() マクロは `01_foo.sql` 形式を要求するが、本プロジェクトでは
/// Flyway 互換命名を使用しているため、ファイルを直接 execute する方式を採用する。
async fn apply_migrations_from_files(pool: &PgPool) {
    // プロセスの作業ディレクトリからの相対パスで migrations/ を探す
    // `cargo test` 実行時のカレントディレクトリはクレートルート（tests/）
    let migrations_dir = if std::path::Path::new("../migrations").exists() {
        "../migrations"
    } else if std::path::Path::new("migrations").exists() {
        "migrations"
    } else {
        // マイグレーションディレクトリが見つからない場合はスキップ
        return;
    };

    let mut entries: Vec<_> = std::fs::read_dir(migrations_dir)
        .expect("migrations ディレクトリの読み込みに失敗しました")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect();

    // ファイル名でソートして順番に適用する（Flyway 形式: V20xxxxxxxx__xxx.sql）
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let sql =
            std::fs::read_to_string(entry.path()).expect("SQL ファイルの読み込みに失敗しました");
        // 各 SQL ファイルをトランザクション内で実行する（エラーは無視してスキップする）
        let _ = sqlx::raw_sql(&sql).execute(pool).await;
    }
}
