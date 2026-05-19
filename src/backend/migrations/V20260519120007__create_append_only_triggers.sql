-- V20260519120007__create_append_only_triggers.sql
-- Append-only テーブルへの UPDATE/DELETE を禁止するトリガーの作成（ADR-010）
-- 対象テーブル:
--   work_events       — イベントストアコア
--   hash_chain_blocks — ハッシュチェーンチェックポイント
--   auth_logs         — 認証イベントログ
--   electronic_signs  — 電子サインレコード
--   evidence_files    — 証拠ファイルメタデータ
--   measurements      — 計測値レコード
--   suspensions       — 作業中断レコード
--   external_key_bindings — 外部キーマッピング
--   outbox_events     — Transactional Outbox（例外: status UPDATE は許可）
-- IQC/リワーク系 Append-only テーブル（ADR-011）:
--   incoming_inspection_measurements
--   concession_approvals
--   dispositions
--   rework_verifications
--   reworked_lot_labels
--   scrap_records
--   return_to_vendor_records
-- =====================================================

-- =====================================================
-- 汎用 Append-only 違反検知トリガー関数
-- =====================================================

-- Append-only 違反時に RAISE EXCEPTION するトリガー関数（UPDATE 禁止）
CREATE OR REPLACE FUNCTION fn_deny_update_append_only()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'APPEND-ONLY violation: UPDATE is forbidden on table %', TG_TABLE_NAME;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fn_deny_update_append_only() IS
    'Append-only テーブルへの UPDATE を禁止するトリガー関数。ADR-010 準拠。対象テーブルへの UPDATE 時に ERR-DB-APPEND-ONLY を発生させる。';

-- Append-only 違反時に RAISE EXCEPTION するトリガー関数（DELETE 禁止）
CREATE OR REPLACE FUNCTION fn_deny_delete_append_only()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'APPEND-ONLY violation: DELETE is forbidden on table %', TG_TABLE_NAME;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fn_deny_delete_append_only() IS
    'Append-only テーブルへの DELETE を禁止するトリガー関数。ADR-010 準拠。対象テーブルへの DELETE 時に ERR-DB-APPEND-ONLY を発生させる。';

-- =====================================================
-- TBL-001: work_events（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_work_events ON work_events;
CREATE TRIGGER trg_deny_update_work_events
    BEFORE UPDATE ON work_events
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_work_events ON work_events;
CREATE TRIGGER trg_deny_delete_work_events
    BEFORE DELETE ON work_events
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_work_events ON work_events IS
    'ADR-010: work_events は Append-only テーブル。UPDATE は全カラムで禁止。ハッシュチェーン整合性を保護する。';
COMMENT ON TRIGGER trg_deny_delete_work_events ON work_events IS
    'ADR-010: work_events は Append-only テーブル。DELETE は禁止。7年以上の監査証跡保存を保証する。';

-- =====================================================
-- TBL-031: hash_chain_blocks（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_hash_chain_blocks ON hash_chain_blocks;
CREATE TRIGGER trg_deny_update_hash_chain_blocks
    BEFORE UPDATE ON hash_chain_blocks
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_hash_chain_blocks ON hash_chain_blocks;
CREATE TRIGGER trg_deny_delete_hash_chain_blocks
    BEFORE DELETE ON hash_chain_blocks
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_hash_chain_blocks ON hash_chain_blocks IS
    'ADR-010: hash_chain_blocks は Append-only テーブル。週次チェックポイントの改ざんを防止する。';
COMMENT ON TRIGGER trg_deny_delete_hash_chain_blocks ON hash_chain_blocks IS
    'ADR-010: hash_chain_blocks は Append-only テーブル。DELETE は禁止。7年以上の監査証跡保存を保証する。';

-- =====================================================
-- TBL-032: auth_logs（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_auth_logs ON auth_logs;
CREATE TRIGGER trg_deny_update_auth_logs
    BEFORE UPDATE ON auth_logs
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_auth_logs ON auth_logs;
CREATE TRIGGER trg_deny_delete_auth_logs
    BEFORE DELETE ON auth_logs
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_auth_logs ON auth_logs IS
    'ADR-010: auth_logs は Append-only テーブル。認証監査ログの改ざんを防止する（ALCOA+ 要件）。';
COMMENT ON TRIGGER trg_deny_delete_auth_logs ON auth_logs IS
    'ADR-010: auth_logs は Append-only テーブル。DELETE は禁止。90日保存後はアーカイブで対応する。';

-- =====================================================
-- TBL-002: electronic_signs（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_electronic_signs ON electronic_signs;
CREATE TRIGGER trg_deny_update_electronic_signs
    BEFORE UPDATE ON electronic_signs
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_electronic_signs ON electronic_signs;
CREATE TRIGGER trg_deny_delete_electronic_signs
    BEFORE DELETE ON electronic_signs
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_electronic_signs ON electronic_signs IS
    'ADR-010: electronic_signs は Append-only テーブル。ALCOA+ Original / Attributable 要件（電子サインの不変性）を保護する。';
