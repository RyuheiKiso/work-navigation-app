-- V20260519120004__create_work_assignment_tables.sql
-- Push 型作業指示テーブルの作成（TBL-052〜053）
-- UC-041: 親機（ERP/MES）からの作業指示受信
-- UC-042: ハンディ端末への SSE 配信と作業開始
-- =====================================================
-- ロール制御方針（V008 で GRANT 実行）:
--   app_event_insert: INSERT + SELECT を許可
--   app_write: status 系列の UPDATE を許可
--   work_assignments の Append-only 列（source_payload / external_order_id / external_system /
--     received_at / idempotency_key）は UPDATE 禁止（ADR-010 準拠）
-- =====================================================

-- =====================================================
-- TBL-052: work_assignments（Push 型作業指示テーブル）
-- =====================================================
-- DDL-052: TBL-052 work_assignments
-- Push 型作業指示テーブル。親機（ERP/MES）から受信した作業指示を管理する。
-- Append-only 列（source_payload / external_order_id / external_system / received_at / idempotency_key）は
-- INSERT 後 UPDATE 禁止（ADR-010 準拠）。
-- status / case_id / dispatched_at / accepted_at / completed_at のみ UPDATE 可。
-- NOTE: factory_id は予約フィールド。ver1.0.0 では定数 UUID '00000000-0000-7000-8000-000000000001' を使用する。
-- NOTE: factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。
CREATE TABLE IF NOT EXISTS work_assignments (
    assignment_id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    external_order_id       TEXT        NOT NULL,
    external_system         TEXT        NOT NULL,
    work_pattern_id         UUID        NOT NULL,
    lot_id                  UUID        NULL,
    target_terminal_id      UUID        NOT NULL,
    suggested_worker_id     UUID        NULL,
    suggested_equipment_id  UUID        NULL,
    due_at                  TIMESTAMPTZ NULL,
    priority                SMALLINT    NOT NULL DEFAULT 2,
    status                  TEXT        NOT NULL DEFAULT 'pending',
    case_id                 UUID        NULL,
    source_payload          JSONB       NOT NULL DEFAULT '{}',
    idempotency_key         UUID        NOT NULL,
    received_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    dispatched_at           TIMESTAMPTZ NULL,
    accepted_at             TIMESTAMPTZ NULL,
    completed_at            TIMESTAMPTZ NULL,
    factory_id              UUID        NOT NULL,

    CONSTRAINT pk_work_assignments PRIMARY KEY (assignment_id),
    CONSTRAINT fk_wa_work_pattern    FOREIGN KEY (work_pattern_id)       REFERENCES work_patterns (work_pattern_id),
    CONSTRAINT fk_wa_lot             FOREIGN KEY (lot_id)                REFERENCES lots (lot_id),
    CONSTRAINT fk_wa_target_terminal FOREIGN KEY (target_terminal_id)    REFERENCES devices (device_id),
    CONSTRAINT fk_wa_suggested_worker FOREIGN KEY (suggested_worker_id)  REFERENCES users (user_id),
    CONSTRAINT fk_wa_suggested_equip  FOREIGN KEY (suggested_equipment_id) REFERENCES equipments (equipment_id),
    CONSTRAINT fk_wa_case            FOREIGN KEY (case_id)               REFERENCES work_executions (work_execution_id),
    CONSTRAINT uq_wa_idempotency UNIQUE (external_system, idempotency_key),
    CONSTRAINT ck_wa_priority CHECK (priority IN (1, 2, 3)),
    CONSTRAINT ck_wa_status CHECK (
        status IN (
            'pending',
            'pending_resolution',
            'dispatched',
            'accepted',
            'in_progress',
            'completed',
            'rejected',
            'cancelled',
            'expired'
        )
    ),
    CONSTRAINT ck_wa_source_payload_is_object CHECK (
        jsonb_typeof(source_payload) = 'object'
    )
);

