// TST-perf-004: JSON Logic 評価速度ベンチマーク
//
// JSON Logic ルールの評価速度を測定する（複雑なルール: AND/OR ネスト深度 5）。
// Step エンジンのアドオン機構で使用する JSON Logic の実用的な評価速度を確認する。
//
// 権威ドキュメント: docs/05_詳細設計/08_テストケース詳細設計/06_セキュリティ・性能テストケース.md TST-perf-004
// 権威ドキュメント: docs/02_企画/システム化計画/18_拡張可能Stepエンジン（アドオン機構）.md

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

/// TST-perf-004: JSON Logic 評価速度測定（AND/OR ネスト深度 5）。
/// 製造現場のポカヨケルールを模した複雑な条件を評価する速度を確認する。
fn bench_json_logic_complex_rule(c: &mut Criterion) {
    // AND/OR ネスト深度 5 の複雑なルール（ポカヨケ条件を模した設計）
    let complex_rule = serde_json::json!({
        "and": [
            {">=": [{"var": "measurement.value"}, {"var": "spec.min"}]},
            {"<=": [{"var": "measurement.value"}, {"var": "spec.max"}]},
            {"or": [
                {"==": [{"var": "material.qc_status"}, "PASSED"]},
                {"==": [{"var": "material.qc_status"}, "CONDITIONAL_PASS"]}
            ]},
            {"and": [
                {"!=": [{"var": "operator.id"}, null]},
                {">": [{"var": "operator.skill_level"}, 0]},
                {"or": [
                    {"==": [{"var": "sop.status"}, "PUBLISHED"]},
                    {"and": [
                        {"==": [{"var": "sop.status"}, "UNDER_REVIEW"]},
                        {"==": [{"var": "override.authorized"}, true]}
                    ]}
                ]}
            ]}
        ]
    });

    // ルールに渡すデータ（PASSED ケース）
    let data_pass = serde_json::json!({
        "measurement": {"value": 50.0},
        "spec": {"min": 0.0, "max": 100.0},
        "material": {"qc_status": "PASSED"},
        "operator": {"id": "OP001", "skill_level": 3},
        "sop": {"status": "PUBLISHED"},
        "override": {"authorized": false}
    });

    // ルールに渡すデータ（FAIL ケース）
    let data_fail = serde_json::json!({
        "measurement": {"value": 150.0},  // 範囲外
        "spec": {"min": 0.0, "max": 100.0},
        "material": {"qc_status": "REJECTED"},
        "operator": {"id": "OP001", "skill_level": 3},
        "sop": {"status": "PUBLISHED"},
        "override": {"authorized": false}
    });

    let mut group = c.benchmark_group("json_logic_complex");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("depth5_pass_case_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = jsonlogic_rs::apply(&complex_rule, &data_pass);
                let _ = result;
            }
        });
    });

    group.bench_function("depth5_fail_case_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = jsonlogic_rs::apply(&complex_rule, &data_fail);
                let _ = result;
            }
        });
    });

    group.finish();
}

/// シンプルなルール（単一条件）のパフォーマンス基準値を測定する。
fn bench_json_logic_simple_rule(c: &mut Criterion) {
    let simple_rule = serde_json::json!({
        "and": [
            {">=": [{"var": "value"}, 0]},
            {"<=": [{"var": "value"}, 100]}
        ]
    });

    let data = serde_json::json!({"value": 50});

    let mut group = c.benchmark_group("json_logic_simple");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("simple_range_check_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = jsonlogic_rs::apply(&simple_rule, &data);
                let _ = result;
            }
        });
    });

    group.finish();
}

/// ネスト深度別のパフォーマンス比較ベンチマーク。
fn bench_json_logic_nesting_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_logic_nesting");

    for depth in [1, 2, 3, 5].iter() {
        let rule = create_nested_and_rule(*depth);
        let data = serde_json::json!({"value": 50, "threshold": 100});

        group.bench_with_input(
            BenchmarkId::new("and_depth", depth),
            &(rule, data),
            |b, (r, d)| {
                b.iter(|| jsonlogic_rs::apply(r, d));
            },
        );
    }

    group.finish();
}

/// 指定した深さの AND ルールを生成するヘルパー関数。
fn create_nested_and_rule(depth: usize) -> serde_json::Value {
    if depth <= 1 {
        serde_json::json!({
            "<=": [{"var": "value"}, {"var": "threshold"}]
        })
    } else {
        serde_json::json!({
            "and": [
                {"<=": [{"var": "value"}, {"var": "threshold"}]},
                create_nested_and_rule(depth - 1)
            ]
        })
    }
}

criterion_group!(
    benches,
    bench_json_logic_complex_rule,
    bench_json_logic_simple_rule,
    bench_json_logic_nesting_comparison
);
criterion_main!(benches);