COMMENT ON TRIGGER trg_deny_delete_electronic_signs ON electronic_signs IS
    'ADR-010: electronic_signs は Append-only テーブル。DELETE は禁止。電子サイン証跡の永続保存を保証する。';

-- =====================================================
-- TBL-009: evidence_files（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_evidence_files ON evidence_files;
CREATE TRIGGER trg_deny_update_evidence_files
    BEFORE UPDATE ON evidence_files
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_evidence_files ON evidence_files;
CREATE TRIGGER trg_deny_delete_evidence_files
    BEFORE DELETE ON evidence_files
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_evidence_files ON evidence_files IS
    'ADR-010: evidence_files は Append-only テーブル。証拠ファイルメタデータの改ざんを防止する。';
COMMENT ON TRIGGER trg_deny_delete_evidence_files ON evidence_files IS
    'ADR-010: evidence_files は Append-only テーブル。DELETE は禁止。7年以上の証拠保存を保証する。';

-- =====================================================
-- TBL-010: measurements（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_measurements ON measurements;
CREATE TRIGGER trg_deny_update_measurements
    BEFORE UPDATE ON measurements
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_measurements ON measurements;
CREATE TRIGGER trg_deny_delete_measurements
    BEFORE DELETE ON measurements
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_measurements ON measurements IS
    'ADR-010: measurements は Append-only テーブル。計測値の改ざんを防止する（ALCOA+ Accurate 要件）。';
COMMENT ON TRIGGER trg_deny_delete_measurements ON measurements IS
    'ADR-010: measurements は Append-only テーブル。DELETE は禁止。7年以上の計測記録保存を保証する。';

-- =====================================================
-- TBL-011: suspensions（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_suspensions ON suspensions;
CREATE TRIGGER trg_deny_update_suspensions
    BEFORE UPDATE ON suspensions
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_suspensions ON suspensions;
CREATE TRIGGER trg_deny_delete_suspensions
    BEFORE DELETE ON suspensions
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_suspensions ON suspensions IS
    'ADR-010: suspensions は Append-only テーブル。中断記録の改ざんを防止する。再開情報は work_events（activity=work_resumed）で記録する。';
COMMENT ON TRIGGER trg_deny_delete_suspensions ON suspensions IS
    'ADR-010: suspensions は Append-only テーブル。DELETE は禁止。7年以上の中断証跡保存を保証する。';

-- =====================================================
-- TBL-027: external_key_bindings（Append-only・UPDATE/DELETE 完全禁止）
-- =====================================================
DROP TRIGGER IF EXISTS trg_deny_update_external_key_bindings ON external_key_bindings;
CREATE TRIGGER trg_deny_update_external_key_bindings
    BEFORE UPDATE ON external_key_bindings
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_external_key_bindings ON external_key_bindings;
CREATE TRIGGER trg_deny_delete_external_key_bindings
    BEFORE DELETE ON external_key_bindings
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_external_key_bindings ON external_key_bindings IS
    'ADR-010: external_key_bindings は Append-only テーブル。変更時は旧レコードの valid_to を設定 + 新レコード INSERT の 2 件操作が必須。自動解決禁止。';
COMMENT ON TRIGGER trg_deny_delete_external_key_bindings ON external_key_bindings IS
    'ADR-010: external_key_bindings は Append-only テーブル。DELETE は禁止。マッピング履歴の永続保存を保証する。';

-- =====================================================
-- TBL-003: outbox_events（Append-only・ただし status UPDATE のみ許可）
-- ADR-010 例外: Transactional Outbox パターンの送信状態管理（PENDING→SENDING→SENT）に必要
-- outbox_events の処理済み DELETE については、90日後アーカイブ後に DELETE を許可する
-- （DLQ・SENT レコードの TTL 管理のため）
-- =====================================================

-- outbox_events は status 列の UPDATE のみ許可するため、
-- UPDATE 禁止トリガーは付与しない（ロールレベルで制御）
-- ただし DELETE についてはトリガーで防ぐ（物理削除禁止方針に準拠）
-- 実際の outbox 処理では SENT/DLQ レコードを定期削除するバッチ（BAT-003）が存在するため
-- DELETE トリガーは付与せず、ロールによる制御のみとする

-- outbox_events は Append-only + status UPDATE 許可の特殊テーブルのため、
-- このファイルではトリガーを付与しない
-- UPDATE/DELETE の制御は V008 のロール権限設定で行う

-- =====================================================
-- IQC/リワーク系 Append-only テーブル（ADR-011 対応）
-- =====================================================

