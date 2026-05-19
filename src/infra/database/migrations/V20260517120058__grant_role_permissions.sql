-- V20260517120058__grant_role_permissions.sql
-- PostgreSQL データベースロールの作成（冪等）と権限設定（GRANT/REVOKE 全件）を行う。
--
-- ロール構成:
--   app_event_writer : Append-only テーブルへの INSERT/SELECT のみ
--   app_read_write   : 通常業務テーブルへの INSERT/SELECT/UPDATE（DELETE 禁止）
--   app_event_insert : case_locks など制御テーブルへの INSERT/UPDATE/DELETE（例外）
--   app_admin        : 全権限（DDL 操作・シーケンス含む）
--
-- 対象ドキュメント: docs/05_詳細設計/01_データベース詳細設計/07_マイグレーションスクリプト設計.md §3-3

-- =============================================================================
-- Step 1: ロールの冪等作成（既存ロールはスキップ）
-- =============================================================================
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_writer') THEN
        CREATE ROLE app_event_writer NOLOGIN;
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_read_write') THEN
        CREATE ROLE app_read_write NOLOGIN;
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_admin') THEN
        CREATE ROLE app_admin NOLOGIN CREATEROLE;
    END IF;
END $$;
-- V20260517120000 で CREATEROLE なしで app_admin が作成されている場合に備えて ALTER で確実に付与する
ALTER ROLE app_admin CREATEROLE;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_insert') THEN
        CREATE ROLE app_event_insert NOLOGIN;
    END IF;
END $$;

-- =============================================================================
-- Step 2: app_event_writer — Append-only テーブルへの INSERT/SELECT のみ
-- 対象: work_events / electronic_signs / evidence_files / measurements /
--       suspensions / hash_chain_blocks / auth_logs / external_key_bindings /
--       IQC・リワーク Append-only テーブル群
-- =============================================================================

-- 作業イベント系 Append-only テーブル
GRANT INSERT, SELECT ON work_events             TO app_event_writer;
GRANT INSERT, SELECT ON electronic_signs        TO app_event_writer;
GRANT INSERT, SELECT ON evidence_files          TO app_event_writer;
GRANT INSERT, SELECT ON measurements            TO app_event_writer;
GRANT INSERT, SELECT ON suspensions             TO app_event_writer;
GRANT INSERT, SELECT ON hash_chain_blocks       TO app_event_writer;
GRANT INSERT, SELECT ON auth_logs               TO app_event_writer;
GRANT INSERT, SELECT ON external_key_bindings   TO app_event_writer;

-- IQC・リワーク Append-only テーブル群
GRANT INSERT, SELECT ON incoming_inspection_measurements TO app_event_writer;
GRANT INSERT, SELECT ON concession_approvals             TO app_event_writer;
GRANT INSERT, SELECT ON dispositions                     TO app_event_writer;
GRANT INSERT, SELECT ON rework_verifications             TO app_event_writer;
GRANT INSERT, SELECT ON reworked_lot_labels              TO app_event_writer;
GRANT INSERT, SELECT ON scrap_records                    TO app_event_writer;
GRANT INSERT, SELECT ON return_to_vendor_records         TO app_event_writer;

-- =============================================================================
-- Step 3: app_read_write — 更新可テーブルへの CRUD + 列限定 UPDATE
-- 対象: 業務状態テーブル・マスタ系テーブル全般
-- =============================================================================

-- 業務状態テーブル
GRANT INSERT, SELECT, UPDATE ON work_executions    TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON andon_alerts       TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON nonconformities    TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON capas              TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON kaizen_proposals   TO app_read_write;
-- outbox_events: 設計書「Append-only + status 系のみ UPDATE 可」に従い列限定 UPDATE を適用する
GRANT INSERT, SELECT ON outbox_events TO app_read_write;
GRANT UPDATE (status, sent_at, next_retry_at, retry_count, dlq_reason) ON outbox_events TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON device_sync_states TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON lots               TO app_read_write;