COMMENT ON TABLE  work_assignments IS 'TBL-052 — Push 型作業指示テーブル。親機（ERP/MES）から受信した作業指示を管理する。source_payload / external_order_id / external_system / received_at / idempotency_key は Append-only（ADR-010）。status 遷移は 05_作業指示テーブルDDL §1-1-2 参照。';
COMMENT ON COLUMN work_assignments.assignment_id           IS 'UUID v4。gen_random_uuid() で自動生成。外部システムへの公開 ID としては使用しない。';
COMMENT ON COLUMN work_assignments.external_order_id       IS '親機（ERP/MES）側の作業指示番号。Append-only（INSERT 後変更不可）。external_system と組み合わせて人間が判読する識別子として機能する。';
COMMENT ON COLUMN work_assignments.external_system         IS '送信元システム識別子（例: "SAP_PP"）。Append-only。idempotency_key との複合 UNIQUE 制約で重複受信を防止する。';
COMMENT ON COLUMN work_assignments.work_pattern_id         IS 'external_key_binding により解決された内部 work_pattern_id。作業手順（SOP）を特定する。';
COMMENT ON COLUMN work_assignments.lot_id                  IS 'lot_id_ext が提供されている場合に external_key_binding で解決した内部 lot_id。NULL = ロット未紐付け。';
COMMENT ON COLUMN work_assignments.target_terminal_id      IS 'target_terminal_key を external_key_binding で解決した端末 device_id。割当先端末を特定する。';
COMMENT ON COLUMN work_assignments.suggested_worker_id     IS '推奨作業員の user_id（参考情報・作業員は拒否可）。解決失敗時は NULL。';
COMMENT ON COLUMN work_assignments.suggested_equipment_id  IS '推奨設備の equipment_id（参考情報・強制しない）。解決失敗時は NULL。';
COMMENT ON COLUMN work_assignments.due_at                  IS '作業期限（TIMESTAMPTZ）。NULL = 期限なし扱い。BAT-015 が定期的に due_at 超過を検知して expired に遷移させる。';
COMMENT ON COLUMN work_assignments.priority                IS '優先度: 1=緊急 / 2=通常 / 3=低。CHECK 制約で 1/2/3 のみ許可。priority=1 の場合 UI で蛍光色強調表示（FR-NV-014）。';
COMMENT ON COLUMN work_assignments.status                  IS 'ステータス遷移は 05_作業指示テーブルDDL §1-1-2 参照。9 種の列挙値のみ許可（CHECK 制約）。';
COMMENT ON COLUMN work_assignments.case_id                 IS 'UC-042 で作業員が「開始」をタップ後に生成・バインドされる work_executions.work_execution_id。それ以前は NULL。';
COMMENT ON COLUMN work_assignments.source_payload          IS '受信したリクエストボディ全体の JSONB スナップショット。Append-only。外部キー解決の事後検証・監査ログとして使用する。';
COMMENT ON COLUMN work_assignments.idempotency_key         IS '親機が送信した Idempotency-Key ヘッダ値（UUID）。external_system との複合 UNIQUE 制約で重複受信を防止する。Append-only。';
COMMENT ON COLUMN work_assignments.received_at             IS 'サーバーがリクエストを受信した時刻（ALCOA+ Contemporaneous）。Append-only。IDX-033（BRIN）のインデックス対象列。';
COMMENT ON COLUMN work_assignments.dispatched_at           IS 'SSE 配信が sent 状態に遷移した時刻。NULL = 未配信。';
COMMENT ON COLUMN work_assignments.accepted_at             IS '作業員が AssignmentDetailDialog で「開始」をタップした時刻。NULL = 未受諾。';
COMMENT ON COLUMN work_assignments.completed_at            IS '作業完了イベントが記録された時刻。NULL = 未完了。';
COMMENT ON COLUMN work_assignments.factory_id              IS 'ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。';

-- IDX-032: 端末向け割当高速取得（部分インデックス）
-- 対象端末の未処理割当（pending / pending_resolution / dispatched）を高速に取得する
CREATE INDEX IF NOT EXISTS idx_wa_terminal_status
    ON work_assignments (target_terminal_id, status)
    WHERE status IN ('pending', 'pending_resolution', 'dispatched');

