# 05 ALCOA+ 検証テストケース

本章は製造業電子記録の国際規格（FDA 21 CFR Part 11・PIC/S・GMP ガイドライン）が要求する ALCOA+ 9 属性の充足を検証するテストケース TST-alcoa-001〜009 を確定する。各属性（Attributable・Legible・Contemporaneous・Original・Accurate・Complete・Consistent・Enduring・Available）に対して 1:1 でテストケースを割付ける。

---

## 1. ALCOA+ 属性とテストケース対応表

| ALCOA+ 属性 | 日本語意味 | TST-ID | 検証対象 |
|---|---|---|---|
| Attributable | 帰属可能 | TST-alcoa-001 | work_events.resource が NULL でない |
| Legible | 判読可能 | TST-alcoa-002 | PDF/A-3 が veraPDF 検証をパス |
| Contemporaneous | 同時性 | TST-alcoa-003 | sync_lag_ms が閾値以内 |
| Original | 原本性 | TST-alcoa-004 | Append-Only 制約（UPDATE/DELETE 拒否）|
| Accurate | 正確性 | TST-alcoa-005 | 範囲外数値が ERR-VAL-002 で拒否 |
| Complete | 完全性 | TST-alcoa-006 | XES 全属性が存在 |
| Consistent | 一貫性 | TST-alcoa-007 | ハッシュチェーン prev_hash 連鎖 |
| Enduring | 永続性 | TST-alcoa-008 | PDF/A-3 適合性（ISO 19005-3）|
| Available | 利用可能 | TST-alcoa-009 | 3 種エクスポート方式の動作確認 |

---

## 2. TST-alcoa-001: Attributable（帰属可能）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-001 |
| テスト観点 | すべての作業イベントに作業者 ID（resource フィールド）が記録される |
| 検証方法 | SQL: `SELECT COUNT(*) FROM work_events WHERE resource IS NULL` |
| 期待結果 | COUNT = 0（resource が NULL の行が存在しない）|
| 追加確認 | PII 匿名化後も UUID は保持されること（BAT-004 実行後に再検証）|
| 対応 FR/NFR | FR-EV-005 |
| 根拠規格 | FDA 21 CFR Part 11 §11.10(e)「audit trail」|

```sql
-- 検証クエリ
SELECT COUNT(*) AS null_resource_count
FROM work_events
WHERE resource IS NULL;
-- 期待値: 0
```

---

## 3. TST-alcoa-002: Legible（判読可能）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-002 |
| テスト観点 | 生成された PDF/A-3 ファイルが機械可読かつ人間可読な状態で保存される |
| 検証方法 | veraPDF（CLI ツール）による PDF/A-3b 適合性チェックを全レポートファイルに実行 |
| 期待結果 | veraPDF の exit code = 0（PASSED）・ValidationResult.isPassed = true |
| コマンド | `verapdf --flavour 3b reports/*.pdf` |
| 対応 FR/NFR | FR-RP-001, NFR-DOC-001 |
| 根拠規格 | PIC/S PE 009-16 §10「Audit Trails」|

```bash
# 検証スクリプト（CI で実行）
#!/bin/bash
set -euo pipefail
for pdf in /var/reports/*.pdf; do
  result=$(verapdf --flavour 3b --format json "$pdf" 2>/dev/null)
  passed=$(echo "$result" | jq '.jobs[0].validationResult.isPassed')
  if [ "$passed" != "true" ]; then
    echo "FAIL: $pdf は PDF/A-3b 非準拠"
    exit 1
  fi
done
echo "PASS: 全レポートが PDF/A-3b 準拠"
```

---

## 4. TST-alcoa-003: Contemporaneous（同時性）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-003 |
| テスト観点 | デバイス記録時刻（timestamp_client）とサーバー受信時刻（timestamp_server）の差が許容範囲内 |
| 検証方法 | SQL: `timestamp_server - timestamp_client` の分布を確認 |
| 期待結果 | P95 sync_lag_ms ≤ CFG-007（2000 ms）/ P99 ≤ 5000 ms |
| 追加確認 | EMERGENCY_MODE 中のイベントは同時性の例外として別途集計する |
| 対応 FR/NFR | NFR-SYNC-001 |
| 根拠規格 | PIC/S PE 009-16 §4.8「audit trail」|

