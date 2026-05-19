-- V20260517120057__create_materialized_views.sql
-- VW-006 mv_daily_work_summary マテリアライズドビューを作成する。
-- REFRESH 方針: 毎日 06:00 に pg_cron または BAT-004 が REFRESH CONCURRENTLY を実行する。
-- UNIQUE インデックス（work_date, sop_id）付与により REFRESH CONCURRENTLY が使用可能。
--
-- 対象ドキュメント: docs/05_詳細設計/01_データベース詳細設計/04_ビュー・マテリアライズドビュー設計（VWカタログ）.md

-- =============================================================================
-- VW-006: mv_daily_work_summary（MATERIALIZED VIEW）
-- 目的: 日次作業件数・完了率・平均所要時間の集計（RP-006 集計レポート用）
-- 集計軸: 日付（Asia/Tokyo）× sop_id × operation_id × process_id
-- WITH DATA: 作成時点のデータを即時取り込む（WITH NO DATA ではなく WITH DATA）
--            ただし初回はデータが存在しない可能性があるため WITH NO DATA でも可
-- =============================================================================
CREATE MATERIALIZED VIEW mv_daily_work_summary AS
SELECT
    -- 日付軸: タイムゾーンを Asia/Tokyo に変換してから DATE 型に変換する
    DATE(we.started_at AT TIME ZONE 'Asia/Tokyo')                   AS work_date,
    we.sop_id,
    s.sop_code,
    op.operation_id,
    op.operation_code,
    pr.process_id,
    pr.process_code,
    -- 件数集計
    COUNT(*)                                                         AS total_sessions,
    COUNT(*) FILTER (WHERE we.status = 'COMPLETED')                  AS completed_sessions,
    COUNT(*) FILTER (WHERE we.status = 'CANCELLED')                  AS cancelled_sessions,
    COUNT(*) FILTER (WHERE we.status = 'SUSPENDED')                  AS suspended_sessions,
    -- 完了率（完了セッション / 全セッション × 100、NULLIF で 0 除算を防止）
    ROUND(
        COUNT(*) FILTER (WHERE we.status = 'COMPLETED')::NUMERIC
        / NULLIF(COUNT(*), 0) * 100,
        2
    )                                                                AS completion_rate_pct,
    -- 平均作業時間（秒）: COMPLETED のセッションのみ集計し、completed_at が NULL の場合は除外
    AVG(
        EXTRACT(EPOCH FROM (we.completed_at - we.started_at))
    ) FILTER (WHERE we.status = 'COMPLETED')                         AS avg_duration_seconds
FROM work_executions we
    INNER JOIN sops       s  ON s.sop_id       = we.sop_id
    INNER JOIN operations op ON op.operation_id = s.operation_id
    INNER JOIN processes  pr ON pr.process_id   = op.process_id
-- started_at が NULL（NOT_STARTED 状態）は集計対象外
WHERE we.started_at IS NOT NULL
GROUP BY
    DATE(we.started_at AT TIME ZONE 'Asia/Tokyo'),
    we.sop_id,
    s.sop_code,
    op.operation_id,
    op.operation_code,
    pr.process_id,
    pr.process_code
WITH DATA;

COMMENT ON MATERIALIZED VIEW mv_daily_work_summary IS
    'VW-006 — 日次作業件数・完了率の集計 MATERIALIZED VIEW。毎日 06:00 に REFRESH CONCURRENTLY を実行する（pg_cron: 0 6 * * *）。RP-006 集計レポートおよびダッシュボード（SCR-MC-005）で参照する。UNIQUE インデックスにより CONCURRENTLY が使用可能。';

-- =============================================================================
-- MV 用インデックス: REFRESH CONCURRENTLY に必要な UNIQUE インデックス
-- idx_mv_daily_work_summary_pk: (work_date, sop_id) で UNIQUE を保証
-- idx_mv_daily_work_summary_date: work_date DESC で日付範囲検索を高速化
-- =============================================================================

-- REFRESH CONCURRENTLY を可能にするための UNIQUE インデックス（必須）
CREATE UNIQUE INDEX idx_mv_daily_work_summary_pk
    ON mv_daily_work_summary (work_date, sop_id);

COMMENT ON INDEX idx_mv_daily_work_summary_pk IS
    'MV 用 UNIQUE インデックス。REFRESH CONCURRENTLY の実行に必須。(work_date, sop_id) の組み合わせが主キーとなる。';

-- 日付範囲クエリを高速化するための補助インデックス
CREATE INDEX idx_mv_daily_work_summary_date
    ON mv_daily_work_summary (work_date DESC);

COMMENT ON INDEX idx_mv_daily_work_summary_date IS
    'MV 用日付降順インデックス。管理画面での直近 N 日間の集計レポート取得を高速化する。';
