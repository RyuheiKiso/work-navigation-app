-- V20260517120009__create_products_table.sql
-- TBL-023 products: 製品マスタ。product_code は外部システム（ERP 等）に合わせた任意形式を許容する。

-- EN-007 Product — 製品マスタ
CREATE TABLE IF NOT EXISTS products (
    -- 製品識別子。UUID v7（時系列順）。Rust 側で生成する。
    product_id    UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- 製品コード。外部システム形式を許容するため長め（128）に設定。変更不可の公開識別子。
    product_code  VARCHAR(128) NOT NULL,
    -- 多言語名称 JSONB。{"ja": "製品名", "en": "Product Name"} 形式。ja キーは必須。
    name          JSONB        NOT NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_products PRIMARY KEY (product_id),
    CONSTRAINT uq_products_code UNIQUE (product_code),
    -- name の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_products_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  products IS 'EN-007 Product — 製品マスタ。product_code は外部システム（ERP 等）に合わせた任意形式を許容する。';
COMMENT ON COLUMN products.product_id   IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN products.product_code IS '外部システム形式を許容するため長め（128）に設定。変更不可の公開識別子。';
COMMENT ON COLUMN products.name         IS '多言語名称 JSONB。{"ja": "製品名", "en": "Product Name"} 形式。ja キーは必須。';
COMMENT ON COLUMN products.is_active    IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN products.created_at   IS 'レコード作成日時。';
COMMENT ON COLUMN products.updated_at   IS 'レコード最終更新日時。';
