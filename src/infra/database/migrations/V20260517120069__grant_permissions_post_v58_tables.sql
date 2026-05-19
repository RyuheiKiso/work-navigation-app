-- V20260517120069__grant_permissions_post_v58_tables.sql
-- V65〜V68 で追加した 4 テーブルへの権限補完。
--
-- 背景:
--   V58（grant_role_permissions）では実行時点に存在するすべてのテーブルへの
--   GRANT SELECT ON ALL TABLES IN SCHEMA public を一括付与した。
--   しかし V65〜V68 で追加した以下のテーブルは V58 実行後に作成されるため、
--   各ロールへの SELECT（および app_admin への ALL）が付与されていない:
--     - hash_chain_verification_results (V65)
--     - batch_job_logs                  (V66)
--     - batch_dlq                       (V67)
--     - alerts                          (V68)
--
-- 対象ロール:
--   app_admin        : 全権限（GRANT ALL）
--   app_event_writer : 上記テーブルの SELECT（ビュー結合・監視 API 参照のため）
--   app_read_write   : hash_chain_verification_results への SELECT
--                      （SCR-MC-008 ハッシュチェーン検証ダッシュボードが参照する）
--   app_event_insert : 上記テーブルの SELECT（V58 の設計方針「全テーブル SELECT 可」を継承）
--
-- 注: batch_job_logs / batch_dlq / alerts への INSERT/SELECT/UPDATE は
--     V66〜V68 で app_read_write に既に付与済みであり、本マイグレーションでは重複付与しない。

-- =============================================================================
-- app_admin — 全テーブル全権限（DDL 操作を含む）
-- =============================================================================
GRANT ALL ON hash_chain_verification_results TO app_admin;
GRANT ALL ON batch_job_logs                  TO app_admin;
GRANT ALL ON batch_dlq                       TO app_admin;
GRANT ALL ON alerts                          TO app_admin;

-- =============================================================================
-- app_event_writer — SELECT のみ（ビュー結合・監視 API 参照）
-- =============================================================================
GRANT SELECT ON batch_job_logs                  TO app_event_writer;
GRANT SELECT ON batch_dlq                       TO app_event_writer;
GRANT SELECT ON alerts                          TO app_event_writer;
-- hash_chain_verification_results は V65 で INSERT/SELECT を付与済み（重複不要）

-- =============================================================================
-- app_read_write — hash_chain_verification_results への SELECT 補完
-- SCR-MC-008 管理コンソール画面がハッシュチェーン検証結果を参照するため必要。
-- batch_job_logs / batch_dlq / alerts は V66〜V68 で INSERT/SELECT/UPDATE 付与済み。
-- =============================================================================
GRANT SELECT ON hash_chain_verification_results TO app_read_write;

-- =============================================================================
-- app_event_insert — 全 4 テーブルへの SELECT（V58 設計方針の継承）
-- =============================================================================
GRANT SELECT ON hash_chain_verification_results TO app_event_insert;
GRANT SELECT ON batch_job_logs                  TO app_event_insert;
GRANT SELECT ON batch_dlq                       TO app_event_insert;
GRANT SELECT ON alerts                          TO app_event_insert;
