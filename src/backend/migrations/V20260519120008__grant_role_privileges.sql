-- V20260519120008__grant_role_privileges.sql
-- テーブル単位での GRANT/REVOKE 設定
-- ロール設計:
--   app_read        — 全テーブルへの SELECT のみ
--   app_write       — マスタテーブルへの SELECT/INSERT/UPDATE（DELETE 不可）
--   app_event_insert — Append-only テーブルへの INSERT のみ
--                      例外: case_locks・idempotency_keys は INSERT/UPDATE/DELETE を許可（ADR-009）
-- =====================================================

-- =====================================================
-- PUBLIC からの権限剥奪（セキュリティ基線）
-- =====================================================
-- PostgreSQL のデフォルトは PUBLIC に EXECUTE 等が付与されているため明示的に剥奪する
REVOKE ALL ON ALL TABLES IN SCHEMA public FROM PUBLIC;
REVOKE ALL ON ALL SEQUENCES IN SCHEMA public FROM PUBLIC;

-- =====================================================
-- app_read: 全テーブルへの SELECT のみ
-- =====================================================

-- マスタ系テーブル
GRANT SELECT ON roles                    TO app_read;
GRANT SELECT ON users                    TO app_read;
GRANT SELECT ON skills                   TO app_read;
GRANT SELECT ON user_roles               TO app_read;
GRANT SELECT ON user_skills              TO app_read;
GRANT SELECT ON devices                  TO app_read;
GRANT SELECT ON electronic_signs         TO app_read;
GRANT SELECT ON master_versions          TO app_read;
GRANT SELECT ON processes                TO app_read;
GRANT SELECT ON operations               TO app_read;
GRANT SELECT ON products                 TO app_read;
GRANT SELECT ON materials                TO app_read;
GRANT SELECT ON suppliers                TO app_read;
GRANT SELECT ON lots                     TO app_read;
GRANT SELECT ON equipments               TO app_read;
GRANT SELECT ON instruments              TO app_read;
GRANT SELECT ON sops                     TO app_read;
GRANT SELECT ON steps                    TO app_read;
GRANT SELECT ON step_flow_rules          TO app_read;
GRANT SELECT ON step_type_definitions    TO app_read;
GRANT SELECT ON work_patterns            TO app_read;
GRANT SELECT ON device_sync_states       TO app_read;
GRANT SELECT ON sampling_plans           TO app_read;
GRANT SELECT ON rework_sop_mapping       TO app_read;

-- トランザクション系テーブル
GRANT SELECT ON work_orders                       TO app_read;
GRANT SELECT ON work_executions                   TO app_read;
GRANT SELECT ON work_events                       TO app_read;
GRANT SELECT ON outbox_events                     TO app_read;
GRANT SELECT ON evidence_files                    TO app_read;
GRANT SELECT ON measurements                      TO app_read;
GRANT SELECT ON suspensions                       TO app_read;
GRANT SELECT ON andon_alerts                      TO app_read;
GRANT SELECT ON nonconformities                   TO app_read;
GRANT SELECT ON capas                             TO app_read;
GRANT SELECT ON kaizen_proposals                  TO app_read;
GRANT SELECT ON external_key_bindings             TO app_read;
GRANT SELECT ON hash_chain_blocks                 TO app_read;
GRANT SELECT ON auth_logs                         TO app_read;
GRANT SELECT ON idempotency_keys                  TO app_read;
GRANT SELECT ON incoming_inspections              TO app_read;
GRANT SELECT ON incoming_inspection_measurements  TO app_read;
GRANT SELECT ON concession_approvals              TO app_read;
GRANT SELECT ON lot_qc_states                     TO app_read;
GRANT SELECT ON reworks                           TO app_read;
GRANT SELECT ON dispositions                      TO app_read;
GRANT SELECT ON rework_verifications              TO app_read;
GRANT SELECT ON reworked_lot_labels               TO app_read;
GRANT SELECT ON rework_cost_records               TO app_read;
GRANT SELECT ON scrap_records                     TO app_read;
GRANT SELECT ON return_to_vendor_records          TO app_read;
GRANT SELECT ON case_locks                        TO app_read;

