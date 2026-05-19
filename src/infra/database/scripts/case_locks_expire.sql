-- case_locks_expire.sql — BAT-013: case_locks の heartbeat タイムアウト処理
-- 権威ドキュメント:
--   src/CLAUDE.md §「マルチデバイス排他原則」（heartbeat 60秒・EXPIRED 閾値 5 分）
--   docs/05_詳細設計/07_アルゴリズム詳細設計/08_Case端末占有アルゴリズム.md（ADR-009）
--
-- 目的: ハートビートが 5 分を超えて更新されていない ACTIVE ロックを EXPIRED に変更する
--       端末の異常終了・通信断によりハートビートが途絶えた場合のデッドロックを防止する
--
-- 実行タイミング: 定期実行（BAT-013: アプリ tokio task として 1 分ごとに実行する）
--
-- 注意:
--   EXPIRED に変更されたロックは次回の端末接続時に再取得できるようになる。
--   正常なシフト交代は suspend → 解放 → resume の順序で実施するため、
--   ハートビートタイムアウトとは区別される。

-- heartbeat_at が 5 分を超過した ACTIVE ロックを EXPIRED に変更する
UPDATE case_locks
SET lock_status = 'EXPIRED'
WHERE
    -- ACTIVE 状態のロックのみを対象とする（EXPIRED / RELEASED は除外する）
    lock_status = 'ACTIVE'
    -- ハートビートが 5 分以上更新されていないレコードを対象とする
    AND heartbeat_at < NOW() - INTERVAL '5 minutes';
