-- V20260517120012__create_lots_table.sql
-- TBL-024 lots: 製造ロットマスタ。7年以上保存。IQC 拡張列（supplier_id / material_id / qc_status /
-- rework_history_count / parent_lot_id）を最初から含める。

-- EN-021 Lot — 製造ロットマスタ。製造ロット単位のトレーサビリティ記録。
-- IQC 拡張列（supplier_id / material_id / qc_status / rework_history_count / parent_lot_id）を含む最終形。
CREATE TABLE IF NOT EXISTS lots (
    -- ロット識別子。UUID v7（時系列順）。Rust 側で生成する。
    lot_id                UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- ロット番号。ERP 等の外部システムと一致させる。変更不可の公開識別子。
    lot_code              VARCHAR(128) NOT NULL,
    -- 製品識別子。この製品のロットであることを示す。
    product_id            UUID         NOT NULL,
    -- ロットステータス。5 値の列挙。
    lot_status            VARCHAR(16)  NOT NULL DEFAULT 'IN_PRODUCTION',
    -- ロット数量。1 以上の正整数。
    quantity              INTEGER      NOT NULL,
    -- 数量単位。UCUM コード。個数: pcs、kg、m 等。
    unit                  VARCHAR(32)  NOT NULL DEFAULT 'pcs',
    -- ロット製造開始日時。NULL は製造未着手。
    lot_started_at        TIMESTAMPTZ  NULL,
    -- ロット完了・クローズ日時。NULL は未完了。
    lot_closed_at         TIMESTAMPTZ  NULL,
    -- レコード作成日時。
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- IQC 拡張列: 入荷元仕入先の supplier_id（NULL は製造品）。
    supplier_id           UUID         NULL,
    -- IQC 拡張列: 材料・部品の material_id（NULL は製造品）。
    material_id           UUID         NULL,
    -- IQC 拡張列: 受入検査 QC ステータス。lot_qc_states（TBL-042）と連動して後工程ゲートに使用する。
    qc_status             VARCHAR(24)  NOT NULL DEFAULT 'PENDING',
    -- IQC 拡張列: このロットのリワーク実施回数。BAT-011 が日次集計で更新する。
    rework_history_count  INTEGER      NOT NULL DEFAULT 0,
    -- IQC 拡張列: リワーク後の新ロット発行時に元ロットの lot_id を設定する。自己参照 FK（自ループ禁止制約付き）。
    parent_lot_id         UUID         NULL,

    CONSTRAINT pk_lots PRIMARY KEY (lot_id),
    CONSTRAINT uq_lots_code UNIQUE (lot_code),
    -- products テーブルへの外部キー。製品削除時は RESTRICT。
    CONSTRAINT fk_lots_product FOREIGN KEY (product_id)
        REFERENCES products (product_id) ON DELETE RESTRICT,
    -- suppliers テーブルへの外部キー。仕入先削除時は RESTRICT。
    CONSTRAINT fk_lots_supplier FOREIGN KEY (supplier_id)
        REFERENCES suppliers (supplier_id) ON DELETE RESTRICT,
    -- materials テーブルへの外部キー。材料削除時は RESTRICT。
    CONSTRAINT fk_lots_material FOREIGN KEY (material_id)
        REFERENCES materials (material_id) ON DELETE RESTRICT,
    -- 自己参照外部キー。親ロットの lot_id を参照する。
    CONSTRAINT fk_lots_parent_lot FOREIGN KEY (parent_lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    -- lot_status は 5 値のみ許可する。
    CONSTRAINT ck_lots_status CHECK (
        lot_status IN ('IN_PRODUCTION', 'ON_HOLD', 'COMPLETED', 'REJECTED', 'SCRAPPED')
    ),
    -- qc_status は 8 値のみ許可する。
    CONSTRAINT ck_lots_qc_status CHECK (
        qc_status IN ('PENDING', 'INSPECTING', 'PASSED', 'CONDITIONAL_PASS', 'SCREENING_REQUIRED', 'REJECTED', 'SCRAPPED', 'RETURNED')
    ),
    -- quantity は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_lots_quantity_positive CHECK (quantity > 0),
    -- rework_history_count は 0 以上の非負整数のみ許可する。
    CONSTRAINT ck_lots_rework_history_non_negative CHECK (rework_history_count >= 0),
    -- 自己参照は禁止する（lot_id = parent_lot_id は不正）。
    CONSTRAINT ck_lots_parent_lot_no_self CHECK (parent_lot_id IS NULL OR parent_lot_id <> lot_id),
    -- lot_closed_at が設定されている場合は lot_started_at も必須かつ closed >= started を保証する。
    CONSTRAINT ck_lots_closed_after_started CHECK (
        NOT (lot_closed_at IS NOT NULL AND lot_started_at IS NULL)
        AND NOT (lot_closed_at IS NOT NULL AND lot_closed_at < lot_started_at)
    )
);

COMMENT ON TABLE  lots IS 'EN-021 Lot — 製造ロットマスタ。7年以上保存。lot_code は ERP/MES 連携時の外部識別子。IQC 拡張列（supplier_id / material_id / qc_status / rework_history_count / parent_lot_id）を含む最終形。';
COMMENT ON COLUMN lots.lot_id               IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN lots.lot_code             IS 'ロット番号。ERP 等の外部システムと一致させる。変更不可の公開識別子。';
COMMENT ON COLUMN lots.product_id           IS '製品識別子。この製品のロットであることを示す。';
COMMENT ON COLUMN lots.lot_status           IS 'IN_PRODUCTION / ON_HOLD / COMPLETED / REJECTED / SCRAPPED の 5 値。';
COMMENT ON COLUMN lots.quantity             IS 'ロット数量。1 以上の正整数。';
COMMENT ON COLUMN lots.unit                 IS 'UCUM コード。個数: pcs、kg、m 等。';
COMMENT ON COLUMN lots.lot_started_at       IS 'ロット製造開始日時。NULL は製造未着手。';
COMMENT ON COLUMN lots.lot_closed_at        IS 'ロット完了・クローズ日時。NULL は未完了。';
COMMENT ON COLUMN lots.supplier_id          IS 'IQC 拡張列。入荷元仕入先の supplier_id（NULL は製造品）。';
COMMENT ON COLUMN lots.material_id          IS 'IQC 拡張列。材料・部品の material_id（NULL は製造品）。';
COMMENT ON COLUMN lots.qc_status            IS 'IQC 拡張列。受入検査 QC ステータス。lot_qc_states（TBL-042）と連動して後工程ゲートに使用する。';
COMMENT ON COLUMN lots.rework_history_count IS 'IQC 拡張列。このロットのリワーク実施回数。BAT-011 が日次集計で更新する。';
COMMENT ON COLUMN lots.parent_lot_id        IS 'IQC 拡張列。リワーク後の新ロット発行時に元ロットの lot_id を設定する。自己参照 FK（自ループ禁止制約付き）。';
COMMENT ON COLUMN lots.created_at           IS 'レコード作成日時。';
COMMENT ON COLUMN lots.updated_at           IS 'レコード最終更新日時。';
