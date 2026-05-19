// TST-perf-003: Idempotency キャッシュヒット率測定ベンチマーク
//
// 1000 件のリクエストのうち 10% が重複（同一 Idempotency-Key）の場合の
// キャッシュヒット率を測定する。
//
// 権威ドキュメント: docs/05_詳細設計/08_テストケース詳細設計/06_セキュリティ・性能テストケース.md TST-perf-003
// 権威ドキュメント: src/backend/CLAUDE.md「Idempotent API」

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::collections::HashMap;

/// TST-perf-003: Idempotency キャッシュヒット率測定（1000 req, 10% 重複）。
/// in-memory キャッシュのシミュレーションで測定する（moka キャッシュ相当）。
fn bench_idempotency_cache_hit_rate(c: &mut Criterion) {
    // 1000 件のリクエストのうち 100 件（10%）が重複する Idempotency-Key を準備する
    let total_requests = 1000_usize;
    let duplicate_rate = 0.1_f64;
    let unique_keys_count = (total_requests as f64 * (1.0 - duplicate_rate)) as usize; // 900 件

    // ユニークな Idempotency-Key を 900 件生成する
    let unique_keys: Vec<uuid::Uuid> = (0..unique_keys_count)
        .map(|_| uuid::Uuid::now_v7())
        .collect();

    // 1000 件のリクエスト（900 件ユニーク + 100 件重複）を構築する
    let mut requests: Vec<uuid::Uuid> = unique_keys.clone();
    // 重複リクエスト: 最初の 100 件のキーを再利用する
    for i in 0..100 {
        requests.push(unique_keys[i % unique_keys_count]);
    }

    let mut group = c.benchmark_group("idempotency_cache");
    group.throughput(Throughput::Elements(total_requests as u64));

    // インメモリ HashMap を使ったキャッシュシミュレーション（moka キャッシュの簡略版）
    group.bench_function("1000_requests_10pct_duplicate", |b| {
        b.iter(|| {
            let mut cache: HashMap<uuid::Uuid, String> = HashMap::with_capacity(unique_keys_count);
            let mut cache_hits = 0_usize;
            let mut cache_misses = 0_usize;

            for &key in &requests {
                if let Some(cached_response) = cache.get(&key) {
                    // キャッシュヒット: レスポンスを返す（DB 書き込みなし）
                    cache_hits += 1;
                    let _ = cached_response; // 使用済みとしてマーク
                } else {
                    // キャッシュミス: 処理してキャッシュに保存する
                    cache_misses += 1;
                    let response = format!("{{\"event_id\": \"{}\", \"status\": 201}}", key);
                    cache.insert(key, response);
                }
            }

            // ヒット率の計算（最適化除去防止）
            (cache_hits, cache_misses)
        });
    });

    group.finish();
}

/// Idempotency-Key の生成コスト（UUID v7 生成）を測定する補助ベンチマーク。
fn bench_idempotency_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("idempotency_key_gen");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("uuid_v7_generation_1000", |b| {
        b.iter(|| {
            let keys: Vec<uuid::Uuid> = (0..1000).map(|_| uuid::Uuid::now_v7()).collect();
            keys
        });
    });

    group.finish();
}

/// Idempotency-Key の TTL チェック（期限切れ判定）のパフォーマンスを測定する。
fn bench_idempotency_ttl_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("idempotency_ttl");
    group.throughput(Throughput::Elements(1000));

    let now = std::time::Instant::now();
    let expires: Vec<std::time::Instant> = (0..1000)
        .map(|i| {
            if i % 2 == 0 {
                // 有効なキャッシュ（24 時間後に期限切れ）
                now + std::time::Duration::from_secs(86400)
            } else {
                // 期限切れのキャッシュ
                now - std::time::Duration::from_secs(1)
            }
        })
        .collect();

    group.bench_function("ttl_check_1000_mixed", |b| {
        b.iter(|| {
            let current = std::time::Instant::now();
            expires.iter().filter(|&&exp| exp > current).count()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_idempotency_cache_hit_rate,
    bench_idempotency_key_generation,
    bench_idempotency_ttl_check
);
criterion_main!(benches);