-- マスタ系テーブル（is_active 更新含む）
GRANT INSERT, SELECT, UPDATE ON users              TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON sops               TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON steps              TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON master_versions    TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON work_patterns      TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON processes          TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON operations         TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON products           TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON equipments         TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON instruments        TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON devices            TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON work_orders        TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON roles              TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON skills             TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON user_roles         TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON user_skills        TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON step_type_definitions TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON step_flow_rules    TO app_read_write;

-- IQC・リワーク 状態更新テーブル
-- reworks: 設計書「限定可変（status のみ更新可）」に従い列限定 UPDATE を適用する
-- status 遷移に伴い rework_case_id / rework_sop_version_id / disposition_id / updated_at も更新される
GRANT INSERT, SELECT ON reworks TO app_read_write;
GRANT UPDATE (status, rework_case_id, rework_sop_version_id, disposition_id, updated_at) ON reworks TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON lot_qc_states      TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON rework_cost_records TO app_read_write;
-- incoming_inspections: 設計書「限定可変（qc_status のみ更新可）」に従い列限定 UPDATE を適用する
GRANT INSERT, SELECT ON incoming_inspections TO app_read_write;
GRANT UPDATE (qc_status, judged_at) ON incoming_inspections TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON materials          TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON suppliers          TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON sampling_plans     TO app_read_write;
GRANT INSERT, SELECT, UPDATE ON rework_sop_mapping TO app_read_write;

-- 列限定 UPDATE: work_assignments（状態遷移のみ許可）
GRANT INSERT, SELECT ON work_assignments TO app_read_write;
GRANT UPDATE (status, case_id, dispatched_at, accepted_at, completed_at) ON work_assignments TO app_read_write;

-- 列限定 UPDATE: sse_dispatch_log（配信状態の更新のみ許可）
GRANT INSERT, SELECT ON sse_dispatch_log TO app_read_write;
GRANT UPDATE (delivery_status, ack_at, retry_count) ON sse_dispatch_log TO app_read_write;

-- idempotency_keys: INSERT/SELECT/DELETE（TTL 削除のため DELETE を許可）
GRANT INSERT, SELECT, DELETE ON idempotency_keys TO app_read_write;

-- =============================================================================
-- Step 4: app_event_insert — case_locks 制御テーブルへの完全操作権限
-- 排他占有の取得・解放・ハートビート更新に必要（ADR-009 端末占有アルゴリズム）
-- =============================================================================
GRANT INSERT, UPDATE, DELETE ON case_locks TO app_event_insert;
GRANT SELECT                  ON case_locks TO app_event_insert;

-- =============================================================================
-- Step 5: Append-only テーブルから UPDATE/DELETE を全ロール・PUBLIC から明示的に剥奪
-- アーキテクチャ原則 P2（Append-only Event Sourcing）を DB 権限レベルで強制する。
-- =============================================================================

-- work_events（作業イベント — 絶対に修正・削除を禁止する）
REVOKE UPDATE, DELETE ON work_events FROM PUBLIC;
REVOKE UPDATE, DELETE ON work_events FROM app_event_writer;
REVOKE UPDATE, DELETE ON work_events FROM app_read_write;

-- electronic_signs（電子署名 — ALCOA+ Original 要件）
REVOKE UPDATE, DELETE ON electronic_signs FROM PUBLIC;
REVOKE UPDATE, DELETE ON electronic_signs FROM app_event_writer;
REVOKE UPDATE, DELETE ON electronic_signs FROM app_read_write;

-- evidence_files（証拠ファイルメタデータ）
REVOKE UPDATE, DELETE ON evidence_files FROM PUBLIC;
REVOKE UPDATE, DELETE ON evidence_files FROM app_event_writer;
REVOKE UPDATE, DELETE ON evidence_files FROM app_read_write;

