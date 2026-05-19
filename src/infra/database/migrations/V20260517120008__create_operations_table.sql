-- V20260517120008__create_operations_table.sql
-- TBL-022 operations: オペレーション（工程）マスタ。process の子。

-- EN-006 Operation — オペレーション（工程）マスタ。process の子。
CREATE TABLE IF NOT EXISTS operations (
    -- オペレーション識別子。UUID v7（時系列順）。Rust 側で生成する。
    operation_id     UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 所属するプロセスの識別子。
    process_id       UUID        NOT NULL,
    -- オペレーションコード。形式: {process_code}-{連番3桁}（例: ASS-001-003）。
    operation_code   VARCHAR(64) NOT NULL,
    -- 多言語名称 JSONB。{"ja": "ボルト締付", "en": "Bolt Tightening"} 形式。ja キーは必須。
    name             JSONB       NOT NULL,
    -- プロセス内での表示順。1 以上の正整数。
    sequence_number  INTEGER     NOT NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_operations PRIMARY KEY (operation_id),
    CONSTRAINT uq_operations_code UNIQUE (operation_code),
    -- processes テーブルへの外部キー。プロセス削除時は RESTRICT。
    CONSTRAINT fk_operations_process FOREIGN KEY (process_id)
        REFERENCES processes (process_id) ON DELETE RESTRICT,
    -- sequence_number は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_operations_sequence_positive CHECK (sequence_number > 0),
    -- name の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_operations_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  operations IS 'EN-006 Operation — オペレーションマスタ。operation_code 形式: {process_code}-{連番3桁}。例: ASS-001-003。';
COMMENT ON COLUMN operations.operation_id    IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN operations.process_id      IS '所属するプロセスの process_id。';
COMMENT ON COLUMN operations.operation_code  IS 'オペレーションコード。形式: {process_code}-{連番3桁}。変更不可の公開識別子。';
COMMENT ON COLUMN operations.name            IS '多言語名称 JSONB。{"ja": "ボルト締付", "en": "Bolt Tightening"} 形式。ja キーは必須。';
COMMENT ON COLUMN operations.sequence_number IS 'プロセス内での表示順。1 以上の正整数。';
COMMENT ON COLUMN operations.is_active       IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN operations.created_at      IS 'レコード作成日時。';
COMMENT ON COLUMN operations.updated_at      IS 'レコード最終更新日時。';