-- 作業指示系テーブル
GRANT SELECT ON work_assignments    TO app_read;
GRANT SELECT ON sse_dispatch_log    TO app_read;

-- ビュー・マテリアライズドビュー
GRANT SELECT ON v_active_work_executions  TO app_read;
GRANT SELECT ON v_published_sops          TO app_read;
GRANT SELECT ON v_user_skills_full        TO app_read;
GRANT SELECT ON v_step_sequence           TO app_read;
GRANT SELECT ON v_work_event_xes          TO app_read;
GRANT SELECT ON mv_daily_work_summary     TO app_read;
GRANT SELECT ON v_andon_active            TO app_read;
GRANT SELECT ON v_hash_chain_latest       TO app_read;

-- =====================================================
-- app_write: マスタ系テーブルへの SELECT/INSERT/UPDATE（DELETE 不可）
-- =====================================================

-- マスタ系テーブル（読み取り・書き込み・更新）
GRANT SELECT, INSERT, UPDATE ON roles                 TO app_write;
GRANT SELECT, INSERT, UPDATE ON users                 TO app_write;
GRANT SELECT, INSERT, UPDATE ON skills                TO app_write;
GRANT SELECT, INSERT, UPDATE ON user_roles            TO app_write;
GRANT SELECT, INSERT, UPDATE ON user_skills           TO app_write;
GRANT SELECT, INSERT, UPDATE ON devices               TO app_write;
GRANT SELECT, INSERT, UPDATE ON master_versions       TO app_write;
GRANT SELECT, INSERT, UPDATE ON processes             TO app_write;
GRANT SELECT, INSERT, UPDATE ON operations            TO app_write;
GRANT SELECT, INSERT, UPDATE ON products              TO app_write;
GRANT SELECT, INSERT, UPDATE ON materials             TO app_write;
GRANT SELECT, INSERT, UPDATE ON suppliers             TO app_write;
GRANT SELECT, INSERT, UPDATE ON lots                  TO app_write;
GRANT SELECT, INSERT, UPDATE ON equipments            TO app_write;
GRANT SELECT, INSERT, UPDATE ON instruments           TO app_write;
GRANT SELECT, INSERT, UPDATE ON sops                  TO app_write;
GRANT SELECT, INSERT, UPDATE ON steps                 TO app_write;
GRANT SELECT, INSERT, UPDATE ON step_flow_rules       TO app_write;
GRANT SELECT, INSERT, UPDATE ON step_type_definitions TO app_write;
GRANT SELECT, INSERT, UPDATE ON work_patterns         TO app_write;
GRANT SELECT, INSERT, UPDATE ON device_sync_states    TO app_write;
GRANT SELECT, INSERT, UPDATE ON sampling_plans        TO app_write;
GRANT SELECT, INSERT, UPDATE ON rework_sop_mapping    TO app_write;

-- 更新可トランザクション系テーブル
GRANT SELECT, INSERT, UPDATE ON work_orders     TO app_write;
GRANT SELECT, INSERT, UPDATE ON work_executions TO app_write;
GRANT SELECT, INSERT, UPDATE ON andon_alerts    TO app_write;
GRANT SELECT, INSERT, UPDATE ON nonconformities TO app_write;
GRANT SELECT, INSERT, UPDATE ON capas           TO app_write;
GRANT SELECT, INSERT, UPDATE ON kaizen_proposals TO app_write;
GRANT SELECT, INSERT, UPDATE ON lot_qc_states   TO app_write;
GRANT SELECT, INSERT, UPDATE ON reworks         TO app_write;
GRANT SELECT, INSERT, UPDATE ON rework_cost_records TO app_write;

-- outbox_events: INSERT + SELECT + status 列のみ UPDATE（ADR-010 例外）
GRANT SELECT, INSERT ON outbox_events TO app_write;
GRANT UPDATE (status, sent_at, next_retry_at, retry_count, dlq_reason) ON outbox_events TO app_write;

