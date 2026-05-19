-- V20260517120045__create_reworks_table.sql
-- TBL-043 reworks: リワーク作業ヘッダ（限定可変: status のみ更新可）。status 10種。

-- EN-032 Rework — リワーク作業ヘッダ（限定可変: status のみ更新可）。
-- parent_case_id は ALCOA+ Original 原則（NFR-DQ-010）に従い不変参照のみ。
-- rework_case_id が新規 WorkExecution を指す。
-- NOTE: disposition_id → dispositions は dispositions テーブル作成後に追加する。
CREATE TABLE IF NOT EXISTS reworks (
    -- リワーク識別子。UUID v4。gen_random_uuid() で自動生成。
    rework_id               UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 起因となった不適合レコードの識別子。
    parent_nonconformity_id UUID        NOT NULL,
    -- 元 WorkExecution ID。ALCOA+ Original — この FK が指すレコードは本テーブルから一切 UPDATE/DELETE しない。
    parent_case_id          UUID        NOT NULL,
    -- 対象ロットの識別子。NULL 許容（ロット未特定の場合）。
    parent_lot_id           UUID        NULL,
    -- 関連する CAPA レコードの識別子。NULL 許容（CAPA なしのリワークも許可）。
    related_capa_id         UUID        NULL,
    -- リワーク種別。TOUCH_UP / REWORK_FULL / SORTING / SCRAP / RETURN の 5 種のみ許可。
    rework_type             VARCHAR(32) NOT NULL,
    -- ステータス。10 種の列挙値のみ許可（初期値: PENDING_DISPOSITION）。status のみ UPDATE 可。
    status                  VARCHAR(32) NOT NULL DEFAULT 'PENDING_DISPOSITION',
    -- リワーク作業用の新 WorkExecution ID。リワーク着手時に採番される（NULL = 未着手）。
    rework_case_id          UUID        NULL,
    -- リワーク SOP バージョンの識別子。NULL 許容（ディスポジション決定前は NULL）。
    rework_sop_version_id   UUID        NULL,
    -- ディスポジション判定レコードの識別子。NULL 許容（dispositions テーブル作成後に FK を追加する）。
    disposition_id          UUID        NULL,
    -- リワーク期限日。NULL = 期限なし。
    due_date                DATE        NULL,
    -- レコード作成時刻。
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新時刻（status 変更時に更新する）。
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 主キー
    CONSTRAINT pk_reworks PRIMARY KEY (rework_id),
    -- nonconformities への外部キー（起因不適合）。
    CONSTRAINT fk_reworks_nonconformity FOREIGN KEY (parent_nonconformity_id)
        REFERENCES nonconformities (nc_id) ON DELETE RESTRICT,
    -- 元 WorkExecution への外部キー（ALCOA+ Original 不変参照）。
    CONSTRAINT fk_reworks_parent_case FOREIGN KEY (parent_case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    -- 対象ロットへの外部キー。NULL 許容。
    CONSTRAINT fk_reworks_parent_lot FOREIGN KEY (parent_lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    -- 関連 CAPA への外部キー。NULL 許容。
    CONSTRAINT fk_reworks_capa FOREIGN KEY (related_capa_id)
        REFERENCES capas (capa_id) ON DELETE RESTRICT,
    -- リワーク用 WorkExecution への外部キー。NULL 許容（着手前は NULL）。
    CONSTRAINT fk_reworks_rework_case FOREIGN KEY (rework_case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    -- リワーク SOP バージョンへの外部キー。NULL 許容。
    CONSTRAINT fk_reworks_sop_version FOREIGN KEY (rework_sop_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT,
    -- rework_type は 5 種の列挙値のみ許可する。
    CONSTRAINT ck_reworks_type CHECK (
        rework_type IN ('TOUCH_UP', 'REWORK_FULL', 'SORTING', 'SCRAP', 'RETURN')
    ),
    -- status は 10 種の列挙値のみ許可する。
    CONSTRAINT ck_reworks_status CHECK (
        status IN (
            'PENDING_DISPOSITION',
            'DISPOSITION_DECIDED',
            'REWORK_IN_PROGRESS',
            'REWORK_COMPLETED',
            'VERIFICATION_IN_PROGRESS',
            'CLOSED_OK_RELEASE',
            'CLOSED_DOWNGRADE',
            'CLOSED_SCRAP',
            'CLOSED_RETURN',
            'RE_REWORK_NEEDED'
        )
    )
);

COMMENT ON TABLE  reworks IS 'EN-032 Rework — リワーク作業ヘッダ。status のみ UPDATE 可。parent_case_id は ALCOA+ Original 原則（NFR-DQ-010）に従い不変参照のみ。rework_case_id が新規 WorkExecution を指す。';
COMMENT ON COLUMN reworks.parent_case_id IS '元 WorkExecution ID。ALCOA+ Original — この FK が指すレコードは本テーブルから一切 UPDATE/DELETE しない。';
COMMENT ON COLUMN reworks.rework_case_id IS 'リワーク作業用の新 WorkExecution ID（execution_type=REWORK）。リワーク着手時に採番。';

-- IDX-024: 不適合別リワーク検索
CREATE INDEX IF NOT EXISTS idx_reworks_nonconformity ON reworks (parent_nonconformity_id);
-- IDX-025: アクティブなリワーク一覧取得（クローズ済みを除外する部分インデックス）
CREATE INDEX IF NOT EXISTS idx_reworks_status ON reworks (status)
    WHERE status NOT IN ('CLOSED_OK_RELEASE', 'CLOSED_DOWNGRADE', 'CLOSED_SCRAP', 'CLOSED_RETURN');
