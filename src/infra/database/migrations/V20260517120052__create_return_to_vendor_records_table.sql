-- V20260517120052__create_return_to_vendor_records_table.sql
-- TBL-050 return_to_vendor_records: 仕入先返品記録（Append-only）。ハッシュチェーン列込み（ADR-011）。追跡番号必須。

-- EN-039 ReturnToVendorRecord — 仕入先返品記録（Append-only）。
-- 追跡番号（tracking_no）必須（ERR-BIZ-025）。
-- qc_case_id = rework_id でハッシュチェーンを構成する（ADR-011）。
-- 1 rework_id に 1 件のため genesis 兼末端ブロック。
-- NOTE: 二段適用方針により qc_case_id / prev_hash / content_hash は NULL 許容で作成する。
CREATE TABLE IF NOT EXISTS return_to_vendor_records (
    -- 返品記録の主キーは rework_id 自体（1 rework_id に 1 件を保証）。
    rework_id               UUID         NOT NULL,
    -- 返品伝票 PDF のファイル識別子（evidence_files テーブルへの参照）。NULL 許容（後添付の場合）。
    return_invoice_pdf_id   UUID         NULL,
    -- 返品先業者（仕入先）の識別子。suppliers テーブルへの参照。
    vendor_id               UUID         NOT NULL,
    -- 配送業者名。例: "ヤマト運輸"。空文字許容（情報なしの場合は BAT でアラートを発する）。
    carrier                 VARCHAR(128) NOT NULL DEFAULT '',
    -- 追跡番号。空白のみは禁止（ERR-BIZ-025）。
    tracking_no             VARCHAR(128) NOT NULL,
    -- 返品実施時刻。
    returned_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- ハッシュチェーン単位 ID（= rework_id）。1 件のため genesis 兼末端ブロック。NULL 許容（二段適用方針）。
    qc_case_id              UUID         NULL,
    -- 前ブロックの content_hash（genesis は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash               CHAR(64)     NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash            CHAR(64)     NULL,

    -- 主キー（rework_id が PK で 1 rework_id に 1 件を保証する）
    CONSTRAINT pk_return_to_vendor PRIMARY KEY (rework_id),
    -- reworks への外部キー。返品記録はリワークが存在する限り削除しない（RESTRICT）。
    CONSTRAINT fk_rtv_rework FOREIGN KEY (rework_id)
        REFERENCES reworks (rework_id) ON DELETE RESTRICT,
    -- 返品先業者への外部キー。
    CONSTRAINT fk_rtv_vendor FOREIGN KEY (vendor_id)
        REFERENCES suppliers (supplier_id) ON DELETE RESTRICT,
    -- tracking_no は空白文字のみを禁止する（ERR-BIZ-025 準拠）。
    CONSTRAINT ck_rtv_tracking CHECK (length(trim(tracking_no)) > 0),
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_rtv_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  return_to_vendor_records IS 'EN-039 ReturnToVendorRecord — Append-only。追跡番号（tracking_no）必須（ERR-BIZ-025）。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN return_to_vendor_records.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。return_to_vendor_records は 1 rework_id に 1 件のため genesis 兼末端ブロック。';
COMMENT ON COLUMN return_to_vendor_records.content_hash IS '本レコードの SHA-256（rework_id / vendor_id / tracking_no / returned_at の canonical JSON）。';

-- Append-only 強制: UPDATE/DELETE を禁止する
REVOKE UPDATE, DELETE ON return_to_vendor_records FROM PUBLIC;
REVOKE UPDATE, DELETE ON return_to_vendor_records FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON return_to_vendor_records TO app_event_writer;
