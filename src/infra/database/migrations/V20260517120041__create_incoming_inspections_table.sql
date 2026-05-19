-- V20260517120041__create_incoming_inspections_table.sql
-- TBL-038 incoming_inspections: 入荷ロット受入検査ヘッダ（限定可変）。ハッシュチェーン列込み（ADR-011）。

-- EN-030 IncomingInspection — 入荷ロット受入検査ヘッダ（限定可変）。
-- qc_status 列のみ UPDATE 可。その他は Append-only に準じる。
-- qc_case_id / prev_hash / content_hash はハッシュチェーン列（ADR-011）。
-- NOTE: 二段適用方針により NULL 許容で作成する。初期データ投入後に NOT NULL 化するマイグレーションを別途適用する。
CREATE TABLE IF NOT EXISTS incoming_inspections (
    -- 受入検査識別子。UUID v4。gen_random_uuid() で自動生成。
    inspection_id           UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 検査対象ロットの識別子。
    lot_id                  UUID            NOT NULL,
    -- 仕入先業者の識別子。
    supplier_id             UUID            NOT NULL,
    -- 検査対象材料の識別子。
    material_id             UUID            NOT NULL,
    -- 適用したサンプリング計画の識別子。
    sampling_plan_id        UUID            NOT NULL,
    -- サンプリング計画バージョン（時点固定コピー）。計画改訂後も判定根拠が追跡可能。
    sampling_plan_version   INTEGER         NOT NULL,
    -- ロット数量。0 より大きい値のみ許可（CHECK 制約）。
    lot_quantity            INTEGER         NOT NULL,
    -- サンプルサイズ n。0 より大きい値のみ許可（CHECK 制約）。
    sample_size_n           INTEGER         NOT NULL,
    -- 合格判定個数 Ac。0 以上の値のみ許可（CHECK 制約）。
    accept_number_ac        INTEGER         NOT NULL,
    -- 不合格判定個数 Re。accept_number_ac より大きい値のみ許可（CHECK 制約）。
    reject_number_re        INTEGER         NOT NULL,
    -- JIS Z 9015-1 §10 の検査の厳しさ状態。NORMAL / TIGHTENED / REDUCED の 3 種のみ許可（初期値: NORMAL）。
    severity_state          VARCHAR(16)     NOT NULL DEFAULT 'NORMAL',
    -- 受入検査 QC ステータス。8 種の列挙値のみ許可（初期値: PENDING）。qc_status のみ UPDATE 可（限定可変）。
    qc_status               VARCHAR(32)     NOT NULL DEFAULT 'PENDING',
    -- 検査担当者ユーザーの識別子。
    inspector_id            UUID            NOT NULL,
    -- 入荷日時（サーバー受信時刻）。
    received_at             TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- 判定完了時刻。NULL = 判定未完了。
    judged_at               TIMESTAMPTZ     NULL,
    -- レコード作成時刻。
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- ハッシュチェーン単位 ID。genesis として自身の inspection_id を設定する（ADR-011）。NULL 許容（二段適用方針）。
    qc_case_id              UUID            NULL,
    -- 前ブロックの content_hash（genesis は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash               CHAR(64)        NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash            CHAR(64)        NULL,

    -- 主キー
    CONSTRAINT pk_incoming_inspections PRIMARY KEY (inspection_id),
    -- ロットへの外部キー。
    CONSTRAINT fk_incoming_insp_lot       FOREIGN KEY (lot_id)           REFERENCES lots (lot_id) ON DELETE RESTRICT,
    -- 仕入先業者への外部キー。
    CONSTRAINT fk_incoming_insp_supplier  FOREIGN KEY (supplier_id)      REFERENCES suppliers (supplier_id) ON DELETE RESTRICT,
    -- 材料への外部キー。
    CONSTRAINT fk_incoming_insp_material  FOREIGN KEY (material_id)      REFERENCES materials (material_id) ON DELETE RESTRICT,
    -- サンプリング計画への外部キー。
    CONSTRAINT fk_incoming_insp_plan      FOREIGN KEY (sampling_plan_id) REFERENCES sampling_plans (plan_id) ON DELETE RESTRICT,
    -- 検査担当者ユーザーへの外部キー。
    CONSTRAINT fk_incoming_insp_inspector FOREIGN KEY (inspector_id)     REFERENCES users (user_id) ON DELETE RESTRICT,
    -- severity_state は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_incoming_insp_severity CHECK (
        severity_state IN ('NORMAL', 'TIGHTENED', 'REDUCED')
    ),
    -- qc_status は 8 種の列挙値のみ許可する。
    CONSTRAINT ck_incoming_insp_status CHECK (
        qc_status IN ('PENDING', 'INSPECTING', 'PASSED', 'CONDITIONAL_PASS', 'SCREENING_REQUIRED', 'REJECTED', 'SCRAPPED', 'RETURNED')
    ),
    -- ロット数量とサンプルサイズは 0 より大きい値のみ許可する。
    CONSTRAINT ck_incoming_insp_qty CHECK (lot_quantity > 0 AND sample_size_n > 0),
    -- accept_number_ac は 0 以上、reject_number_re は accept_number_ac より大きい値のみ許可する。
    CONSTRAINT ck_incoming_insp_acre CHECK (accept_number_ac >= 0 AND reject_number_re > accept_number_ac),
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_incoming_insp_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  incoming_inspections IS 'EN-030 IncomingInspection — 入荷受入検査ヘッダ。qc_status のみ UPDATE 可。per qc_case_id genesis ハッシュチェーン（ADR-011）により改ざん検知を保証。';
COMMENT ON COLUMN incoming_inspections.sampling_plan_version IS 'sampling_plans.version の時点固定コピー。サンプリング計画改訂後も判定根拠が追跡可能。';
COMMENT ON COLUMN incoming_inspections.severity_state IS 'JIS Z 9015-1 §10 の検査の厳しさ状態（なみ/きつい/ゆるい）。';
COMMENT ON COLUMN incoming_inspections.qc_case_id IS 'ハッシュチェーン単位 ID。incoming_inspections では genesis として自身の inspection_id を設定する（ADR-011）。';
COMMENT ON COLUMN incoming_inspections.prev_hash IS '前ブロックの content_hash（genesis は "0"×64）。';
COMMENT ON COLUMN incoming_inspections.content_hash IS '本レコードの SHA-256（inspection_id / lot_id / supplier_id / material_id / sampling_plan_id / sampling_plan_version / lot_quantity / sample_size_n / accept_number_ac / reject_number_re / severity_state / inspector_id / received_at の canonical JSON）。qc_status・judged_at は可変フィールドのためハッシュ対象外。';

-- IDX-021: ロット別検索（入荷検査ヘッダのロット軸検索）
CREATE INDEX IF NOT EXISTS idx_incoming_insp_lot ON incoming_inspections (lot_id);
-- IDX-022: 仕入先×ステータス複合検索（サプライヤーポータルおよび品質レポート）
CREATE INDEX IF NOT EXISTS idx_incoming_insp_supplier_status ON incoming_inspections (supplier_id, qc_status);

-- qc_status 列の UPDATE のみ app_read_write に許可する（限定可変）
GRANT UPDATE (qc_status, judged_at) ON incoming_inspections TO app_read_write;
