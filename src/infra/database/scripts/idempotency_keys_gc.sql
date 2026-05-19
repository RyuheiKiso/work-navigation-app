-- idempotency_keys_gc.sql — TBL-035 idempotency_keys の TTL 超過レコード削除
-- 権威ドキュメント:
--   src/CLAUDE.md §3「Idempotent API」原則（TTL 24h）
--   docs/05_詳細設計/01_データベース詳細設計/ TBL-035 定義
--
-- 目的: Idempotency-Key のキャッシュ（TTL 24h）を定期的に削除してテーブルを小さく保つ
--       24 時間以上経過した Idempotency-Key は再送チェックの対象外となるため安全に削除できる
--
-- 実行タイミング: 毎時（pg_cron 設定またはアプリ tokio task として定期実行する）
--
-- 注意:
--   このクエリはテーブルロックを最小化するため DELETE ... WHERE の単純な形式を使用する。
--   大量のレコードが蓄積している場合はバッチサイズを設けること。

-- 24 時間（TTL 上限）を超過した Idempotency-Key レコードを削除する
DELETE FROM idempotency_keys
WHERE created_at < NOW() - INTERVAL '24 hours';
