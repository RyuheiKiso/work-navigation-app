-- V20260517120021__create_work_patterns_table.sql
-- TBL-028 work_patterns: external_key_bindings の解決先。外部 ERP ロット → 内部 SOP を仲介する。
-- NOTE: factory_id は予約フィールド。ver1.0.0 では定数 UUID '00000000-0000-7000-8000-000000000001' を使用する。
-- NOTE: factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。

-- EN-025 WorkPattern — external_key_bindings の解決先。外部 ID と SOP の中間エンティティ。
CREATE TABLE IF NOT EXISTS work_patterns (
    -- ワークパターン識別子。UUID v7（時系列順）。Rust 側で生成する。
    work_pattern_id  UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 対応する SOP の識別子。
    sop_id           UUID        NOT NULL,
    -- 対応するオペレーションの識別子。
    operation_id     UUID        NOT NULL,
    -- 多言語パターン名 JSONB。{"ja": "パターン名", "en": "Pattern Name"} 形式。ja キーは必須。
    pattern_name     JSONB       NOT NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    -- 工場識別子（予約フィールド）。ver1.0.0 では定数 UUID を使用する。将来のマルチファクトリー拡張時に使用する。
    factory_id       UUID        NOT NULL DEFAULT '00000000-0000-7000-8000-000000000001',
    -- レコード作成日時。
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_patterns PRIMARY KEY (work_pattern_id),
    -- sops テーブルへの外部キー。SOP 削除時は RESTRICT。
    CONSTRAINT fk_work_patterns_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    -- operations テーブルへの外部キー。オペレーション削除時は RESTRICT。
    CONSTRAINT fk_work_patterns_operation FOREIGN KEY (operation_id)
        REFERENCES operations (operation_id) ON DELETE RESTRICT,
    -- pattern_name の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_work_patterns_name_has_ja CHECK (
        jsonb_typeof(pattern_name -> 'ja') = 'string'
        AND length(pattern_name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  work_patterns IS 'EN-025 WorkPattern — external_key_bindings（TBL-027）の解決先。外部 ERP ロット → 内部 SOP を仲介する。';
COMMENT ON COLUMN work_patterns.work_pattern_id IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN work_patterns.sop_id          IS '対応する SOP の sop_id。';
COMMENT ON COLUMN work_patterns.operation_id    IS '対応するオペレーションの operation_id。';
COMMENT ON COLUMN work_patterns.pattern_name    IS '多言語パターン名 JSONB。{"ja": "パターン名", "en": "Pattern Name"} 形式。ja キーは必須。';
COMMENT ON COLUMN work_patterns.is_active       IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN work_patterns.factory_id      IS 'ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。';
COMMENT ON COLUMN work_patterns.created_at      IS 'レコード作成日時。';
COMMENT ON COLUMN work_patterns.updated_at      IS 'レコード最終更新日時。';
