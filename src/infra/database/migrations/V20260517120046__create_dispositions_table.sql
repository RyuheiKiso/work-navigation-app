-- V20260517120046__create_dispositions_table.sql
-- TBL-044 dispositions: ディスポジション判定（Append-only）。ハッシュチェーン列、Two-Person Integrity CHECK。

-- EN-033 Disposition — ディスポジション判定（Append-only）。
-- Two-Person Integrity: quality_admin_sign_id と supervisor_sign_id は異なる worker_id でなければならない。
-- トリガ check_disposition_distinct_signers で NFR-SEC-048 を実装する（V47 で定義）。
-- NOTE: 二段適用方針により qc_case_id / prev_hash / content_hash は NULL 許容で作成する。
CREATE TABLE IF NOT EXISTS dispositions (
    -- ディスポジション識別子。UUID v4。gen_random_uuid() で自動生成。
    disposition_id          UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付く不適合レコードの識別子。
    nonconformity_id        UUID        NOT NULL,
    -- 処分判定種別。REWORK / SCRAP / RETURN / USE_AS_IS の 4 種のみ許可。
    decision                VARCHAR(16) NOT NULL,
    -- 処分判定理由。空白文字のみは禁止（CHECK 制約）。
    decision_reason         TEXT        NOT NULL,
    -- 品質管理者の電子サイン識別子（Two-Person Integrity の 1 人目）。
    quality_admin_sign_id   UUID        NOT NULL,
    -- 監督者の電子サイン識別子（Two-Person Integrity の 2 人目）。supervisor_sign_id ≠ quality_admin_sign_id。
    supervisor_sign_id      UUID        NOT NULL,
    -- 処分判定時刻。
    decided_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ハッシュチェーン単位 ID（= nonconformity_id）。同一 NC に対する複数ディスポジションを時系列で連結する。NULL 許容（二段適用方針）。
    qc_case_id              UUID        NULL,
    -- 前ブロックの content_hash（genesis は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash               CHAR(64)    NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash            CHAR(64)    NULL,

    -- 主キー
    CONSTRAINT pk_dispositions PRIMARY KEY (disposition_id),
    -- nonconformities への外部キー。
    CONSTRAINT fk_disp_nonconformity FOREIGN KEY (nonconformity_id)
        REFERENCES nonconformities (nc_id) ON DELETE RESTRICT,
    -- 品質管理者電子サインへの外部キー。
    CONSTRAINT fk_disp_qa_sign FOREIGN KEY (quality_admin_sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    -- 監督者電子サインへの外部キー。
    CONSTRAINT fk_disp_sup_sign FOREIGN KEY (supervisor_sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    -- decision は 4 種の列挙値のみ許可する。
    CONSTRAINT ck_disp_decision CHECK (
        decision IN ('REWORK', 'SCRAP', 'RETURN', 'USE_AS_IS')
    ),
    -- decision_reason は空白文字のみを禁止する（有意な理由が必要）。
    CONSTRAINT ck_disp_reason CHECK (length(trim(decision_reason)) > 0),
    -- 2 つの電子サインは異なる sign_id でなければならない（サイン ID レベルの Two-Person Integrity）。
    -- ※ 署名者（signer_id）レベルの検証は DB トリガ（V47）で行う（NFR-SEC-048）。
    CONSTRAINT ck_disp_distinct_signs CHECK (quality_admin_sign_id <> supervisor_sign_id),
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_disp_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  dispositions IS 'EN-033 Disposition — Append-only。Two-Person Integrity をトリガ check_disposition_distinct_signers で保証（NFR-SEC-048）。qc_case_id = nonconformity_id でチェーン（ADR-011）。';
COMMENT ON COLUMN dispositions.qc_case_id IS 'ハッシュチェーン単位 ID（= nonconformity_id）。同一 NC に対する複数ディスポジションを時系列で連結する。';
COMMENT ON COLUMN dispositions.content_hash IS '本レコードの SHA-256（nonconformity_id / decision / quality_admin_sign_id / supervisor_sign_id / decided_at の canonical JSON）。';

-- dispositions テーブル作成後に reworks.disposition_id への外部キーを追加する
ALTER TABLE reworks
    ADD CONSTRAINT fk_reworks_disposition FOREIGN KEY (disposition_id)
        REFERENCES dispositions (disposition_id) ON DELETE RESTRICT;

-- Append-only 強制: UPDATE/DELETE を禁止する
REVOKE UPDATE, DELETE ON dispositions FROM PUBLIC;
REVOKE UPDATE, DELETE ON dispositions FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON dispositions TO app_event_writer;
