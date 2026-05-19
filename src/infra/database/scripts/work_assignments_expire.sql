-- work_assignments_expire.sql — BAT-015: work_assignments の期限切れ処理
-- 権威ドキュメント:
--   docs/05_詳細設計/01_データベース詳細設計/ TBL-work_assignments 定義
--
-- 目的: due_at（期限日時）を超過した pending / dispatched 状態の作業指示を expired に変更する
--       期限超過した作業指示を放置すると監督者が状況を把握できなくなるため、
--       自動的に expired 状態に移行してアラートの契機とする
--
-- 実行タイミング: 定期実行（BAT-015: アプリ tokio task として定期実行する）
--
-- 注意:
--   expired 状態への変更は Append-only 例外（work_assignments は状態遷移テーブルのため
--   状態カラムの UPDATE が許可されている）。

-- due_at を超過した pending / dispatched 状態の作業指示を expired に変更する
UPDATE work_assignments
SET status = 'expired'
WHERE
    -- pending（未着手）または dispatched（割り当て済み・未着手）の状態のみを対象とする
    status IN ('pending', 'dispatched')
    -- due_at が設定されており（NULL は期限なしのため除外する）
    AND due_at IS NOT NULL
    -- 現在時刻が due_at を超過している場合
    AND due_at < NOW();
