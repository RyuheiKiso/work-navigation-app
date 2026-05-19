-- V20260517120053__create_work_assignments_table.sql
-- TBL-052 work_assignments: Push 型作業指示テーブル（限定可変）。status 9種。Append-only 列あり。

-- DDL-052: TBL-052 work_assignments
-- Push 型作業指示テーブル。親機（ERP/MES）から受信した作業指示を管理する。
-- Append-only 列（source_payload / external_order_id / external_system / received_at / idempotency_key）は
-- INSERT 後 UPDATE 禁止（ADR-010 準拠）。
-- status / case_id / dispatched_at / accepted_at / completed_at のみ UPDATE 可。
-- NOTE: factory_id は予約フィールド。ver1.0.0 では定数 UUID '00000000-0000-7000-8000-000000000001' を使用する。
-- NOTE: factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。
-- NOTE: case_id は work_executions.work_execution_id を参照するが、ドキュメントの FK 定義が case_id で記載されているため合わせる。
CREATE TABLE IF NOT EXISTS work_assignments (
    -- 割当識別子。UUID v4。gen_random_uuid() で自動生成。外部システムへの公開 ID としては使用しない。
    assignment_id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 親機（ERP/MES）側の作業指示番号。Append-only（INSERT 後変更不可）。
    external_order_id       TEXT        NOT NULL,
    -- 送信元システム識別子（例: "SAP_PP"）。Append-only。idempotency_key との複合 UNIQUE 制約で重複受信を防止する。
    external_system         TEXT        NOT NULL,
    -- external_key_binding により解決された内部 work_pattern_id。作業手順（SOP）を特定する。
    work_pattern_id         UUID        NOT NULL,
    -- lot_id_ext が提供されている場合に external_key_binding で解決した内部 lot_id。NULL = ロット未紐付け。
    lot_id                  UUID        NULL,
    -- target_terminal_key を external_key_binding で解決した端末 device_id。割当先端末を特定する。
    target_terminal_id      UUID        NOT NULL,
    -- 推奨作業員の user_id（参考情報・作業員は拒否可）。解決失敗時は NULL（ERR-BIZ-027 は発生しない）。
    suggested_worker_id     UUID        NULL,
    -- 推奨設備の equipment_id（参考情報・強制しない）。解決失敗時は NULL。
    suggested_equipment_id  UUID        NULL,
    -- 作業期限（TIMESTAMPTZ）。NULL = 期限なし扱い。BAT-015 が定期的に due_at 超過を検知して expired に遷移させる。
    due_at                  TIMESTAMPTZ NULL,
    -- 優先度: 1=緊急 / 2=通常 / 3=低。CHECK 制約で 1/2/3 のみ許可。priority=1 の場合 UI で蛍光色強調表示（FR-NV-014）。
    priority                SMALLINT    NOT NULL DEFAULT 2,
    -- ステータス遷移は本ドキュメント §1-1-2 参照。9 種の列挙値のみ許可（CHECK 制約）。
    status                  VARCHAR(32) NOT NULL DEFAULT 'pending',
    -- UC-042 で作業員が「開始」をタップ後に生成・バインドされる work_executions の識別子。それ以前は NULL。
    case_id                 UUID        NULL,
    -- 受信したリクエストボディ全体の JSONB スナップショット。Append-only。外部キー解決の事後検証・監査ログとして使用する。
    source_payload          JSONB       NOT NULL,
    -- 親機が送信した Idempotency-Key ヘッダ値（UUID）。external_system との複合 UNIQUE 制約で重複受信を防止する。Append-only。
    idempotency_key         UUID        NOT NULL,
    -- サーバーがリクエストを受信した時刻（ALCOA+ Contemporaneous）。Append-only。IDX-033（BRIN）のインデックス対象列。
    received_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- SSE 配信が sent 状態に遷移した時刻。NULL = 未配信。
    dispatched_at           TIMESTAMPTZ NULL,
    -- 作業員が AssignmentDetailDialog で「開始」をタップした時刻。NULL = 未受諾。
    accepted_at             TIMESTAMPTZ NULL,
    -- 作業完了イベントが記録された時刻。NULL = 未完了。
    completed_at            TIMESTAMPTZ NULL,
    -- ファクトリー識別子。ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。
    factory_id              UUID        NOT NULL DEFAULT '00000000-0000-7000-8000-000000000001',

    -- 主キー
    CONSTRAINT pk_work_assignments PRIMARY KEY (assignment_id),
    -- work_patterns への外部キー。
    CONSTRAINT fk_wa_work_pattern FOREIGN KEY (work_pattern_id)
        REFERENCES work_patterns (work_pattern_id) ON DELETE RESTRICT,
    -- lots への外部キー。NULL 許容（ロット未紐付けの場合）。
    CONSTRAINT fk_wa_lot FOREIGN KEY (lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    -- 割当先端末への外部キー。
    CONSTRAINT fk_wa_terminal FOREIGN KEY (target_terminal_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    -- 推奨作業員ユーザーへの外部キー。NULL 許容。
    CONSTRAINT fk_wa_suggested_worker FOREIGN KEY (suggested_worker_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- 推奨設備への外部キー。NULL 許容。
    CONSTRAINT fk_wa_suggested_equipment FOREIGN KEY (suggested_equipment_id)
        REFERENCES equipments (equipment_id) ON DELETE RESTRICT,
    -- 作業セッションへの外部キー。NULL 許容（作業開始前は NULL）。
    CONSTRAINT fk_wa_case FOREIGN KEY (case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    -- priority は 1/2/3 のみ許可する（1=緊急 / 2=通常 / 3=低）。
    CONSTRAINT ck_wa_priority CHECK (priority IN (1, 2, 3)),
    -- status は 9 種の列挙値のみ許可する。
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
    -- external_system + idempotency_key の組み合わせは UNIQUE でなければならない（重複受信防止）。
    CONSTRAINT uq_wa_idempotency UNIQUE (external_system, idempotency_key)
);

COMMENT ON TABLE  work_assignments IS 'TBL-052 — Push 型作業指示テーブル。親機（ERP/MES）から受信した作業指示を管理する。source_payload / external_order_id / external_system / received_at / idempotency_key は Append-only（ADR-010）。status 遷移は 05_作業指示テーブルDDL §1-1-2 参照。';
COMMENT ON COLUMN work_assignments.assignment_id        IS 'UUID v4。gen_random_uuid() で自動生成。外部システムへの公開 ID としては使用しない。';
COMMENT ON COLUMN work_assignments.external_order_id    IS '親機（ERP/MES）側の作業指示番号。Append-only（INSERT 後変更不可）。external_system と組み合わせて人間が判読する識別子として機能する。';
COMMENT ON COLUMN work_assignments.external_system      IS '送信元システム識別子（例: "SAP_PP"）。Append-only。idempotency_key との複合 UNIQUE 制約で重複受信を防止する。';
COMMENT ON COLUMN work_assignments.work_pattern_id      IS 'external_key_binding により解決された内部 work_pattern_id。作業手順（SOP）を特定する。';
COMMENT ON COLUMN work_assignments.lot_id               IS 'lot_id_ext が提供されている場合に external_key_binding で解決した内部 lot_id。NULL = ロット未紐付け。';
COMMENT ON COLUMN work_assignments.target_terminal_id   IS 'target_terminal_key を external_key_binding で解決した端末 device_id。割当先端末を特定する。';
COMMENT ON COLUMN work_assignments.suggested_worker_id  IS '推奨作業員の user_id（参考情報・作業員は拒否可）。解決失敗時は NULL（ERR-BIZ-027 は発生しない）。';
COMMENT ON COLUMN work_assignments.suggested_equipment_id IS '推奨設備の equipment_id（参考情報・強制しない）。解決失敗時は NULL。';
COMMENT ON COLUMN work_assignments.due_at               IS '作業期限（TIMESTAMPTZ）。NULL = 期限なし扱い。BAT-015 が定期的に due_at 超過を検知して expired に遷移させる。';
COMMENT ON COLUMN work_assignments.priority             IS '優先度: 1=緊急 / 2=通常 / 3=低。CHECK 制約で 1/2/3 のみ許可。priority=1 の場合 UI で蛍光色強調表示（FR-NV-014）。';
COMMENT ON COLUMN work_assignments.status               IS 'ステータス遷移は 05_作業指示テーブルDDL §1-1-2 参照。9 種の列挙値のみ許可（CHECK 制約）。';
COMMENT ON COLUMN work_assignments.case_id              IS 'UC-042 で作業員が「開始」をタップ後に生成・バインドされる work_executions.work_execution_id。それ以前は NULL。';
COMMENT ON COLUMN work_assignments.source_payload       IS '受信したリクエストボディ全体の JSONB スナップショット。Append-only。外部キー解決の事後検証・監査ログとして使用する。';
COMMENT ON COLUMN work_assignments.idempotency_key      IS '親機が送信した Idempotency-Key ヘッダ値（UUID）。external_system との複合 UNIQUE 制約で重複受信を防止する。Append-only。';
COMMENT ON COLUMN work_assignments.received_at          IS 'サーバーがリクエストを受信した時刻（ALCOA+ Contemporaneous）。Append-only。IDX-033（BRIN）のインデックス対象列。';
COMMENT ON COLUMN work_assignments.dispatched_at        IS 'SSE 配信が sent 状態に遷移した時刻。NULL = 未配信。';
COMMENT ON COLUMN work_assignments.accepted_at          IS '作業員が AssignmentDetailDialog で「開始」をタップした時刻。NULL = 未受諾。';
COMMENT ON COLUMN work_assignments.completed_at         IS '作業完了イベントが記録された時刻。NULL = 未完了。';
COMMENT ON COLUMN work_assignments.factory_id           IS 'ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。';

-- IDX-032: 端末向け割当高速取得（対象端末の未処理割当を高速に取得するための部分インデックス）
CREATE INDEX IF NOT EXISTS idx_wa_terminal_status ON work_assignments (target_terminal_id, status)
    WHERE status IN ('pending', 'pending_resolution', 'dispatched');

-- IDX-033: 時系列検索（received_at の範囲検索向け BRIN インデックス。書き込みコスト最小化）
CREATE INDEX IF NOT EXISTS idx_wa_received ON work_assignments USING BRIN (received_at);

-- IDX-034: 重複受信防止（uq_wa_idempotency UNIQUE 制約と対応する明示的なインデックス）
CREATE UNIQUE INDEX IF NOT EXISTS idx_wa_idempotency ON work_assignments (external_system, idempotency_key);

-- app_event_writer に INSERT と SELECT のみを許可する（Append-only 列の保護）
REVOKE UPDATE, DELETE ON work_assignments FROM PUBLIC;
GRANT INSERT, SELECT ON work_assignments TO app_event_writer;
-- app_read_write に status 系列の UPDATE のみを許可する
GRANT UPDATE (status, case_id, dispatched_at, accepted_at, completed_at) ON work_assignments TO app_read_write;