-- IDX-033: 時系列検索（BRIN）
-- received_at の範囲検索（管理画面・バッチ処理）向け
CREATE INDEX IF NOT EXISTS idx_wa_received
    ON work_assignments USING BRIN (received_at);

-- IDX-034: 重複受信防止（UNIQUE インデックス）
-- uq_wa_idempotency UNIQUE 制約と対応する明示的インデックス
CREATE UNIQUE INDEX IF NOT EXISTS idx_wa_idempotency
    ON work_assignments (external_system, idempotency_key);

-- =====================================================
-- TBL-053: sse_dispatch_log（SSE 配信ログ）
-- =====================================================
-- DDL-053: TBL-053 sse_dispatch_log
-- SSE 配信ログ。work_assignments の各端末への配信試行を記録する。
-- Append-only ベース。delivery_status / ack_at / retry_count のみ UPDATE 可。
CREATE TABLE IF NOT EXISTS sse_dispatch_log (
    dispatch_id      UUID        NOT NULL DEFAULT gen_random_uuid(),
    assignment_id    UUID        NOT NULL,
    terminal_id      UUID        NOT NULL,
    dispatched_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivery_status  TEXT        NOT NULL DEFAULT 'queued',
    ack_at           TIMESTAMPTZ NULL,
    retry_count      INTEGER     NOT NULL DEFAULT 0,

    CONSTRAINT pk_sse_dispatch_log PRIMARY KEY (dispatch_id),
    CONSTRAINT fk_sdl_assignment FOREIGN KEY (assignment_id) REFERENCES work_assignments (assignment_id),
    CONSTRAINT fk_sdl_terminal   FOREIGN KEY (terminal_id)   REFERENCES devices (device_id),
    CONSTRAINT ck_sdl_delivery_status CHECK (
        delivery_status IN (
            'queued',
            'sent',
            'ack',
            'failed',
            'expired'
        )
    ),
    CONSTRAINT ck_sdl_retry_non_negative CHECK (retry_count >= 0)
);

COMMENT ON TABLE  sse_dispatch_log IS 'TBL-053 — SSE 配信ログ。assignment_id × terminal_id の配信試行を記録する。Append-only ベース（delivery_status / ack_at / retry_count のみ UPDATE 可）。配信リトライ上限は CFG-030 で設定する。';
COMMENT ON COLUMN sse_dispatch_log.dispatch_id     IS 'UUID v4。gen_random_uuid() で自動生成。';
COMMENT ON COLUMN sse_dispatch_log.assignment_id   IS '配信対象の work_assignments.assignment_id への FK。';
COMMENT ON COLUMN sse_dispatch_log.terminal_id     IS '配信先端末の device_id への FK。work_assignments.target_terminal_id と原則一致する（リトライ先変更時は除く）。';
COMMENT ON COLUMN sse_dispatch_log.dispatched_at   IS '配信キューへの登録時刻（queued 状態への INSERT 時刻）。Append-only。';
COMMENT ON COLUMN sse_dispatch_log.delivery_status IS '配信状態の 5 値列挙。queued=配信待ち / sent=送信済み / ack=端末 ACK 受信 / failed=リトライ上限超過 / expired=割当取消。';
COMMENT ON COLUMN sse_dispatch_log.ack_at          IS '端末が POST /ack を送信した時刻。delivery_status=ack 時に設定される。NULL = 未 ACK。';
COMMENT ON COLUMN sse_dispatch_log.retry_count     IS '再送試行回数。0 = 初回配信。CFG-030 の上限（デフォルト 5）に達すると delivery_status=failed に遷移する。';

-- IDX-035: 配信状態検索（複合インデックス）
-- assignment_id + terminal_id の複合インデックス。配信状態確認・リトライ判定で使用する。
CREATE INDEX IF NOT EXISTS idx_sdl_assignment_terminal
    ON sse_dispatch_log (assignment_id, terminal_id);