-- TBL-040: incoming_inspection_measurements（Append-only）
DROP TRIGGER IF EXISTS trg_deny_update_incoming_inspection_measurements ON incoming_inspection_measurements;
CREATE TRIGGER trg_deny_update_incoming_inspection_measurements
    BEFORE UPDATE ON incoming_inspection_measurements
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_incoming_inspection_measurements ON incoming_inspection_measurements;
CREATE TRIGGER trg_deny_delete_incoming_inspection_measurements
    BEFORE DELETE ON incoming_inspection_measurements
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_incoming_inspection_measurements ON incoming_inspection_measurements IS
    'ADR-011: incoming_inspection_measurements は Append-only テーブル。IQC ハッシュチェーンの整合性を保護する。';

-- TBL-041: concession_approvals（Append-only）
DROP TRIGGER IF EXISTS trg_deny_update_concession_approvals ON concession_approvals;
CREATE TRIGGER trg_deny_update_concession_approvals
    BEFORE UPDATE ON concession_approvals
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_concession_approvals ON concession_approvals;
CREATE TRIGGER trg_deny_delete_concession_approvals
    BEFORE DELETE ON concession_approvals
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_concession_approvals ON concession_approvals IS
    'ADR-011: concession_approvals は Append-only テーブル。特採承認の改ざんを防止する。';

-- TBL-044: dispositions（Append-only）
DROP TRIGGER IF EXISTS trg_deny_update_dispositions ON dispositions;
CREATE TRIGGER trg_deny_update_dispositions
    BEFORE UPDATE ON dispositions
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_dispositions ON dispositions;
CREATE TRIGGER trg_deny_delete_dispositions
    BEFORE DELETE ON dispositions
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_dispositions ON dispositions IS
    'ADR-011: dispositions は Append-only テーブル。ディスポジション判定の改ざんを防止する。Two-Person Integrity と組み合わせて第三層防御（ADR-011）を完成させる。';

-- TBL-045: rework_verifications（Append-only）
DROP TRIGGER IF EXISTS trg_deny_update_rework_verifications ON rework_verifications;
CREATE TRIGGER trg_deny_update_rework_verifications
    BEFORE UPDATE ON rework_verifications
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_rework_verifications ON rework_verifications;
CREATE TRIGGER trg_deny_delete_rework_verifications
    BEFORE DELETE ON rework_verifications
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_rework_verifications ON rework_verifications IS
    'ADR-011: rework_verifications は Append-only テーブル。リワーク検証結果の改ざんを防止する。';

-- TBL-047: reworked_lot_labels（Append-only）
DROP TRIGGER IF EXISTS trg_deny_update_reworked_lot_labels ON reworked_lot_labels;
CREATE TRIGGER trg_deny_update_reworked_lot_labels
    BEFORE UPDATE ON reworked_lot_labels
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_reworked_lot_labels ON reworked_lot_labels;
CREATE TRIGGER trg_deny_delete_reworked_lot_labels
    BEFORE DELETE ON reworked_lot_labels
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_reworked_lot_labels ON reworked_lot_labels IS
    'ADR-011: reworked_lot_labels は Append-only テーブル。修正品 QR ラベルの改ざんを防止する。';

-- TBL-049: scrap_records（Append-only）
DROP TRIGGER IF EXISTS trg_deny_update_scrap_records ON scrap_records;
CREATE TRIGGER trg_deny_update_scrap_records
    BEFORE UPDATE ON scrap_records
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_scrap_records ON scrap_records;
CREATE TRIGGER trg_deny_delete_scrap_records
    BEFORE DELETE ON scrap_records
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_scrap_records ON scrap_records IS
    'ADR-011: scrap_records は Append-only テーブル。廃棄記録の改ざんを防止する（廃棄物処理法の観点からも不変性が必要）。';

-- TBL-050: return_to_vendor_records（Append-only）
DROP TRIGGER IF EXISTS trg_deny_update_return_to_vendor_records ON return_to_vendor_records;
CREATE TRIGGER trg_deny_update_return_to_vendor_records
    BEFORE UPDATE ON return_to_vendor_records
    FOR EACH ROW EXECUTE FUNCTION fn_deny_update_append_only();

DROP TRIGGER IF EXISTS trg_deny_delete_return_to_vendor_records ON return_to_vendor_records;
CREATE TRIGGER trg_deny_delete_return_to_vendor_records
    BEFORE DELETE ON return_to_vendor_records
    FOR EACH ROW EXECUTE FUNCTION fn_deny_delete_append_only();

COMMENT ON TRIGGER trg_deny_update_return_to_vendor_records ON return_to_vendor_records IS
    'ADR-011: return_to_vendor_records は Append-only テーブル。返品記録の改ざんを防止する（追跡番号等の不変性を保証）。';
