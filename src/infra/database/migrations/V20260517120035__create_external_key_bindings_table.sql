-- V20260517120035__create_external_key_bindings_table.sql
-- TBL-027 external_key_bindings: 外部キーマッピング（Append-only）。有効期間管理。

-- EN-022 ExternalKeyBinding — 外部 ERP/MES キーと内部 work_pattern_id のマッピング（Append-only）。
-- NOTE: factory_id は予約フィールド。ver1.0.0 では定数 UUID '00000000-0000-7000-8000-000000000001' を使用する。
-- NOTE: factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。
CREATE TABLE IF NOT EXISTS external_key_bindings (
    -- バインディング識別子。UUID v4。gen_random_uuid() で自動生成。
    binding_id       UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 外部システム識別子。例: "SAP_PP"。
    external_system  VARCHAR(64) NOT NULL,
    -- 外部キー JSONB。例: {"lot_id": "L001", "product_code": "P-A001-REV2"}。GIN インデックスで検索する。
    external_key     JSONB       NOT NULL,
    -- 内部 work_pattern_id への参照。external_key に対応するワークパターンを特定する。
    work_pattern_id  UUID        NOT NULL,
    -- 有効期間開始日。
    valid_from       DATE        NOT NULL,
    -- 有効期間終了日。NULL = 無期限有効。変更時は旧レコードに valid_to を設定 + 新レコード INSERT の 2 件操作が必須。
    valid_to         DATE        NULL,
    -- 同期ステータス。ACTIVE / CONFLICT / DEPRECATED の 3 種のみ許可（初期値: ACTIVE）。
    sync_status      VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
    -- ファクトリー識別子。ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。
    factory_id       UUID        NOT NULL DEFAULT '00000000-0000-7000-8000-000000000001',
    -- レコード作成時刻。
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 主キー
    CONSTRAINT pk_external_key_bindings PRIMARY KEY (binding_id),
    -- work_patterns への外部キー。
    CONSTRAINT fk_external_key_work_pattern FOREIGN KEY (work_pattern_id)
        REFERENCES work_patterns (work_pattern_id) ON DELETE RESTRICT,
    -- sync_status は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_external_key_sync_status CHECK (
        sync_status IN ('ACTIVE', 'CONFLICT', 'DEPRECATED')
    ),
    -- valid_to は NULL または valid_from 以降の日付でなければならない。
    CONSTRAINT ck_external_key_valid_range CHECK (
        valid_to IS NULL OR valid_to >= valid_from
    ),
    -- external_key は JSONB オブジェクト型でなければならない。
    CONSTRAINT ck_external_key_is_object CHECK (
        jsonb_typeof(external_key) = 'object'
    )
);

COMMENT ON TABLE  external_key_bindings IS 'EN-022 ExternalKeyBinding — 外部キーマッピング。Append-only。変更時は旧レコードの valid_to を設定 + 新レコード INSERT の 2 件操作が必須。自動解決禁止。';
COMMENT ON COLUMN external_key_bindings.external_key IS 'JSONB 形式の外部キー。例: {"lot_id": "L001", "product_code": "P-A001-REV2"}。GIN インデックス（IDX-013）で検索する。';
COMMENT ON COLUMN external_key_bindings.sync_status  IS 'ACTIVE: 有効 / CONFLICT: 複数マッピング競合（手動解決必要）/ DEPRECATED: 廃止。';
COMMENT ON COLUMN external_key_bindings.factory_id   IS 'ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。';

-- Append-only 強制: app_event_writer ロールから UPDATE/DELETE を剥奪する
REVOKE UPDATE, DELETE ON external_key_bindings FROM PUBLIC;
REVOKE UPDATE, DELETE ON external_key_bindings FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON external_key_bindings TO app_event_writer;
