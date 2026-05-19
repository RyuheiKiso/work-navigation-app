-- V20260519120003__create_transaction_tables.sql
-- トランザクション系テーブルの全量作成
-- 作成順序（外部キー依存を考慮）:
--   work_orders (→ lots, sops)
--   work_executions (→ sops, master_versions, users, devices, work_orders)
--   work_events (→ work_executions, users, master_versions, steps, devices) [PARTITION BY RANGE]
--   evidence_files (→ work_events)
--   measurements (→ work_events, instruments)
--   suspensions (→ work_executions, users, electronic_signs)
--   andon_alerts (→ work_executions, users)
--   nonconformities (→ work_executions, users)
--   capas (→ nonconformities, users, electronic_signs)
--   kaizen_proposals (→ users, processes)
--   outbox_events
--   external_key_bindings (→ work_patterns)
--   hash_chain_blocks
--   auth_logs (→ users, devices)
--   idempotency_keys
--   incoming_inspections (→ lots, suppliers, materials, sampling_plans, users)
--   incoming_inspection_measurements (→ incoming_inspections, evidence_files)
--   concession_approvals (→ incoming_inspections, users, electronic_signs)
--   lot_qc_states (→ lots, incoming_inspections)
--   reworks (→ nonconformities, work_executions, lots, capas, master_versions, dispositions)
--   dispositions (→ nonconformities, electronic_signs) [トリガ含む]
--   rework_verifications (→ reworks, work_executions, users)
--   reworked_lot_labels (→ reworks, lots, users)
--   rework_cost_records (→ reworks)
--   scrap_records (→ reworks, users)
--   return_to_vendor_records (→ reworks, suppliers)
--   case_locks (→ work_executions, devices, users)
-- =====================================================

