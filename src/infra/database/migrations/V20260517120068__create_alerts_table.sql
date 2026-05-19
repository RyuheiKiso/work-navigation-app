-- V20260517120068__create_alerts_table.sql
-- alerts: システムアラート管理テーブル。バッチジョブ失敗・ハッシュチェーン破断等の
-- P1〜P3 アラートを記録し、SCR-MC-001 ダッシュボードに反映する（30秒リフレッシュ）。
-- 権威: docs/04_概要設計/08_運用方式設計/09_ジョブ実行失敗時の自動回復方針.md §3
-- 原本は BIGSERIAL PK だが、コーディング規約（UUID 統一）に準拠して UUID PK に変換する。
-- resolved_by（原本は TEXT）は users テーブルへの FK に変換する（NULL 許容）。

CREATE TABLE IF NOT EXISTS alerts (
    -- アラート識別子。UUID v4。gen_random_uuid() で自動生成。
    id          UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- アラートコード。'ERR-SYS-005' / 'ERR-SEC-002' 等の内部エラーコード。
    alert_code  TEXT         NOT NULL,
    -- アラート優先度。P1（即時対応）/ P2（1時間以内）/ P3（24時間以内）の 3 値。
    priority    TEXT         NOT NULL,
    -- アラートを発生させたジョブ識別子。'BAT-002' 等。NULL は手動発生アラート。
    source_job  TEXT         NULL,
    -- アラートの説明メッセージ。
    message     TEXT         NOT NULL,
    -- アラートステータス。active（未対応）/ acknowledged（確認済）/ resolved（解決済）。
    status      TEXT         NOT NULL DEFAULT 'active',
    -- レコード作成日時。
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- 解決日時。NULL = 未解決。
    resolved_at TIMESTAMPTZ  NULL,
    -- 解決担当者の user_id。NULL = 未解決または自動解決。
    resolved_by UUID         NULL,

    CONSTRAINT pk_alerts PRIMARY KEY (id),
    -- resolved_by は users テーブルへの外部キー（NULL 許容）
    CONSTRAINT fk_alerts_resolved_by FOREIGN KEY (resolved_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- priority は 3 値のみ許可する。
    CONSTRAINT ck_alerts_priority CHECK (priority IN ('P1', 'P2', 'P3')),
    -- status は 3 値のみ許可する。
    CONSTRAINT ck_alerts_status CHECK (status IN ('active', 'acknowledged', 'resolved')),
    -- resolved 時は resolved_at が必須。
    CONSTRAINT ck_alerts_resolved_requires_at CHECK (
        status <> 'resolved' OR resolved_at IS NOT NULL
    )
);

COMMENT ON TABLE alerts IS
    'システムアラート管理。バッチジョブ失敗・ハッシュチェーン破断等の P1〜P3 アラートを記録し、SCR-MC-001 ダッシュボードに反映する（30秒リフレッシュ）。';
COMMENT ON COLUMN alerts.alert_code  IS '内部エラーコード（例: ERR-SYS-005 / ERR-SEC-002）。';
COMMENT ON COLUMN alerts.priority    IS 'P1（即時対応）/ P2（1時間以内対応）/ P3（24時間以内対応）。';
COMMENT ON COLUMN alerts.source_job  IS 'アラートを発生させたバッチ識別子（例: BAT-002）。NULL は手動発生。';
COMMENT ON COLUMN alerts.status      IS 'active（未対応）/ acknowledged（確認済）/ resolved（解決済）。';

-- 未解決アラートの検索インデックス（ダッシュボードの 30 秒リフレッシュ用）
CREATE INDEX idx_alerts_active
    ON alerts USING BTREE (priority, created_at DESC)
    WHERE status IN ('active', 'acknowledged');

-- app_read_write に INSERT/SELECT/UPDATE を許可する（status/resolved_at の更新に必要）
GRANT INSERT, SELECT, UPDATE ON alerts TO app_read_write;
