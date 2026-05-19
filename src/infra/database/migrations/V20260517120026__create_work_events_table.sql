-- V20260517120026__create_work_events_table.sql
-- TBL-001 work_events: イベントストアコアテーブル。Append-only / SHA-256 ハッシュチェーン / XES 互換 / 月次 RANGE パーティション。

-- EN-012 WorkEvent — イベントストアのコアテーブル。
-- Append-only。SHA-256 ハッシュチェーン。XES 互換。月次 RANGE パーティション。
CREATE TABLE IF NOT EXISTS work_events (
    -- イベント識別子。UUID v7（時系列順）。Rust 側で生成する。Idempotency Key として機能する。
    event_id            UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- XES Case ID。work_executions.work_execution_id に対応する。同一作業セッション内の全イベントが共有する。
    case_id             UUID            NOT NULL,
    -- XES Activity。9 種の列挙値のみ許可（CHECK 制約）。
    activity            VARCHAR(64)     NOT NULL,
    -- XES Timestamp（端末申告時刻）。オフライン時は端末ローカル時刻が記録される。ALCOA+ Contemporaneous 補足情報。
    timestamp_client    TIMESTAMPTZ     NOT NULL,
    -- サーバー受信時刻（証拠時刻）。パーティション KEY。ALCOA+ Contemporaneous 主証拠。
    timestamp_server    TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- XES Resource。実施者の user_id。
    resource            UUID            NOT NULL,
    -- 時点参照固定。作業実施時点の master_version_id を記録し、SOP 改訂後も当時の版を特定可能にする。
    sop_version_id      UUID            NOT NULL,
    -- 対象ステップ識別子。step 系 activity の場合は NOT NULL となる（CHECK 制約で担保）。
    step_id             UUID            NULL,
    -- activity 固有データ JSONB。スキーマは 05_JSONBスキーマ定義.md §4 参照（9 種それぞれ定義）。
    payload             JSONB           NOT NULL DEFAULT '{}',
    -- SHA-256 ハッシュチェーン前ブロック。チェーン先頭は "0000000000000000000000000000000000000000000000000000000000000000"。
    prev_hash           CHAR(64)        NOT NULL,
    -- このレコードの SHA-256。計算対象列: event_id + case_id + activity + timestamp_client + timestamp_server + resource + sop_version_id + payload(canonical JSON) + terminal_id + prev_hash。
    content_hash        CHAR(64)        NOT NULL,
    -- ALCOA+ Attributable 担保。どの端末で記録されたかを特定する。
    terminal_id         UUID            NOT NULL,
    -- TRUE のとき、オフライン状態で記録され後から同期されたことを示す。
    is_offline          BOOLEAN         NOT NULL DEFAULT FALSE,
    -- オフライン同期遅延（ミリ秒）。is_offline=TRUE 時に設定される。NULL は即時記録。
    sync_lag_ms         INTEGER         NULL,

    -- 複合主キー：event_id + timestamp_server（パーティションキー必須）
    CONSTRAINT pk_work_events PRIMARY KEY (event_id, timestamp_server),
    -- work_executions への外部キー。作業セッションが削除された場合も RESTRICT。
    CONSTRAINT fk_work_events_case FOREIGN KEY (case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- 実施者ユーザーへの外部キー。
    CONSTRAINT fk_work_events_resource FOREIGN KEY (resource)
        REFERENCES users (user_id) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- SOP 版への外部キー（時点参照固定）。
    CONSTRAINT fk_work_events_sop_version FOREIGN KEY (sop_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- ステップへの外部キー。step_id が NULL でない場合にのみ有効。
    CONSTRAINT fk_work_events_step FOREIGN KEY (step_id)
        REFERENCES steps (step_id) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- 端末への外部キー。
    CONSTRAINT fk_work_events_terminal FOREIGN KEY (terminal_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    -- activity は 9 種の列挙値のみ許可する。
    CONSTRAINT ck_work_events_activity CHECK (
        activity IN (
            'work_started',
            'step_completed',
            'step_skipped',
            'step_rejected',
            'work_suspended',
            'work_resumed',
            'work_completed',
            'work_cancelled',
            'evidence_attached'
        )
    ),
    -- prev_hash と content_hash は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_work_events_hash_length CHECK (
        length(prev_hash) = 64 AND length(content_hash) = 64
    ),
    -- sync_lag_ms は NULL または 0 以上でなければならない。
    CONSTRAINT ck_work_events_sync_lag_non_negative CHECK (
        sync_lag_ms IS NULL OR sync_lag_ms >= 0
    ),
    -- step 系 activity（step_completed / step_skipped / step_rejected / evidence_attached）の場合は step_id が必須。
    CONSTRAINT ck_work_events_step_required_for_step_activities CHECK (
        NOT (activity IN ('step_completed', 'step_skipped', 'step_rejected', 'evidence_attached')
            AND step_id IS NULL)
    )
) PARTITION BY RANGE (timestamp_server);

COMMENT ON TABLE  work_events IS 'EN-012 WorkEvent — イベントストアコアテーブル。Append-only / ハッシュチェーン / XES 互換 / 月次パーティション。7年以上保存。';
COMMENT ON COLUMN work_events.event_id         IS 'UUID v7（時系列順）。Rust 側で生成。Idempotency Key として機能する。';
COMMENT ON COLUMN work_events.case_id          IS 'XES Case ID。work_executions.work_execution_id に対応する。同一作業セッション内の全イベントが共有する。';
COMMENT ON COLUMN work_events.activity         IS 'XES Activity。9 種の列挙値のみ許可（CHECK 制約）。';
COMMENT ON COLUMN work_events.timestamp_client IS 'XES Timestamp（端末申告時刻）。オフライン時は端末ローカル時刻が記録される。ALCOA+ Contemporaneous 補足情報。';
COMMENT ON COLUMN work_events.timestamp_server IS 'サーバー受信時刻（証拠時刻）。パーティション KEY。ALCOA+ Contemporaneous 主証拠。';
COMMENT ON COLUMN work_events.resource         IS 'XES Resource。実施者の user_id。';
COMMENT ON COLUMN work_events.sop_version_id   IS '時点参照固定。作業実施時点の master_version_id を記録し、SOP 改訂後も当時の版を特定可能にする。';
COMMENT ON COLUMN work_events.payload          IS 'activity 固有データ JSONB。スキーマは 05_JSONBスキーマ定義.md §4 参照（9 種それぞれ定義）。';
COMMENT ON COLUMN work_events.prev_hash        IS 'SHA-256 ハッシュチェーン前ブロック。チェーン先頭は "0000000000000000000000000000000000000000000000000000000000000000"。';
COMMENT ON COLUMN work_events.content_hash     IS 'このレコードの SHA-256。計算対象列: event_id + case_id + activity + timestamp_client + timestamp_server + resource + sop_version_id + payload(canonical JSON) + terminal_id + prev_hash。';
COMMENT ON COLUMN work_events.terminal_id      IS 'ALCOA+ Attributable 担保。どの端末で記録されたかを特定する。';
COMMENT ON COLUMN work_events.is_offline       IS 'TRUE のとき、オフライン状態で記録され後から同期されたことを示す。';
COMMENT ON COLUMN work_events.sync_lag_ms      IS 'オフライン同期遅延（ミリ秒）。is_offline=TRUE 時に設定される。NULL は即時記録。';

-- 月次パーティション（2026年1月〜12月の初期定義）
-- 毎月1日に BAT-004 が翌月のパーティションを自動作成する
CREATE TABLE IF NOT EXISTS work_events_y2026m01
    PARTITION OF work_events
    FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-02-01 00:00:00+00');

-- 2026年2月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m02
    PARTITION OF work_events
    FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

-- 2026年3月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m03
    PARTITION OF work_events
    FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');

-- 2026年4月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m04
    PARTITION OF work_events
    FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');

-- 2026年5月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m05
    PARTITION OF work_events
    FOR VALUES FROM ('2026-05-01 00:00:00+00') TO ('2026-06-01 00:00:00+00');

-- 2026年6月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m06
    PARTITION OF work_events
    FOR VALUES FROM ('2026-06-01 00:00:00+00') TO ('2026-07-01 00:00:00+00');

-- 2026年7月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m07
    PARTITION OF work_events
    FOR VALUES FROM ('2026-07-01 00:00:00+00') TO ('2026-08-01 00:00:00+00');

-- 2026年8月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m08
    PARTITION OF work_events
    FOR VALUES FROM ('2026-08-01 00:00:00+00') TO ('2026-09-01 00:00:00+00');

-- 2026年9月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m09
    PARTITION OF work_events
    FOR VALUES FROM ('2026-09-01 00:00:00+00') TO ('2026-10-01 00:00:00+00');

-- 2026年10月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m10
    PARTITION OF work_events
    FOR VALUES FROM ('2026-10-01 00:00:00+00') TO ('2026-11-01 00:00:00+00');

-- 2026年11月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m11
    PARTITION OF work_events
    FOR VALUES FROM ('2026-11-01 00:00:00+00') TO ('2026-12-01 00:00:00+00');

-- 2026年12月分パーティション
CREATE TABLE IF NOT EXISTS work_events_y2026m12
    PARTITION OF work_events
    FOR VALUES FROM ('2026-12-01 00:00:00+00') TO ('2027-01-01 00:00:00+00');

-- Append-only 強制: app_event_writer ロールから UPDATE/DELETE を剥奪する
REVOKE UPDATE, DELETE ON work_events FROM PUBLIC;
REVOKE UPDATE, DELETE ON work_events FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON work_events TO app_event_writer;
