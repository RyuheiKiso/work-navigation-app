-- V20260517120007__create_processes_table.sql
-- TBL-021 processes: プロセス（工程群）マスタ。process_code 形式: {英大文字2-4字}-{連番3桁}。

-- EN-005 Process — プロセス（工程群）マスタ
CREATE TABLE IF NOT EXISTS processes (
    -- プロセス識別子。UUID v7（時系列順）。Rust 側で生成する。
    process_id      UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- プロセスコード。形式: {英大文字2-4字}-{連番3桁}（例: ASS-001）。変更不可の公開識別子。
    process_code    VARCHAR(64) NOT NULL,
    -- 多言語名称 JSONB。{"ja": "組立", "en": "Assembly"} 形式。ja キーは必須。
    name            JSONB       NOT NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_processes PRIMARY KEY (process_id),
    CONSTRAINT uq_processes_code UNIQUE (process_code),
    -- process_code は {英大文字2-4字}-{連番3桁} の形式のみ許可する（正規表現チェック）。
    CONSTRAINT ck_processes_code_format CHECK (
        process_code ~ '^[A-Z]{2,4}-[0-9]{3}$'
    ),
    -- name の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_processes_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  processes IS 'EN-005 Process — プロセスマスタ。process_code 形式: {英大文字2-4字}-{連番3桁}。例: ASS-001。';
COMMENT ON COLUMN processes.process_id   IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN processes.process_code IS 'プロセスコード。形式: {英大文字2-4字}-{連番3桁}。変更不可の公開識別子。';
COMMENT ON COLUMN processes.name         IS '多言語名称 JSONB。{"ja": "組立", "en": "Assembly"} 形式。ja キーは必須。';
COMMENT ON COLUMN processes.is_active    IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN processes.created_at   IS 'レコード作成日時。';
COMMENT ON COLUMN processes.updated_at   IS 'レコード最終更新日時。';
