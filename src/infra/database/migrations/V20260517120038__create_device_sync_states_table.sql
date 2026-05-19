-- V20260517120038__create_device_sync_states_table.sql
-- TBL-034 device_sync_states: デバイス同期状態管理テーブル（更新可）。

-- TBL-034 device_sync_states — デバイスごとのマスタ同期状態を管理する（更新可）。
-- デバイスが削除された場合はカスケード削除する（ON DELETE CASCADE）。
CREATE TABLE IF NOT EXISTS device_sync_states (
    -- デバイス識別子。PRIMARY KEY。devices テーブルへの FK（1 デバイスに 1 件）。
    device_id              UUID        NOT NULL,
    -- 最終同期時刻。NULL = 未同期。
    last_sync_at           TIMESTAMPTZ NULL,
    -- 最終同期したマスタバージョンの識別子。NULL = 未同期または初期状態。
    last_master_version_id UUID        NULL,
    -- 同期ステータス。SYNCED / PENDING / CONFLICT の 3 種のみ許可（初期値: PENDING）。
    sync_status            VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    -- レコード最終更新時刻。
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 主キー（device_id 単体が PK）
    CONSTRAINT pk_device_sync_states PRIMARY KEY (device_id),
    -- devices テーブルへの外部キー。デバイス削除時はカスケード削除する。
    CONSTRAINT fk_device_sync_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE CASCADE,
    -- master_versions テーブルへの外部キー。NULL 許容（未同期の場合）。
    CONSTRAINT fk_device_sync_master_version FOREIGN KEY (last_master_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT,
    -- sync_status は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_device_sync_status CHECK (
        sync_status IN ('SYNCED', 'PENDING', 'CONFLICT')
    )
);

COMMENT ON TABLE  device_sync_states IS 'TBL-034 — デバイスごとのマスタ同期状態管理テーブル。1 デバイスに 1 件。デバイス削除時はカスケード削除する。';
COMMENT ON COLUMN device_sync_states.sync_status IS 'SYNCED: 最新マスタ同期済み / PENDING: 同期待ち / CONFLICT: 競合（手動解決必要）。';
COMMENT ON COLUMN device_sync_states.last_master_version_id IS '最終同期したマスタバージョン。NULL = 未同期または初期状態。';
