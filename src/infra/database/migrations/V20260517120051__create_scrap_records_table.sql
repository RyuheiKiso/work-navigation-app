-- V20260517120051__create_scrap_records_table.sql
-- TBL-049 scrap_records: 廃棄記録（Append-only）。ハッシュチェーン列込み（ADR-011）。1 rework_id に 1 件。

-- EN-038 ScrapRecord — 廃棄記録（Append-only）。
-- witness_id ≠ 廃棄実施者（API 層で検証）。
-- qc_case_id = rework_id でハッシュチェーンを構成する（ADR-011）。
-- 1 rework_id に 1 件のため genesis 兼末端ブロック。
-- NOTE: 二段適用方針により qc_case_id / prev_hash / content_hash は NULL 許容で作成する。
CREATE TABLE IF NOT EXISTS scrap_records (
    -- 廃棄記録の主キーは rework_id 自体（1 rework_id に 1 件を保証）。
    rework_id               UUID        NOT NULL,
    -- 廃棄物処理票 PDF のファイル識別子（evidence_files テーブルへの参照）。NULL 許容（後添付の場合）。
    waste_manifest_pdf_id   UUID        NULL,
    -- 廃棄物分類コード。例: "産業廃棄物/廃プラスチック"。空文字のみの場合は BAT でアラートを発する。
    waste_classification    VARCHAR(64) NOT NULL DEFAULT '',
    -- 廃棄立会者ユーザーの識別子。廃棄実施者とは異なる人物であることが必要（API 層で検証）。
    witness_id              UUID        NOT NULL,
    -- 廃棄記録時刻。
    recorded_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ハッシュチェーン単位 ID（= rework_id）。1 件のため genesis 兼末端ブロック。NULL 許容（二段適用方針）。
    qc_case_id              UUID        NULL,
    -- 前ブロックの content_hash（genesis は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash               CHAR(64)    NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash            CHAR(64)    NULL,

    -- 主キー（rework_id が PK で 1 rework_id に 1 件を保証する）
    CONSTRAINT pk_scrap_records PRIMARY KEY (rework_id),
    -- reworks への外部キー。廃棄記録はリワークが存在する限り削除しない（RESTRICT）。
    CONSTRAINT fk_sr_rework FOREIGN KEY (rework_id)
        REFERENCES reworks (rework_id) ON DELETE RESTRICT,
    -- 廃棄立会者ユーザーへの外部キー。
    CONSTRAINT fk_sr_witness FOREIGN KEY (witness_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_sr_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  scrap_records IS 'EN-038 ScrapRecord — Append-only。廃棄物処理票・立会者サイン（witness_id ≠ 廃却実施者を API 層で検証）。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN scrap_records.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。scrap_records は 1 rework_id に 1 件のため genesis 兼末端ブロック。';
COMMENT ON COLUMN scrap_records.content_hash IS '本レコードの SHA-256（rework_id / witness_id / recorded_at の canonical JSON）。';

-- Append-only 強制: UPDATE/DELETE を禁止する
REVOKE UPDATE, DELETE ON scrap_records FROM PUBLIC;
REVOKE UPDATE, DELETE ON scrap_records FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON scrap_records TO app_event_writer;
