-- V20260517120014__create_instruments_table.sql
-- TBL-026 instruments: 計測器マスタ（校正管理付き）。ALCOA+ Accurate 要件。

-- EN-024 Instrument — 計測器マスタ（校正管理付き）
CREATE TABLE IF NOT EXISTS instruments (
    -- 計測器識別子。UUID v7（時系列順）。Rust 側で生成する。
    instrument_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 計測器コード。変更不可の公開識別子。
    instrument_code       VARCHAR(64) NOT NULL,
    -- 多言語名称 JSONB。{"ja": "計測器名", "en": "Instrument Name"} 形式。ja キーは必須。
    name                  JSONB       NOT NULL,
    -- 計測器種別の自由記述文字列（例: TORQUE_GAUGE, MICROMETER, THERMOMETER）。
    instrument_type       VARCHAR(64) NOT NULL,
    -- 次回校正期限。NULL は校正不要な参考計器。UI 警告トリガとして使用（アプリ層で制御）。
    calibration_due_date  DATE        NULL,
    -- 校正証明書の参照パス（NAS 上のパスまたは URL）。
    calibration_cert_ref  TEXT        NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active             BOOLEAN     NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_instruments PRIMARY KEY (instrument_id),
    CONSTRAINT uq_instruments_code UNIQUE (instrument_code),
    -- name の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_instruments_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  instruments IS 'EN-024 Instrument — 計測器マスタ。校正期限管理（calibration_due_date）を含む。ALCOA+ Accurate 要件。';
COMMENT ON COLUMN instruments.instrument_id        IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN instruments.instrument_code      IS '計測器コード。変更不可の公開識別子。';
COMMENT ON COLUMN instruments.name                 IS '多言語名称 JSONB。{"ja": "計測器名", "en": "Instrument Name"} 形式。ja キーは必須。';
COMMENT ON COLUMN instruments.instrument_type      IS '計測器種別の自由記述文字列（例: TORQUE_GAUGE, MICROMETER, THERMOMETER）。';
COMMENT ON COLUMN instruments.calibration_due_date IS '次回校正期限。NULL は校正不要な参考計器。UI 警告トリガとして使用（アプリ層で制御）。';
COMMENT ON COLUMN instruments.calibration_cert_ref IS '校正証明書の参照パス（NAS 上のパスまたは URL）。';
COMMENT ON COLUMN instruments.is_active            IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN instruments.created_at           IS 'レコード作成日時。';
COMMENT ON COLUMN instruments.updated_at           IS 'レコード最終更新日時。';
