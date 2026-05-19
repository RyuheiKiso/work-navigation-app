-- pii_anonymize.sql — BAT-004: PII 匿名化スクリプト
-- 権威ドキュメント:
--   docs/05_詳細設計/07_アルゴリズム詳細設計/06_バッチジョブ処理詳細（BAT-001〜010）.md §5 BAT-004
--
-- 目的: 退職（is_active = FALSE）してまだ匿名化されていないユーザーの PII を匿名化する
-- 実行タイミング: 毎日 01:00 JST（BAT-004: wnav_master_api 内 tokio task）
--
-- 匿名化対象列:
--   display_name : 'anonymized-{user_id 前 8 文字}' に置換する
--   login_id     : 'anonymized-{user_id 全 36 文字（ハイフン除去）}' に置換する
--
-- 実装注記（DDL との整合）:
--   BAT-004 設計書は email / phone / deactivated_at 列を参照しているが、
--   TBL-016（users）の権威 DDL にこれらの列は存在しない。
--   本スクリプトは実際の DDL（V20260517120003）に従い、存在する列のみを操作する。
--   deactivated_at が存在しないため、60 日の猶予期間はアプリ層スケジューラで制御する。
--   （wnav_master_api の BAT-004 タスクで updated_at から 60 日経過後に起動する）
--
-- ALCOA+ 要件（FR-EV-005）:
--   work_events.resource は UUID のまま保持する（AttributableトレーサビリティのためUUID変更不可）
--
-- 冪等性:
--   anonymized_at IS NULL のレコードのみを対象とするため二重実行は安全

-- is_active = FALSE かつ未匿名化のユーザーに対して PII を匿名化する
UPDATE users
SET
    -- 表示名を匿名化する（ALCOA+ Attributable 要件に従い user_id 前 8 文字で識別可能にする）
    display_name  = 'anonymized-' || LEFT(REPLACE(user_id::TEXT, '-', ''), 8),
    -- ログイン ID を匿名化する（外部システム連携を無効化するため UUID 全体に置換する）
    login_id      = 'anonymized-' || REPLACE(user_id::TEXT, '-', ''),
    -- 匿名化実施時刻を記録する（BAT-004 冪等性チェックキー）
    anonymized_at = NOW(),
    -- 更新時刻を更新する
    updated_at    = NOW()
WHERE
    -- 退職済み（論理削除）のユーザーのみを対象とする
    is_active     = FALSE
    -- 未匿名化のレコードのみを対象とする（冪等性保証）
    AND anonymized_at IS NULL;
