-- V20260517120061__alter_sops_add_sop_type.sql
-- TBL-007 sops に IQC/リワーク対応の sop_type 列を追加する。
-- 権威: docs/04_概要設計/04_データ設計/02_物理テーブル一覧（TBLカタログ）.md §1c「既存テーブル拡張」
-- ENUM 禁止（コーディング規約 §3）のため VARCHAR + CHECK で代替する。

-- SOP の種別を追加する（NORMAL / REWORK / INSPECTION / SCRAP_RECORD / RETURN_RECORD / IQC）
ALTER TABLE sops
    ADD COLUMN IF NOT EXISTS sop_type VARCHAR(32) NOT NULL DEFAULT 'NORMAL';

-- sop_type は 6 値のみ許可する
ALTER TABLE sops
    ADD CONSTRAINT ck_sops_sop_type CHECK (
        sop_type IN ('NORMAL', 'REWORK', 'INSPECTION', 'SCRAP_RECORD', 'RETURN_RECORD', 'IQC')
    );

COMMENT ON COLUMN sops.sop_type IS 'SOP 種別。NORMAL（通常作業）/ REWORK（リワーク）/ INSPECTION（受入検査）/ SCRAP_RECORD（廃棄）/ RETURN_RECORD（返品）/ IQC（受入品質管理）の 6 種。';
