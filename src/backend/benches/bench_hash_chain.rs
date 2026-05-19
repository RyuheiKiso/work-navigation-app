// TST-perf-001/002: ハッシュチェーン計算ベンチマーク
//
// hash chain 計算のスループット測定（1000 件のシーケンシャル計算）と
// canonical_json の処理速度（ネスト深度 3 / 5 / 10）を測定する。
//
// 権威ドキュメント: docs/05_詳細設計/08_テストケース詳細設計/06_セキュリティ・性能テストケース.md TST-perf-003

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wnav_hash_chain::{
    GENESIS_PREV_HASH, canonical_json, compute_chain_hash, compute_content_hash,
};

/// TST-perf-001: hash chain 計算のスループット測定（1000 件のシーケンシャル計算）。
/// NFR-PERF-003: 100 万件のハッシュチェーン検証が 60 秒以内であることを確認する前提として
/// 1000 件のシーケンシャル計算のスループットを計測する。
fn bench_hash_chain_sequential_1000(c: &mut Criterion) {
    let case_id = uuid::Uuid::now_v7();

    let mut group = c.benchmark_group("hash_chain_sequential");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("1000_blocks", |b| {
        b.iter(|| {
            let mut prev_hash = GENESIS_PREV_HASH;

            // 1000 件のブロックを順番にハッシュ計算する
            for i in 0..1000u64 {
                let payload = serde_json::json!({
                    "activity": "step.completed",
                    "case_id": case_id.to_string(),
                    "sequence": i,
                });
                let canonical = canonical_json(&payload);
                let content_hash = compute_content_hash(&canonical);
                let block_hash = compute_chain_hash(&prev_hash, &content_hash);
                prev_hash = block_hash;
            }

            // 最終チェーンハッシュを返す（最適化によりベンチマークが除去されないようにする）
            prev_hash
        });
    });

    group.finish();
}

/// TST-perf-002: canonical_json の処理速度測定（ネスト深度 3 / 5 / 10）。
/// ネスト深度によるパフォーマンス劣化を測定し、製造現場での実用性を確認する。
fn bench_canonical_json_nesting_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonical_json_depth");

    // ネスト深度 3 の JSON
    let depth_3 = serde_json::json!({
        "activity": "step.completed",
        "case_id": uuid::Uuid::now_v7().to_string(),
        "payload": {
            "measurement": {
                "value": 50.0,
                "unit": "mm",
                "valid": true,
            }
        }
    });

    // ネスト深度 5 の JSON
    let depth_5 = serde_json::json!({
        "activity": "step.completed",
        "case_id": uuid::Uuid::now_v7().to_string(),
        "payload": {
            "measurement": {
                "details": {
                    "sensor": {
                        "calibration": {
                            "last_calibrated": "2026-05-17",
                            "accuracy": 0.001
                        },
                        "id": "SENSOR-001"
                    },
                    "raw_value": 50.123
                },
                "value": 50.0,
                "unit": "mm"
            }
        }
    });

    // ネスト深度 10 の JSON（極端なケース）
    let depth_10 = create_nested_json(10);

    for (depth, payload) in [(3, depth_3), (5, depth_5), (10, depth_10)] {
        group.bench_with_input(BenchmarkId::new("depth", depth), &payload, |b, p| {
            b.iter(|| {
                let canonical = canonical_json(p);
                // canonical JSON から SHA-256 を計算する
                compute_content_hash(&canonical)
            });
        });
    }

    group.finish();
}

/// TST-perf-001: 単一ハッシュ計算の基本パフォーマンス測定。
/// 各ブロックの compute_chain_hash の実行時間を計測する。
fn bench_single_chain_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_hash_operations");

    let canonical = r#"{"activity":"step.completed","case_id":"test-case-001","sequence":1}"#;
    let content_hash = compute_content_hash(canonical);

    group.bench_function("compute_chain_hash", |b| {
        b.iter(|| compute_chain_hash(&GENESIS_PREV_HASH, &content_hash));
    });

    group.bench_function("compute_content_hash", |b| {
        b.iter(|| compute_content_hash(canonical));
    });

    group.bench_function("canonical_json_simple", |b| {
        let payload = serde_json::json!({
            "z_key": "last",
            "a_key": "first",
            "m_key": "middle",
        });
        b.iter(|| canonical_json(&payload));
    });

    group.finish();
}

/// 深くネストされた JSON を生成するヘルパー関数。
fn create_nested_json(depth: u32) -> serde_json::Value {
    if depth == 0 {
        serde_json::json!({ "value": "leaf", "depth": 0 })
    } else {
        serde_json::json!({
            "level": depth,
            "nested": create_nested_json(depth - 1),
        })
    }
}

criterion_group!(
    benches,
    bench_hash_chain_sequential_1000,
    bench_canonical_json_nesting_depth,
    bench_single_chain_hash
);
criterion_main!(benches);
