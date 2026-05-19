-- V20260517120042__create_incoming_inspection_measurements_table.sql
-- TBL-040 incoming_inspection_measurements: サンプル測定値明細（Append-only）。ハッシュチェーン列込み（ADR-011）。

-- EN-030 詳細 — サンプル測定値明細（Append-only）。
-- qc_case_id = inspection_id でハッシュチェーンを構成する（ADR-011）。
-- NOTE: 二段適用方針により qc_case_id / prev_hash / content_hash は NULL 許容で作成する。
CREATE TABLE IF NOT EXISTS incoming_inspection_measurements (
    -- 測定値識別子。UUID v4。gen_random_uuid() で自動生成。
    measurement_id      UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付く受入検査の識別子。
    inspection_id       UUID            NOT NULL,
    -- サンプル番号。1 以上の値のみ許可（CHECK 制約）。
    sample_no           INTEGER         NOT NULL,
    -- 測定値。NULL 許容（官能検査など測定値なしの場合）。
    measured_value      NUMERIC(18,6)   NULL,
    -- 欠陥フラグ。TRUE = 不良品（defect として記録）。
    defect_flag         BOOLEAN         NOT NULL DEFAULT FALSE,
    -- 証拠写真の識別子。NULL 許容（証拠写真なしの場合）。
    evidence_file_id    UUID            NULL,
    -- 測定時刻。
    measured_at         TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- ハッシュチェーン単位 ID（= inspection_id）。同一検査内のサンプル列を時系列で連結する。NULL 許容（二段適用方針）。
    qc_case_id          UUID            NULL,
    -- 前ブロックの content_hash（同一 qc_case_id 内の直前レコード。初回は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash           CHAR(64)        NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash        CHAR(64)        NULL,

    -- 主キー
    CONSTRAINT pk_incoming_insp_meas PRIMARY KEY (measurement_id),
    -- incoming_inspections への外部キー。
    CONSTRAINT fk_insp_meas_inspection FOREIGN KEY (inspection_id)
        REFERENCES incoming_inspections (inspection_id) ON DELETE RESTRICT,
    -- evidence_files への外部キー。NULL 許容（証拠写真なしの場合）。
    CONSTRAINT fk_insp_meas_evidence FOREIGN KEY (evidence_file_id)
        REFERENCES evidence_files (evidence_id) ON DELETE RESTRICT,
    -- sample_no は 1 以上の値のみ許可する。
    CONSTRAINT ck_insp_meas_sample_no CHECK (sample_no >= 1),
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_insp_meas_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  incoming_inspection_measurements IS 'EN-030 詳細 — Append-only。サンプル 1 個の測定値・証拠写真を記録する。UPDATE/DELETE 禁止。qc_case_id = inspection_id でチェーン（ADR-011）。';
COMMENT ON COLUMN incoming_inspection_measurements.qc_case_id IS 'ハッシュチェーン単位 ID（= inspection_id）。同一検査内のサンプル列を時系列で連結する。';
COMMENT ON COLUMN incoming_inspection_measurements.prev_hash IS '前ブロックの content_hash（同一 qc_case_id 内の直前レコード。初回は "0"×64）。';
COMMENT ON COLUMN incoming_inspection_measurements.content_hash IS '本レコードの SHA-256（inspection_id / sample_no / measured_value / defect_flag / measured_at の canonical JSON）。';

-- IDX-023: 検査ヘッダ別の測定値明細取得
CREATE INDEX IF NOT EXISTS idx_insp_meas_inspection ON incoming_inspection_measurements (inspection_id);

-- Append-only 強制: UPDATE/DELETE を禁止する
REVOKE UPDATE, DELETE ON incoming_inspection_measurements FROM PUBLIC;
REVOKE UPDATE, DELETE ON incoming_inspection_measurements FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON incoming_inspection_measurements TO app_event_writer;
