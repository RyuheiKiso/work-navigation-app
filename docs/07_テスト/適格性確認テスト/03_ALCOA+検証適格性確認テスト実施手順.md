# 03 ALCOA+ 検証適格性確認テスト実施手順

本章は TST-alcoa-001〜009 の各属性ごとの検証コマンド・SQL・期待値・根拠規格を確定する。
ALCOA+ 9 属性の全 Pass が規制対応品質の必要条件である。

---

## 1. 適用スコープと根拠規格

| テスト識別子 | 対応 ALCOA+ 属性 | 根拠規格 |
|---|---|---|
| TST-alcoa-001 | Attributable（帰属可能）| 21 CFR Part 11 §11.10(e) |
| TST-alcoa-002 | Legible（読取可能）| PIC/S PE 009-16 §10 |
| TST-alcoa-003 | Contemporaneous（同時記録）| NFR-SYNC-001 |
| TST-alcoa-004 | Original（元データ）| NFR-SEC-003 |
| TST-alcoa-005 | Accurate（正確）| NFR-VAL-001 |
| TST-alcoa-006 | Complete（完全）| IEEE XES 2.0 |
| TST-alcoa-007 | Consistent（一貫）| NFR-SEC-040 |
| TST-alcoa-008 | Enduring（耐久）| ISO 19005-3 |
| TST-alcoa-009 | Available（利用可能）| FR-AU-004/005 |

権威ソース: `05_詳細設計/08_テストケース詳細設計/05_ALCOA+検証テストケース.md`

**本節で確定した方針**
- **TST-alcoa-001〜009 の 9 件全 Pass を ALCOA+ 検証テストの合否基準として確定する。1 件でも Fail の場合は当該テストをブロッカーとして扱い、リリースを保留する。**
- **各根拠規格への適合は本テストの結果でもって「21 CFR Part 11 準拠する」「PIC/S PE 009-16 対応する」「ISO 19005-3 準拠する」の根拠証跡とする。**

---

## 2. TST-alcoa-001: Attributable（帰属可能）

**根拠規格**: 21 CFR Part 11 §11.10(e)

```sql
-- 全 work_events に resource（操作者 ID）が記録されているか検証
SELECT COUNT(*) FROM work_events WHERE resource IS NULL;
-- 期待値: 0 件
```

```bash
psql -U app -d work_nav_prod -c \
  "SELECT COUNT(*) FROM work_events WHERE resource IS NULL;"
```

Pass 判定: 返却値 = 0

---

## 3. TST-alcoa-002: Legible（読取可能）

**根拠規格**: PIC/S PE 009-16 §10

```bash
# veraPDF で全 PDF 報告書の PDF/A-3b 適合を検証
veraPDF --flavour 3b reports/*.pdf
```

Pass 判定: exit code = 0（全ファイル適合）

---

## 4. TST-alcoa-003: Contemporaneous（同時記録）

**根拠規格**: NFR-SYNC-001

```bash
# k6 で sync_lag_ms を測定
k6 run scripts/k6/sync-lag.js \
  --out json=results/alcoa-003.json

# P95 抽出
jq '.metrics.sync_lag_ms.values["p(95)"]' results/alcoa-003.json
```

Pass 判定: `timestamp_server - timestamp_client` P95 ≤ 2000ms

---

## 5. TST-alcoa-004: Original（元データ）

**根拠規格**: NFR-SEC-003

```bash
# PostgreSQL トリガによる UPDATE 拒否確認
psql -U app -d work_nav_prod -c \
  "UPDATE work_events SET step_id = 'TAMPERED' WHERE id = '00000000-0000-0000-0000-000000000001';"
```

Pass 判定: `ERROR: immutable_record: UPDATE is not allowed on work_events` がエラー出力されること

---

## 6. TST-alcoa-005: Accurate（正確）

**根拠規格**: NFR-VAL-001

