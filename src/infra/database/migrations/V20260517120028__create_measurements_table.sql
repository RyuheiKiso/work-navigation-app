-- V20260517120028__create_measurements_table.sql
-- TBL-010 measurements: 計測値レコード（Append-only）。ALCOA+ Accurate 要件。

-- EN-014 Measurement — 計測値レコード（Append-only）。SI 基本単位で保存し、表示単位は別列に持つ。
CREATE TABLE IF NOT EXISTS measurements (
    -- 計測値識別子。UUID v4。gen_random_uuid() で自動生成。
    measurement_id      UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付く作業イベントの識別子。work_events への DEFERRABLE 外部キー。
    event_id            UUID            NOT NULL,
    -- 使用した計測器の識別子。NULL = 手動入力または計測器不明。
    instrument_id       UUID            NULL,
    -- 使用した校正証明書番号または参照パス。instrument_id が NULL の場合でも記録可能。
    calibration_ref     VARCHAR(128)    NULL,
    -- SI 基本単位での測定値。例: 長さは mm（社内規約）。
    measured_value      NUMERIC(20, 6)  NOT NULL,
    -- 測定不確かさ（拡張不確かさ U）。JCSS 校正証明書の値を転記する。
    uncertainty_u       NUMERIC(10, 6)  NULL,
    -- 包含係数 k。通常 k=2（信頼水準約 95%）。0 より大きい値のみ許可（CHECK 制約）。
    uncertainty_k       NUMERIC(5, 2)   NULL,
    -- UCUM コード。例: mm, Cel（摂氏）, kPa, kg, s。
    unit_ucum           VARCHAR(32)     NOT NULL,
    -- 表示用換算値。UI 表示時に使用する。NULL = measured_value をそのまま表示。
    display_value       NUMERIC(20, 6)  NULL,
    -- 表示用 UCUM コード。NULL = unit_ucum と同一。
    display_unit_ucum   VARCHAR(32)     NULL,
    -- 判定結果。OK / NG / WARNING の 3 種のみ許可（CHECK 制約）。
    judgment            VARCHAR(8)      NOT NULL,
    -- サーバー受信時刻。Append-only 全テーブルの created_at 必須方針に準拠。IDX-020 のインデックス対象列。
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    -- 主キー
    CONSTRAINT pk_measurements PRIMARY KEY (measurement_id),
    -- work_events への外部キー。パーティション化されたテーブルへの参照のため DEFERRABLE INITIALLY DEFERRED。
    CONSTRAINT fk_measurements_event FOREIGN KEY (event_id)
        REFERENCES work_events (event_id, timestamp_server) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- 計測器への外部キー。NULL 許容（計測器不明の場合）。
    CONSTRAINT fk_measurements_instrument FOREIGN KEY (instrument_id)
        REFERENCES instruments (instrument_id) ON DELETE RESTRICT,
    -- judgment は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_measurements_judgment CHECK (
        judgment IN ('OK', 'NG', 'WARNING')
    ),
    -- uncertainty_k は NULL または 0 より大きい値のみ許可する。
    CONSTRAINT ck_measurements_uncertainty_k_positive CHECK (
        uncertainty_k IS NULL OR uncertainty_k > 0
    )
);

COMMENT ON TABLE  measurements IS 'EN-014 Measurement — 計測値レコード。Append-only。SI 基本単位で保存し、表示単位は別列に持つ。7年以上保存。';
COMMENT ON COLUMN measurements.measured_value    IS 'SI 基本単位での測定値。例: 長さは mm（m でなく mm を基準とする社内規約による）。';
COMMENT ON COLUMN measurements.uncertainty_u     IS '測定不確かさ（拡張不確かさ U）。JCSS 校正証明書の値を転記する。';
COMMENT ON COLUMN measurements.uncertainty_k     IS '包含係数 k。通常 k=2（信頼水準約 95%）。';
COMMENT ON COLUMN measurements.unit_ucum         IS 'UCUM コード。例: mm, Cel（摂氏）, kPa, kg, s。';
COMMENT ON COLUMN measurements.calibration_ref   IS '使用した校正証明書番号または参照パス。instrument_id が NULL の場合でも記録可能。';
COMMENT ON COLUMN measurements.created_at        IS 'サーバー受信時刻。Append-only 全テーブルの created_at 必須方針（06_インデックス §1）に準拠。IDX-020 のインデックス対象列。';

-- Append-only 強制: app_event_writer ロールから UPDATE/DELETE を剥奪する
REVOKE UPDATE, DELETE ON measurements FROM PUBLIC;
REVOKE UPDATE, DELETE ON measurements FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON measurements TO app_event_writer;