-- incoming_inspections: INSERT + SELECT + qc_status 列のみ UPDATE（限定可変）
GRANT SELECT, INSERT ON incoming_inspections TO app_write;
GRANT UPDATE (qc_status, judged_at) ON incoming_inspections TO app_write;

-- 作業指示系テーブル: status 系列の UPDATE を許可
GRANT SELECT, INSERT ON work_assignments TO app_write;
GRANT UPDATE (status, case_id, dispatched_at, accepted_at, completed_at) ON work_assignments TO app_write;
GRANT SELECT, INSERT ON sse_dispatch_log TO app_write;
GRANT UPDATE (delivery_status, ack_at, retry_count) ON sse_dispatch_log TO app_write;

-- idempotency_keys: INSERT + SELECT + DELETE（TTL 削除・唯一の例外）
GRANT SELECT, INSERT, DELETE ON idempotency_keys TO app_write;

-- app_write は Append-only テーブルへの SELECT のみ（INSERT は app_event_insert が担当）
GRANT SELECT ON electronic_signs         TO app_write;
GRANT SELECT ON evidence_files           TO app_write;
GRANT SELECT ON measurements             TO app_write;
GRANT SELECT ON suspensions              TO app_write;
GRANT SELECT ON external_key_bindings    TO app_write;
GRANT SELECT ON hash_chain_blocks        TO app_write;
GRANT SELECT ON auth_logs                TO app_write;
GRANT SELECT ON incoming_inspection_measurements TO app_write;
GRANT SELECT ON concession_approvals     TO app_write;
GRANT SELECT ON dispositions             TO app_write;
GRANT SELECT ON rework_verifications     TO app_write;
GRANT SELECT ON reworked_lot_labels      TO app_write;
GRANT SELECT ON scrap_records            TO app_write;
GRANT SELECT ON return_to_vendor_records TO app_write;
GRANT SELECT ON work_events              TO app_write;
GRANT SELECT ON case_locks               TO app_write;

-- ビュー・マテリアライズドビュー（app_write は SELECT のみ）
GRANT SELECT ON v_active_work_executions  TO app_write;
GRANT SELECT ON v_published_sops          TO app_write;
GRANT SELECT ON v_user_skills_full        TO app_write;
GRANT SELECT ON v_step_sequence           TO app_write;
GRANT SELECT ON v_work_event_xes          TO app_write;
GRANT SELECT ON mv_daily_work_summary     TO app_write;
GRANT SELECT ON v_andon_active            TO app_write;
GRANT SELECT ON v_hash_chain_latest       TO app_write;

-- =====================================================
-- app_event_insert: Append-only テーブルへの INSERT のみ（作業ログ書き込み専用）
-- ADR-010: UPDATE/DELETE は一切付与しない
-- =====================================================

-- Append-only テーブルへの INSERT + SELECT
GRANT SELECT, INSERT ON work_events                     TO app_event_insert;
GRANT SELECT, INSERT ON electronic_signs                TO app_event_insert;
GRANT SELECT, INSERT ON evidence_files                  TO app_event_insert;
GRANT SELECT, INSERT ON measurements                    TO app_event_insert;
GRANT SELECT, INSERT ON suspensions                     TO app_event_insert;
GRANT SELECT, INSERT ON external_key_bindings           TO app_event_insert;
GRANT SELECT, INSERT ON hash_chain_blocks               TO app_event_insert;
GRANT SELECT, INSERT ON auth_logs                       TO app_event_insert;
GRANT SELECT, INSERT ON outbox_events                   TO app_event_insert;
GRANT SELECT, INSERT ON incoming_inspections            TO app_event_insert;
GRANT SELECT, INSERT ON incoming_inspection_measurements TO app_event_insert;
GRANT SELECT, INSERT ON concession_approvals            TO app_event_insert;
GRANT SELECT, INSERT ON dispositions                    TO app_event_insert;
GRANT SELECT, INSERT ON rework_verifications            TO app_event_insert;
GRANT SELECT, INSERT ON reworked_lot_labels             TO app_event_insert;
GRANT SELECT, INSERT ON scrap_records                   TO app_event_insert;
GRANT SELECT, INSERT ON return_to_vendor_records        TO app_event_insert;

