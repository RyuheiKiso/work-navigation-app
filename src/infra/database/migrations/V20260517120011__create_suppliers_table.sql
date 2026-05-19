-- V20260517120011__create_suppliers_table.sql
-- TBL-037 suppliers: 仕入先マスタ（版管理）。物理削除禁止。

-- EN-029 Supplier — 仕入先マスタ（版管理）
CREATE TABLE IF NOT EXISTS suppliers (
    -- 仕入先識別子。UUID v7（時系列順）。Rust 側で生成する。
    supplier_id     UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 仕入先コード（購買システム連携キー）。UNIQUE 制約。
    supplier_code   VARCHAR(64)     NOT NULL,
    -- 仕入先名称。空白のみ禁止（trim 後 1 文字以上必須）。
    name            VARCHAR(256)    NOT NULL,
    -- 仕入先住所。
    address         TEXT            NOT NULL DEFAULT '',
    -- 仕入先連絡先（電話番号・メールアドレス等）。
    contact         VARCHAR(256)    NOT NULL DEFAULT '',
    -- レコードバージョン番号。1 以上の正整数。更新のたびにインクリメントする。
    version         INTEGER         NOT NULL DEFAULT 1,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active       BOOLEAN         NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_suppliers PRIMARY KEY (supplier_id),
    CONSTRAINT uq_suppliers_code UNIQUE (supplier_code),
    -- name は空白のみ禁止する（trim 後 1 文字以上必須）。
    CONSTRAINT ck_suppliers_name_not_empty CHECK (length(trim(name)) > 0),
    -- version は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_suppliers_version CHECK (version >= 1)
);

COMMENT ON TABLE  suppliers IS 'EN-029 Supplier — 仕入先マスタ。物理削除禁止。';
COMMENT ON COLUMN suppliers.supplier_id   IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN suppliers.supplier_code IS '仕入先コード（購買システム連携キー）。UNIQUE 制約。';
COMMENT ON COLUMN suppliers.name          IS '仕入先名称。空白のみ禁止（trim 後 1 文字以上必須）。';
COMMENT ON COLUMN suppliers.address       IS '仕入先住所。';
COMMENT ON COLUMN suppliers.contact       IS '仕入先連絡先（電話番号・メールアドレス等）。';
COMMENT ON COLUMN suppliers.version       IS 'レコードバージョン番号。1 以上の正整数。更新のたびにインクリメントする。';
COMMENT ON COLUMN suppliers.is_active     IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN suppliers.created_at    IS 'レコード作成日時。';
COMMENT ON COLUMN suppliers.updated_at    IS 'レコード最終更新日時。';
