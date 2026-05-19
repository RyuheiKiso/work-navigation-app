-- V20260517120063__alter_nonconformities_add_disposition_id.sql
-- TBL-013 nonconformities に disposition_id 列と FK を追加する。
-- 権威: docs/04_概要設計/04_データ設計/02_物理テーブル一覧（TBLカタログ）.md §1c「既存テーブル拡張」
-- 前提: dispositions（V46）が作成済みであること。

-- 処置決定レコードの識別子を追加する（dispositions との関連付け）
ALTER TABLE nonconformities
    ADD COLUMN IF NOT EXISTS disposition_id UUID NULL;

-- dispositions テーブルへの外部キーを追加する
ALTER TABLE nonconformities
    ADD CONSTRAINT fk_nc_disposition FOREIGN KEY (disposition_id)
        REFERENCES dispositions (disposition_id) ON DELETE RESTRICT;

COMMENT ON COLUMN nonconformities.disposition_id IS '処置決定レコードの disposition_id（TBL-044）。処置未決定の場合は NULL。';
