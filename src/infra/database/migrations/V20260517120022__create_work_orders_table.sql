-- V20260517120022__create_work_orders_table.sql
-- TBL-006 work_orders: ワークオーダー（製造指示）。ERP から同期または手動発行。永続保存。

-- EN-011 WorkOrder — ワークオーダーマスタ（ERP 連携または手動発行）
CREATE TABLE IF NOT EXISTS work_orders (
    -- ワークオーダー識別子。UUID v7（時系列順）。Rust 側で生成する。
    work_order_id    UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- ERP 連携時の外部ワークオーダー番号。変更不可の公開識別子。
    work_order_code  VARCHAR(128) NOT NULL,
    -- 対象ロットの識別子。NULL は単体 SOP 実行（ロット不明または無関係）。
    lot_id           UUID         NULL,
    -- 対象 SOP の識別子。
    sop_id           UUID         NOT NULL,
    -- 計画数量。1 以上の正整数。
    quantity_planned  INTEGER     NOT NULL,
    -- 実績数量。0 以上の非負整数。
    quantity_actual   INTEGER     NOT NULL DEFAULT 0,
    -- ワークオーダーステータス。5 値のみ許可する。
    status           VARCHAR(16)  NOT NULL DEFAULT 'OPEN',
    -- 完了期日。NULL は期日なし。
    due_date         DATE         NULL,
    -- レコード作成日時。
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_orders PRIMARY KEY (work_order_id),
    -- work_order_code は UNIQUE 制約（ERP 連携時の重複防止）。
    CONSTRAINT uq_work_orders_code UNIQUE (work_order_code),
    -- lots テーブルへの外部キー。ロット削除時は RESTRICT。
    CONSTRAINT fk_work_orders_lot FOREIGN KEY (lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    -- sops テーブルへの外部キー。SOP 削除時は RESTRICT。
    CONSTRAINT fk_work_orders_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    -- status は 5 値のみ許可する。
    CONSTRAINT ck_work_orders_status CHECK (
        status IN ('OPEN', 'IN_PROGRESS', 'COMPLETED', 'CANCELLED', 'ON_HOLD')
    ),
    -- quantity_planned は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_work_orders_quantity_planned_positive CHECK (quantity_planned > 0),
    -- quantity_actual は 0 以上の非負整数のみ許可する。
    CONSTRAINT ck_work_orders_quantity_actual_non_negative CHECK (quantity_actual >= 0)
);

COMMENT ON TABLE  work_orders IS 'EN-011 WorkOrder — ワークオーダー（製造指示）。ERP から同期または手動発行。work_executions から FK 参照される。永続保存。';
COMMENT ON COLUMN work_orders.work_order_id    IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN work_orders.work_order_code  IS 'ERP 連携時の外部ワークオーダー番号。変更不可の公開識別子。';
COMMENT ON COLUMN work_orders.lot_id           IS '対象ロットの識別子。NULL は単体 SOP 実行（ロット不明または無関係）。';
COMMENT ON COLUMN work_orders.sop_id           IS '対象 SOP の sop_id。';
COMMENT ON COLUMN work_orders.quantity_planned IS '計画数量。1 以上の正整数。';
COMMENT ON COLUMN work_orders.quantity_actual  IS '実績数量。0 以上の非負整数。';
COMMENT ON COLUMN work_orders.status           IS 'OPEN / IN_PROGRESS / COMPLETED / CANCELLED / ON_HOLD の 5 値。';
COMMENT ON COLUMN work_orders.due_date         IS '完了期日。NULL は期日なし。';
COMMENT ON COLUMN work_orders.created_at       IS 'レコード作成日時。';
COMMENT ON COLUMN work_orders.updated_at       IS 'レコード最終更新日時。';
