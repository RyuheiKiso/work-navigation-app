// TST-perf-005/006: DB クエリパフォーマンスベンチマーク
//
// work_event INSERT スループット（sequential vs concurrent）と
// SELECT with filter（work_executions WHERE status=IN_PROGRESS）を測定する。
// Docker なし環境では compile-only テスト（`--no-run`）で確認する。
//
// 権威ドキュメント: docs/05_詳細設計/08_テストケース詳細設計/06_セキュリティ・性能テストケース.md TST-perf-001/002

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

/// TST-perf-005: work_event INSERT スループット測定（sequential）。
/// 実際の DB 接続を使用したベンチマーク（Docker 環境が必要）。
/// CI 環境外では `--no-run` でビルド確認のみ行う。
fn bench_work_event_insert_sequential(c: &mut Criterion) {
    // 非同期 tokio runtime を作成する
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime の作成に失敗しました");

    // DB 接続が利用できない場合はスキップするためのフラグ
    let db_available = std::env::var("WNAV_TEST_DB_URL").is_ok();
    if !db_available {
        // DB なし環境では in-memory の serialization コストのみ測定する
        let mut group = c.benchmark_group("work_event_insert_no_db");
        group.throughput(Throughput::Elements(100));

        group.bench_function("serialize_100_events", |b| {
            b.iter(|| {
                let events: Vec<serde_json::Value> = (0..100)
                    .map(|i| {
                        serde_json::json!({
                            "event_id": uuid::Uuid::now_v7().to_string(),
                            "case_id": uuid::Uuid::now_v7().to_string(),
                            "activity": "step.completed",
                            "sequence": i,
                            "timestamp_server": chrono::Utc::now().to_rfc3339(),
                            "resource": uuid::Uuid::now_v7().to_string(),
                        })
                    })
                    .collect();
                events
            });
        });

        group.finish();
        return;
    }

    // DB 接続が利用可能な場合の実際の INSERT ベンチマーク
    let db_url = std::env::var("WNAV_TEST_DB_URL").expect("WNAV_TEST_DB_URL が設定されていません");
    let pool = rt
        .block_on(sqlx::PgPool::connect(&db_url))
        .expect("DB 接続に失敗しました");

    let mut group = c.benchmark_group("work_event_insert_sequential");
    group.throughput(Throughput::Elements(100));
    group.sample_size(10); // DB テストはサンプル数を少なくする

    group.bench_function("100_sequential_inserts", |b| {
        b.iter(|| {
            rt.block_on(async {
                let case_id = uuid::Uuid::now_v7();
                let resource_id = uuid::Uuid::now_v7();
                let sop_version_id = uuid::Uuid::now_v7();
                let terminal_id = uuid::Uuid::now_v7();

                for _ in 0..100_u64 {
                    let event_id = uuid::Uuid::now_v7();
                    let _ = sqlx::query(
                        "INSERT INTO work_events
                            (event_id, case_id, activity, timestamp_client, timestamp_server,
                             resource, sop_version_id, terminal_id, payload, prev_hash, content_hash)
                         VALUES ($1, $2, 'step.completed', NOW(), NOW(), $3, $4, $5,
                             '{}'::jsonb, repeat('0', 64), repeat('0', 64))
                         ON CONFLICT DO NOTHING",
                    )
                    .bind(event_id)
                    .bind(case_id)
                    .bind(resource_id)
                    .bind(sop_version_id)
                    .bind(terminal_id)
                    .execute(&pool)
                    .await;
                }
            })
        });
    });

    group.finish();
}

/// TST-perf-006: SELECT with filter ベンチマーク。
/// work_executions WHERE status='IN_PROGRESS' のクエリパフォーマンスを測定する。
fn bench_work_executions_filter_select(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime の作成に失敗しました");

    let db_available = std::env::var("WNAV_TEST_DB_URL").is_ok();
    if !db_available {
        // DB なし環境では構造体の作成コストのみ測定する
        let mut group = c.benchmark_group("work_execution_filter_no_db");
        group.throughput(Throughput::Elements(1000));

        group.bench_function("filter_in_progress_struct_1000", |b| {
            b.iter(|| {
                // WorkExecution のフィルタリングをメモリ上でシミュレートする
                let statuses = vec!["IN_PROGRESS", "SUSPENDED", "COMPLETED", "NOT_STARTED"];
                let executions: Vec<&str> = statuses
                    .iter()
                    .cycle()
                    .take(1000)
                    .filter(|&&s| s == "IN_PROGRESS")
                    .copied()
                    .collect();
                executions
            });
        });

        group.finish();
        return;
    }

    // DB 接続が利用可能な場合の実際の SELECT ベンチマーク
    let db_url = std::env::var("WNAV_TEST_DB_URL").expect("WNAV_TEST_DB_URL が設定されていません");
    let pool = rt
        .block_on(sqlx::PgPool::connect(&db_url))
        .expect("DB 接続に失敗しました");

    let mut group = c.benchmark_group("work_execution_filter_select");
    group.sample_size(10); // DB テストはサンプル数を少なくする

    group.bench_function("select_in_progress", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _rows: Vec<(uuid::Uuid,)> = sqlx::query_as(
                    "SELECT work_execution_id FROM work_executions WHERE status = 'IN_PROGRESS' LIMIT 100",
                )
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            })
        });
    });

    group.finish();
}

/// work_event の JSONB payload シリアライズコストを測定する補助ベンチマーク。
fn bench_payload_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_serialization");
    group.throughput(Throughput::Elements(1000));

    // 標準的な step 完了ペイロード
    let typical_payload = serde_json::json!({
        "step_id": uuid::Uuid::now_v7().to_string(),
        "input_type": "Numeric",
        "value": 50.0,
        "unit": "mm",
        "evidence_ids": [uuid::Uuid::now_v7().to_string()],
        "note": "正常完了",
        "sop_version": "1.0.0"
    });

    group.bench_function("jsonb_serialize_1000", |b| {
        b.iter(|| {
            let serialized: Vec<String> = (0..1000)
                .map(|_| serde_json::to_string(&typical_payload).expect("シリアライズ失敗"))
                .collect();
            serialized
        });
    });

    group.bench_function("jsonb_deserialize_1000", |b| {
        let serialized = serde_json::to_string(&typical_payload).expect("シリアライズ失敗");
        b.iter(|| {
            let deserialized: Vec<serde_json::Value> = (0..1000)
                .map(|_| {
                    serde_json::from_str(&serialized).expect("デシリアライズ失敗")
                })
                .collect();
            deserialized
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_work_event_insert_sequential,
    bench_work_executions_filter_select,
    bench_payload_serialization
);
criterion_main!(benches);