```sql
-- 検証クエリ（正常モード時）
SELECT
    PERCENTILE_CONT(0.95) WITHIN GROUP (
        ORDER BY EXTRACT(EPOCH FROM (timestamp_server - timestamp_client)) * 1000
    ) AS p95_lag_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (
        ORDER BY EXTRACT(EPOCH FROM (timestamp_server - timestamp_client)) * 1000
    ) AS p99_lag_ms
FROM work_events
WHERE timestamp_server > NOW() - INTERVAL '1 day'
  AND was_offline = FALSE;
-- 期待値: p95_lag_ms ≤ 2000
```

---

## 5. TST-alcoa-004: Original（原本性）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-004 |
| テスト観点 | work_events テーブルへの UPDATE・DELETE が PostgreSQL トリガにより拒否される |
| 検証方法 | PostgreSQL セッションから直接 UPDATE/DELETE を試行 |
| 期待結果 | `UPDATE work_events SET ...` → `ERROR: cannot update work_events (append-only)` |
| | `DELETE FROM work_events WHERE ...` → `ERROR: cannot delete from work_events (append-only)` |
| 追加確認 | hash_chain_blocks・hash_chain_verification_results も同様に Append-Only であること |
| 対応 FR/NFR | FR-EV-002, NFR-SEC-003 |
| 根拠規格 | FDA 21 CFR Part 11 §11.10(e)「protect records」|

```sql
-- 検証クエリ（エラーが発生することを確認）
UPDATE work_events SET activity = 'tampered' WHERE id = (SELECT id FROM work_events LIMIT 1);
-- 期待: ERROR: trigger deny_update_work_events raised exception
```

---

## 6. TST-alcoa-005: Accurate（正確性）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-005 |
| テスト観点 | ステップ定義の数値範囲外の入力値が ERR-VAL-002 で拒否される |
| 検証方法 | step.range = {min:0, max:100} に value=150 でステップ完了 API を呼び出す |
| 期待結果 | HTTP 400, `{"error": "ERR-VAL-002", "field": "input.value", "constraint": "max:100"}` |
| 追加確認 | 境界値テスト: value=0（有効）・value=100（有効）・value=-1（無効）・value=100.001（無効）|
| 対応 FR/NFR | FR-NV-002, NFR-VAL-001 |
| 根拠規格 | GMP ガイドライン §5「データ整合性」|

---

## 7. TST-alcoa-006: Complete（完全性）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-006 |
| テスト観点 | XES エクスポートで全必須属性（case:id/activity/time:timestamp/org:resource）が全イベントに存在する |
| 検証方法 | `GET /api/v1/exports/xes?exec_id=<uuid>` でダウンロードした XES ファイルを解析 |
| 期待結果 | XES の全 `<event>` 要素に `case:id`・`concept:name`（activity）・`time:timestamp`・`org:resource` の 4 属性が存在する |
| 追加確認 | NULL 値を含む属性が 1 件も存在しないこと |
| 対応 FR/NFR | FR-EV-004 |
| 根拠規格 | IEEE XES Standard（XES 2.0）|

```python
# 検証スクリプト（Python）
import xml.etree.ElementTree as ET

REQUIRED_XES_KEYS = {'case:id', 'concept:name', 'time:timestamp', 'org:resource'}

def verify_xes_complete(xes_path: str) -> bool:
    tree = ET.parse(xes_path)
    for event in tree.findall('.//{http://www.xes-standard.org/}event'):
        keys = {attr.get('key') for attr in event}
        if not REQUIRED_XES_KEYS.issubset(keys):
            missing = REQUIRED_XES_KEYS - keys
            raise AssertionError(f'Missing XES attributes: {missing}')
    return True
```

---

## 8. TST-alcoa-007: Consistent（一貫性）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-007 |
| テスト観点 | hash_chain_blocks の prev_hash 連鎖が全ブロックで一貫している |
| 検証方法 | `HashChainService::verify_chain()` を実行し PASSED を確認 |
| 期待結果 | VerificationResult.status = 'PASSED' / broken_at = NULL |
| 追加確認 | 1 件のブロックを手動で改ざんした場合に broken_at が検出されること（破壊的テスト）|
| 対応 FR/NFR | FR-EV-001/003 |
| 根拠規格 | FDA 21 CFR Part 11 §11.10(e)「detect record tampering」|

