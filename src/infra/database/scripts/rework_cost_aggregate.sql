-- rework_cost_aggregate.sql — BAT-011: rework_cost_records 日次集計
-- 権威ドキュメント:
--   docs/05_詳細設計/01_データベース詳細設計/ TBL-rework_cost_records・TBL-reworks 定義
--
-- 目的: 完了済みリワーク案件の作業時間・材料費・廃棄損失を日次集計してアップサートする
--       rework_cost_records は唯一 UPDATE が許可されているテーブル（集計データは上書き可能）
--
-- 実行タイミング: 毎日 03:00 JST（BAT-011）
--
-- 注意:
--   material_cost および scrap_loss は別途 ERP / scrap_records から取得する必要がある。
--   本クエリでは作業時間（labor_seconds）のみを work_executions から計算し、
--   コスト項目は 0 でプレースホルダとして挿入する。

-- 完了済みリワーク案件の作業時間を集計してアップサートする
INSERT INTO rework_cost_records (
    record_id,
    rework_id,
    additional_labor_seconds,
    additional_material_cost_yen,
    scrap_loss_yen,
    aggregated_at
)
SELECT
    -- 新規レコードの場合は UUID を生成する（ON CONFLICT で既存レコードを更新する）
    gen_random_uuid(),
    r.rework_id,
    -- 作業時間: リワーク案件に紐づく work_executions の合計時間（秒単位）
    COALESCE(
        (
            SELECT EXTRACT(EPOCH FROM SUM(we.completed_at - we.started_at))::INTEGER
            FROM work_executions we
            WHERE
                we.work_execution_id = r.rework_case_id
                AND we.started_at IS NOT NULL
                AND we.completed_at IS NOT NULL
        ),
        0
    ),
    -- 材料費: 別途 ERP から取得するため 0 でプレースホルダとする
    0,
    -- 廃棄損失: scrap_records から別途集計するため 0 でプレースホルダとする
    0,
    NOW()
FROM reworks r
WHERE
    -- 完了済み状態のリワーク案件のみを対象とする
    r.status IN (
        'REWORK_COMPLETED',
        'CLOSED_OK_RELEASE',
        'CLOSED_DOWNGRADE',
        'CLOSED_SCRAP',
        'CLOSED_RETURN'
    )
    -- rework_case_id が設定されているもののみ作業時間を集計できる
    AND r.rework_case_id IS NOT NULL
-- 既存レコードは作業時間と集計日時を更新する（材料費・廃棄損失は ERP 連携後に別途更新する）
ON CONFLICT (rework_id) DO UPDATE SET
    additional_labor_seconds = EXCLUDED.additional_labor_seconds,
    aggregated_at = NOW();
