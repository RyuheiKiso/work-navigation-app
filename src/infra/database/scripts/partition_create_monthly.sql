-- partition_create_monthly.sql — BAT-004: 翌月の work_events パーティション作成
-- 権威ドキュメント:
--   docs/05_詳細設計/01_データベース詳細設計/06_パーティション・アーカイブ詳細設計.md §2
--
-- 実行タイミング: 毎月 25 日 02:00 JST（UTC 16:00 前日）に実行する
-- 目的: 翌月のパーティションを月末までに事前作成し、月初の INSERT 欠落を防止する
--
-- 使用方法:
--   psql -f partition_create_monthly.sql
--   または sqlx 経由で Rust の BAT-004 タスクから実行する

DO $$
DECLARE
    -- 翌月の開始日を計算する（現在時刻 + 1 ヶ月で翌月を確実に取得する）
    next_month_start DATE;
    next_month_end DATE;
    partition_name TEXT;
BEGIN
    -- 翌月の開始日を計算する
    next_month_start := DATE_TRUNC('month', NOW() + INTERVAL '1 month')::DATE;

    -- 翌々月の開始日をパーティション終端として計算する（半開区間 [from, to) を使用する）
    next_month_end := (next_month_start + INTERVAL '1 month')::DATE;

    -- パーティション名を命名規約に従って生成する（形式: work_events_yYYYYmMM）
    partition_name := 'work_events_y' || TO_CHAR(next_month_start, 'YYYY') || 'm' || TO_CHAR(next_month_start, 'MM');

    -- 既存でなければパーティションを作成する（冪等性を保証するため IF NOT EXISTS を使用する）
    IF NOT EXISTS (SELECT 1 FROM pg_tables WHERE tablename = partition_name) THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF work_events FOR VALUES FROM (%L) TO (%L)',
            partition_name, next_month_start, next_month_end
        );
        RAISE NOTICE 'Created partition: %', partition_name;
    ELSE
        RAISE NOTICE 'Partition already exists: %', partition_name;
    END IF;
END;
$$;
