-- V20260517120032__create_capas_table.sql
-- TBL-014 capas: 是正予防措置レコード（更新可）。CLOSED 時は root_cause 必須。

-- EN-019 CAPA — 是正予防措置レコード（更新可）。
-- nonconformities との相互参照を DEFERRABLE 外部キーで実現する。
CREATE TABLE IF NOT EXISTS capas (
    -- CAPA 識別子。UUID v4。gen_random_uuid() で自動生成。
    capa_id      UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 紐付く不適合レコードの識別子。NULL 許容（予防措置の場合は不適合なし）。
    nc_id        UUID        NULL,
    -- CAPA 種別。CORRECTIVE（是正）/ PREVENTIVE（予防）の 2 種のみ許可。
    capa_type    VARCHAR(16) NOT NULL,
    -- CAPA の説明。自由記述。
    description  TEXT        NOT NULL,
    -- 根本原因分析結果。status=CLOSED 時は必須（CHECK 制約）。
    root_cause   TEXT        NULL,
    -- ステータス。OPEN / CORRECTIVE_ACTION / VERIFICATION / CLOSED の 4 種のみ許可（初期値: OPEN）。
    status       VARCHAR(24) NOT NULL DEFAULT 'OPEN',
    -- 担当者ユーザーの識別子。
    assigned_to  UUID        NOT NULL,
    -- 承認電子サインの識別子。NULL = 承認未取得。
    sign_id      UUID        NULL,
    -- CAPA 開始時刻。
    opened_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- CAPA クローズ時刻。NULL = 未クローズ。
    closed_at    TIMESTAMPTZ NULL,

    -- 主キー
    CONSTRAINT pk_capas PRIMARY KEY (capa_id),
    -- nonconformities への外部キー。NULL 許容（予防措置の場合）。DEFERRABLE で相互参照を解消する。
    CONSTRAINT fk_capas_nc FOREIGN KEY (nc_id)
        REFERENCES nonconformities (nc_id) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- 担当者ユーザーへの外部キー。
    CONSTRAINT fk_capas_assigned_to FOREIGN KEY (assigned_to)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- 承認電子サインへの外部キー。NULL 許容。
    CONSTRAINT fk_capas_sign FOREIGN KEY (sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    -- capa_type は 2 種の列挙値のみ許可する。
    CONSTRAINT ck_capas_type CHECK (
        capa_type IN ('CORRECTIVE', 'PREVENTIVE')
    ),
    -- status は 4 種の列挙値のみ許可する。
    CONSTRAINT ck_capas_status CHECK (
        status IN ('OPEN', 'CORRECTIVE_ACTION', 'VERIFICATION', 'CLOSED')
    ),
    -- CLOSED 状態では root_cause が必須である（根本原因分析が完了していることを保証）。
    CONSTRAINT ck_capas_closed_requires_root_cause CHECK (
        NOT (status = 'CLOSED' AND root_cause IS NULL)
    ),
    -- クローズ時刻は開始時刻より後でなければならない。
    CONSTRAINT ck_capas_closed_after_opened CHECK (
        NOT (closed_at IS NOT NULL AND closed_at < opened_at)
    )
);

COMMENT ON TABLE  capas IS 'EN-019 CAPA — 是正予防措置。CLOSED 時は root_cause 必須（CHECK 制約）。sign_id により承認電子サインと連携。7年以上保存。';

-- capas テーブル作成後に nonconformities.capa_id への外部キーを追加する
-- nonconformities と capas の相互参照を DEFERRABLE で解消する
ALTER TABLE nonconformities
    ADD CONSTRAINT fk_nc_capa FOREIGN KEY (capa_id)
        REFERENCES capas (capa_id) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED;
