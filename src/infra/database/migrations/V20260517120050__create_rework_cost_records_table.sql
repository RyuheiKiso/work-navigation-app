-- V20260517120050__create_rework_cost_records_table.sql
-- TBL-048 rework_cost_records: リワークコスト集計テーブル（BAT-011 が日次上書き）。ハッシュチェーン非対象（ADR-011 §除外対象）。

-- EN-037 ReworkCostRecord — BAT-011 が日次で集計・上書きする（唯一の上書き可能集計テーブル）。
-- ハッシュチェーン非対象（ADR-011 §除外対象）。
-- NOTE: BAT-011 が daily で aggregate して UPSERT するため、全列の UPDATE が許可される例外テーブル。
CREATE TABLE IF NOT EXISTS rework_cost_records (
    -- コストレコード識別子。UUID v4。gen_random_uuid() で自動生成。
    record_id                       UUID          NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付くリワークレコードの識別子。1 rework_id に 1 件（UNIQUE 制約）。
    rework_id                       UUID          NOT NULL,
    -- 追加工数（秒）。0 以上の値のみ許可（CHECK 制約）。BAT-011 が work_events から集計する。
    additional_labor_seconds        INTEGER       NOT NULL DEFAULT 0,
    -- 追加材料費（円）。0 以上の値のみ許可（CHECK 制約）。
    additional_material_cost_yen    NUMERIC(12,2) NOT NULL DEFAULT 0,
    -- スクラップ損失額（円）。0 以上の値のみ許可（CHECK 制約）。
    scrap_loss_yen                  NUMERIC(12,2) NOT NULL DEFAULT 0,
    -- 最終集計時刻。BAT-011 実行時刻を記録する。NULL = 未集計。
    aggregated_at                   TIMESTAMPTZ   NULL,
    -- レコード作成時刻。
    created_at                      TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    -- レコード最終更新時刻（BAT-011 更新時に更新する）。
    updated_at                      TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    -- 主キー
    CONSTRAINT pk_rework_cost_records PRIMARY KEY (record_id),
    -- 1 rework_id に 1 件のコストレコードを保証する（UNIQUE 制約）。
    CONSTRAINT uq_rework_cost_records_rework UNIQUE (rework_id),
    -- reworks への外部キー。
    CONSTRAINT fk_rcr_rework FOREIGN KEY (rework_id)
        REFERENCES reworks (rework_id) ON DELETE RESTRICT,
    -- additional_labor_seconds は 0 以上の値のみ許可する。
    CONSTRAINT ck_rcr_labor_non_negative CHECK (additional_labor_seconds >= 0),
    -- additional_material_cost_yen は 0 以上の値のみ許可する。
    CONSTRAINT ck_rcr_material_non_negative CHECK (additional_material_cost_yen >= 0),
    -- scrap_loss_yen は 0 以上の値のみ許可する。
    CONSTRAINT ck_rcr_scrap_non_negative CHECK (scrap_loss_yen >= 0)
);

COMMENT ON TABLE  rework_cost_records IS 'EN-037 ReworkCostRecord — BAT-011 が日次で集計・上書きする（唯一の上書き可能集計テーブル）。ハッシュチェーン非対象（ADR-011）。';
COMMENT ON COLUMN rework_cost_records.additional_labor_seconds IS '追加工数（秒）。BAT-011 が work_events の timestamp_client/server 差分から集計する。';
COMMENT ON COLUMN rework_cost_records.aggregated_at IS '最終集計時刻。BAT-011 の実行時刻を記録する。NULL = 未集計（レコード作成直後）。';