```rust
// 破壊的テスト（テスト環境のみ）
#[sqlx::test]
async fn test_tst_alcoa_007_tamper_detection(pool: PgPool) {
    // 正常なハッシュチェーンを構築
    setup_valid_hash_chain(&pool, 10).await;

    // 5 番目のブロックのコンテンツを改ざん
    sqlx::query!("UPDATE hash_chain_blocks SET content_hash = '\\x' || repeat('ff', 32)::bytea WHERE block_id = (SELECT block_id FROM hash_chain_blocks ORDER BY block_id LIMIT 1 OFFSET 4)")
        .execute(&pool).await.unwrap();

    let service = HashChainService::new(pool);
    let result = service.verify_chain().await.unwrap();

    assert_eq!(result.status, VerificationStatus::Failed);
    assert!(result.broken_at.is_some());
}
```

---

## 9. TST-alcoa-008: Enduring（永続性）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-008 |
| テスト観点 | 帳票 PDF が PDF/A-3 規格（ISO 19005-3）に準拠し長期保存可能 |
| 検証方法 | veraPDF による ISO 19005-3 Part 3 (PDF/A-3b) 適合性検証 |
| 期待結果 | veraPDF ValidationResult: isPassed=true, flavorId='3b' |
| 追加確認 | 埋め込みフォント（IPA フォント）の存在確認・外部リソース参照がゼロ |
| 対応 FR/NFR | NFR-DOC-001（保存期間 10 年以上）|
| 根拠規格 | PIC/S PE 009-16「retained in a durable medium」|

---

## 10. TST-alcoa-009: Available（利用可能）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-alcoa-009 |
| テスト観点 | すべての記録が 3 種のエクスポート方式（PDF/XES/CSV）で取得可能 |
| 検証方法 | 同一 execution_id に対して 3 つのエクスポートエンドポイントを呼び出す |
| 期待結果 | PDF: HTTP 200（Content-Type: application/pdf）/ XES: HTTP 200（text/xml）/ CSV: HTTP 200（text/csv）|
| 追加確認 | 各ファイルに同一 event_id が含まれること（内容の一貫性）|
| 対応 FR/NFR | FR-EV-004, NFR-DOC-002 |
| 根拠規格 | FDA 21 CFR Part 11 §11.10(d)「protect records」|

```typescript
// 検証テストスケルトン
it('TST-alcoa-009: 3 種エクスポートが全て利用可能', async () => {
  const execId = await createCompletedExecution();

  const pdfRes = await api.get(`/api/v1/exports/pdf?exec_id=${execId}`);
  expect(pdfRes.status).toBe(200);
  expect(pdfRes.headers['content-type']).toMatch(/application\/pdf/);

  const xesRes = await api.get(`/api/v1/exports/xes?exec_id=${execId}`);
  expect(xesRes.status).toBe(200);
  expect(xesRes.headers['content-type']).toMatch(/text\/xml/);

  const csvRes = await api.get(`/api/v1/exports/csv?exec_id=${execId}`);
  expect(csvRes.status).toBe(200);
  expect(csvRes.headers['content-type']).toMatch(/text\/csv/);
});
```

---

**本節で確定した方針**
- **TST-alcoa-001〜009 の 9 件は ALCOA+ の 9 属性に 1:1 で対応し、各属性の充足を具体的な SQL・CLI コマンド・API 呼び出しで検証することを確定した。**
- **TST-alcoa-004（Original）は Append-Only トリガが PostgreSQL セッションからの直接 UPDATE/DELETE を拒否することを確認する。アプリケーション層のバイパスを防ぐ唯一の防御線であり、必須の検証とすることを確定した。**
- **TST-alcoa-007（Consistent）は破壊的テスト（ブロック改ざん後の検証）を含む。この破壊的テストはテスト環境のみで実行し、`#[sqlx::test]` によるトランザクションロールバックで本番データへの影響を完全に排除することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../90_業界分析/21_電子記録の法規制とALCOA+.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
