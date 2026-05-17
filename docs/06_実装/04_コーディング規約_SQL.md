# 04 コーディング規約_SQL

## 1. PostgreSQL 17 採用

本プロジェクトのデータベースは **PostgreSQL 17** を採用する。拡張機能は `pgcrypto` のみを許容する。

### pgcrypto 採用根拠

`pgcrypto` は PostgreSQL 公式の暗号化拡張であり、以下の用途に使用する。

- `gen_random_uuid()`: UUID v4 の生成（`DEFAULT gen_random_uuid()`）
- `crypt()` / `digest()`: SHA-256 ハッシュチェーン計算のサポート
- `gen_random_bytes()`: ランダムバイト生成

### 追加拡張の申請手順

`pgcrypto` 以外の拡張を追加する場合は以下の手順を踏む。

1. `docs/01_管理/変更管理/ADR-IMPL-NNN.md` に根拠・代替案却下理由・セキュリティ評価を記載する。
2. PostgreSQL 公式またはコミュニティの信頼できる拡張であることを確認する。
3. 本番環境への適用は `08_移行` ドキュメントを改訂してから実施する。

拡張の追加は DDL として `migrations/` に記録し、`down.sql` での削除手順をセットで記述する。

**本節で確定した方針**
- **PostgreSQL 17 を採用し、`pgcrypto` のみを拡張として使用する。**
- **追加拡張は ADR-IMPL-NNN を作成し、事後追加を禁止する。**
- **拡張の有効化 SQL をマイグレーションファイルに記録し、再現可能なセットアップを保証する。**

---

## 2. 命名規約

### テーブル・ビュー・インデックス

| 要素 | 規則 | 例 |
|---|---|---|
| テーブル | `snake_case` 単数形 | `work_event`, `work_order`, `operator` |
| ビュー | `vw_<name>` | `vw_active_work_order`, `vw_audit_trail` |
| マテリアライズドビュー | `mv_<name>` | `mv_daily_production_summary` |
| インデックス | `idx_<table>_<columns>` | `idx_work_event_case_id`, `idx_operator_employee_id` |
| ユニーク制約 | `uq_<table>_<columns>` | `uq_outbox_idempotency_key` |
| 外部キー | `fk_<child>_<parent>` | `fk_work_event_work_order` |
| チェック制約 | `chk_<table>_<condition>` | `chk_work_event_activity_valid` |
| シーケンス | `<table>_<col>_seq` | `operator_code_seq` |
| プライマリキー | `pk_<table>` | `pk_work_event` |

### カラム命名規則

```sql
-- タイムスタンプ系: _at サフィックス（WITH TIME ZONE 必須）
created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
deleted_at        TIMESTAMPTZ,                       -- 論理削除
server_received_at TIMESTAMPTZ NOT NULL,             -- サーバー付与
client_recorded_at TIMESTAMPTZ NOT NULL,             -- クライアント申告

-- フラグ系: is_ プレフィックス
is_active         BOOLEAN NOT NULL DEFAULT TRUE,

-- 論理削除: deleted_at で管理（boolean フラグ禁止）
deleted_at        TIMESTAMPTZ,                       -- NULL = 有効

-- 外部キー: 参照先テーブル名 + _id
work_order_id     UUID NOT NULL REFERENCES work_order(id),
operator_id       UUID NOT NULL REFERENCES operator(id),
```

**本節で確定した方針**
- **テーブル名は `snake_case` 単数形・インデックスは `idx_<table>_<columns>` 形式で統一する。**
- **論理削除は `deleted_at TIMESTAMPTZ` で管理し、`is_deleted BOOLEAN` フラグを禁止する。**
- **外部キー制約名は `fk_<child>_<parent>` 形式で必ず命名し、無名制約を禁止する。**

---

## 3. DDL 規約

### NOT NULL と DEFAULT