-- measurements（計測値）
REVOKE UPDATE, DELETE ON measurements FROM PUBLIC;
REVOKE UPDATE, DELETE ON measurements FROM app_event_writer;
REVOKE UPDATE, DELETE ON measurements FROM app_read_write;

-- suspensions（中断記録）
REVOKE UPDATE, DELETE ON suspensions FROM PUBLIC;
REVOKE UPDATE, DELETE ON suspensions FROM app_event_writer;
REVOKE UPDATE, DELETE ON suspensions FROM app_read_write;

-- hash_chain_blocks（ハッシュチェーン）
REVOKE UPDATE, DELETE ON hash_chain_blocks FROM PUBLIC;
REVOKE UPDATE, DELETE ON hash_chain_blocks FROM app_event_writer;
REVOKE UPDATE, DELETE ON hash_chain_blocks FROM app_read_write;

-- auth_logs（認証ログ）
REVOKE UPDATE, DELETE ON auth_logs FROM PUBLIC;
REVOKE UPDATE, DELETE ON auth_logs FROM app_event_writer;
REVOKE UPDATE, DELETE ON auth_logs FROM app_read_write;

-- external_key_bindings（外部キーバインディング）
REVOKE UPDATE, DELETE ON external_key_bindings FROM PUBLIC;
REVOKE UPDATE, DELETE ON external_key_bindings FROM app_event_writer;
REVOKE UPDATE, DELETE ON external_key_bindings FROM app_read_write;

-- IQC・リワーク Append-only テーブル群（ADR-011 ハッシュチェーン保護）
REVOKE UPDATE, DELETE ON incoming_inspection_measurements FROM PUBLIC;
REVOKE UPDATE, DELETE ON incoming_inspection_measurements FROM app_event_writer;
REVOKE UPDATE, DELETE ON incoming_inspection_measurements FROM app_read_write;

REVOKE UPDATE, DELETE ON concession_approvals FROM PUBLIC;
REVOKE UPDATE, DELETE ON concession_approvals FROM app_event_writer;
REVOKE UPDATE, DELETE ON concession_approvals FROM app_read_write;

REVOKE UPDATE, DELETE ON dispositions FROM PUBLIC;
REVOKE UPDATE, DELETE ON dispositions FROM app_event_writer;
REVOKE UPDATE, DELETE ON dispositions FROM app_read_write;

REVOKE UPDATE, DELETE ON rework_verifications FROM PUBLIC;
REVOKE UPDATE, DELETE ON rework_verifications FROM app_event_writer;
REVOKE UPDATE, DELETE ON rework_verifications FROM app_read_write;

REVOKE UPDATE, DELETE ON reworked_lot_labels FROM PUBLIC;
REVOKE UPDATE, DELETE ON reworked_lot_labels FROM app_event_writer;
REVOKE UPDATE, DELETE ON reworked_lot_labels FROM app_read_write;

REVOKE UPDATE, DELETE ON scrap_records FROM PUBLIC;
REVOKE UPDATE, DELETE ON scrap_records FROM app_event_writer;
REVOKE UPDATE, DELETE ON scrap_records FROM app_read_write;

REVOKE UPDATE, DELETE ON return_to_vendor_records FROM PUBLIC;
REVOKE UPDATE, DELETE ON return_to_vendor_records FROM app_event_writer;
REVOKE UPDATE, DELETE ON return_to_vendor_records FROM app_read_write;

-- =============================================================================
-- Step 6: app_admin — 全テーブル・全シーケンスへの全権限
-- 読み取り SELECT を他ロールにも付与する（ビュー等の参照のため）
-- =============================================================================

-- app_admin に全権限を付与
GRANT ALL ON ALL TABLES    IN SCHEMA public TO app_admin;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO app_admin;

-- 全テーブルへの SELECT 権限を各ロールに付与（ビュー結合で他テーブル参照が必要なため）
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_event_writer;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_read_write;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_event_insert;