-- =====================================================
-- TBL-006: work_orders（ワークオーダーマスタ）
-- =====================================================
-- DDL-006: TBL-006 work_orders
-- EN-011 WorkOrder — ワークオーダーマスタ（ERP 連携または手動発行）
CREATE TABLE IF NOT EXISTS work_orders (
    work_order_id     UUID         NOT NULL DEFAULT gen_random_uuid(),
    work_order_code   VARCHAR(128) NOT NULL,
    lot_id            UUID         NULL,
    sop_id            UUID         NOT NULL,
    quantity_planned  INTEGER      NOT NULL,
    quantity_actual   INTEGER      NOT NULL DEFAULT 0,
    status            VARCHAR(16)  NOT NULL DEFAULT 'OPEN',
    due_date          DATE         NULL,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_orders PRIMARY KEY (work_order_id),
    CONSTRAINT uq_work_orders_code UNIQUE (work_order_code),
    CONSTRAINT fk_work_orders_lot FOREIGN KEY (lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_orders_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    CONSTRAINT ck_work_orders_status CHECK (
        status IN ('OPEN', 'IN_PROGRESS', 'COMPLETED', 'CANCELLED', 'ON_HOLD')
    ),
    CONSTRAINT ck_work_orders_quantity_planned_positive CHECK (quantity_planned > 0),
    CONSTRAINT ck_work_orders_quantity_actual_non_negative CHECK (quantity_actual >= 0)
);

COMMENT ON TABLE  work_orders IS 'EN-011 WorkOrder — ワークオーダー（製造指示）。ERP から同期または手動発行。work_executions から FK 参照される。永続保存。';
COMMENT ON COLUMN work_orders.work_order_code IS 'ERP 連携時の外部ワークオーダー番号。変更不可の公開識別子。';

-- =====================================================
-- TBL-005: work_executions（作業セッション管理テーブル）
-- =====================================================
-- DDL-005: TBL-005 work_executions
-- EN-011 WorkInstance — 作業セッション管理テーブル（更新可）
CREATE TABLE IF NOT EXISTS work_executions (
    work_execution_id    UUID        NOT NULL DEFAULT gen_random_uuid(),
    sop_id               UUID        NOT NULL,
    sop_version_id       UUID        NOT NULL,
    primary_worker_id    UUID        NOT NULL,
    device_id            UUID        NOT NULL,
    work_order_id        UUID        NULL,
    status               VARCHAR(16) NOT NULL DEFAULT 'NOT_STARTED',
    started_at           TIMESTAMPTZ NULL,
    completed_at         TIMESTAMPTZ NULL,
    current_step_index   SMALLINT    NOT NULL DEFAULT 0,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_executions PRIMARY KEY (work_execution_id),
    CONSTRAINT fk_work_executions_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_executions_version FOREIGN KEY (sop_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_executions_worker FOREIGN KEY (primary_worker_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_executions_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_executions_order FOREIGN KEY (work_order_id)
        REFERENCES work_orders (work_order_id) ON DELETE RESTRICT,
    CONSTRAINT ck_work_executions_status CHECK (
        status IN ('NOT_STARTED', 'IN_PROGRESS', 'SUSPENDED', 'COMPLETED', 'CANCELLED')
    ),
    CONSTRAINT ck_work_executions_step_index_non_negative CHECK (current_step_index >= 0),
    CONSTRAINT ck_work_executions_completed_after_started CHECK (
        NOT (completed_at IS NOT NULL AND started_at IS NULL)
        AND NOT (completed_at IS NOT NULL AND completed_at < started_at)
    )
);

COMMENT ON TABLE  work_executions IS 'EN-011 WorkInstance — 作業セッション管理。1 件 = 1 回の SOP 実行。work_events.case_id の外部キー参照元。7年以上保存。';
COMMENT ON COLUMN work_executions.sop_version_id     IS '作業開始時点の master_version_id を固定。SOP 改訂後も当時の版を参照可能にする。';
COMMENT ON COLUMN work_executions.current_step_index IS 'プレースキーパー。中断・再開時に次に実行するステップ位置を保持する。';

-- =====================================================
-- TBL-001: work_events（イベントストア・Append-only・月次パーティション・ハッシュチェーン）
-- =====================================================
-- DDL-001: TBL-001 work_events
-- EN-012 WorkEvent — イベントストアのコアテーブル。
-- Append-only。SHA-256 ハッシュチェーン。XES 互換。月次 RANGE パーティション。
CREATE TABLE IF NOT EXISTS work_events (
    event_id            UUID            NOT NULL DEFAULT gen_random_uuid(),
    case_id             UUID            NOT NULL,
    activity            VARCHAR(64)     NOT NULL,
    timestamp_client    TIMESTAMPTZ     NOT NULL,
    timestamp_server    TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    resource            UUID            NOT NULL,
    sop_version_id      UUID            NOT NULL,
    step_id             UUID            NULL,
    payload             JSONB           NOT NULL DEFAULT '{}',
    prev_hash           CHAR(64)        NOT NULL,
    content_hash        CHAR(64)        NOT NULL,
    terminal_id         UUID            NOT NULL,
    is_offline          BOOLEAN         NOT NULL DEFAULT FALSE,
    sync_lag_ms         INTEGER         NULL,
    server_received_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_events PRIMARY KEY (event_id, timestamp_server),
    CONSTRAINT fk_work_events_case FOREIGN KEY (case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_events_resource FOREIGN KEY (resource)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_events_sop_version FOREIGN KEY (sop_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_events_step FOREIGN KEY (step_id)
        REFERENCES steps (step_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_events_terminal FOREIGN KEY (terminal_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
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
    CONSTRAINT ck_work_events_hash_length CHECK (
        length(prev_hash) = 64 AND length(content_hash) = 64
    ),
    CONSTRAINT ck_work_events_sync_lag_non_negative CHECK (
        sync_lag_ms IS NULL OR sync_lag_ms >= 0
    ),
    CONSTRAINT ck_work_events_step_required_for_step_activities CHECK (
        NOT (activity IN ('step_completed', 'step_skipped', 'step_rejected', 'evidence_attached')
            AND step_id IS NULL)
    )
) PARTITION BY RANGE (timestamp_server);

COMMENT ON TABLE  work_events IS 'EN-012 WorkEvent — イベントストアコアテーブル。Append-only / ハッシュチェーン / XES 互換 / 月次パーティション。7年以上保存。';
COMMENT ON COLUMN work_events.event_id          IS 'UUID v7（時系列順）。Rust 側で生成。Idempotency Key として機能する。';
COMMENT ON COLUMN work_events.case_id           IS 'XES Case ID。work_executions.work_execution_id に対応する。同一作業セッション内の全イベントが共有する。';
COMMENT ON COLUMN work_events.activity          IS 'XES Activity。9 種の列挙値のみ許可（CHECK 制約）。';
COMMENT ON COLUMN work_events.timestamp_client  IS 'XES Timestamp（端末申告時刻）。オフライン時は端末ローカル時刻が記録される。ALCOA+ Contemporaneous 補足情報。';
COMMENT ON COLUMN work_events.timestamp_server  IS 'サーバー受信時刻（証拠時刻）。パーティション KEY。ALCOA+ Contemporaneous 主証拠。';
COMMENT ON COLUMN work_events.resource          IS 'XES Resource。実施者の user_id。';
COMMENT ON COLUMN work_events.sop_version_id    IS '時点参照固定。作業実施時点の master_version_id を記録し、SOP 改訂後も当時の版を特定可能にする。';
COMMENT ON COLUMN work_events.payload           IS 'activity 固有データ JSONB。スキーマは 05_JSONBスキーマ定義.md §4 参照（9 種それぞれ定義）。';
COMMENT ON COLUMN work_events.prev_hash         IS 'SHA-256 ハッシュチェーン前ブロック。チェーン先頭は "0000000000000000000000000000000000000000000000000000000000000000"。';
COMMENT ON COLUMN work_events.content_hash      IS 'このレコードの SHA-256。計算対象列: event_id + case_id + activity + timestamp_client + timestamp_server + resource + sop_version_id + payload(canonical JSON) + terminal_id + prev_hash。';
COMMENT ON COLUMN work_events.terminal_id       IS 'ALCOA+ Attributable 担保。どの端末で記録されたかを特定する。';
COMMENT ON COLUMN work_events.is_offline        IS 'TRUE のとき、オフライン状態で記録され後から同期されたことを示す。';
COMMENT ON COLUMN work_events.sync_lag_ms       IS 'オフライン同期遅延（ミリ秒）。is_offline=TRUE 時に設定される。NULL は即時記録。';
COMMENT ON COLUMN work_events.server_received_at IS 'サーバー受信時刻（UTC）。クライアントによる上書き不可。DEFAULT NOW() で強制付与する。ALCOA+ Contemporaneous 要件。';

-- 月次パーティション（2026年1月〜12月の初期定義）
-- 毎月25日に BAT-004 が翌月のパーティションを自動作成する
CREATE TABLE IF NOT EXISTS work_events_y2026m01
    PARTITION OF work_events
    FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-02-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m02
    PARTITION OF work_events
    FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m03
    PARTITION OF work_events
    FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m04
    PARTITION OF work_events
    FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m05
    PARTITION OF work_events
    FOR VALUES FROM ('2026-05-01 00:00:00+00') TO ('2026-06-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m06
    PARTITION OF work_events
    FOR VALUES FROM ('2026-06-01 00:00:00+00') TO ('2026-07-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m07
    PARTITION OF work_events
    FOR VALUES FROM ('2026-07-01 00:00:00+00') TO ('2026-08-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m08
    PARTITION OF work_events
    FOR VALUES FROM ('2026-08-01 00:00:00+00') TO ('2026-09-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m09
    PARTITION OF work_events
    FOR VALUES FROM ('2026-09-01 00:00:00+00') TO ('2026-10-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m10
    PARTITION OF work_events
    FOR VALUES FROM ('2026-10-01 00:00:00+00') TO ('2026-11-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m11
    PARTITION OF work_events
    FOR VALUES FROM ('2026-11-01 00:00:00+00') TO ('2026-12-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS work_events_y2026m12
    PARTITION OF work_events
    FOR VALUES FROM ('2026-12-01 00:00:00+00') TO ('2027-01-01 00:00:00+00');

-- =====================================================
-- TBL-003: outbox_events（Transactional Outbox 送信キュー）
-- =====================================================
-- DDL-003: TBL-003 outbox_events
-- EN-021 OutboxEvent — Outbox パターンの送信キュー。INSERT + status UPDATE のみ許可。
CREATE TABLE IF NOT EXISTS outbox_events (
    outbox_id        UUID        NOT NULL DEFAULT gen_random_uuid(),
    event_id         UUID        NOT NULL,
    event_type       VARCHAR(64) NOT NULL,
    status           VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    payload          JSONB       NOT NULL DEFAULT '{}',
    idempotency_key  UUID        NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at          TIMESTAMPTZ NULL,
    next_retry_at    TIMESTAMPTZ NULL,
    retry_count      SMALLINT    NOT NULL DEFAULT 0,
    dlq_reason       TEXT        NULL,

    CONSTRAINT pk_outbox_events PRIMARY KEY (outbox_id),
    CONSTRAINT uq_outbox_events_event_id UNIQUE (event_id),
    CONSTRAINT uq_outbox_events_idempotency UNIQUE (idempotency_key),
    CONSTRAINT ck_outbox_events_status CHECK (
        status IN ('PENDING', 'SENDING', 'SENT', 'FAILED', 'DLQ')
    ),
    CONSTRAINT ck_outbox_events_event_type CHECK (
        event_type IN (
            'work_event', 'electronic_sign', 'evidence_file',
            'measurement', 'suspension', 'andon_alert',
            'nonconformity', 'capa', 'kaizen_proposal',
            'incoming_inspection', 'incoming_inspection_measurement',
            'concession_approval', 'rework', 'disposition',
            'rework_verification', 'reworked_lot_label',
            'scrap_record', 'return_to_vendor'
        )
    ),
    CONSTRAINT ck_outbox_events_retry_non_negative CHECK (retry_count >= 0),
    CONSTRAINT ck_outbox_events_dlq_requires_reason CHECK (
        NOT (status = 'DLQ' AND dlq_reason IS NULL)
    )
);

COMMENT ON TABLE  outbox_events IS 'EN-021 OutboxEvent — Transactional Outbox パターン送信キュー。INSERT は常に許可。status UPDATE のみ app_write に許可（PENDING→SENDING→SENT 等）。90日後アーカイブ。';
COMMENT ON COLUMN outbox_events.idempotency_key IS 'バックエンド→外部送信時の Idempotency-Key ヘッダ値。重複送信防止。';
COMMENT ON COLUMN outbox_events.retry_count     IS '再送試行回数。上限は CFG-002（デフォルト 5）。上限到達時に DLQ ステータスへ遷移。';
COMMENT ON COLUMN outbox_events.dlq_reason      IS 'DLQ 移行理由テキスト。status=DLQ 時は必須（CHECK 制約）。';

-- =====================================================
-- TBL-009: evidence_files（証拠ファイルメタデータ・Append-only）
-- =====================================================
-- DDL-009: TBL-009 evidence_files
-- EN-013 EvidenceFile — 証拠ファイルメタデータ（Append-only）。バイナリは NAS 保存。
CREATE TABLE IF NOT EXISTS evidence_files (
    evidence_id       UUID        NOT NULL DEFAULT gen_random_uuid(),
    event_id          UUID        NOT NULL,
    file_type         VARCHAR(16) NOT NULL,
    file_path         TEXT        NOT NULL,
    file_hash         CHAR(64)    NOT NULL,
    file_size_bytes   INTEGER     NOT NULL,
    mime_type         VARCHAR(64) NOT NULL,
    captured_at       TIMESTAMPTZ NOT NULL,
    exif_stripped     BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    server_received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_evidence_files PRIMARY KEY (evidence_id),
    CONSTRAINT fk_evidence_files_event FOREIGN KEY (event_id)
        REFERENCES work_events (event_id, timestamp_server) ON DELETE RESTRICT
            DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT ck_evidence_files_type CHECK (
        file_type IN ('PHOTO', 'AUDIO', 'DOCUMENT', 'VIDEO')
    ),
    CONSTRAINT ck_evidence_files_size_positive CHECK (file_size_bytes > 0),
    CONSTRAINT ck_evidence_files_hash_length CHECK (length(file_hash) = 64)
);

COMMENT ON TABLE  evidence_files IS 'EN-013 EvidenceFile — 証拠ファイルメタデータ。Append-only。バイナリは NAS /evidence/ 配下に UUID 命名で保存。7年以上保存。';
COMMENT ON COLUMN evidence_files.file_path        IS 'NAS 上の相対パス。形式: {year}/{month}/{uuid}.{ext}。バイナリを DB に保存しない設計（NFR-PRF-015 対応）。';
COMMENT ON COLUMN evidence_files.file_hash        IS 'SHA-256 ハッシュ（64 文字 hex）。改ざん検知および重複排除に使用する。ALCOA+ Original 要件。';
COMMENT ON COLUMN evidence_files.exif_stripped     IS 'TRUE = Exif（位置情報・機器情報）削除済み。プライバシー保護のため撮影直後に除去する（アプリ層で処理）。';
COMMENT ON COLUMN evidence_files.created_at        IS 'サーバー受信時刻。captured_at がクライアント側の撮影時刻であるのに対し、こちらはサーバーへのアップロード受信時刻を示す。IDX-019 のインデックス対象列。';
COMMENT ON COLUMN evidence_files.server_received_at IS 'サーバー受信時刻（UTC）。クライアントによる上書き不可。ALCOA+ Contemporaneous 要件。';

-- =====================================================
-- TBL-010: measurements（計測値レコード・Append-only）
-- =====================================================
-- DDL-010: TBL-010 measurements
-- EN-014 Measurement — 計測値レコード（Append-only）。ALCOA+ Accurate 要件。
CREATE TABLE IF NOT EXISTS measurements (
    measurement_id      UUID            NOT NULL DEFAULT gen_random_uuid(),
    event_id            UUID            NOT NULL,
    instrument_id       UUID            NULL,
    calibration_ref     VARCHAR(128)    NULL,
    measured_value      NUMERIC(20, 6)  NOT NULL,
    uncertainty_u       NUMERIC(10, 6)  NULL,
    uncertainty_k       NUMERIC(5, 2)   NULL,
    unit_ucum           VARCHAR(32)     NOT NULL,
    display_value       NUMERIC(20, 6)  NULL,
    display_unit_ucum   VARCHAR(32)     NULL,
    judgment            VARCHAR(8)      NOT NULL,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    server_received_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_measurements PRIMARY KEY (measurement_id),
    CONSTRAINT fk_measurements_event FOREIGN KEY (event_id)
        REFERENCES work_events (event_id, timestamp_server) ON DELETE RESTRICT
            DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT fk_measurements_instrument FOREIGN KEY (instrument_id)
        REFERENCES instruments (instrument_id) ON DELETE RESTRICT,
    CONSTRAINT ck_measurements_judgment CHECK (
        judgment IN ('OK', 'NG', 'WARNING')
    ),
    CONSTRAINT ck_measurements_uncertainty_k_positive CHECK (
        uncertainty_k IS NULL OR uncertainty_k > 0
    )
);

COMMENT ON TABLE  measurements IS 'EN-014 Measurement — 計測値レコード。Append-only。SI 基本単位で保存し、表示単位は別列に持つ。7年以上保存。';
COMMENT ON COLUMN measurements.measured_value     IS 'SI 基本単位での測定値。例: 長さは mm（m でなく mm を基準とする社内規約による）。';
COMMENT ON COLUMN measurements.uncertainty_u      IS '測定不確かさ（拡張不確かさ U）。JCSS 校正証明書の値を転記する。';
COMMENT ON COLUMN measurements.uncertainty_k      IS '包含係数 k。通常 k=2（信頼水準約 95%）。';
COMMENT ON COLUMN measurements.unit_ucum          IS 'UCUM コード。例: mm, Cel（摂氏）, kPa, kg, s。';
COMMENT ON COLUMN measurements.calibration_ref    IS '使用した校正証明書番号または参照パス。instrument_id が NULL の場合でも記録可能。';
COMMENT ON COLUMN measurements.created_at         IS 'サーバー受信時刻。Append-only 全テーブルの created_at 必須方針（06_インデックス §1）に準拠。IDX-020 のインデックス対象列。';
COMMENT ON COLUMN measurements.server_received_at IS 'サーバー受信時刻（UTC）。クライアントによる上書き不可。ALCOA+ Contemporaneous 要件。';

-- =====================================================
-- TBL-011: suspensions（作業中断レコード・Append-only）
-- ADR-010: Append-only テーブルへの後埋め列禁止規約に準拠
-- resumed_at / resume_sign_id を持たない（再開は work_events.activity='work_resumed' で表現）
-- =====================================================
-- DDL-011: TBL-011 suspensions
-- EN-016 Suspension — 作業中断レコード（Append-only）
CREATE TABLE IF NOT EXISTS suspensions (
    suspension_id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    work_execution_id       UUID        NOT NULL,
    suspended_by            UUID        NOT NULL,
    suspend_reason_category VARCHAR(32) NOT NULL,
    suspend_comment         TEXT        NULL,
    sign_id                 UUID        NULL,
    suspended_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    step_index_at_suspend   SMALLINT    NOT NULL,

    CONSTRAINT pk_suspensions PRIMARY KEY (suspension_id),
    CONSTRAINT fk_suspensions_execution FOREIGN KEY (work_execution_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    CONSTRAINT fk_suspensions_suspended_by FOREIGN KEY (suspended_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_suspensions_sign FOREIGN KEY (sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    CONSTRAINT ck_suspensions_reason CHECK (
        suspend_reason_category IN (
            'MATERIAL_WAIT', 'EQUIPMENT_FAILURE', 'QUALITY_HOLD',
            'WORKER_ABSENCE', 'SAFETY_ISSUE', 'PROCESS_CHANGE', 'OTHER'
        )
    ),
    CONSTRAINT ck_suspensions_step_non_negative CHECK (step_index_at_suspend >= 0)
);

COMMENT ON TABLE  suspensions IS 'EN-016 Suspension — 作業中断レコード。Append-only。中断状態のみ記録する。再開は work_events.activity=''work_resumed'' イベントで記録する（ADR-010 準拠）。7年以上保存。';
COMMENT ON COLUMN suspensions.step_index_at_suspend IS '中断時の current_step_index。再開時に work_executions.current_step_index に転写される（アプリ層で制御）。';

-- =====================================================
-- TBL-012: andon_alerts（アンドン発報レコード）
-- =====================================================
-- DDL-012: TBL-012 andon_alerts
-- EN-017 AndonAlert — アンドン発報レコード（更新可）
CREATE TABLE IF NOT EXISTS andon_alerts (
    alert_id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    work_execution_id  UUID        NULL,
    raised_by          UUID        NOT NULL,
    alert_type         VARCHAR(32) NOT NULL,
    status             VARCHAR(16) NOT NULL DEFAULT 'ALERTING',
    raised_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_by    UUID        NULL,
    acknowledged_at    TIMESTAMPTZ NULL,
    resolved_by        UUID        NULL,
    resolved_at        TIMESTAMPTZ NULL,
    resolution_note    TEXT        NULL,

    CONSTRAINT pk_andon_alerts PRIMARY KEY (alert_id),
    CONSTRAINT fk_andon_alerts_execution FOREIGN KEY (work_execution_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    CONSTRAINT fk_andon_alerts_raised_by FOREIGN KEY (raised_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_andon_alerts_acknowledged_by FOREIGN KEY (acknowledged_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_andon_alerts_resolved_by FOREIGN KEY (resolved_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT ck_andon_alerts_type CHECK (
        alert_type IN ('QUALITY', 'EQUIPMENT', 'MATERIAL', 'PROCESS', 'SAFETY')
    ),
    CONSTRAINT ck_andon_alerts_status CHECK (
        status IN ('ALERTING', 'ACKNOWLEDGED', 'RESOLVED')
    ),
    CONSTRAINT ck_andon_alerts_resolved_requires_note CHECK (
        NOT (status = 'RESOLVED' AND resolution_note IS NULL)
    ),
    CONSTRAINT ck_andon_alerts_acknowledged_consistency CHECK (
        NOT (acknowledged_by IS NOT NULL AND acknowledged_at IS NULL)
    ),
    CONSTRAINT ck_andon_alerts_resolved_consistency CHECK (
        NOT (resolved_by IS NOT NULL AND resolved_at IS NULL)
    )
);

COMMENT ON TABLE  andon_alerts IS 'EN-017 AndonAlert — アンドン発報レコード。ALERTING→ACKNOWLEDGED→RESOLVED の順序で遷移。RESOLVED 時は resolution_note 必須（CHECK 制約）。5年以上保存。';

-- =====================================================
-- TBL-013: nonconformities（不適合レコード）
-- =====================================================
-- DDL-013: TBL-013 nonconformities
-- EN-018 Nonconformity — 不適合レコード（更新可）
CREATE TABLE IF NOT EXISTS nonconformities (
    nc_id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    work_execution_id  UUID        NULL,
    reported_by        UUID        NOT NULL,
    nc_category        VARCHAR(16) NOT NULL,
    description        TEXT        NOT NULL,
    status             VARCHAR(16) NOT NULL DEFAULT 'OPEN',
    opened_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at          TIMESTAMPTZ NULL,
    capa_id            UUID        NULL,

    CONSTRAINT pk_nonconformities PRIMARY KEY (nc_id),
    CONSTRAINT fk_nc_execution FOREIGN KEY (work_execution_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    CONSTRAINT fk_nc_reported_by FOREIGN KEY (reported_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT ck_nc_category CHECK (
        nc_category IN ('MAN', 'MACHINE', 'MATERIAL', 'METHOD', 'ENVIRONMENT')
    ),
    CONSTRAINT ck_nc_status CHECK (
        status IN ('OPEN', 'INVESTIGATING', 'CLOSED')
    ),
    CONSTRAINT ck_nc_closed_after_opened CHECK (
        NOT (closed_at IS NOT NULL AND closed_at < opened_at)
    )
);

COMMENT ON TABLE  nonconformities IS 'EN-018 Nonconformity — 不適合レコード。4M+E カテゴリで分類。CAPA（TBL-014）と関連付け可能。7年以上保存。';
COMMENT ON COLUMN nonconformities.nc_category IS '4M+E 分類: MAN（人）/ MACHINE（機械）/ MATERIAL（材料）/ METHOD（方法）/ ENVIRONMENT（環境）。';

-- =====================================================
-- TBL-014: capas（是正予防措置レコード）
-- =====================================================
-- DDL-014: TBL-014 capas
-- EN-019 CAPA — 是正予防措置レコード（更新可）
CREATE TABLE IF NOT EXISTS capas (
    capa_id      UUID        NOT NULL DEFAULT gen_random_uuid(),
    nc_id        UUID        NULL,
    capa_type    VARCHAR(16) NOT NULL,
    description  TEXT        NOT NULL,
    root_cause   TEXT        NULL,
    status       VARCHAR(24) NOT NULL DEFAULT 'OPEN',
    assigned_to  UUID        NOT NULL,
    sign_id      UUID        NULL,
    opened_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at    TIMESTAMPTZ NULL,

    CONSTRAINT pk_capas PRIMARY KEY (capa_id),
    CONSTRAINT fk_capas_nc FOREIGN KEY (nc_id)
        REFERENCES nonconformities (nc_id) ON DELETE RESTRICT,
    CONSTRAINT fk_capas_assigned_to FOREIGN KEY (assigned_to)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_capas_sign FOREIGN KEY (sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    CONSTRAINT ck_capas_type CHECK (
        capa_type IN ('CORRECTIVE', 'PREVENTIVE')
    ),
    CONSTRAINT ck_capas_status CHECK (
        status IN ('OPEN', 'CORRECTIVE_ACTION', 'VERIFICATION', 'CLOSED')
    ),
    CONSTRAINT ck_capas_closed_requires_root_cause CHECK (
        NOT (status = 'CLOSED' AND root_cause IS NULL)
    ),
    CONSTRAINT ck_capas_closed_after_opened CHECK (
        NOT (closed_at IS NOT NULL AND closed_at < opened_at)
    )
);

COMMENT ON TABLE  capas IS 'EN-019 CAPA — 是正予防措置。CLOSED 時は root_cause 必須（CHECK 制約）。sign_id により承認電子サインと連携。7年以上保存。';

-- nonconformities → capas の循環 FK を後から設定
ALTER TABLE nonconformities
    ADD CONSTRAINT fk_nc_capa FOREIGN KEY (capa_id)
        REFERENCES capas (capa_id) ON DELETE RESTRICT;

-- =====================================================
-- TBL-015: kaizen_proposals（改善提案レコード）
-- =====================================================
-- DDL-015: TBL-015 kaizen_proposals
-- EN-020 Kaizen — 改善提案レコード（更新可）
CREATE TABLE IF NOT EXISTS kaizen_proposals (
    kaizen_id          UUID         NOT NULL DEFAULT gen_random_uuid(),
    proposed_by        UUID         NOT NULL,
    category           VARCHAR(64)  NOT NULL,
    title              VARCHAR(256) NOT NULL,
    description        TEXT         NOT NULL,
    target_process_id  UUID         NULL,
    status             VARCHAR(16)  NOT NULL DEFAULT 'PROPOSED',
    proposed_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    reviewed_at        TIMESTAMPTZ  NULL,
    reviewed_by        UUID         NULL,

    CONSTRAINT pk_kaizen_proposals PRIMARY KEY (kaizen_id),
    CONSTRAINT fk_kaizen_proposed_by FOREIGN KEY (proposed_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_kaizen_target_process FOREIGN KEY (target_process_id)
        REFERENCES processes (process_id) ON DELETE RESTRICT,
    CONSTRAINT fk_kaizen_reviewed_by FOREIGN KEY (reviewed_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT ck_kaizen_status CHECK (
        status IN ('PROPOSED', 'REVIEWING', 'ADOPTED', 'REJECTED')
    ),
    CONSTRAINT ck_kaizen_reviewed_consistency CHECK (
        NOT (reviewed_by IS NOT NULL AND reviewed_at IS NULL)
    )
);

COMMENT ON TABLE  kaizen_proposals IS 'EN-020 Kaizen — 改善提案。PROPOSED→REVIEWING→ADOPTED/REJECTED の遷移。5年以上保存。';

-- =====================================================
-- TBL-027: external_key_bindings（外部キーマッピング・Append-only）
-- =====================================================
-- DDL-027: TBL-027 external_key_bindings
-- EN-022 ExternalKeyBinding — 外部 ERP/MES キーと内部 work_pattern_id のマッピング（Append-only）
-- NOTE: factory_id は予約フィールド。ver1.0.0 では定数 UUID '00000000-0000-7000-8000-000000000001' を使用する。
CREATE TABLE IF NOT EXISTS external_key_bindings (
    binding_id       UUID        NOT NULL DEFAULT gen_random_uuid(),
    external_system  VARCHAR(64) NOT NULL,
    external_key     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    work_pattern_id  UUID        NOT NULL,
    valid_from       DATE        NOT NULL,
    valid_to         DATE        NULL,
    sync_status      VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
    factory_id       UUID        NOT NULL,

    CONSTRAINT pk_external_key_bindings PRIMARY KEY (binding_id),
    CONSTRAINT fk_external_key_work_pattern FOREIGN KEY (work_pattern_id)
        REFERENCES work_patterns (work_pattern_id) ON DELETE RESTRICT,
    CONSTRAINT ck_external_key_sync_status CHECK (
        sync_status IN ('ACTIVE', 'CONFLICT', 'DEPRECATED')
    ),
    CONSTRAINT ck_external_key_valid_range CHECK (
        valid_to IS NULL OR valid_to >= valid_from
    ),
    CONSTRAINT ck_external_key_is_object CHECK (
        jsonb_typeof(external_key) = 'object'
    )
);

COMMENT ON TABLE  external_key_bindings IS 'EN-022 ExternalKeyBinding — 外部キーマッピング。Append-only。変更時は旧レコードの valid_to を設定 + 新レコード INSERT の 2 件操作が必須。自動解決禁止。';
COMMENT ON COLUMN external_key_bindings.external_key IS 'JSONB 形式の外部キー。例: {"lot_id": "L001", "product_code": "P-A001-REV2"}。GIN インデックス（IDX-013）で検索する。';
COMMENT ON COLUMN external_key_bindings.sync_status  IS 'ACTIVE: 有効 / CONFLICT: 複数マッピング競合（手動解決必要）/ DEPRECATED: 廃止。';
COMMENT ON COLUMN external_key_bindings.factory_id   IS 'ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。';

-- =====================================================
-- TBL-031: hash_chain_blocks（SHA-256 ハッシュチェーン週次チェックポイント・Append-only）
-- =====================================================
-- DDL-031: TBL-031 hash_chain_blocks
-- EN-025 HashChainBlock — SHA-256 ハッシュチェーンの週次チェックポイント（Append-only）
CREATE TABLE IF NOT EXISTS hash_chain_blocks (
    block_id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    block_period       DATE        NOT NULL,
    event_count        INTEGER     NOT NULL,
    last_event_id      UUID        NOT NULL,
    last_content_hash  CHAR(64)    NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    block_hash         CHAR(64)    NOT NULL,
    server_received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_hash_chain_blocks PRIMARY KEY (block_id),
    CONSTRAINT uq_hash_chain_blocks_period UNIQUE (block_period),
    CONSTRAINT ck_hash_chain_event_count_positive CHECK (event_count > 0),
    CONSTRAINT ck_hash_chain_hash_lengths CHECK (
        length(last_content_hash) = 64 AND length(block_hash) = 64
    )
);

COMMENT ON TABLE  hash_chain_blocks IS 'EN-025 HashChainBlock — 週次ハッシュチェーンチェックポイント。BAT-001 が週次で生成する。7年以上保存。Append-only。';
COMMENT ON COLUMN hash_chain_blocks.block_period       IS '集計週の開始日（月曜日）。UNIQUE 制約により週次 1 レコードを保証。';
COMMENT ON COLUMN hash_chain_blocks.last_content_hash  IS '期間内最終 WorkEvent の content_hash。次週の BAT-001 がチェーン継続点として使用する。';
COMMENT ON COLUMN hash_chain_blocks.block_hash         IS 'このブロック自体のハッシュ。SHA-256(block_period || last_content_hash)。';
COMMENT ON COLUMN hash_chain_blocks.server_received_at IS 'サーバー受信時刻（UTC）。クライアントによる上書き不可。ALCOA+ Contemporaneous 要件。';

-- =====================================================
-- TBL-032: auth_logs（認証イベントログ・Append-only）
-- =====================================================
-- DDL-032: TBL-032 auth_logs
-- EN-022 AuthLog — 認証イベントログ（Append-only）
CREATE TABLE IF NOT EXISTS auth_logs (
    log_id             UUID        NOT NULL DEFAULT gen_random_uuid(),
    user_id            UUID        NULL,
    action             VARCHAR(64) NOT NULL,
    device_id          UUID        NULL,
    ip_address         INET        NULL,
    occurred_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    server_received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_auth_logs PRIMARY KEY (log_id),
    CONSTRAINT fk_auth_logs_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_auth_logs_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    CONSTRAINT ck_auth_logs_action CHECK (
        action IN (
            'LOGIN_SUCCESS', 'LOGIN_FAILURE', 'LOGOUT',
            'TOKEN_REFRESH', 'PASSWORD_CHANGE', 'ACCOUNT_LOCKED'
        )
    )
);

COMMENT ON TABLE  auth_logs IS 'EN-022 AuthLog — 認証イベントログ。Append-only。user_id は NULL 許容（未認証ユーザーの失敗ログ）。90日保存後アーカイブ。';
COMMENT ON COLUMN auth_logs.ip_address       IS 'クライアント IP アドレス。プライバシー保護のため /24 マスク推奨（例: 192.168.1.0）。アプリ層でマスク処理する。';
COMMENT ON COLUMN auth_logs.user_id          IS 'LOGIN_FAILURE 時は NULL または存在しない user_id の可能性あり。外部キー制約は ON DELETE RESTRICT のため廃止ユーザーの参照が残る。';
COMMENT ON COLUMN auth_logs.server_received_at IS 'サーバー受信時刻（UTC）。クライアントによる上書き不可。ALCOA+ Contemporaneous 要件。';

-- =====================================================
-- TBL-035: idempotency_keys（API 冪等性キーキャッシュ・TTL 24h）
-- =====================================================
-- DDL-035: TBL-035 idempotency_keys
-- 制御テーブル — API 冪等性キーキャッシュ。TTL 24 時間。唯一 DELETE が許可される制御テーブル。
CREATE TABLE IF NOT EXISTS idempotency_keys (
    idempotency_key  UUID         NOT NULL,
    endpoint         VARCHAR(128) NOT NULL,
    response_status  SMALLINT     NOT NULL,
    response_body    JSONB        NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_idempotency_keys PRIMARY KEY (idempotency_key),
    CONSTRAINT ck_idempotency_response_status CHECK (
        response_status BETWEEN 100 AND 599
    )
);

COMMENT ON TABLE  idempotency_keys IS '制御テーブル。API 冪等性キーキャッシュ（TTL 24 時間）。pg_cron または BAT-003 が 24 時間超のレコードを DELETE する（全体方針の DELETE 禁止の唯一例外）。';
COMMENT ON COLUMN idempotency_keys.idempotency_key IS 'クライアントが送信する Idempotency-Key ヘッダ値（UUID v4）。PRIMARY KEY により一意性を保証。';
COMMENT ON COLUMN idempotency_keys.response_body   IS '最初のリクエスト応答 JSON。同一キーの再送時にこのボディを返却する（アプリ層で制御）。';

-- =====================================================
-- TBL-038: incoming_inspections（入荷ロット受入検査ヘッダ）
-- =====================================================
-- DDL-038: TBL-038 incoming_inspections
-- EN-030 IncomingInspection — 入荷ロット受入検査ヘッダ（限定可変）
CREATE TABLE IF NOT EXISTS incoming_inspections (
    inspection_id           UUID            NOT NULL DEFAULT gen_random_uuid(),
    lot_id                  UUID            NOT NULL,
    supplier_id             UUID            NOT NULL,
    material_id             UUID            NOT NULL,
    sampling_plan_id        UUID            NOT NULL,
    sampling_plan_version   INTEGER         NOT NULL,
    lot_quantity            INTEGER         NOT NULL,
    sample_size_n           INTEGER         NOT NULL,
    accept_number_ac        INTEGER         NOT NULL,
    reject_number_re        INTEGER         NOT NULL,
    severity_state          TEXT            NOT NULL DEFAULT 'NORMAL',
    qc_status               TEXT            NOT NULL DEFAULT 'PENDING',
    inspector_id            UUID            NOT NULL,
    received_at             TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    judged_at               TIMESTAMPTZ     NULL,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- ADR-011: ハッシュチェーン列（IQC per qc_case_id genesis 方式）
    qc_case_id              UUID            NOT NULL,
    prev_hash               CHAR(64)        NOT NULL,
    content_hash            CHAR(64)        NOT NULL,

    CONSTRAINT pk_incoming_inspections PRIMARY KEY (inspection_id),
    CONSTRAINT fk_incoming_insp_lot       FOREIGN KEY (lot_id)          REFERENCES lots (lot_id),
    CONSTRAINT fk_incoming_insp_supplier  FOREIGN KEY (supplier_id)     REFERENCES suppliers (supplier_id),
    CONSTRAINT fk_incoming_insp_material  FOREIGN KEY (material_id)     REFERENCES materials (material_id),
    CONSTRAINT fk_incoming_insp_plan      FOREIGN KEY (sampling_plan_id) REFERENCES sampling_plans (plan_id),
    CONSTRAINT fk_incoming_insp_inspector FOREIGN KEY (inspector_id)    REFERENCES users (user_id),
    CONSTRAINT ck_incoming_insp_severity CHECK (
        severity_state IN ('NORMAL', 'TIGHTENED', 'REDUCED')
    ),
    CONSTRAINT ck_incoming_insp_status CHECK (
        qc_status IN ('PENDING', 'INSPECTING', 'PASSED', 'CONDITIONAL_PASS', 'SCREENING_REQUIRED', 'REJECTED', 'SCRAPPED', 'RETURNED')
    ),
    CONSTRAINT ck_incoming_insp_qty CHECK (lot_quantity > 0 AND sample_size_n > 0),
    CONSTRAINT ck_incoming_insp_acre CHECK (accept_number_ac >= 0 AND reject_number_re > accept_number_ac),
    CONSTRAINT ck_incoming_insp_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

COMMENT ON TABLE  incoming_inspections IS 'EN-030 IncomingInspection — 入荷受入検査ヘッダ。qc_status のみ UPDATE 可。per qc_case_id genesis ハッシュチェーン（ADR-011）により改ざん検知を保証。';
COMMENT ON COLUMN incoming_inspections.sampling_plan_version IS 'sampling_plans.version の時点固定コピー。サンプリング計画改訂後も判定根拠が追跡可能。';
COMMENT ON COLUMN incoming_inspections.severity_state IS 'JIS Z 9015-1 §10 の検査の厳しさ状態（なみ/きつい/ゆるい）。';
COMMENT ON COLUMN incoming_inspections.qc_case_id IS 'ハッシュチェーン単位 ID。incoming_inspections では genesis として自身の inspection_id を設定する（ADR-011）。';
COMMENT ON COLUMN incoming_inspections.prev_hash IS '前ブロックの content_hash（genesis は "0"×64）。';
COMMENT ON COLUMN incoming_inspections.content_hash IS '本レコードの SHA-256（inspection_id / lot_id / supplier_id / material_id / sampling_plan_id / sampling_plan_version / lot_quantity / sample_size_n / accept_number_ac / reject_number_re / severity_state / inspector_id / received_at の canonical JSON）。qc_status・judged_at は可変フィールドのためハッシュ対象外。';

-- =====================================================
-- TBL-040: incoming_inspection_measurements（サンプル測定値明細・Append-only）
-- =====================================================
-- DDL-040: TBL-040 incoming_inspection_measurements
-- EN-030 詳細 — サンプル測定値明細（Append-only）
CREATE TABLE IF NOT EXISTS incoming_inspection_measurements (
    measurement_id   UUID            NOT NULL DEFAULT gen_random_uuid(),
    inspection_id    UUID            NOT NULL,
    sample_no        INTEGER         NOT NULL,
    measured_value   NUMERIC(18,6)   NULL,
    defect_flag      BOOLEAN         NOT NULL DEFAULT FALSE,
    evidence_file_id UUID            NULL,
    measured_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- ADR-011: ハッシュチェーン列
    qc_case_id       UUID            NOT NULL,
    prev_hash        CHAR(64)        NOT NULL,
    content_hash     CHAR(64)        NOT NULL,

    CONSTRAINT pk_incoming_insp_meas PRIMARY KEY (measurement_id),
    CONSTRAINT fk_insp_meas_inspection FOREIGN KEY (inspection_id) REFERENCES incoming_inspections (inspection_id),
    CONSTRAINT fk_insp_meas_evidence   FOREIGN KEY (evidence_file_id) REFERENCES evidence_files (evidence_id),
    CONSTRAINT ck_insp_meas_sample_no  CHECK (sample_no >= 1),
    CONSTRAINT ck_insp_meas_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

COMMENT ON TABLE incoming_inspection_measurements IS 'EN-030 詳細 — Append-only。サンプル 1 個の測定値・証拠写真を記録する。UPDATE/DELETE 禁止。qc_case_id = inspection_id でチェーン（ADR-011）。';
COMMENT ON COLUMN incoming_inspection_measurements.qc_case_id IS 'ハッシュチェーン単位 ID（= inspection_id）。同一検査内のサンプル列を時系列で連結する。';
COMMENT ON COLUMN incoming_inspection_measurements.prev_hash IS '前ブロックの content_hash（同一 qc_case_id 内の直前レコード。初回は "0"×64）。';
COMMENT ON COLUMN incoming_inspection_measurements.content_hash IS '本レコードの SHA-256（inspection_id / sample_no / measured_value / defect_flag / measured_at の canonical JSON）。';

-- =====================================================
-- TBL-041: concession_approvals（特採承認記録・Append-only）
-- =====================================================
-- DDL-041: TBL-041 concession_approvals
-- EN-030 承認詳細 — 特採承認記録（Append-only）
CREATE TABLE IF NOT EXISTS concession_approvals (
    approval_id         UUID            NOT NULL DEFAULT gen_random_uuid(),
    inspection_id       UUID            NOT NULL,
    decision            TEXT            NOT NULL DEFAULT 'CONCESSION',
    reason              TEXT            NOT NULL,
    validity_scope      JSONB           NOT NULL DEFAULT '{}',
    valid_until         DATE            NULL,
    approver_id         UUID            NOT NULL,
    electronic_sign_id  UUID            NOT NULL,
    approved_at         TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- ADR-011: ハッシュチェーン列
    qc_case_id          UUID            NOT NULL,
    prev_hash           CHAR(64)        NOT NULL,
    content_hash        CHAR(64)        NOT NULL,

    CONSTRAINT pk_concession_approvals PRIMARY KEY (approval_id),
    CONSTRAINT fk_concession_inspection FOREIGN KEY (inspection_id)      REFERENCES incoming_inspections (inspection_id),
    CONSTRAINT fk_concession_approver   FOREIGN KEY (approver_id)        REFERENCES users (user_id),
    CONSTRAINT fk_concession_sign       FOREIGN KEY (electronic_sign_id) REFERENCES electronic_signs (sign_id),
    CONSTRAINT ck_concession_reason     CHECK (length(trim(reason)) > 0),
    CONSTRAINT ck_concession_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

COMMENT ON TABLE concession_approvals IS 'EN-030 承認詳細 — 特採承認は Append-only。valid_until を超過した場合は BAT-009 拡張が lot_qc_states を REJECTED に遷移させる。qc_case_id = inspection_id でチェーン（ADR-011）。';
COMMENT ON COLUMN concession_approvals.qc_case_id IS 'ハッシュチェーン単位 ID（= inspection_id）。同一検査の特採承認列を連結する。';
COMMENT ON COLUMN concession_approvals.content_hash IS '本レコードの SHA-256（inspection_id / decision / reason / approver_id / approved_at の canonical JSON）。';

-- =====================================================
-- TBL-042: lot_qc_states（ロット現在 QC ステータス・後工程ゲート判定用）
-- =====================================================
-- DDL-042: TBL-042 lot_qc_states
-- EN-030 × EN-021 — ロット現在 QC ステータス（後工程ゲート判定用・更新可）
CREATE TABLE IF NOT EXISTS lot_qc_states (
    lot_id              UUID    NOT NULL,
    qc_status           TEXT    NOT NULL DEFAULT 'PENDING',
    last_inspection_id  UUID    NOT NULL,
    last_updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_lot_qc_states PRIMARY KEY (lot_id),
    CONSTRAINT fk_lot_qc_lot        FOREIGN KEY (lot_id)             REFERENCES lots (lot_id),
    CONSTRAINT fk_lot_qc_inspection FOREIGN KEY (last_inspection_id) REFERENCES incoming_inspections (inspection_id),
    CONSTRAINT ck_lot_qc_status CHECK (
        qc_status IN ('PENDING', 'INSPECTING', 'PASSED', 'CONDITIONAL_PASS', 'SCREENING_REQUIRED', 'REJECTED', 'SCRAPPED', 'RETURNED')
    )
);

COMMENT ON TABLE lot_qc_states IS 'EN-030×EN-021 — ロットの現在 QC ステータス。後工程 QR スキャン時の ERR-BIZ-015 ゲート判定に使用する。qc_status は incoming_inspections.qc_status 変化に連動して UPDATE する。';

-- =====================================================
-- TBL-043: reworks（リワーク作業ヘッダ）
-- dispositions より先に宣言するが dispositions FK は後で追加する
-- =====================================================
-- DDL-043: TBL-043 reworks
-- EN-032 Rework — リワーク作業ヘッダ（限定可変: status のみ更新可）
CREATE TABLE IF NOT EXISTS reworks (
    rework_id               UUID            NOT NULL DEFAULT gen_random_uuid(),
    parent_nonconformity_id UUID            NOT NULL,
    parent_case_id          UUID            NOT NULL,
    parent_lot_id           UUID            NULL,
    related_capa_id         UUID            NULL,
    rework_type             TEXT            NOT NULL,
    status                  TEXT            NOT NULL DEFAULT 'PENDING_DISPOSITION',
    rework_case_id          UUID            NULL,
    rework_sop_version_id   UUID            NULL,
    disposition_id          UUID            NULL,
    due_date                DATE            NULL,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_reworks PRIMARY KEY (rework_id),
    CONSTRAINT fk_reworks_nonconformity FOREIGN KEY (parent_nonconformity_id) REFERENCES nonconformities (nc_id),
    CONSTRAINT fk_reworks_parent_case   FOREIGN KEY (parent_case_id)          REFERENCES work_executions (work_execution_id),
    CONSTRAINT fk_reworks_parent_lot    FOREIGN KEY (parent_lot_id)           REFERENCES lots (lot_id),
    CONSTRAINT fk_reworks_capa          FOREIGN KEY (related_capa_id)         REFERENCES capas (capa_id),
    CONSTRAINT fk_reworks_rework_case   FOREIGN KEY (rework_case_id)          REFERENCES work_executions (work_execution_id),
    CONSTRAINT fk_reworks_sop_version   FOREIGN KEY (rework_sop_version_id)   REFERENCES master_versions (master_version_id),
    CONSTRAINT ck_reworks_type CHECK (
        rework_type IN ('TOUCH_UP', 'REWORK_FULL', 'SORTING', 'SCRAP', 'RETURN')
    ),
    CONSTRAINT ck_reworks_status CHECK (
        status IN ('PENDING_DISPOSITION', 'DISPOSITION_DECIDED', 'REWORK_IN_PROGRESS', 'REWORK_COMPLETED', 'VERIFICATION_IN_PROGRESS', 'CLOSED_OK_RELEASE', 'CLOSED_DOWNGRADE', 'CLOSED_SCRAP', 'CLOSED_RETURN', 'RE_REWORK_NEEDED')
    )
);

COMMENT ON TABLE  reworks IS 'EN-032 Rework — リワーク作業ヘッダ。status のみ UPDATE 可。parent_case_id は ALCOA+ Original 原則（NFR-DQ-010）に従い不変参照のみ。rework_case_id が新規 WorkExecution を指す。';
COMMENT ON COLUMN reworks.parent_case_id IS '元 WorkExecution ID。ALCOA+ Original — この FK が指すレコードは本テーブルから一切 UPDATE/DELETE しない。';
COMMENT ON COLUMN reworks.rework_case_id IS 'リワーク作業用の新 WorkExecution ID（execution_type=REWORK）。リワーク着手時に採番。';

-- =====================================================
-- TBL-044: dispositions（ディスポジション判定・Append-only）
-- Two-Person Integrity: quality_admin_sign_id と supervisor_sign_id は異なる worker_id でなければならない
-- =====================================================
-- DDL-044: TBL-044 dispositions
-- EN-033 Disposition — ディスポジション判定（Append-only）
CREATE TABLE IF NOT EXISTS dispositions (
    disposition_id          UUID    NOT NULL DEFAULT gen_random_uuid(),
    nonconformity_id        UUID    NOT NULL,
    decision                TEXT    NOT NULL,
    decision_reason         TEXT    NOT NULL,
    quality_admin_sign_id   UUID    NOT NULL,
    supervisor_sign_id      UUID    NOT NULL,
    decided_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ADR-011: ハッシュチェーン列
    qc_case_id              UUID    NOT NULL,
    prev_hash               CHAR(64) NOT NULL,
    content_hash            CHAR(64) NOT NULL,

    CONSTRAINT pk_dispositions PRIMARY KEY (disposition_id),
    CONSTRAINT fk_disp_nonconformity  FOREIGN KEY (nonconformity_id)      REFERENCES nonconformities (nc_id),
    CONSTRAINT fk_disp_qa_sign        FOREIGN KEY (quality_admin_sign_id) REFERENCES electronic_signs (sign_id),
    CONSTRAINT fk_disp_sup_sign       FOREIGN KEY (supervisor_sign_id)    REFERENCES electronic_signs (sign_id),
    CONSTRAINT ck_disp_decision CHECK (
        decision IN ('REWORK', 'SCRAP', 'RETURN', 'USE_AS_IS')
    ),
    CONSTRAINT ck_disp_reason CHECK (length(trim(decision_reason)) > 0),
    CONSTRAINT ck_disp_distinct_signs CHECK (quality_admin_sign_id <> supervisor_sign_id),
    CONSTRAINT ck_disp_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

-- DB トリガ: 署名者 worker_id の異一性検証（NFR-SEC-048）
CREATE OR REPLACE FUNCTION check_disposition_distinct_signers()
RETURNS TRIGGER AS $$
DECLARE
    qa_signer  UUID;
    sup_signer UUID;
BEGIN
    SELECT signer_id INTO qa_signer  FROM electronic_signs WHERE sign_id = NEW.quality_admin_sign_id;
    SELECT signer_id INTO sup_signer FROM electronic_signs WHERE sign_id = NEW.supervisor_sign_id;
    IF qa_signer = sup_signer THEN
        RAISE EXCEPTION 'ERR-BIZ-021: disposition requires two distinct signers (same worker_id detected)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_disposition_distinct_signers
    BEFORE INSERT ON dispositions
    FOR EACH ROW EXECUTE FUNCTION check_disposition_distinct_signers();

COMMENT ON TABLE  dispositions IS 'EN-033 Disposition — Append-only。Two-Person Integrity をトリガ check_disposition_distinct_signers で保証（NFR-SEC-048）。qc_case_id = nonconformity_id でチェーン（ADR-011）。';
COMMENT ON COLUMN dispositions.qc_case_id IS 'ハッシュチェーン単位 ID（= nonconformity_id）。同一 NC に対する複数ディスポジションを時系列で連結する。';
COMMENT ON COLUMN dispositions.content_hash IS '本レコードの SHA-256（nonconformity_id / decision / quality_admin_sign_id / supervisor_sign_id / decided_at の canonical JSON）。';

-- reworks → dispositions の循環 FK を後から設定
ALTER TABLE reworks
    ADD CONSTRAINT fk_reworks_disposition FOREIGN KEY (disposition_id)
        REFERENCES dispositions (disposition_id);

-- =====================================================
-- TBL-045: rework_verifications（リワーク検証・Append-only）
-- =====================================================
-- DDL-045: TBL-045 rework_verifications（Append-only）
CREATE TABLE IF NOT EXISTS rework_verifications (
    verification_id            UUID    NOT NULL DEFAULT gen_random_uuid(),
    rework_id                  UUID    NOT NULL,
    verification_case_id       UUID    NOT NULL,
    verifier_id                UUID    NOT NULL,
    verdict                    TEXT    NOT NULL,
    follow_up_nonconformity_id UUID    NULL,
    verified_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ADR-011: ハッシュチェーン列
    qc_case_id                 UUID    NOT NULL,
    prev_hash                  CHAR(64) NOT NULL,
    content_hash               CHAR(64) NOT NULL,

    CONSTRAINT pk_rework_verifications PRIMARY KEY (verification_id),
    CONSTRAINT fk_rv_rework   FOREIGN KEY (rework_id)             REFERENCES reworks (rework_id),
    CONSTRAINT fk_rv_case     FOREIGN KEY (verification_case_id)  REFERENCES work_executions (work_execution_id),
    CONSTRAINT fk_rv_verifier FOREIGN KEY (verifier_id)           REFERENCES users (user_id),
    CONSTRAINT ck_rv_verdict  CHECK (verdict IN ('OK', 'NG', 'DOWNGRADE')),
    CONSTRAINT ck_rv_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

COMMENT ON TABLE rework_verifications IS 'EN-034 ReworkVerification — Append-only。verifier_id ≠ リワーク実施者（API 層で ERR-BIZ-023 を返す）。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN rework_verifications.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。';
COMMENT ON COLUMN rework_verifications.content_hash IS '本レコードの SHA-256（rework_id / verifier_id / verdict / verified_at の canonical JSON）。';

-- =====================================================
-- TBL-047: reworked_lot_labels（修正品 QR ラベル・Append-only）
-- =====================================================
-- DDL-047: TBL-047 reworked_lot_labels（Append-only）
CREATE TABLE IF NOT EXISTS reworked_lot_labels (
    label_id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    rework_id             UUID        NOT NULL,
    qr_payload            TEXT        NOT NULL,
    giai                  VARCHAR(30) NOT NULL UNIQUE,
    parent_lot_id         UUID        NOT NULL,
    rework_sop_version_id UUID        NOT NULL,
    issued_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    issued_by             UUID        NOT NULL,
    -- ADR-011: ハッシュチェーン列
    qc_case_id            UUID        NOT NULL,
    prev_hash             CHAR(64)    NOT NULL,
    content_hash          CHAR(64)    NOT NULL,

    CONSTRAINT pk_reworked_lot_labels PRIMARY KEY (label_id),
    CONSTRAINT fk_rll_rework FOREIGN KEY (rework_id)             REFERENCES reworks (rework_id),
    CONSTRAINT fk_rll_lot    FOREIGN KEY (parent_lot_id)         REFERENCES lots (lot_id),
    CONSTRAINT fk_rll_issuer FOREIGN KEY (issued_by)             REFERENCES users (user_id),
    CONSTRAINT ck_rll_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

COMMENT ON TABLE reworked_lot_labels IS 'EN-036 ReworkedLotLabel — GS1 AI 8003（GIAI）+ AI 91 形式の修正品 QR ラベル。Append-only。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN reworked_lot_labels.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。';
COMMENT ON COLUMN reworked_lot_labels.content_hash IS '本レコードの SHA-256（rework_id / giai / parent_lot_id / issued_by / issued_at の canonical JSON）。';

-- =====================================================
-- TBL-048: rework_cost_records（リワークコスト集計・BAT-011 上書き可）
-- =====================================================
-- DDL-048: TBL-048 rework_cost_records（BAT-011 が日次上書き）
-- NOTE: BAT-011 が日次で上書きするため Append-only 対象外。ハッシュチェーン非適用（ADR-011 §除外対象）。
CREATE TABLE IF NOT EXISTS rework_cost_records (
    record_id                   UUID    NOT NULL DEFAULT gen_random_uuid(),
    rework_id                   UUID    NOT NULL UNIQUE,
    additional_labor_seconds    INTEGER     NOT NULL DEFAULT 0,
    additional_material_cost_yen NUMERIC(12,2) NOT NULL DEFAULT 0,
    scrap_loss_yen              NUMERIC(12,2) NOT NULL DEFAULT 0,
    aggregated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_rework_cost_records PRIMARY KEY (record_id),
    CONSTRAINT fk_rcr_rework FOREIGN KEY (rework_id) REFERENCES reworks (rework_id)
);

COMMENT ON TABLE rework_cost_records IS 'EN-037 ReworkCostRecord — BAT-011 が日次で集計・上書きする（唯一の上書き可能集計テーブル）。ハッシュチェーン非対象（ADR-011）。';

-- =====================================================
-- TBL-049: scrap_records（廃棄記録・Append-only）
-- =====================================================
-- DDL-049: TBL-049 scrap_records（Append-only）
CREATE TABLE IF NOT EXISTS scrap_records (
    rework_id              UUID        NOT NULL UNIQUE,
    waste_manifest_pdf_id  UUID        NULL,
    waste_classification   VARCHAR(64) NOT NULL DEFAULT '',
    witness_id             UUID        NOT NULL,
    recorded_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ADR-011: ハッシュチェーン列
    qc_case_id             UUID        NOT NULL,
    prev_hash              CHAR(64)    NOT NULL,
    content_hash           CHAR(64)    NOT NULL,

    CONSTRAINT pk_scrap_records PRIMARY KEY (rework_id),
    CONSTRAINT fk_sr_rework  FOREIGN KEY (rework_id)  REFERENCES reworks (rework_id),
    CONSTRAINT fk_sr_witness FOREIGN KEY (witness_id) REFERENCES users (user_id),
    CONSTRAINT ck_sr_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

COMMENT ON TABLE scrap_records IS 'EN-038 ScrapRecord — Append-only。廃棄物処理票・立会者サイン（witness_id ≠ 廃却実施者を API 層で検証）。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN scrap_records.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。scrap_records は 1 rework_id に 1 件のため genesis 兼末端ブロック。';
COMMENT ON COLUMN scrap_records.content_hash IS '本レコードの SHA-256（rework_id / witness_id / recorded_at の canonical JSON）。';

-- =====================================================
-- TBL-050: return_to_vendor_records（返品記録・Append-only）
-- =====================================================
-- DDL-050: TBL-050 return_to_vendor_records（Append-only）
CREATE TABLE IF NOT EXISTS return_to_vendor_records (
    rework_id              UUID        NOT NULL UNIQUE,
    return_invoice_pdf_id  UUID        NULL,
    vendor_id              UUID        NOT NULL,
    carrier                VARCHAR(128) NOT NULL DEFAULT '',
    tracking_no            VARCHAR(128) NOT NULL,
    returned_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- ADR-011: ハッシュチェーン列
    qc_case_id             UUID        NOT NULL,
    prev_hash              CHAR(64)    NOT NULL,
    content_hash           CHAR(64)    NOT NULL,

    CONSTRAINT pk_return_to_vendor PRIMARY KEY (rework_id),
    CONSTRAINT fk_rtv_rework FOREIGN KEY (rework_id)  REFERENCES reworks (rework_id),
    CONSTRAINT fk_rtv_vendor FOREIGN KEY (vendor_id)  REFERENCES suppliers (supplier_id),
    CONSTRAINT ck_rtv_tracking CHECK (length(trim(tracking_no)) > 0),
    CONSTRAINT ck_rtv_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
);

COMMENT ON TABLE return_to_vendor_records IS 'EN-039 ReturnToVendorRecord — Append-only。追跡番号（tracking_no）必須（ERR-BIZ-025）。qc_case_id = rework_id でチェーン（ADR-011）。';
COMMENT ON COLUMN return_to_vendor_records.qc_case_id IS 'ハッシュチェーン単位 ID（= rework_id）。return_to_vendor_records は 1 rework_id に 1 件のため genesis 兼末端ブロック。';
COMMENT ON COLUMN return_to_vendor_records.content_hash IS '本レコードの SHA-256（rework_id / vendor_id / tracking_no / returned_at の canonical JSON）。';

-- =====================================================
-- TBL-051: case_locks（Case 端末排他占有テーブル）
-- ADR-009 マルチデバイス排他方式。制御テーブルのため app_event_insert に INSERT/UPDATE/DELETE を許可（例外）
-- heartbeat_at は 60 秒ごとに端末が更新、5 分超過で BAT-013 が自動解放（EXPIRED）
-- =====================================================
-- DDL-051: TBL-051 case_locks
-- Case 端末排他占有テーブル（FR-SY-011 / ADR-009）
CREATE TABLE IF NOT EXISTS case_locks (
    case_id       UUID        NOT NULL,
    terminal_id   UUID        NOT NULL,
    user_id       UUID        NOT NULL,
    acquired_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    heartbeat_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    lock_status   TEXT        NOT NULL DEFAULT 'ACTIVE',

    CONSTRAINT pk_case_locks PRIMARY KEY (case_id),
    CONSTRAINT fk_case_locks_case FOREIGN KEY (case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE CASCADE,
    CONSTRAINT fk_case_locks_terminal FOREIGN KEY (terminal_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    CONSTRAINT fk_case_locks_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT ck_case_locks_status CHECK (
        lock_status IN ('ACTIVE', 'RELEASED', 'EXPIRED')
    )
);

COMMENT ON TABLE case_locks IS 'TBL-051 — case_id 単位の端末排他占有テーブル。1 case_id に同時 1 端末のみ ACTIVE 可（ADR-009）。BAT-013 が heartbeat_at 5 分超過で EXPIRED 化。制御テーブルのため app_event_insert ロールに INSERT/UPDATE/DELETE を許可（例外）。';
COMMENT ON COLUMN case_locks.heartbeat_at IS '端末が 60 秒ごとに更新する。BAT-013 が NOW() - heartbeat_at > 5 分の ACTIVE レコードを EXPIRED に更新する（FR-SY-011）。';
