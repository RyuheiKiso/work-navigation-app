-- V20260519120009__create_partitions.sql
-- パーティション関連の追加設定
-- work_events の月次パーティションは V003 で 2026年1月〜12月分を作成済み
-- 本ファイルでは:
--   1. 2026年5〜7月（要件指定の最低3ヶ月分）の存在確認コメント
--   2. 翌月パーティション自動作成関数の定義
--   3. outbox_events のパーティション設計確認（ドキュメントではパーティション非適用）
-- =====================================================

-- =====================================================
-- work_events パーティション状況確認
-- V003 で以下の月次パーティションが作成済み:
--   work_events_y2026m01 〜 work_events_y2026m12
-- 要件指定の最低3ヶ月分（2026-05, 2026-06, 2026-07）は V003 で作成済み
-- =====================================================

-- =====================================================
-- 翌月パーティション自動作成関数
-- BAT-004 が毎月 25 日 02:00（JST）に呼び出す
-- 引数: 作成対象の年（INTEGER）と月（INTEGER）
-- =====================================================
CREATE OR REPLACE FUNCTION fn_create_work_events_partition(
    p_year  INTEGER,
    p_month INTEGER
)
RETURNS VOID AS $$
DECLARE
    v_partition_name TEXT;
    v_from_ts        TEXT;
    v_to_year        INTEGER;
    v_to_month       INTEGER;
    v_to_ts          TEXT;
    v_sql            TEXT;
BEGIN
    -- パーティション名の生成（例: work_events_y2026m05）
    v_partition_name := FORMAT('work_events_y%04dm%02d', p_year, p_month);

    -- FROM タイムスタンプ
    v_from_ts := FORMAT('%04d-%02d-01 00:00:00+00', p_year, p_month);

    -- TO タイムスタンプ（翌月）
    IF p_month = 12 THEN
        v_to_year  := p_year + 1;
        v_to_month := 1;
    ELSE
        v_to_year  := p_year;
        v_to_month := p_month + 1;
    END IF;
    v_to_ts := FORMAT('%04d-%02d-01 00:00:00+00', v_to_year, v_to_month);

    -- パーティションが存在しない場合のみ作成する（冪等性保証）
    IF NOT EXISTS (
        SELECT 1
        FROM   pg_class c
               INNER JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE  n.nspname = 'public'
          AND  c.relname = v_partition_name
    ) THEN
        v_sql := FORMAT(
            'CREATE TABLE %I PARTITION OF work_events FOR VALUES FROM (%L) TO (%L)',
            v_partition_name,
            v_from_ts,
            v_to_ts
        );
        EXECUTE v_sql;

        RAISE NOTICE 'Created partition: % (% to %)', v_partition_name, v_from_ts, v_to_ts;
    ELSE
        RAISE NOTICE 'Partition already exists: %', v_partition_name;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fn_create_work_events_partition(INTEGER, INTEGER) IS
    'work_events の月次パーティションを作成する。BAT-004 が毎月 25 日 02:00（JST）に呼び出す。引数: 作成対象の年・月。既存パーティションは冪等スキップ。パーティション命名規約: work_events_y{YYYY}m{MM}（06_パーティション詳細設計 §1-2 準拠）。';

-- =====================================================
-- 現在の翌月以降のパーティション事前作成
-- 本マイグレーション実行時点（2026-05-19）から
-- 要件指定の最低3ヶ月分（2026-05〜07）を確実に保証する
-- V003 で既に作成済みだが冪等関数を通じて確認する
-- =====================================================
SELECT fn_create_work_events_partition(2026, 5);
SELECT fn_create_work_events_partition(2026, 6);
SELECT fn_create_work_events_partition(2026, 7);

-- =====================================================
-- outbox_events のパーティション設計
-- ドキュメント（06_パーティション詳細設計）によれば、
-- outbox_events は月次パーティション非適用（推定行数が少ない・TTL 90日管理のため）
-- パーティションは work_events のみに適用する設計
-- =====================================================

-- =====================================================
-- パーティション管理クエリ（DBA 運用用・参考）
-- =====================================================

-- 現在の全 work_events パーティション一覧を確認するクエリ（参考コメント）
-- SELECT
--     c.relname                                   AS partition_name,
--     pg_size_pretty(pg_relation_size(c.oid))     AS table_size,
--     pg_get_expr(c.relpartbound, c.oid, TRUE)    AS partition_bound
-- FROM pg_class c
--     JOIN pg_inherits i ON i.inhrelid = c.oid
--     JOIN pg_class parent ON parent.oid = i.inhparent
-- WHERE parent.relname = 'work_events'
-- ORDER BY c.relname;

-- =====================================================
-- 月次パーティション自動作成スケジューラ連携コメント
-- BAT-004 実装方針:
--   実行スケジュール: 毎月 25 日 02:00（JST）= UTC 17:00
--   実行方法: Rust バックエンドからの sqlx::query!("SELECT fn_create_work_events_partition($1, $2)", year, month)
--   year/month: 実行時点から見た翌月の年月を計算して渡す
--   pg_cron 連携（オプション）:
--     SELECT cron.schedule('create-next-month-partition', '0 17 25 * *',
--       $$SELECT fn_create_work_events_partition(
--           EXTRACT(YEAR FROM (NOW() + INTERVAL '35 days'))::INTEGER,
--           EXTRACT(MONTH FROM (NOW() + INTERVAL '35 days'))::INTEGER
--       )$$
--     );
-- =====================================================
