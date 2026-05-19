-- V20260517120030__create_andon_alerts_table.sql
-- TBL-012 andon_alerts: アンドン発報レコード（更新可）。

-- EN-017 AndonAlert — アンドン発報レコード（更新可）。
-- ALERTING→ACKNOWLEDGED→RESOLVED の順序で遷移する。RESOLVED 時は resolution_note 必須。
CREATE TABLE IF NOT EXISTS andon_alerts (
    -- アンドン発報識別子。UUID v4。gen_random_uuid() で自動生成。
    alert_id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 発報が発生した作業セッション識別子。NULL 許容（作業外での発報も許可）。
    work_execution_id  UUID        NULL,
    -- 発報したユーザーの識別子。
    raised_by          UUID        NOT NULL,
    -- 発報種別。QUALITY / EQUIPMENT / MATERIAL / PROCESS / SAFETY の 5 種のみ許可。
    alert_type         VARCHAR(32) NOT NULL,
    -- ステータス。ALERTING / ACKNOWLEDGED / RESOLVED の 3 種のみ許可（初期値: ALERTING）。
    status             VARCHAR(16) NOT NULL DEFAULT 'ALERTING',
    -- 発報時刻。
    raised_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 確認（承認）したユーザーの識別子。NULL = 未確認。
    acknowledged_by    UUID        NULL,
    -- 確認時刻。NULL = 未確認。
    acknowledged_at    TIMESTAMPTZ NULL,
    -- 解決したユーザーの識別子。NULL = 未解決。
    resolved_by        UUID        NULL,
    -- 解決時刻。NULL = 未解決。
    resolved_at        TIMESTAMPTZ NULL,
    -- 解決備考。status=RESOLVED 時は必須（CHECK 制約）。
    resolution_note    TEXT        NULL,

    -- 主キー
    CONSTRAINT pk_andon_alerts PRIMARY KEY (alert_id),
    -- work_executions への外部キー。NULL 許容。
    CONSTRAINT fk_andon_alerts_execution FOREIGN KEY (work_execution_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    -- 発報者ユーザーへの外部キー。
    CONSTRAINT fk_andon_alerts_raised_by FOREIGN KEY (raised_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- 確認者ユーザーへの外部キー。NULL 許容。
    CONSTRAINT fk_andon_alerts_acknowledged_by FOREIGN KEY (acknowledged_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- 解決者ユーザーへの外部キー。NULL 許容。
    CONSTRAINT fk_andon_alerts_resolved_by FOREIGN KEY (resolved_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- alert_type は 5 種の列挙値のみ許可する。
    CONSTRAINT ck_andon_alerts_type CHECK (
        alert_type IN ('QUALITY', 'EQUIPMENT', 'MATERIAL', 'PROCESS', 'SAFETY')
    ),
    -- status は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_andon_alerts_status CHECK (
        status IN ('ALERTING', 'ACKNOWLEDGED', 'RESOLVED')
    ),
    -- RESOLVED 状態では resolution_note が必須である。
    CONSTRAINT ck_andon_alerts_resolved_requires_note CHECK (
        NOT (status = 'RESOLVED' AND resolution_note IS NULL)
    ),
    -- acknowledged_by が設定されている場合は acknowledged_at も必須である。
    CONSTRAINT ck_andon_alerts_acknowledged_consistency CHECK (
        NOT (acknowledged_by IS NOT NULL AND acknowledged_at IS NULL)
    ),
    -- resolved_by が設定されている場合は resolved_at も必須である。
    CONSTRAINT ck_andon_alerts_resolved_consistency CHECK (
        NOT (resolved_by IS NOT NULL AND resolved_at IS NULL)
    )
);

COMMENT ON TABLE  andon_alerts IS 'EN-017 AndonAlert — アンドン発報レコード。ALERTING→ACKNOWLEDGED→RESOLVED の順序で遷移。RESOLVED 時は resolution_note 必須（CHECK 制約）。5年以上保存。';
