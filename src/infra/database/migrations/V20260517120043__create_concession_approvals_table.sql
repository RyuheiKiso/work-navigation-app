-- V20260517120043__create_concession_approvals_table.sql
-- TBL-041 concession_approvals: 特採承認記録（Append-only）。ハッシュチェーン列込み（ADR-011）。

-- EN-030 承認詳細 — 特採承認記録（Append-only）。
-- qc_case_id = inspection_id でハッシュチェーンを構成する（ADR-011）。
-- NOTE: 二段適用方針により qc_case_id / prev_hash / content_hash は NULL 許容で作成する。
CREATE TABLE IF NOT EXISTS concession_approvals (
    -- 特採承認識別子。UUID v4。gen_random_uuid() で自動生成。
    approval_id         UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付く受入検査の識別子。
    inspection_id       UUID            NOT NULL,
    -- 特採判定種別。デフォルト: CONCESSION（特採）。
    decision            VARCHAR(32)     NOT NULL DEFAULT 'CONCESSION',
    -- 特採理由。空白文字のみは禁止（CHECK 制約）。
    reason              TEXT            NOT NULL,
    -- 特採の適用範囲 JSONB。例: {"lot_range": "L001-L010", "product": "P-A001"}。
    validity_scope      JSONB           NOT NULL DEFAULT '{}',
    -- 特採有効期限。NULL = 無期限。BAT-009 拡張が期限超過時に lot_qc_states を REJECTED に遷移させる。
    valid_until         DATE            NULL,
    -- 承認者ユーザーの識別子。
    approver_id         UUID            NOT NULL,
    -- 承認に使用した電子サインの識別子。
    electronic_sign_id  UUID            NOT NULL,
    -- 承認時刻。
    approved_at         TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- ハッシュチェーン単位 ID（= inspection_id）。同一検査の特採承認列を連結する。NULL 許容（二段適用方針）。
    qc_case_id          UUID            NULL,
    -- 前ブロックの content_hash（genesis は "0"×64）。NULL 許容（二段適用方針）。
    prev_hash           CHAR(64)        NULL,
    -- 本レコードの SHA-256。NULL 許容（二段適用方針）。
    content_hash        CHAR(64)        NULL,

    -- 主キー
    CONSTRAINT pk_concession_approvals PRIMARY KEY (approval_id),
    -- incoming_inspections への外部キー。
    CONSTRAINT fk_concession_inspection FOREIGN KEY (inspection_id)
        REFERENCES incoming_inspections (inspection_id) ON DELETE RESTRICT,
    -- 承認者ユーザーへの外部キー。
    CONSTRAINT fk_concession_approver FOREIGN KEY (approver_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- 電子サインへの外部キー。
    CONSTRAINT fk_concession_sign FOREIGN KEY (electronic_sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    -- reason は空白文字のみを禁止する（有意な説明文が必要）。
    CONSTRAINT ck_concession_reason CHECK (length(trim(reason)) > 0),
    -- ハッシュ列が設定されている場合は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_concession_hash_length CHECK (
        (prev_hash IS NULL OR length(prev_hash) = 64) AND
        (content_hash IS NULL OR length(content_hash) = 64)
    )
);

COMMENT ON TABLE  concession_approvals IS 'EN-030 承認詳細 — 特採承認は Append-only。valid_until を超過した場合は BAT-009 拡張が lot_qc_states を REJECTED に遷移させる。qc_case_id = inspection_id でチェーン（ADR-011）。';
COMMENT ON COLUMN concession_approvals.qc_case_id IS 'ハッシュチェーン単位 ID（= inspection_id）。同一検査の特採承認列を連結する。';
COMMENT ON COLUMN concession_approvals.content_hash IS '本レコードの SHA-256（inspection_id / decision / reason / approver_id / approved_at の canonical JSON）。';

-- Append-only 強制: UPDATE/DELETE を禁止する
REVOKE UPDATE, DELETE ON concession_approvals FROM PUBLIC;
REVOKE UPDATE, DELETE ON concession_approvals FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON concession_approvals TO app_event_writer;
