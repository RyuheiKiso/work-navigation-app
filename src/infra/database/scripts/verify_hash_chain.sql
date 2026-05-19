-- verify_hash_chain.sql — BAT-001: ハッシュチェーン整合性検証（LAG ベース）
-- 権威ドキュメント:
--   src/CLAUDE.md §4「SHA-256 ハッシュチェーン」原則
--   docs/04_概要設計/08_運用方式設計/04_バックアップ・リストア方式.md §5 Step 5
--   docs/09_運用・保守/運用手順/10_ActiveStandby切替手順（OPS-PROC-010）.md §4.8
--
-- 目的: case_id 単位で prev_hash の連続性を検証し、改ざん・データ欠落を検出する
-- 出力: 不整合が検出された場合は ERR-DB-003 として結果セットに含める
--       結果が 0 行であればチェーンは整合している
--
-- 実行タイミング:
--   - リストア後の整合性確認（restore.sh Step 5）
--   - Active-Standby 切替後の確認（failover.sh Step 後）
--   - 定期検証（BAT-001 の一部として実行する）

-- work_events の各 case_id について、prev_hash が前レコードの content_hash と一致するか検証する
WITH event_chain AS (
    SELECT
        event_id,
        case_id,
        content_hash,
        prev_hash,
        -- LAG 関数で直前レコードの content_hash を取得する（case_id 単位・server 受信時刻順）
        LAG(content_hash) OVER (
            PARTITION BY case_id
            ORDER BY timestamp_server ASC
        ) AS prev_content_hash,
        timestamp_server
    FROM work_events
    ORDER BY case_id, timestamp_server ASC
)
-- 不整合レコード（prev_hash が直前の content_hash と異なるもの）を抽出する
SELECT
    event_id,
    case_id,
    content_hash,
    prev_hash,
    prev_content_hash,
    timestamp_server,
    -- エラーコード ERR-DB-003: ハッシュチェーン破断
    'ERR-DB-003: hash_chain_broken' AS error_code
FROM event_chain
WHERE
    -- prev_content_hash が NULL でない（最初のレコードは NULL で正常）
    prev_content_hash IS NOT NULL
    -- prev_hash が直前レコードの content_hash と一致しない場合は不整合
    AND prev_hash <> prev_content_hash;

-- 結果が 0 行であればチェーンは整合している
-- 1 行以上の場合は system_admin に報告し、business 再開を保留すること（OPS-PROC-010 §4.8）
