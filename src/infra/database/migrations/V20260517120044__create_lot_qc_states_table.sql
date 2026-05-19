-- V20260517120044__create_lot_qc_states_table.sql
-- TBL-042 lot_qc_states: ロット現在 QC ステータス（後工程ゲート判定用・更新可）。

-- EN-030 × EN-021 — ロット現在 QC ステータス（後工程ゲート判定用・更新可）。
-- 後工程 QR スキャン時の ERR-BIZ-015 ゲート判定に使用する。
-- qc_status は incoming_inspections.qc_status 変化に連動して UPDATE する（アプリ層で制御）。
CREATE TABLE IF NOT EXISTS lot_qc_states (
    -- ロット識別子。PRIMARY KEY（1 ロットに 1 件の QC ステータス）。
    lot_id              UUID        NOT NULL,
    -- ロットの現在 QC ステータス。8 種の列挙値のみ許可。incoming_inspections.qc_status と同期する。
    qc_status           VARCHAR(32) NOT NULL DEFAULT 'PENDING',
    -- 最新の受入検査レコードの識別子。
    last_inspection_id  UUID        NOT NULL,
    -- レコード最終更新時刻。
    last_updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 主キー（lot_id 単体が PK）
    CONSTRAINT pk_lot_qc_states PRIMARY KEY (lot_id),
    -- lots テーブルへの外部キー。
    CONSTRAINT fk_lot_qc_lot FOREIGN KEY (lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    -- incoming_inspections テーブルへの外部キー（最新検査レコードへの参照）。
    CONSTRAINT fk_lot_qc_inspection FOREIGN KEY (last_inspection_id)
        REFERENCES incoming_inspections (inspection_id) ON DELETE RESTRICT,
    -- qc_status は 8 種の列挙値のみ許可する（incoming_inspections.qc_status と同一の値セット）。
    CONSTRAINT ck_lot_qc_status CHECK (
        qc_status IN (
            'PENDING',
            'INSPECTING',
            'PASSED',
            'CONDITIONAL_PASS',
            'SCREENING_REQUIRED',
            'REJECTED',
            'SCRAPPED',
            'RETURNED'
        )
    )
);

COMMENT ON TABLE  lot_qc_states IS 'EN-030×EN-021 — ロットの現在 QC ステータス。後工程 QR スキャン時の ERR-BIZ-015 ゲート判定に使用する。qc_status は incoming_inspections.qc_status 変化に連動して UPDATE する。';
