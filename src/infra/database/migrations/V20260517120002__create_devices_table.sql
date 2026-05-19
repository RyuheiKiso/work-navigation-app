-- V20260517120002__create_devices_table.sql
-- TBL-033 devices: ハンディ端末デバイスマスタ。ALCOA+ Attributable 要件（どの端末で記録されたかを特定する）。

-- EN-023 Device — ハンディ端末デバイスマスタ
CREATE TABLE IF NOT EXISTS devices (
    -- デバイス識別子。UUID v7（時系列順）。Rust 側で生成する。
    device_id      UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- 端末製造番号。デバイスの物理識別子。UNIQUE 制約。
    serial_number  VARCHAR(128) NOT NULL,
    -- デバイス種別。android / ios / windows の 3 値（CLAUDE.md 対応 OS と一致）。
    device_type    VARCHAR(16)  NOT NULL,
    -- 有効フラグ。廃棄・無効化時に FALSE に設定する。物理 DELETE は禁止。
    is_active      BOOLEAN      NOT NULL DEFAULT TRUE,
    -- デバイス登録日時。
    registered_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_devices PRIMARY KEY (device_id),
    CONSTRAINT uq_devices_serial UNIQUE (serial_number),
    -- デバイス種別は android / ios / windows の 3 値のみ許可する。
    CONSTRAINT ck_devices_type CHECK (
        device_type IN ('android', 'ios', 'windows')
    )
);

COMMENT ON TABLE  devices IS 'EN-023 Device — ハンディ端末デバイスマスタ。work_events.terminal_id の外部キー参照元。ALCOA+ Attributable 要件（どの端末で記録されたかを特定する）。';
COMMENT ON COLUMN devices.device_id     IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN devices.serial_number IS '端末製造番号。デバイスの物理識別子。変更不可の公開識別子。';
COMMENT ON COLUMN devices.device_type   IS 'android / ios / windows の 3 値（CLAUDE.md 対応 OS と一致）。';
COMMENT ON COLUMN devices.is_active     IS '廃棄・無効化時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN devices.registered_at IS 'デバイス初回登録日時。';
COMMENT ON COLUMN devices.updated_at    IS 'レコード最終更新日時。';