```bash
# 境界値テスト: USL + 1 でバリデーションエラーを確認
curl -X POST http://localhost:8080/api/v1/measurements \
  -H "Authorization: Bearer $OPERATOR_JWT" \
  -H "Content-Type: application/json" \
  -d '{"value": 100.1, "usl": 100.0, "lsl": 0.0}' \
  -w "\n%{http_code}"
# 期待値: HTTP 400 + {"error_code": "ERR-VAL-002"}

# 境界値テスト: LSL - 1 でバリデーションエラーを確認
curl -X POST http://localhost:8080/api/v1/measurements \
  -H "Authorization: Bearer $OPERATOR_JWT" \
  -H "Content-Type: application/json" \
  -d '{"value": -0.1, "usl": 100.0, "lsl": 0.0}' \
  -w "\n%{http_code}"
# 期待値: HTTP 400 + {"error_code": "ERR-VAL-002"}
```

Pass 判定: 両リクエストとも HTTP 400 かつ `error_code: ERR-VAL-002` を返すこと

---

## 7. TST-alcoa-006: Complete（完全）

**根拠規格**: IEEE XES 2.0

```bash
# XES エクスポートし必須属性の NOT NULL を検証
curl -s -H "Authorization: Bearer $QUALITY_ADMIN_JWT" \
  "http://localhost:8080/api/v1/reports/audit-log?format=xes" \
  -o results/audit-log.xes

# Python スクリプトで必須属性チェック
python3 scripts/validate-xes.py results/audit-log.xes
# → concept:name, time:timestamp, org:resource の全 <event> NOT NULL を確認
```

```python
# scripts/validate-xes.py の検証観点（抜粋）
# <event> 要素ごとに concept:name / time:timestamp / org:resource が存在し値が空でないこと
```

Pass 判定: validate-xes.py exit code = 0（全 `<event>` の必須属性 NOT NULL 確認）

---

## 8. TST-alcoa-007: Consistent（一貫）

**根拠規格**: NFR-SEC-040

```bash
# ハッシュチェーン検証 PASS
cd backend && cargo test verify_chain -- --nocapture

# 破壊的テスト: hash 改ざん → broken_at 検出
cd backend && cargo test hash_chain_broken_detection -- --nocapture
# → broken_at フィールドにレコード ID が記録されることを確認
```

Pass 判定: `verify_chain()` PASS かつ破壊的テストで `broken_at` が正しく検出されること

---

## 9. TST-alcoa-008: Enduring（耐久）

**根拠規格**: ISO 19005-3（PDF/A-3b）

```bash
# PDF/A-3b 適合確認（TST-alcoa-002 との複合確認）
veraPDF --flavour 3b reports/*.pdf

# Noto フォント埋込確認（poppler-utils）
pdffonts reports/*.pdf | grep "Noto"
# → Noto フォントが全 PDF に埋込済みであることを確認
```

Pass 判定: veraPDF exit code = 0 かつ `pdffonts` で Noto フォント埋込が確認できること

---

## 10. TST-alcoa-009: Available（利用可能）

**根拠規格**: FR-AU-004 / FR-AU-005

```bash
# quality_admin: 全エクスポート形式で HTTP 200
curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $QUALITY_ADMIN_JWT" \
  "http://localhost:8080/api/v1/reports/audit-log?format=xes"
# → 200

curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $QUALITY_ADMIN_JWT" \
  "http://localhost:8080/api/v1/reports/audit-log?format=pdf"
# → 200

curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $QUALITY_ADMIN_JWT" \
  "http://localhost:8080/api/v1/reports/audit-log?format=csv"
# → 200

# quality_admin 以外（operator）: 403 確認
curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $OPERATOR_JWT" \
  "http://localhost:8080/api/v1/reports/audit-log?format=xes"
# → 403
```

Pass 判定: quality_admin で PDF/XES/CSV 全エクスポート HTTP 200 かつ operator で HTTP 403

**本節で確定した方針**
- **TST-alcoa-001〜009 の実施コマンド・SQL・期待値は本章を実施根拠とし、実行結果は `07_適格性確認テスト実施結果テンプレート.md` の ALCOA+ 列に記録する。**
- **TST-alcoa-004（Original）の PostgreSQL トリガ検証は、本番 DB 相当の TENV-006 環境でのみ実施する。**

---

## 参照業界分析

- `../../../90_業界分析/22_規制別トレーサビリティ要件詳論.md`（必須）
- `../../../90_業界分析/06_品質管理とトレーサビリティ.md`（必須）

---

## 版数履歴

| バージョン | 日付 | 著者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | RyuheiKiso | 初版 |
