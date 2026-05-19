-- V20260517120064__alter_capas_add_rework_summary.sql
-- TBL-014 capas に rework_summary 列を追加する。
-- 権威: docs/04_概要設計/04_データ設計/02_物理テーブル一覧（TBLカタログ）.md §1c「既存テーブル拡張」
-- rework_summary はリワーク作業の概要情報を JSONB で保持する。FK なし（参照整合性は JSONB で柔軟に対応）。

-- リワーク作業概要情報を JSONB で追加する
ALTER TABLE capas
    ADD COLUMN IF NOT EXISTS rework_summary JSONB NULL;

-- rework_summary が設定されている場合はオブジェクト型であることを保証する
ALTER TABLE capas
    ADD CONSTRAINT ck_capas_rework_summary_object CHECK (
        rework_summary IS NULL OR jsonb_typeof(rework_summary) = 'object'
    );

COMMENT ON COLUMN capas.rework_summary IS 'リワーク作業概要（JSONB）。rework_id・実施日・要因分析結果等を格納する。NULL は CAPA がリワークに関連しないことを示す。';
