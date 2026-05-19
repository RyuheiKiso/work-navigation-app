-- V20260517120062__alter_work_executions_add_iqc_columns.sql
-- TBL-005 work_executions に IQC/リワーク対応の拡張列を追加する。
-- 権威: docs/04_概要設計/04_データ設計/02_物理テーブル一覧（TBLカタログ）.md §1c「既存テーブル拡張」
-- 前提: reworks（V45）および incoming_inspections（V41）が作成済みであること。
-- ENUM 禁止（コーディング規約 §3）のため VARCHAR + CHECK で代替する。

-- 実行種別列を追加する（NORMAL / REWORK / VERIFICATION / IQC）
ALTER TABLE work_executions
    ADD COLUMN IF NOT EXISTS execution_type     VARCHAR(16) NOT NULL DEFAULT 'NORMAL',
    -- リワーク元のリワークセッション識別子。通常作業では NULL。
    ADD COLUMN IF NOT EXISTS source_rework_id   UUID        NULL,
    -- IQC 受入検査起源の検査識別子。IQC 種別時のみ設定する。通常作業では NULL。
    ADD COLUMN IF NOT EXISTS source_inspection_id UUID      NULL;

-- execution_type は 4 値のみ許可する
ALTER TABLE work_executions
    ADD CONSTRAINT ck_work_executions_execution_type CHECK (
        execution_type IN ('NORMAL', 'REWORK', 'VERIFICATION', 'IQC')
    );

-- source_rework_id: reworks テーブルへの外部キー
ALTER TABLE work_executions
    ADD CONSTRAINT fk_work_executions_source_rework FOREIGN KEY (source_rework_id)
        REFERENCES reworks (rework_id) ON DELETE RESTRICT;

-- source_inspection_id: incoming_inspections テーブルへの外部キー
ALTER TABLE work_executions
    ADD CONSTRAINT fk_work_executions_source_inspection FOREIGN KEY (source_inspection_id)
        REFERENCES incoming_inspections (inspection_id) ON DELETE RESTRICT;

COMMENT ON COLUMN work_executions.execution_type      IS '実行種別。NORMAL（通常）/ REWORK（リワーク）/ VERIFICATION（検証）/ IQC（受入検査）の 4 種。';
COMMENT ON COLUMN work_executions.source_rework_id    IS 'リワーク元の rework_id。execution_type=REWORK/VERIFICATION 時に設定する。';
COMMENT ON COLUMN work_executions.source_inspection_id IS 'IQC 受入検査起源の inspection_id。execution_type=IQC 時に設定する。';