```sql
-- NOT NULL を明示する（暗黙の NULL 許容を禁止する）
CREATE TABLE work_event (
    id               UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id          UUID NOT NULL,
    activity         VARCHAR(50) NOT NULL,
    -- デフォルト値は NULL の代替として使用しない
    -- （NULL は「不明」、DEFAULT は「既知の初期値」として区別する）
    retry_count      INT NOT NULL DEFAULT 0,
    -- created_at は必ず NOT NULL + DEFAULT で自動設定する
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### ENUM は REFERENCES テーブル化

PostgreSQL の `ENUM` 型は追加のみ可能で削除ができないため、使用禁止とする。代わりに参照テーブルとして定義する。

```sql
-- 禁止: ENUM 型の使用
-- CREATE TYPE activity_type AS ENUM ('start', 'complete', 'skip');

-- 推奨: 参照テーブル化（変更・削除が柔軟に行える）
CREATE TABLE event_activity (
    code         VARCHAR(50) NOT NULL PRIMARY KEY,
    name_json    JSONB NOT NULL CHECK (jsonb_typeof(name_json) = 'object'),
    is_active    BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order   INT NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO event_activity (code, name_json) VALUES
    ('start',    '{"ja": "開始", "en": "Start", "zh": "开始"}'),
    ('complete', '{"ja": "完了", "en": "Complete", "zh": "完成"}'),
    ('skip',     '{"ja": "スキップ", "en": "Skip", "zh": "跳过"}'),
    ('rollback', '{"ja": "差し戻し", "en": "Rollback", "zh": "回退"}'),
    ('suspend',  '{"ja": "中断", "en": "Suspend", "zh": "暂停"}');

-- work_event テーブルの activity は参照テーブルを参照する
ALTER TABLE work_event
    ADD CONSTRAINT fk_work_event_event_activity
    FOREIGN KEY (activity) REFERENCES event_activity(code);
```

### TIMESTAMP WITH TIME ZONE 必須

すべての日時カラムは `TIMESTAMPTZ`（`TIMESTAMP WITH TIME ZONE`）を使用する。`TIMESTAMP`（タイムゾーンなし）を禁止する。

```sql
-- 禁止: タイムゾーンなし
recorded_at  TIMESTAMP NOT NULL,

-- 推奨: タイムゾーンあり
recorded_at  TIMESTAMPTZ NOT NULL,
```

**本節で確定した方針**
- **全カラムに `NOT NULL` を明示し、暗黙の NULL 許容を禁止する（`NULL` が必要な場合のみ明示的に許容）。**
- **PostgreSQL `ENUM` 型を禁止し、参照テーブルと `FOREIGN KEY` で制約する。**
- **全日時カラムを `TIMESTAMPTZ` とし、`TIMESTAMP`（タイムゾーンなし）を禁止する。**

---

## 4. 制約

### 制約の使い分け

```sql
-- CHECK 制約: カラム値の範囲・形式を保証する
ALTER TABLE step_measurement
    ADD CONSTRAINT chk_step_measurement_value_range
    CHECK (measured_value >= 0.0 AND measured_value <= 9999.9);

-- UNIQUE 制約: 重複防止
ALTER TABLE outbox
    ADD CONSTRAINT uq_outbox_idempotency_key
    UNIQUE (idempotency_key);

-- EXCLUSION 制約: 期間の重複防止（作業指示の同時実行防止）
-- PostgreSQL 17 の btree_gist 拡張が必要（使用する場合は ADR 必須）
-- ALTER TABLE work_session
--     ADD CONSTRAINT excl_work_session_no_overlap
--     EXCLUDE USING gist (operator_id WITH =, tstzrange(started_at, ended_at) WITH &&);

-- FOREIGN KEY ON DELETE の使い分け
-- RESTRICT: 子レコードが存在する場合に親の削除を禁止（既定動作）
ALTER TABLE work_event
    ADD CONSTRAINT fk_work_event_work_order
    FOREIGN KEY (work_order_id) REFERENCES work_order(id)
    ON DELETE RESTRICT;

-- CASCADE: 親の削除時に子も削除（使用は最小限に限定・Append-only に反するため work_events には使用禁止）
-- SET NULL: 親の削除時に外部キーを NULL に設定（参照が任意の場合）
```

### NULL 許容の基準

| 状況 | NULL 許容 | 理由 |
|---|---|---|
| 必須項目（常に値を持つ） | NOT NULL | 不明な値は受け付けない |
| 任意項目（未入力の可能性） | NULL 許容 | 「値がない」ことを表す |
| 論理削除タイムスタンプ | NULL 許容 | NULL = 有効、非 NULL = 削除済み |
| サーバー受信時刻（未同期） | NULL 許容 | 同期前は NULL、同期後に埋める |

**本節で確定した方針**
- **CHECK 制約を全数値カラム（測定値・スコア等）に設定し、範囲外値の挿入を DB レベルで防止する。**
- **FOREIGN KEY は全外部キーカラムに明示し、無名制約・暗黙の参照整合性を禁止する。**
- **EXCLUSION 制約（btree_gist）は ADR 確認なしに使用しない。**

---

## 5. JSONB 規約

### 多言語テキストの形式

```sql
-- 多言語テキストは必ず {"ja":"","en":"","zh":""} の 3 キー形式とする
name_json    JSONB NOT NULL CHECK (
    jsonb_typeof(name_json) = 'object' AND
    name_json ? 'ja' AND
    name_json ? 'en' AND
    name_json ? 'zh'
),
```

### JSONB の使用基準

JSONB は以下の場合にのみ使用する。スキーマレスデータを乱用しない。

| 用途 | 使用 | 理由 |
|---|---|---|
| 多言語テキスト `{"ja":"","en":"","zh":""}` | 対応する | 3 言語の構造が固定されているため |
| Step ごとに異なる測定値スキーマ | 対応する | Step 種別ごとに測定項目が異なるため |
| イベントの拡張属性 | 対応する | Step アドオンの柔軟な追加に対応するため |
| 本来は正規化できるデータ | 対象外と判断する | JSONB は正規化の回避手段ではない |

### GIN インデックス対象フィールド

JSONB カラムに対する検索性能を確保するため、以下のフィールドに GIN インデックスを作成する。

```sql
-- 多言語テキスト検索用 GIN インデックス
CREATE INDEX idx_sop_step_name_gin ON sop_step USING GIN (name_json);
CREATE INDEX idx_equipment_name_gin ON equipment USING GIN (name_json);

-- Step 拡張属性の検索用 GIN インデックス
CREATE INDEX idx_work_event_attributes_gin ON work_event USING GIN (attributes);
```

**本節で確定した方針**
- **多言語テキストは `{"ja":"","en":"","zh":""}` の 3 キー形式を CHECK 制約で保証する。**
- **JSONB カラムに `CHECK (jsonb_typeof(col) = 'object')` と必須キーチェックを必ず設定する。**
- **全文検索・キー検索が必要な JSONB カラムに GIN インデックスを設定する。**

---

## 6. インデックス戦略

### インデックスの種類と使用場面

| 種類 | 使用場面 | 例 |
|---|---|---|
| B-tree | 等値・範囲検索・ソート | `idx_work_event_case_id` |
| GIN | JSONB・配列・全文検索 | `idx_sop_step_name_gin` |
| BRIN | 時系列データ（挿入順） | `idx_work_event_server_received_at_brin` |
| Partial index | 特定条件のみを対象 | 有効レコードのみ |
| 複合インデックス | 複数カラムの組み合わせ検索 | `idx_work_event_case_id_activity` |

### インデックス設計例

```sql
-- 作業イベントの主要検索パターンに対応したインデックス
-- case_id での検索（最も頻繁なクエリ）
CREATE INDEX idx_work_event_case_id
    ON work_event (case_id);

-- 時系列検索（監査証跡・進捗確認）
CREATE INDEX idx_work_event_server_received_at
    ON work_event (server_received_at DESC);

-- case_id + 時系列の複合検索（ハッシュチェーン検証）
CREATE INDEX idx_work_event_case_id_server_received_at
    ON work_event (case_id, server_received_at ASC);

-- 有効な作業指示のみを対象とした Partial index
CREATE INDEX idx_work_order_active
    ON work_order (work_order_number, scheduled_start_at)
    WHERE deleted_at IS NULL;

-- INCLUDE 句で Index-Only Scan を可能にする
CREATE INDEX idx_outbox_created_at_include_event_id
    ON outbox (created_at ASC)
    INCLUDE (event_id, idempotency_key);
```

### CONCURRENTLY 作成必須

本番環境でのインデックス作成はテーブルロックを伴わない `CONCURRENTLY` を使用する。

```sql
-- 本番環境でのインデックス追加は必ず CONCURRENTLY を使用する
CREATE INDEX CONCURRENTLY idx_work_event_operator_id
    ON work_event (operator_id);
```

**本節で確定した方針**
- **時系列データ（`server_received_at`）には BRIN インデックスを検討し、B-tree と比較して採用する。**
- **本番環境でのインデックス作成は `CONCURRENTLY` を必須とし、テーブルロックを禁止する。**
- **複合インデックスのカラム順序は高選択度カラムを先頭に配置する。**

---

## 7. パーティション戦略

### event_store の月次 RANGE パーティション

作業イベントは月次 RANGE パーティションで管理し、古いデータのアーカイブを効率化する。

```sql
-- 親テーブル（パーティション宣言）
CREATE TABLE work_event (
    id                 UUID NOT NULL DEFAULT gen_random_uuid(),
    case_id            UUID NOT NULL,
    activity           VARCHAR(50) NOT NULL,
    client_recorded_at TIMESTAMPTZ NOT NULL,
    server_received_at TIMESTAMPTZ NOT NULL,
    hash               VARCHAR(64) NOT NULL,
    attributes         JSONB,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (server_received_at);

-- 月次パーティション（マイグレーションで定期作成する）
CREATE TABLE work_event_2026_05
    PARTITION OF work_event
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

CREATE TABLE work_event_2026_06
    PARTITION OF work_event
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');
```

### 3 層アーカイブ戦略

| 層 | 保存先 | 保存期間 | アクセス頻度 |
|---|---|---|---|
| Hot（直近 3 ヶ月） | PostgreSQL パーティション | 随時 | 高（通常業務） |
| Warm（3 ヶ月〜1 年） | PostgreSQL パーティション（別テーブルスペース） | 3 ヶ月ごとに移行 | 低（監査・集計） |
| Cold（1 年以上） | Parquet 等への外部エクスポート | 恒久保存 | 最低（規制対応時） |

```sql
-- アーカイブ時のパーティション DETACH（ロックなし）
ALTER TABLE work_event
    DETACH PARTITION work_event_2025_01 CONCURRENTLY;
```

**本節で確定した方針**
- **`work_event` テーブルを `server_received_at` で月次 RANGE パーティションに分割する。**
- **パーティションの DETACH は `CONCURRENTLY` を使用し、アーカイブ時のサービス停止を防止する。**
- **3 層アーカイブ（Hot/Warm/Cold）でデータライフサイクルを管理し、規制対応期間を満たす。**

---

## 8. マイグレーション

### sqlx migrate の使用

```bash
# 新規マイグレーションの作成（up.sql + down.sql のペアが自動生成される）
sqlx migrate add --reversible create_work_event_table

# 適用
sqlx migrate run

# ロールバック
sqlx migrate revert
```

### TypeORM Migration の up/down 必須

handy（SQLite）側の TypeORM マイグレーションも同様に `up`/`down` をセットで記述する（`03_コーディング規約_TypeScript.md §11` 参照）。

### マイグレーション記述規則

```sql
-- migrations/20260517_001_create_work_event.up.sql

-- 冪等性を持たせるために IF NOT EXISTS を使用する
CREATE TABLE IF NOT EXISTS work_event (
    id                 UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id            UUID NOT NULL,
    activity           VARCHAR(50) NOT NULL,
    client_recorded_at TIMESTAMPTZ NOT NULL,
    server_received_at TIMESTAMPTZ NOT NULL,
    hash               VARCHAR(64) NOT NULL,
    attributes         JSONB,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (server_received_at);

-- インデックスも CONCURRENTLY で作成する
-- ただし、マイグレーション内の CONCURRENTLY はトランザクション外で実行が必要
-- （sqlx では --no-transaction フラグを使用する）
```

```sql
-- migrations/20260517_001_create_work_event.down.sql
DROP TABLE IF EXISTS work_event;
```

### ロールバックテスト必須

統合テストで各マイグレーションの `down.sql` を実行し、ロールバックが成功することを確認する。

```bash
# CI でのマイグレーション検証
sqlx migrate run
# テスト実行
cargo nextest run --test migration_tests
# ロールバック検証
sqlx migrate revert
```

**本節で確定した方針**
- **`sqlx migrate add --reversible` で `up.sql`/`down.sql` ペアを必ず作成する。**
- **本番でのインデックス作成は `CONCURRENTLY` を使用し、`--no-transaction` フラグで実行する。**
- **統合テストでロールバック（`down.sql`）の実行可能性を自動検証する。**

---

## 9. ロール権限分離

### 3 ロールの定義

```sql
-- ロールの作成
CREATE ROLE app_write NOLOGIN;
CREATE ROLE app_event_insert NOLOGIN;
CREATE ROLE app_read NOLOGIN;

-- app_write: マスタテーブルの CRUD（SELECT/INSERT/UPDATE のみ）
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO app_write;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO app_write;
-- work_events へのアクセスは削除する（マスタ担当ロールは作業ログにアクセス不要）
REVOKE ALL ON work_event FROM app_write;

-- app_event_insert: 作業ログテーブルへの INSERT のみ
GRANT INSERT ON work_event TO app_event_insert;
GRANT INSERT ON outbox TO app_event_insert;
GRANT SELECT ON work_event TO app_event_insert;  -- 直前ハッシュの取得に必要
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO app_event_insert;

-- app_read: 全テーブルの SELECT のみ（監査・ダッシュボード用）
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_read;
```

### 接続プール分離

```rust
// AppState に用途別接続プールを持ち、用途の混用を防止する
pub struct AppState {
    /// マスタ CRUD 用（app_write ロール）
    pub write_pool: PgPool,
    /// 作業ログ記録用（app_event_insert ロール）
    pub event_pool: PgPool,
    /// 読み取り専用（app_read ロール）
    pub read_pool: PgPool,
}
```

**本節で確定した方針**
- **3 ロール（app_write/app_event_insert/app_read）を DB レベルで定義し、接続プールを用途別に分離する。**
- **`app_write` ロールから `work_event` テーブルへのアクセス権を `REVOKE` する。**
- **接続プールの混用（例: `write_pool` で SELECT 専用クエリを実行する）を型レベルで防止する。**

---

## 10. Append-only 物理保証

### app_event_insert ロールへの制限

```sql
-- Append-only の物理保証: app_event_insert ロールに UPDATE/DELETE 権限を付与しない
-- work_events テーブルは INSERT のみを許可する
REVOKE UPDATE, DELETE ON work_event FROM app_event_insert;
REVOKE UPDATE, DELETE ON work_event FROM app_write;
REVOKE UPDATE, DELETE ON work_event FROM app_read;

-- 確認クエリ（統合テストで実行する）
SELECT grantee, privilege_type
FROM information_schema.role_table_grants
WHERE table_name = 'work_event'
  AND grantee IN ('app_event_insert', 'app_write', 'app_read')
  AND privilege_type IN ('UPDATE', 'DELETE');
-- 上記クエリの結果が 0 行であることを確認する
```

### 統合テストによる権限検証

```rust
#[tokio::test]
async fn test_work_event_append_only() {
    // app_event_insert ロールで接続する
    let pool = create_test_pool_with_role("app_event_insert").await;

    // INSERT は成功するはず
    let result = sqlx::query!(
        "INSERT INTO work_event (case_id, activity, ...) VALUES ($1, $2, ...)",
        case_id, "start"
    )
    .execute(&pool)
    .await;
    assert!(result.is_ok());

    // UPDATE は失敗するはず（権限なし）
    let update_result = sqlx::query!(
        "UPDATE work_event SET activity = 'modified' WHERE id = $1",
        event_id
    )
    .execute(&pool)
    .await;
    assert!(update_result.is_err(), "app_event_insert は UPDATE 権限を持つべきではない");

    // DELETE は失敗するはず（権限なし）
    let delete_result = sqlx::query!("DELETE FROM work_event WHERE id = $1", event_id)
        .execute(&pool)
        .await;
    assert!(delete_result.is_err(), "app_event_insert は DELETE 権限を持つべきではない");
}
```

**本節で確定した方針**
- **`REVOKE UPDATE, DELETE ON work_event FROM app_event_insert` を DDL に明記し、権限付与の漏れを防止する。**
- **統合テストで `app_event_insert` ロールの UPDATE/DELETE 禁止を自動検証し、権限変更を検出する。**
- **マイグレーション後に権限設定が正しいことを CI で確認する手順を `09_ビルド手順.md` に記載する。**

---

## 11. クエリパフォーマンス

### EXPLAIN (ANALYZE, BUFFERS) 必須

本番に近い環境で新規クエリを追加する際は `EXPLAIN (ANALYZE, BUFFERS)` で実行計画を確認する。

```sql
-- 作業イベントの取得クエリの実行計画確認
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
SELECT id, case_id, activity, server_received_at, hash
FROM work_event
WHERE case_id = 'a3f8b2c1-d4e5-f607-a890-b12345678901'
ORDER BY server_received_at ASC;
```

確認すべきポイント：
- `Seq Scan` → `Index Scan` に変わっているか
- `Buffers: shared hit=N` の N が過大でないか（N > 1000 は要改善）
- `Actual Time` が 100ms 以下か（スロークエリ閾値）

### N+1 回避

```sql
-- 禁止: N+1 クエリ（work_order を 1 件取得後、関連 work_event を N 回クエリ）
-- SELECT * FROM work_order WHERE id = $1;
-- SELECT * FROM work_event WHERE case_id = $1;  -- N 回実行される

-- 推奨: JOIN で一括取得する
SELECT
    wo.id AS work_order_id,
    wo.work_order_number,
    we.id AS event_id,
    we.activity,
    we.server_received_at
FROM work_order wo
LEFT JOIN work_event we ON wo.id = we.work_order_id
WHERE wo.id = $1
ORDER BY we.server_received_at ASC;
```

### CTE と Window 関数の活用

```sql
-- ハッシュチェーン検証クエリ（Window 関数による前後ハッシュの比較）
WITH event_chain AS (
    SELECT
        id,
        hash,
        LAG(hash) OVER (PARTITION BY case_id ORDER BY server_received_at ASC) AS prev_hash,
        server_received_at
    FROM work_event
    WHERE case_id = $1
)
SELECT *
FROM event_chain
WHERE prev_hash IS NOT NULL
  AND hash != compute_expected_hash(prev_hash, id);
-- 結果が 0 行であればチェーンは整合している
```

### スロークエリ閾値

`log_min_duration_statement = 100` を PostgreSQL の設定に追加し、100ms を超えるクエリをログに記録する。

**本節で確定した方針**
- **新規クエリには `EXPLAIN (ANALYZE, BUFFERS)` での実行計画確認を必須とし、`Seq Scan` の放置を禁止する。**
- **スロークエリ閾値を 100ms と定め、`log_min_duration_statement` でログに記録する。**
- **N+1 問題を `JOIN` またはバッチ取得で解消し、ループ内のクエリ実行を禁止する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)
- [`90_業界分析/21_作業ログ分析とプロセスマイニング.md`](../../90_業界分析/21_作業ログ分析とプロセスマイニング.md)
