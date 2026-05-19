-- V20260517120049__create_reworked_lot_labels_table.sql
-- TBL-047 reworked_lot_labels: GS1 AI 8003（GIAI）形式の修正品 QR ラベル（Append-only）。ハッシュチェーン列込み（ADR-011）。

-- EN-036 ReworkedLotLabel — GS1 AI 8003（GIAI）+ AI 91 形式の修正品 QR ラベル（Append-only）。
-- qc_case_id = rework_id でハッシュチェーンを構成する（ADR-011）。
-- NOTE: 二段適用方針により qc_case_id / prev_hash / content_hash は NULL 許容で作成する。
CREATE TABLE IF NOT EXISTS reworked_lot_labels (
    -- ラベル識別子。UUID v4。gen_random_uuid() で自動生成。
    label_id            UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付くリワークレコードの識別子。
    rework_id           UUID        NOT NULL,
    -- QR コードのペイロード文字列（GS1 AI 8003 + AI 91 形式）。
    qr_payload          TEXT        NOT NULL,
    -- GS1 個別資産識別子（Global Individual Asset Identifier）。30 文字以内。UNIQUE 制約。
    giai                VARCHAR(30) NOT NULL,
    -- 元ロットの識別子（修正品のトレサビリティ確保のため元ロットを必ず参照する）。
    parent_lot_id       UUID        NOT NULL,
    -- 適用したリワーク SOP バージョンの識別子（時点参照固定）。
    rework_sop_version_id UUID      NOT NULL,
    -- ラベル発行時刻。
    issued_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ラベル発行者ユーザーの識別子。
    issued_by           UUID        NOT NULL,
    -- ハッシュチェーン単位 ID（= rework_id）。同一リワークのラベル列を時系列で連結する。NULL 許容（二段適用方針）。
    qc_case_id          UUID        NULL,
    -- 前ブロックの content_hash（genesis は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash           CHAR(64)    NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash        CHAR(64)    NULL,

    -- 主キー
    CONSTRAINT pk_reworked_lot_labels PRIMARY KEY (label_id),
    -- GIAI は全体で UNIQUE でなければならない（QR スキャン時の重複排除）。
    CONSTRAINT uq_reworked_lot_labels_giai UNIQUE (giai),
    -- reworks への外部キー。
    CONSTRAINT fk_rll_rework FOREIGN KEY (rework_id)
        REFERENCES reworks (rework_id) ON DELETE RESTRICT,
    -- 元ロットへの外部キー（修正品のトレサビリティ確保）。
    CONSTRAINT fk_rll_lot FOREIGN KEY (parent_lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    -- リワーク SOP バージョンへの外部キー（時点参照固定）。
    CONSTRAINT fk_rll_sop_version FOREIGN KEY (rework_sop_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT,
    -- ラベル発行者ユーザーへの外部キー。
    CONSTRAINT fk_rll_issuer FOREIGN KEY (issued_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_rll_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  reworked_lot_labels IS 'EN-036 ReworkedLotLabel — GS1 AI 8003（GIAI）+ AI 91 形式の修正品 QR ラベル。Append-only。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN reworked_lot_labels.giai IS 'GS1 Global Individual Asset Identifier（AI 8003）。30 文字以内。UNIQUE 制約。修正品のスキャン検索に使用する。';
COMMENT ON COLUMN reworked_lot_labels.qr_payload IS 'QR コードに埋め込む GS1 データ文字列（AI 8003 GIAI + AI 91 拡張）。';
COMMENT ON COLUMN reworked_lot_labels.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。';
COMMENT ON COLUMN reworked_lot_labels.content_hash IS '本レコードの SHA-256（rework_id / giai / parent_lot_id / issued_by / issued_at の canonical JSON）。';

-- Append-only 強制: UPDATE/DELETE を禁止する
REVOKE UPDATE, DELETE ON reworked_lot_labels FROM PUBLIC;
REVOKE UPDATE, DELETE ON reworked_lot_labels FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON reworked_lot_labels TO app_event_writer;
