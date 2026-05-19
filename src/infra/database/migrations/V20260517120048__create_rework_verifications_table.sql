-- V20260517120048__create_rework_verifications_table.sql
-- TBL-045 rework_verifications: リワーク検証記録（Append-only）。ハッシュチェーン列込み（ADR-011）。

-- EN-034 ReworkVerification — リワーク検証記録（Append-only）。
-- verifier_id ≠ リワーク実施者（API 層で ERR-BIZ-023 を返す）。
-- qc_case_id = rework_id でハッシュチェーンを構成する（ADR-011）。
-- NOTE: 二段適用方針により qc_case_id / prev_hash / content_hash は NULL 許容で作成する。
CREATE TABLE IF NOT EXISTS rework_verifications (
    -- リワーク検証識別子。UUID v4。gen_random_uuid() で自動生成。
    verification_id             UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 検証対象のリワークレコードの識別子。
    rework_id                   UUID        NOT NULL,
    -- 検証作業を実施した WorkExecution の識別子。
    verification_case_id        UUID        NOT NULL,
    -- 検証実施者ユーザーの識別子。リワーク実施者とは異なる人物であることが必要（API 層で検証）。
    verifier_id                 UUID        NOT NULL,
    -- 検証結果判定。OK / NG / DOWNGRADE の 3 種のみ許可（CHECK 制約）。
    verdict                     VARCHAR(16) NOT NULL,
    -- NG 判定時に新規作成した不適合レコードの識別子。NULL = NG または DOWNGRADE なし。
    follow_up_nonconformity_id  UUID        NULL,
    -- 検証完了時刻。
    verified_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ハッシュチェーン単位 ID（= rework_id）。同一リワークの検証列を時系列で連結する。NULL 許容（二段適用方針）。
    qc_case_id                  UUID        NULL,
    -- 前ブロックの content_hash（genesis は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash                   CHAR(64)    NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash                CHAR(64)    NULL,

    -- 主キー
    CONSTRAINT pk_rework_verifications PRIMARY KEY (verification_id),
    -- reworks への外部キー。
    CONSTRAINT fk_rv_rework FOREIGN KEY (rework_id)
        REFERENCES reworks (rework_id) ON DELETE RESTRICT,
    -- 検証 WorkExecution への外部キー。
    CONSTRAINT fk_rv_case FOREIGN KEY (verification_case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    -- 検証者ユーザーへの外部キー。
    CONSTRAINT fk_rv_verifier FOREIGN KEY (verifier_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- フォローアップ不適合への外部キー。NULL 許容。
    CONSTRAINT fk_rv_followup_nc FOREIGN KEY (follow_up_nonconformity_id)
        REFERENCES nonconformities (nc_id) ON DELETE RESTRICT,
    -- verdict は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_rv_verdict CHECK (
        verdict IN ('OK', 'NG', 'DOWNGRADE')
    ),
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_rv_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  rework_verifications IS 'EN-034 ReworkVerification — Append-only。verifier_id ≠ リワーク実施者（API 層で ERR-BIZ-023 を返す）。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN rework_verifications.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。';
COMMENT ON COLUMN rework_verifications.content_hash IS '本レコードの SHA-256（rework_id / verifier_id / verdict / verified_at の canonical JSON）。';

-- Append-only 強制: UPDATE/DELETE を禁止する
REVOKE UPDATE, DELETE ON rework_verifications FROM PUBLIC;
REVOKE UPDATE, DELETE ON rework_verifications FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON rework_verifications TO app_event_writer;
