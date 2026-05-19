-- V20260517120031__create_nonconformities_table.sql
-- TBL-013 nonconformities: 不適合レコード（更新可）。4M+E カテゴリで分類。

-- EN-018 Nonconformity — 不適合レコード（更新可）。
-- capas テーブルへの FK（capa_id）は capas テーブル作成後に追加する。
-- NOTE: nonconformities と capas は相互参照関係にある。
-- nonconformities.capa_id → capas（DEFERRABLE）
-- capas.nc_id → nonconformities（DEFERRABLE）
-- の 2 本の外部キーで双方向参照を実現する。
CREATE TABLE IF NOT EXISTS nonconformities (
    -- 不適合識別子。UUID v4。gen_random_uuid() で自動生成。
    nc_id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 不適合が発生した作業セッション識別子。NULL 許容（作業外での不適合も許可）。
    work_execution_id  UUID        NULL,
    -- 不適合を報告したユーザーの識別子。
    reported_by        UUID        NOT NULL,
    -- 不適合カテゴリ。4M+E: MAN / MACHINE / MATERIAL / METHOD / ENVIRONMENT の 5 種のみ許可。
    nc_category        VARCHAR(16) NOT NULL,
    -- 不適合の説明。自由記述。
    description        TEXT        NOT NULL,
    -- ステータス。OPEN / INVESTIGATING / CLOSED の 3 種のみ許可（初期値: OPEN）。
    status             VARCHAR(16) NOT NULL DEFAULT 'OPEN',
    -- 不適合発生時刻。
    opened_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 不適合クローズ時刻。NULL = 未クローズ。
    closed_at          TIMESTAMPTZ NULL,
    -- 紐付く CAPA レコードの識別子。NULL = CAPA 未作成。capas テーブル作成後に FK を追加する。
    capa_id            UUID        NULL,

    -- 主キー
    CONSTRAINT pk_nonconformities PRIMARY KEY (nc_id),
    -- work_executions への外部キー。NULL 許容。
    CONSTRAINT fk_nc_execution FOREIGN KEY (work_execution_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    -- 報告者ユーザーへの外部キー。
    CONSTRAINT fk_nc_reported_by FOREIGN KEY (reported_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- nc_category は 4M+E の 5 種の列挙値のみ許可する。
    CONSTRAINT ck_nc_category CHECK (
        nc_category IN ('MAN', 'MACHINE', 'MATERIAL', 'METHOD', 'ENVIRONMENT')
    ),
    -- status は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_nc_status CHECK (
        status IN ('OPEN', 'INVESTIGATING', 'CLOSED')
    ),
    -- クローズ時刻は発生時刻より後でなければならない。
    CONSTRAINT ck_nc_closed_after_opened CHECK (
        NOT (closed_at IS NOT NULL AND closed_at < opened_at)
    )
);

COMMENT ON TABLE  nonconformities IS 'EN-018 Nonconformity — 不適合レコード。4M+E カテゴリで分類。CAPA（TBL-014）と関連付け可能。7年以上保存。';
COMMENT ON COLUMN nonconformities.nc_category IS '4M+E 分類: MAN（人）/ MACHINE（機械）/ MATERIAL（材料）/ METHOD（方法）/ ENVIRONMENT（環境）。';