-- 作業実行にも SELECT は必要（参照整合性確認のため）
GRANT SELECT ON work_executions TO app_event_insert;

-- =====================================================
-- app_event_insert 例外テーブル（ADR-009 より）
-- case_locks: heartbeat_at 更新（UPDATE）・解放（DELETE）が必要な制御テーブル
-- idempotency_keys: TTL 削除（DELETE）が必要な制御テーブル
-- =====================================================

-- case_locks: INSERT/UPDATE/DELETE を許可（ADR-009 マルチデバイス排他制御）
GRANT SELECT, INSERT, UPDATE, DELETE ON case_locks TO app_event_insert;

-- idempotency_keys: INSERT/SELECT/DELETE を許可（TTL 24h 管理）
GRANT SELECT, INSERT, DELETE ON idempotency_keys TO app_event_insert;

-- =====================================================
-- Append-only テーブルからの UPDATE/DELETE 明示的剥奪
-- トリガーで防ぐが、ロールレベルでも二重防御する
-- =====================================================
REVOKE UPDATE, DELETE ON work_events              FROM app_event_insert;
REVOKE UPDATE, DELETE ON electronic_signs         FROM app_event_insert;
REVOKE UPDATE, DELETE ON evidence_files           FROM app_event_insert;
REVOKE UPDATE, DELETE ON measurements             FROM app_event_insert;
REVOKE UPDATE, DELETE ON suspensions              FROM app_event_insert;
REVOKE UPDATE, DELETE ON external_key_bindings    FROM app_event_insert;
REVOKE UPDATE, DELETE ON hash_chain_blocks        FROM app_event_insert;
REVOKE UPDATE, DELETE ON auth_logs                FROM app_event_insert;
REVOKE UPDATE, DELETE ON incoming_inspection_measurements FROM app_event_insert;
REVOKE UPDATE, DELETE ON concession_approvals     FROM app_event_insert;
REVOKE UPDATE, DELETE ON dispositions             FROM app_event_insert;
REVOKE UPDATE, DELETE ON rework_verifications     FROM app_event_insert;
REVOKE UPDATE, DELETE ON reworked_lot_labels      FROM app_event_insert;
REVOKE UPDATE, DELETE ON scrap_records            FROM app_event_insert;
REVOKE UPDATE, DELETE ON return_to_vendor_records FROM app_event_insert;

-- app_write も Append-only テーブルへの UPDATE/DELETE を禁止
REVOKE UPDATE, DELETE ON work_events              FROM app_write;
REVOKE UPDATE, DELETE ON electronic_signs         FROM app_write;
REVOKE UPDATE, DELETE ON evidence_files           FROM app_write;
REVOKE UPDATE, DELETE ON measurements             FROM app_write;
REVOKE UPDATE, DELETE ON suspensions              FROM app_write;
REVOKE UPDATE, DELETE ON external_key_bindings    FROM app_write;
REVOKE UPDATE, DELETE ON hash_chain_blocks        FROM app_write;
REVOKE UPDATE, DELETE ON auth_logs                FROM app_write;
REVOKE UPDATE, DELETE ON incoming_inspection_measurements FROM app_write;
REVOKE UPDATE, DELETE ON concession_approvals     FROM app_write;
REVOKE UPDATE, DELETE ON dispositions             FROM app_write;
REVOKE UPDATE, DELETE ON rework_verifications     FROM app_write;
REVOKE UPDATE, DELETE ON reworked_lot_labels      FROM app_write;
REVOKE UPDATE, DELETE ON scrap_records            FROM app_write;
REVOKE UPDATE, DELETE ON return_to_vendor_records FROM app_write;

-- =====================================================
-- シーケンスへの USAGE 付与（gen_random_uuid() には不要だが自動採番シーケンスがある場合に備える）
-- =====================================================
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO app_write;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO app_event_insert;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO app_read;
