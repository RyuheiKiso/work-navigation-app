-- V20260517120033__create_kaizen_proposals_table.sql
-- TBL-015 kaizen_proposals: 改善提案レコード（更新可）。PROPOSED→REVIEWING→ADOPTED/REJECTED の遷移。

-- EN-020 Kaizen — 改善提案レコード（更新可）。
CREATE TABLE IF NOT EXISTS kaizen_proposals (
    -- 改善提案識別子。UUID v4。gen_random_uuid() で自動生成。
    kaizen_id          UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- 提案者ユーザーの識別子。
    proposed_by        UUID         NOT NULL,
    -- 改善提案カテゴリ。自由記述（64 文字以内）。
    category           VARCHAR(64)  NOT NULL,
    -- 改善提案タイトル（256 文字以内）。
    title              VARCHAR(256) NOT NULL,
    -- 改善提案の詳細説明。自由記述。
    description        TEXT         NOT NULL,
    -- 改善対象プロセスの識別子。NULL = プロセス未特定。
    target_process_id  UUID         NULL,
    -- ステータス。PROPOSED / REVIEWING / ADOPTED / REJECTED の 4 種のみ許可（初期値: PROPOSED）。
    status             VARCHAR(16)  NOT NULL DEFAULT 'PROPOSED',
    -- 提案時刻。
    proposed_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- レビュー完了時刻。NULL = 未レビュー。
    reviewed_at        TIMESTAMPTZ  NULL,
    -- レビュー者ユーザーの識別子。NULL = 未レビュー。
    reviewed_by        UUID         NULL,

    -- 主キー
    CONSTRAINT pk_kaizen_proposals PRIMARY KEY (kaizen_id),
    -- 提案者ユーザーへの外部キー。
    CONSTRAINT fk_kaizen_proposed_by FOREIGN KEY (proposed_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- 対象プロセスへの外部キー。NULL 許容。
    CONSTRAINT fk_kaizen_target_process FOREIGN KEY (target_process_id)
        REFERENCES processes (process_id) ON DELETE RESTRICT,
    -- レビュー者ユーザーへの外部キー。NULL 許容。
    CONSTRAINT fk_kaizen_reviewed_by FOREIGN KEY (reviewed_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- status は 4 種の列挙値のみ許可する。
    CONSTRAINT ck_kaizen_status CHECK (
        status IN ('PROPOSED', 'REVIEWING', 'ADOPTED', 'REJECTED')
    ),
    -- reviewed_by が設定されている場合は reviewed_at も必須である。
    CONSTRAINT ck_kaizen_reviewed_consistency CHECK (
        NOT (reviewed_by IS NOT NULL AND reviewed_at IS NULL)
    )
);

COMMENT ON TABLE  kaizen_proposals IS 'EN-020 Kaizen — 改善提案。PROPOSED→REVIEWING→ADOPTED/REJECTED の遷移。5年以上保存。';
