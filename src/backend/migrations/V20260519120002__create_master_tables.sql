-- V20260519120002__create_master_tables.sql
-- マスタ系テーブルの全量作成
-- 作成順序は外部キー依存を考慮する:
--   roles → users → skills → user_roles → user_skills
--   → processes → operations → products
--   → materials → suppliers
--   → lots (supplier/material FK を最後に追加)
--   → equipments → instruments
--   → electronic_signs (→ users, devices)
--   → master_versions (→ users, electronic_signs)
--   → sops (→ operations, master_versions)
--   → steps (→ sops)
--   → step_type_definitions
--   → step_flow_rules (→ sops, steps)
--   → work_patterns (→ sops, operations)
--   → devices → device_sync_states (→ devices, master_versions)
--   → sampling_plans (→ materials, suppliers, users)
--   → rework_sop_mapping (→ sops)
-- =====================================================

-- =====================================================
-- TBL-017: roles（固定 6 種権限ロールマスタ）
-- =====================================================
-- DDL-017: TBL-017 roles
-- EN-002 Role — 固定 6 種の権限ロールマスタ。アプリ起動時にシードデータを INSERT し、以後 UPDATE しない。
CREATE TABLE IF NOT EXISTS roles (
    role_id     UUID            NOT NULL,
    role_name   VARCHAR(64)     NOT NULL,
    description TEXT            NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_roles PRIMARY KEY (role_id),
    CONSTRAINT uq_roles_name UNIQUE (role_name),
    CONSTRAINT ck_roles_name_valid CHECK (
        role_name IN ('operator', 'supervisor', 'master_admin', 'quality_admin', 'system_admin', 'executive')
    )
);

COMMENT ON TABLE  roles IS 'EN-002 Role — 固定 6 種の権限ロール。CHECK 制約で新種追加を防止する（スキーマ変更必須）。';
COMMENT ON COLUMN roles.role_id   IS '固定 UUID。シードデータで定数値を使用する（例: 00000000-0000-7000-8000-000000000001）。';
COMMENT ON COLUMN roles.role_name IS 'operator / supervisor / master_admin / quality_admin / system_admin / executive の 6 値のみ許可。';

-- =====================================================
-- TBL-016: users（作業員・管理者・システムユーザーマスタ）
-- =====================================================
-- DDL-016: TBL-016 users
-- EN-001 User — 作業員・管理者・システムユーザーを統合管理するマスタ
CREATE TABLE IF NOT EXISTS users (
    user_id              UUID            NOT NULL DEFAULT gen_random_uuid(),
    login_id             VARCHAR(128)    NOT NULL,
    display_name         VARCHAR(256)    NOT NULL,
    password_hash        VARCHAR(255)    NOT NULL DEFAULT '',
    pin_hash             VARCHAR(255)    NULL,
    factory_id           UUID            NOT NULL DEFAULT gen_random_uuid(),
    roles                JSONB           NOT NULL DEFAULT '[]'::jsonb,
    failed_login_count   INTEGER         NOT NULL DEFAULT 0,
    locked_until         TIMESTAMPTZ     NULL,
    is_active            BOOLEAN         NOT NULL DEFAULT TRUE,
    anonymized_at        TIMESTAMPTZ     NULL,
    deleted_at           TIMESTAMPTZ     NULL,
    created_at           TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_users PRIMARY KEY (user_id),
    CONSTRAINT uq_users_login_id UNIQUE (login_id),
    CONSTRAINT ck_users_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT ck_users_anonymized_active CHECK (
        NOT (is_active = TRUE AND anonymized_at IS NOT NULL)
    ),
    CONSTRAINT ck_users_roles_is_array CHECK (jsonb_typeof(roles) = 'array')
);

COMMENT ON TABLE  users IS 'EN-001 User — 作業員・管理者・システムユーザーマスタ。物理削除禁止。退職後 is_active=FALSE、60日後に BAT-004 が PII を匿名化する。';
COMMENT ON COLUMN users.user_id             IS 'UUID v7（時系列順）。Rust 側で生成し INSERT する。WorkEvent.resource FK として不変。';
COMMENT ON COLUMN users.login_id            IS 'ログイン識別子。LDAP 連携時は LDAP DN 形式。匿名化後は内部 UUID 文字列に置換される。';
COMMENT ON COLUMN users.display_name        IS '表示名。匿名化後は "anonymized-{user_id 前 8 桁}" に置換される。';
COMMENT ON COLUMN users.password_hash       IS 'bcrypt でハッシュ化されたパスワード（コスト係数 12）。ユーザー作成時に必ず設定する。LDAP 認証失敗時のフォールバック認証に使用する。';
COMMENT ON COLUMN users.pin_hash            IS 'bcrypt でハッシュ化された電子サイン用 PIN（コスト係数 10）。NULL は PIN 未設定。PIN 未設定ユーザーは電子サイン不可。';
COMMENT ON COLUMN users.factory_id          IS '所属工場の UUID。ver1.0.0 はシングルファクトリー運用のため定数 UUID を使用する。JWT クレームに埋め込み高速認証を実現する。';
COMMENT ON COLUMN users.roles               IS '付与されているロール名の非正規化 JSONB 配列（例: ["operator","supervisor"]）。高速認証のため denormalize している。user_roles テーブルが正規化ソース。';
COMMENT ON COLUMN users.failed_login_count  IS 'ブルートフォース対策の連続認証失敗カウンタ。5 回失敗でアカウントを 30 分ロックする。認証成功時に 0 にリセットする。';
COMMENT ON COLUMN users.locked_until        IS 'アカウントロック解除時刻（UTC）。NULL はロックなし。ログイン時に NOW() と比較してロック中か判定する。';
COMMENT ON COLUMN users.is_active           IS '退職・無効化時に FALSE。物理 DELETE は禁止。';
COMMENT ON COLUMN users.anonymized_at       IS 'PII 匿名化実施時刻。CFG-010 で設定された日数（デフォルト 60 日）経過後に BAT-004 が設定する。';
COMMENT ON COLUMN users.deleted_at          IS '論理削除時刻（UTC）。NULL は現役ユーザー。退職・削除時に BAT-004 または管理者操作で設定する。認証クエリの WHERE 条件に使用する。';

-- =====================================================
-- TBL-018: skills（作業員スキル定義マスタ）
-- =====================================================
-- DDL-018: TBL-018 skills
-- EN-003 Skill — 作業員スキル定義マスタ
CREATE TABLE IF NOT EXISTS skills (
    skill_id      UUID          NOT NULL DEFAULT gen_random_uuid(),
    skill_code    VARCHAR(64)   NOT NULL,
    skill_name    VARCHAR(128)  NOT NULL,
    skill_level   SMALLINT      NOT NULL DEFAULT 1,
    description   TEXT          NOT NULL DEFAULT '',
    is_active     BOOLEAN       NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_skills PRIMARY KEY (skill_id),
    CONSTRAINT uq_skills_code UNIQUE (skill_code),
    CONSTRAINT ck_skills_level CHECK (skill_level BETWEEN 1 AND 5)
);

COMMENT ON TABLE  skills IS 'EN-003 Skill — 作業員スキル定義マスタ。skill_level は定義上の最高レベルを示す。';
COMMENT ON COLUMN skills.skill_code  IS 'スキルコード。形式: {カテゴリ英字}-{連番3桁}。例: WLD-001（溶接技能）。変更不可の公開識別子。';
COMMENT ON COLUMN skills.skill_level IS '1〜5。このスキル定義が想定する上限レベル。';

-- =====================================================
-- TBL-019: user_roles（ユーザー×ロール N:M 中間テーブル）
-- =====================================================
-- DDL-019: TBL-019 user_roles
-- EN-001 × EN-002 N:M 中間テーブル — ユーザーへのロール付与記録
CREATE TABLE IF NOT EXISTS user_roles (
    user_id     UUID        NOT NULL,
    role_id     UUID        NOT NULL,
    granted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by  UUID        NOT NULL,

    CONSTRAINT pk_user_roles PRIMARY KEY (user_id, role_id),
    CONSTRAINT fk_user_roles_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_user_roles_role FOREIGN KEY (role_id)
        REFERENCES roles (role_id) ON DELETE RESTRICT,
    CONSTRAINT fk_user_roles_granted_by FOREIGN KEY (granted_by)
        REFERENCES users (user_id) ON DELETE RESTRICT
);

COMMENT ON TABLE  user_roles IS 'EN-001×EN-002 N:M 中間テーブル。ロール付与の証跡として granted_by を必須とする。';
COMMENT ON COLUMN user_roles.granted_by IS 'ロールを付与したユーザーの user_id。system_admin ロールのみ操作可能（アプリ層で制御）。';

-- =====================================================
-- TBL-020: user_skills（ユーザー×スキル N:M 中間テーブル）
-- =====================================================
-- DDL-020: TBL-020 user_skills
-- EN-001 × EN-003 N:M 中間テーブル — ユーザーへのスキル認定記録
CREATE TABLE IF NOT EXISTS user_skills (
    user_id         UUID        NOT NULL,
    skill_id        UUID        NOT NULL,
    achieved_level  SMALLINT    NOT NULL,
    certified_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    certified_by    UUID        NOT NULL,

    CONSTRAINT pk_user_skills PRIMARY KEY (user_id, skill_id),
    CONSTRAINT fk_user_skills_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_user_skills_skill FOREIGN KEY (skill_id)
        REFERENCES skills (skill_id) ON DELETE RESTRICT,
    CONSTRAINT fk_user_skills_certified_by FOREIGN KEY (certified_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT ck_user_skills_level CHECK (achieved_level BETWEEN 1 AND 5)
);

COMMENT ON TABLE  user_skills IS 'EN-001×EN-003 N:M 中間テーブル。スキルゲート（BR-BUS-015）の判定源。';
COMMENT ON COLUMN user_skills.achieved_level IS '1〜5。この認定時点での達成レベル。スキルゲートは steps.skill_level_required と比較する。';
COMMENT ON COLUMN user_skills.certified_by   IS '認定者の user_id。supervisor 以上が操作可能（アプリ層で制御）。';

-- =====================================================
-- TBL-033: devices（ハンディ端末デバイスマスタ）
-- =====================================================
-- DDL-033: TBL-033 devices
-- EN-023 Device — ハンディ端末デバイスマスタ
CREATE TABLE IF NOT EXISTS devices (
    device_id          UUID         NOT NULL DEFAULT gen_random_uuid(),
    serial_number      VARCHAR(128) NOT NULL,
    device_type        VARCHAR(16)  NOT NULL,
    device_public_key  TEXT         NULL,
    is_active          BOOLEAN      NOT NULL DEFAULT TRUE,
    registered_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_devices PRIMARY KEY (device_id),
    CONSTRAINT uq_devices_serial UNIQUE (serial_number),
    CONSTRAINT ck_devices_type CHECK (
        device_type IN ('android', 'ios', 'windows')
    )
);

COMMENT ON TABLE  devices IS 'EN-023 Device — ハンディ端末デバイスマスタ。work_events.terminal_id の外部キー参照元。ALCOA+ Attributable 要件（どの端末で記録されたかを特定する）。';
COMMENT ON COLUMN devices.device_type        IS 'android / ios / windows の 3 値（CLAUDE.md 対応 OS と一致）。';
COMMENT ON COLUMN devices.device_public_key  IS 'base64 エンコードされた Ed25519 公開鍵（32 バイト）。電子サイン作成時のデバイス署名検証に使用する。NULL は公開鍵未登録（デバイス登録前または検証スキップ対象）。';

-- =====================================================
-- TBL-002: electronic_signs（電子サインレコード・Append-only）
-- ※ master_versions が electronic_signs を参照するため先に作成する
-- =====================================================
-- DDL-002: TBL-002 electronic_signs
-- EN-015 ElectronicSign — 電子サインレコード。Append-only。ALCOA+ 承認証拠。
CREATE TABLE IF NOT EXISTS electronic_signs (
    sign_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    signer_id       UUID        NOT NULL,
    signed_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sign_purpose    VARCHAR(64) NOT NULL,
    target_type     VARCHAR(32) NOT NULL,
    target_id       UUID        NOT NULL,
    sign_method     VARCHAR(32) NOT NULL DEFAULT 'PIN',
    credential_hash CHAR(64)    NOT NULL,
    ip_address      INET        NULL,
    device_id       UUID        NULL,

    CONSTRAINT pk_electronic_signs PRIMARY KEY (sign_id),
    CONSTRAINT fk_electronic_signs_signer FOREIGN KEY (signer_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_electronic_signs_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    CONSTRAINT ck_electronic_signs_purpose CHECK (
        sign_purpose IN (
            'step_completed_approval',
            'work_completed_approval',
            'master_publish_approval',
            'suspension_approval',
            'capa_closure_approval',
            'nonconformity_closure_approval'
        )
    ),
    CONSTRAINT ck_electronic_signs_target_type CHECK (
        target_type IN ('work_event', 'master_version', 'suspension', 'capa', 'nonconformity')
    ),
    CONSTRAINT ck_electronic_signs_method CHECK (
        sign_method IN ('PIN', 'BIOMETRIC', 'PASSWORD', 'HARDWARE_TOKEN')
    )
);

COMMENT ON TABLE  electronic_signs IS 'EN-015 ElectronicSign — 電子サインレコード。Append-only。ALCOA+ Original / Attributable 要件。sign_id は master_versions・suspensions・capas から FK 参照される。';
COMMENT ON COLUMN electronic_signs.sign_purpose    IS '署名目的の列挙値。';
COMMENT ON COLUMN electronic_signs.credential_hash IS '認証資格情報のハッシュ（生 PIN・パスワードは保存しない）。';
COMMENT ON COLUMN electronic_signs.ip_address      IS '署名時のクライアント IP。プライバシー保護のため /24 マスクを推奨（アプリ層で制御）。';

-- =====================================================
-- TBL-004: master_versions（SOP・Step 版数管理）
-- =====================================================
-- DDL-004: TBL-004 master_versions
-- EN-010 MasterVersion — SOP・Step・StepTypeDefinition・StepFlowRule の版数管理
CREATE TABLE IF NOT EXISTS master_versions (
    master_version_id  UUID         NOT NULL DEFAULT gen_random_uuid(),
    master_type        VARCHAR(32)  NOT NULL,
    master_id          UUID         NOT NULL,
    version_number     SMALLINT     NOT NULL,
    status             VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    effective_date     DATE         NULL,
    created_by         UUID         NOT NULL,
    published_by       UUID         NULL,
    sign_id            UUID         NULL,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_master_versions PRIMARY KEY (master_version_id),
    CONSTRAINT uq_master_versions_composite UNIQUE (master_type, master_id, version_number),
    CONSTRAINT fk_master_versions_created_by FOREIGN KEY (created_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_master_versions_published_by FOREIGN KEY (published_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    CONSTRAINT fk_master_versions_sign FOREIGN KEY (sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    CONSTRAINT ck_master_versions_type CHECK (
        master_type IN ('SOP', 'STEP', 'STEP_TYPE', 'FLOW_RULE')
    ),
    CONSTRAINT ck_master_versions_status CHECK (
        status IN ('DRAFT', 'UNDER_REVIEW', 'PUBLISHED', 'ARCHIVED')
    ),
    CONSTRAINT ck_master_versions_version_positive CHECK (version_number > 0),
    CONSTRAINT ck_master_versions_published_requires_publisher CHECK (
        NOT (status IN ('PUBLISHED', 'ARCHIVED') AND published_by IS NULL)
    )
);

COMMENT ON TABLE  master_versions IS 'EN-010 MasterVersion — SOP/Step/StepTypeDefinition/StepFlowRule の版数管理テーブル。PUBLISHED 状態への遷移後は status 以外の列を変更しない（アプリ層で強制）。';
COMMENT ON COLUMN master_versions.master_type    IS 'SOP / STEP / STEP_TYPE / FLOW_RULE の 4 値。';
COMMENT ON COLUMN master_versions.master_id      IS '版数対象マスタの主キー（sop_id / step_id / step_type_definition_id / step_flow_rule_id）。';
COMMENT ON COLUMN master_versions.version_number IS '1 から始まる増分番号。同一 master_type + master_id の最大値 + 1 を Rust 層で計算して INSERT する。';
COMMENT ON COLUMN master_versions.status         IS 'DRAFT → UNDER_REVIEW → PUBLISHED → ARCHIVED の順序で遷移。逆遷移不可（アプリ層で制御）。';
COMMENT ON COLUMN master_versions.sign_id        IS 'PUBLISHED 遷移時に付与する電子サインの sign_id。ALCOA+ Original 要件。';

-- =====================================================
-- TBL-021: processes（プロセス（工程群）マスタ）
-- =====================================================
-- DDL-021: TBL-021 processes
-- EN-005 Process — プロセス（工程群）マスタ
CREATE TABLE IF NOT EXISTS processes (
    process_id      UUID        NOT NULL DEFAULT gen_random_uuid(),
    process_code    VARCHAR(64) NOT NULL,
    name            JSONB       NOT NULL DEFAULT '{}'::jsonb,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    deleted_at      TIMESTAMPTZ NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_processes PRIMARY KEY (process_id),
    CONSTRAINT uq_processes_code UNIQUE (process_code),
    CONSTRAINT ck_processes_code_format CHECK (
        process_code ~ '^[A-Z]{2,4}-[0-9]{3}$'
    ),
    CONSTRAINT ck_processes_name_is_object CHECK (
        jsonb_typeof(name) = 'object'
    ),
    CONSTRAINT ck_processes_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  processes IS 'EN-005 Process — プロセスマスタ。process_code 形式: {英大文字2-4字}-{連番3桁}。例: ASS-001。';
COMMENT ON COLUMN processes.name       IS '多言語名称 JSONB。{"ja": "組立", "en": "Assembly"} 形式。ja キーは必須。';
COMMENT ON COLUMN processes.deleted_at IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。is_active=FALSE と併用する。';

-- =====================================================
-- TBL-022: operations（オペレーション（工程）マスタ）
-- =====================================================
-- DDL-022: TBL-022 operations
-- EN-006 Operation — オペレーション（工程）マスタ。process の子。
CREATE TABLE IF NOT EXISTS operations (
    operation_id     UUID        NOT NULL DEFAULT gen_random_uuid(),
    process_id       UUID        NOT NULL,
    operation_code   VARCHAR(64) NOT NULL,
    name             JSONB       NOT NULL DEFAULT '{}'::jsonb,
    sequence_number  INTEGER     NOT NULL,
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    deleted_at       TIMESTAMPTZ NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_operations PRIMARY KEY (operation_id),
    CONSTRAINT uq_operations_code UNIQUE (operation_code),
    CONSTRAINT fk_operations_process FOREIGN KEY (process_id)
        REFERENCES processes (process_id) ON DELETE RESTRICT,
    CONSTRAINT ck_operations_sequence_positive CHECK (sequence_number > 0),
    CONSTRAINT ck_operations_name_is_object CHECK (
        jsonb_typeof(name) = 'object'
    ),
    CONSTRAINT ck_operations_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  operations IS 'EN-006 Operation — オペレーションマスタ。operation_code 形式: {process_code}-{連番3桁}。例: ASS-001-003。';
COMMENT ON COLUMN operations.sequence_number IS 'プロセス内での表示順。1 以上の正整数。';
COMMENT ON COLUMN operations.deleted_at      IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-023: products（製品マスタ）
-- =====================================================
-- DDL-023: TBL-023 products
-- EN-007 Product — 製品マスタ
CREATE TABLE IF NOT EXISTS products (
    product_id    UUID         NOT NULL DEFAULT gen_random_uuid(),
    product_code  VARCHAR(128) NOT NULL,
    name          JSONB        NOT NULL DEFAULT '{}'::jsonb,
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    deleted_at    TIMESTAMPTZ  NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_products PRIMARY KEY (product_id),
    CONSTRAINT uq_products_code UNIQUE (product_code),
    CONSTRAINT ck_products_name_is_object CHECK (
        jsonb_typeof(name) = 'object'
    ),
    CONSTRAINT ck_products_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  products IS 'EN-007 Product — 製品マスタ。product_code は外部システム（ERP 等）に合わせた任意形式を許容する。';
COMMENT ON COLUMN products.product_code IS '外部システム形式を許容するため長め（128）に設定。変更不可の公開識別子。';
COMMENT ON COLUMN products.deleted_at   IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-036（materials）・TBL-037（suppliers）
-- lots より先に作成する（lots が supplier_id / material_id を FK 参照するため）
-- =====================================================

-- TBL-036: materials（材料・部品・工具・包材マスタ）
-- DDL-036: TBL-036 materials
-- EN-028 Material — 材料・部品・工具・包材マスタ（版管理）
CREATE TABLE IF NOT EXISTS materials (
    material_id     UUID            NOT NULL DEFAULT gen_random_uuid(),
    material_code   VARCHAR(64)     NOT NULL,
    name            VARCHAR(256)    NOT NULL,
    material_type   TEXT            NOT NULL,
    description     TEXT            NOT NULL DEFAULT '',
    version         INTEGER         NOT NULL DEFAULT 1,
    is_active       BOOLEAN         NOT NULL DEFAULT TRUE,
    deleted_at      TIMESTAMPTZ     NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_materials PRIMARY KEY (material_id),
    CONSTRAINT uq_materials_code UNIQUE (material_code),
    CONSTRAINT ck_materials_type CHECK (
        material_type IN ('RAW_MATERIAL', 'COMPONENT', 'TOOL', 'PACKAGING')
    ),
    CONSTRAINT ck_materials_version CHECK (version >= 1)
);

COMMENT ON TABLE  materials IS 'EN-028 Material — 材料マスタ。物理削除禁止（is_active=FALSE で論理削除）。';
COMMENT ON COLUMN materials.material_code IS '材料コード（購買システム連携キー）。UNIQUE 制約。';
COMMENT ON COLUMN materials.material_type IS 'RAW_MATERIAL=原材料 / COMPONENT=部品 / TOOL=工具 / PACKAGING=包材。';
COMMENT ON COLUMN materials.deleted_at    IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- TBL-037: suppliers（仕入先マスタ）
-- DDL-037: TBL-037 suppliers
-- EN-029 Supplier — 仕入先マスタ（版管理）
CREATE TABLE IF NOT EXISTS suppliers (
    supplier_id     UUID            NOT NULL DEFAULT gen_random_uuid(),
    supplier_code   VARCHAR(64)     NOT NULL,
    name            VARCHAR(256)    NOT NULL,
    address         TEXT            NOT NULL DEFAULT '',
    contact         VARCHAR(256)    NOT NULL DEFAULT '',
    version         INTEGER         NOT NULL DEFAULT 1,
    is_active       BOOLEAN         NOT NULL DEFAULT TRUE,
    deleted_at      TIMESTAMPTZ     NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_suppliers PRIMARY KEY (supplier_id),
    CONSTRAINT uq_suppliers_code UNIQUE (supplier_code),
    CONSTRAINT ck_suppliers_version CHECK (version >= 1)
);

COMMENT ON TABLE  suppliers IS 'EN-029 Supplier — 仕入先マスタ。物理削除禁止。';
COMMENT ON COLUMN suppliers.supplier_code IS '仕入先コード（購買システム連携キー）。UNIQUE 制約。';
COMMENT ON COLUMN suppliers.deleted_at    IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-024: lots（ロット追跡マスタ）
-- IQC 拡張列（supplier_id / material_id / qc_status / rework_history_count / parent_lot_id）を含む最終形 DDL
-- =====================================================
-- DDL-024: TBL-024 lots
-- EN-021 Lot — ロット追跡マスタ。製造ロット単位のトレーサビリティ記録。
CREATE TABLE IF NOT EXISTS lots (
    lot_id                UUID         NOT NULL DEFAULT gen_random_uuid(),
    lot_code              VARCHAR(128) NOT NULL,
    product_id            UUID         NOT NULL,
    lot_status            VARCHAR(16)  NOT NULL DEFAULT 'IN_PRODUCTION',
    quantity              INTEGER      NOT NULL,
    unit                  VARCHAR(32)  NOT NULL DEFAULT 'pcs',
    lot_started_at        TIMESTAMPTZ  NULL,
    lot_closed_at         TIMESTAMPTZ  NULL,
    -- IQC/リワーク拡張列（ADR-011 / DBレビュー指摘3対応）
    supplier_id           UUID         NULL,
    material_id           UUID         NULL,
    qc_status             VARCHAR(24)  NOT NULL DEFAULT 'PENDING',
    rework_history_count  INTEGER      NOT NULL DEFAULT 0,
    parent_lot_id         UUID         NULL,
    deleted_at            TIMESTAMPTZ  NULL,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_lots PRIMARY KEY (lot_id),
    CONSTRAINT uq_lots_code UNIQUE (lot_code),
    CONSTRAINT fk_lots_product FOREIGN KEY (product_id)
        REFERENCES products (product_id) ON DELETE RESTRICT,
    CONSTRAINT fk_lots_supplier FOREIGN KEY (supplier_id)
        REFERENCES suppliers (supplier_id) ON DELETE RESTRICT,
    CONSTRAINT fk_lots_material FOREIGN KEY (material_id)
        REFERENCES materials (material_id) ON DELETE RESTRICT,
    CONSTRAINT fk_lots_parent_lot FOREIGN KEY (parent_lot_id)
        REFERENCES lots (lot_id) ON DELETE RESTRICT,
    CONSTRAINT ck_lots_status CHECK (
        lot_status IN ('IN_PRODUCTION', 'ON_HOLD', 'COMPLETED', 'REJECTED', 'SCRAPPED')
    ),
    CONSTRAINT ck_lots_qc_status CHECK (
        qc_status IN ('PENDING', 'INSPECTING', 'PASSED', 'CONDITIONAL_PASS', 'SCREENING_REQUIRED', 'REJECTED', 'SCRAPPED', 'RETURNED')
    ),
    CONSTRAINT ck_lots_quantity_positive CHECK (quantity > 0),
    CONSTRAINT ck_lots_rework_history_non_negative CHECK (rework_history_count >= 0),
    CONSTRAINT ck_lots_parent_lot_no_self CHECK (parent_lot_id IS NULL OR parent_lot_id <> lot_id),
    CONSTRAINT ck_lots_closed_after_started CHECK (
        NOT (lot_closed_at IS NOT NULL AND lot_started_at IS NULL)
        AND NOT (lot_closed_at IS NOT NULL AND lot_closed_at < lot_started_at)
    )
);

COMMENT ON TABLE  lots IS 'EN-021 Lot — 製造ロットマスタ。7年以上保存。lot_code は ERP/MES 連携時の外部識別子。IQC 拡張列（supplier_id / material_id / qc_status / rework_history_count / parent_lot_id）を含む最終形（指摘3対応）。';
COMMENT ON COLUMN lots.lot_code              IS 'ロット番号。ERP 等の外部システムと一致させる。変更不可の公開識別子。';
COMMENT ON COLUMN lots.lot_status            IS 'IN_PRODUCTION / ON_HOLD / COMPLETED / REJECTED / SCRAPPED の 5 値。';
COMMENT ON COLUMN lots.unit                  IS 'UCUM コード。個数: pcs、kg、m 等。';
COMMENT ON COLUMN lots.supplier_id           IS 'IQC 拡張列。入荷元仕入先の supplier_id（NULL は製造品）。';
COMMENT ON COLUMN lots.material_id           IS 'IQC 拡張列。材料・部品の material_id（NULL は製造品）。';
COMMENT ON COLUMN lots.qc_status             IS 'IQC 拡張列。受入検査 QC ステータス。後工程ゲートに使用する。';
COMMENT ON COLUMN lots.rework_history_count  IS 'IQC 拡張列。このロットのリワーク実施回数。BAT-011 が日次集計で更新する。';
COMMENT ON COLUMN lots.parent_lot_id         IS 'IQC 拡張列。リワーク後の新ロット発行時に元ロットの lot_id を設定する。自己参照 FK（自ループ禁止制約付き）。';
COMMENT ON COLUMN lots.deleted_at            IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-025: equipments（生産設備マスタ）
-- =====================================================
-- DDL-025: TBL-025 equipments
-- EN-019 Equipment — 生産設備マスタ
CREATE TABLE IF NOT EXISTS equipments (
    equipment_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    equipment_code       VARCHAR(64) NOT NULL,
    name                 JSONB       NOT NULL DEFAULT '{}'::jsonb,
    equipment_type       VARCHAR(64) NOT NULL,
    process_id           UUID        NULL,
    scan_code            VARCHAR(64) NULL,
    tool_subtype         VARCHAR(64) NULL,
    calibration_due_date DATE        NULL,
    is_active            BOOLEAN     NOT NULL DEFAULT TRUE,
    deleted_at           TIMESTAMPTZ NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_equipments PRIMARY KEY (equipment_id),
    CONSTRAINT uq_equipments_code UNIQUE (equipment_code),
    CONSTRAINT uq_equipments_scan_code UNIQUE (scan_code),
    CONSTRAINT fk_equipments_process FOREIGN KEY (process_id)
        REFERENCES processes (process_id) ON DELETE RESTRICT,
    CONSTRAINT ck_equipments_name_is_object CHECK (
        jsonb_typeof(name) = 'object'
    ),
    CONSTRAINT ck_equipments_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    ),
    CONSTRAINT ck_equipments_jig_subtype CHECK (
        tool_subtype IS NULL OR tool_subtype IN (
            'TORQUE_WRENCH', 'FIXTURE_JIG', 'DRILL_GUIDE',
            'TEMPLATE', 'GAUGE', 'ASSEMBLY_JIG', 'OTHER'
        )
    )
);

COMMENT ON TABLE  equipments IS 'EN-019 Equipment — 生産設備マスタ。アンドン発報（TBL-012）の equipment_type 参照元。';
COMMENT ON COLUMN equipments.equipment_type       IS '設備種別の自由記述文字列（例: INJECTION_MOLD, ASSEMBLY_JIG, CONVEYOR）。コード体系は外部定義。';
COMMENT ON COLUMN equipments.process_id           IS '主に使用するプロセスの process_id。NULL は汎用設備。';
COMMENT ON COLUMN equipments.scan_code            IS 'スキャン照合用 ID（GS1 EID/AI 8004 互換）。NULL は照合対象外。FR-EV-013。';
COMMENT ON COLUMN equipments.tool_subtype         IS '工具・治具のサブ種別（例: TORQUE_WRENCH, FIXTURE_JIG）。NULL は設備（生産機械等）。';
COMMENT ON COLUMN equipments.calibration_due_date IS '治具点検期限（NULL は点検不要）。BR-BUS-007 のハードブロック対象範囲を計測器から治具に拡張。';
COMMENT ON COLUMN equipments.deleted_at           IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-026: instruments（計測器マスタ）
-- =====================================================
-- DDL-026: TBL-026 instruments
-- EN-024 Instrument — 計測器マスタ（校正管理付き）
CREATE TABLE IF NOT EXISTS instruments (
    instrument_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    instrument_code       VARCHAR(64) NOT NULL,
    name                  JSONB       NOT NULL DEFAULT '{}'::jsonb,
    instrument_type       VARCHAR(64) NOT NULL,
    calibration_due_date  DATE        NULL,
    calibration_cert_ref  TEXT        NULL,
    is_active             BOOLEAN     NOT NULL DEFAULT TRUE,
    deleted_at            TIMESTAMPTZ NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_instruments PRIMARY KEY (instrument_id),
    CONSTRAINT uq_instruments_code UNIQUE (instrument_code),
    CONSTRAINT ck_instruments_name_is_object CHECK (
        jsonb_typeof(name) = 'object'
    ),
    CONSTRAINT ck_instruments_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  instruments IS 'EN-024 Instrument — 計測器マスタ。校正期限管理（calibration_due_date）を含む。ALCOA+ Accurate 要件。';
COMMENT ON COLUMN instruments.calibration_due_date IS '次回校正期限。NULL は校正不要な参考計器。UI 警告トリガとして使用（アプリ層で制御）。';
COMMENT ON COLUMN instruments.calibration_cert_ref IS '校正証明書の参照パス（NAS 上のパスまたは URL）。';
COMMENT ON COLUMN instruments.deleted_at           IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-007: sops（作業手順書マスタ・版管理対応）
-- =====================================================
-- DDL-007: TBL-007 sops
-- EN-008 SOP — 作業手順書マスタ（版管理）
CREATE TABLE IF NOT EXISTS sops (
    sop_id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    operation_id        UUID        NOT NULL,
    sop_code            VARCHAR(64) NOT NULL,
    current_version_id  UUID        NULL,
    is_active           BOOLEAN     NOT NULL DEFAULT TRUE,
    deleted_at          TIMESTAMPTZ NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_sops PRIMARY KEY (sop_id),
    CONSTRAINT uq_sops_code UNIQUE (sop_code),
    CONSTRAINT fk_sops_operation FOREIGN KEY (operation_id)
        REFERENCES operations (operation_id) ON DELETE RESTRICT,
    CONSTRAINT fk_sops_current_version FOREIGN KEY (current_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT
);

COMMENT ON TABLE  sops IS 'EN-008 SOP — 作業手順書マスタ。sop_code 形式: {operation_code}-SOP-{連番3桁}。current_version_id は PUBLISHED 状態の最新版を指す。';
COMMENT ON COLUMN sops.current_version_id IS '現在有効版の master_version_id。PUBLISHED 状態のレコードのみ設定する（アプリ層で制御）。NULL は未公開。';
COMMENT ON COLUMN sops.deleted_at         IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-008: steps（SOP ステップマスタ・JSON Logic ルール含む）
-- =====================================================
-- DDL-008: TBL-008 steps
-- EN-009 Step — SOP を構成する作業ステップマスタ（版管理）
CREATE TABLE IF NOT EXISTS steps (
    step_id                UUID        NOT NULL DEFAULT gen_random_uuid(),
    sop_id                 UUID        NOT NULL,
    step_number            SMALLINT    NOT NULL,
    input_type             VARCHAR(32) NOT NULL,
    instruction_text       JSONB       NOT NULL DEFAULT '{}'::jsonb,
    judgment_condition     JSONB       NULL,
    evidence_required      BOOLEAN     NOT NULL DEFAULT FALSE,
    fmea_rpn_flag          BOOLEAN     NOT NULL DEFAULT FALSE,
    skill_level_required   SMALLINT    NOT NULL DEFAULT 1,
    expected_unit          VARCHAR(32) NULL,
    media_refs             JSONB       NULL,
    tips_refs              JSONB       NULL,
    required_scans         JSONB       NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_steps PRIMARY KEY (step_id),
    CONSTRAINT fk_steps_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    CONSTRAINT ck_steps_input_type CHECK (
        input_type IN (
            'boolean_check', 'numeric_input', 'photo_capture',
            'text_input', 'slider_range', 'multi_select',
            'signature', 'barcode_scan', 'nfc_read'
        )
    ),
    CONSTRAINT ck_steps_skill_level CHECK (skill_level_required BETWEEN 1 AND 5),
    CONSTRAINT ck_steps_step_number_positive CHECK (step_number > 0),
    CONSTRAINT ck_steps_instruction_is_object CHECK (
        jsonb_typeof(instruction_text) = 'object'
    ),
    CONSTRAINT ck_steps_instruction_has_ja CHECK (
        jsonb_typeof(instruction_text -> 'ja') = 'string'
        AND length(instruction_text ->> 'ja') > 0
    ),
    CONSTRAINT ck_steps_numeric_requires_condition CHECK (
        NOT (input_type = 'numeric_input' AND judgment_condition IS NULL)
    )
);

COMMENT ON TABLE  steps IS 'EN-009 Step — SOP ステップマスタ。PUBLISHED 後は内容変更禁止（DB トリガで強制）。';
COMMENT ON COLUMN steps.input_type           IS 'boolean_check / numeric_input / photo_capture / text_input / slider_range / multi_select / signature / barcode_scan / nfc_read の 9 値。';
COMMENT ON COLUMN steps.instruction_text     IS '多言語作業指示 JSONB。{"ja": "...", "en": "...", "ja-simple": "..."} 形式。ja キー必須。';
COMMENT ON COLUMN steps.judgment_condition   IS '合否判定条件 JSONB。{"usl": 10.5, "lsl": 9.5, "unit": "mm", "tolerance": 0.5} 形式。numeric_input 時は必須。';
COMMENT ON COLUMN steps.evidence_required    IS 'TRUE のとき、このステップ完了時に evidence_files の登録がゲート条件となる（BR-BUS-003）。';
COMMENT ON COLUMN steps.fmea_rpn_flag        IS 'FMEA の RPN 高リスク項目フラグ。TRUE のとき強調表示・監督承認が必要（アプリ層で制御）。';
COMMENT ON COLUMN steps.skill_level_required IS '1〜5。作業員の user_skills.achieved_level >= この値でなければ作業開始不可（FR-NV-008）。';
COMMENT ON COLUMN steps.expected_unit        IS 'UCUM コード。例: mm, celsius, kPa。numeric_input 時の表示単位。';
COMMENT ON COLUMN steps.media_refs           IS '参照メディア JSONB。[{"type": "VIDEO", "url": "...", "title": {"ja": "..."}}] 形式の配列。';
COMMENT ON COLUMN steps.tips_refs            IS 'コツ・注意事項 JSONB。[{"text": {"ja": "...", "en": "..."}, "severity": "WARNING"}] 形式の配列。';
COMMENT ON COLUMN steps.required_scans       IS 'FR-EV-013 ポカヨケ照合対象の配列。NULL は照合不要（旧来挙動）。スキーマ: urn:wnav:schema:required_scans:v1。';

-- =====================================================
-- TBL-030: step_flow_rules（JSON Logic 条件分岐ルール・JSONB）
-- =====================================================
-- DDL-030: TBL-030 step_flow_rules
-- EN-027 StepFlowRule — JSON Logic ベースの条件分岐ルール定義マスタ（版管理）
CREATE TABLE IF NOT EXISTS step_flow_rules (
    rule_id          UUID        NOT NULL DEFAULT gen_random_uuid(),
    sop_id           UUID        NOT NULL,
    from_step_id     UUID        NOT NULL,
    to_step_id       UUID        NOT NULL,
    rule_priority    SMALLINT    NOT NULL DEFAULT 0,
    rule_definition  JSONB       NOT NULL DEFAULT '{}'::jsonb,
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    deleted_at       TIMESTAMPTZ NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_step_flow_rules PRIMARY KEY (rule_id),
    CONSTRAINT fk_step_flow_rules_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    CONSTRAINT fk_step_flow_rules_from FOREIGN KEY (from_step_id)
        REFERENCES steps (step_id) ON DELETE RESTRICT,
    CONSTRAINT fk_step_flow_rules_to FOREIGN KEY (to_step_id)
        REFERENCES steps (step_id) ON DELETE RESTRICT,
    CONSTRAINT ck_step_flow_rules_no_self_loop CHECK (from_step_id <> to_step_id),
    CONSTRAINT ck_step_flow_rules_priority_non_negative CHECK (rule_priority >= 0),
    CONSTRAINT ck_step_flow_rules_definition_is_object CHECK (
        jsonb_typeof(rule_definition) = 'object'
    )
);

COMMENT ON TABLE  step_flow_rules IS 'EN-027 StepFlowRule — JSON Logic 条件分岐ルール。StepEngine が条件評価し次ステップを決定する。PUBLISHED 後は内容変更禁止。';
COMMENT ON COLUMN step_flow_rules.rule_priority   IS '同一 from_step_id に複数ルールが存在する場合の評価優先度。小さい値が優先。';
COMMENT ON COLUMN step_flow_rules.rule_definition IS 'JSON Logic 形式の条件式 JSONB。詳細スキーマは 05_JSONBスキーマ定義.md §1 参照。スキーマ: urn:wnav:schema:step_flow_rule_definition:v1。';
COMMENT ON COLUMN step_flow_rules.deleted_at      IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-029: step_type_definitions（ステップ入力型定義マスタ）
-- =====================================================
-- DDL-029: TBL-029 step_type_definitions
-- EN-026 StepTypeDefinition — ステップ入力型の詳細定義マスタ（版管理）
CREATE TABLE IF NOT EXISTS step_type_definitions (
    step_type_def_id   UUID        NOT NULL DEFAULT gen_random_uuid(),
    type_code          VARCHAR(32) NOT NULL,
    display_name       JSONB       NOT NULL DEFAULT '{}'::jsonb,
    schema_definition  JSONB       NOT NULL DEFAULT '{}'::jsonb,
    is_active          BOOLEAN     NOT NULL DEFAULT TRUE,
    deleted_at         TIMESTAMPTZ NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_step_type_definitions PRIMARY KEY (step_type_def_id),
    CONSTRAINT uq_step_type_definitions_code UNIQUE (type_code),
    CONSTRAINT ck_step_type_def_type_code CHECK (
        type_code IN (
            'boolean_check', 'numeric_input', 'photo_capture',
            'text_input', 'slider_range', 'multi_select',
            'signature', 'barcode_scan', 'nfc_read'
        )
    ),
    CONSTRAINT ck_step_type_def_display_name_is_object CHECK (
        jsonb_typeof(display_name) = 'object'
    ),
    CONSTRAINT ck_step_type_def_display_name_has_ja CHECK (
        jsonb_typeof(display_name -> 'ja') = 'string'
        AND length(display_name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  step_type_definitions IS 'EN-026 StepTypeDefinition — ステップ入力型の JSON Schema 定義マスタ。steps.input_type と 1:1 対応。version 管理のため master_versions（TBL-004）を使用する。';
COMMENT ON COLUMN step_type_definitions.type_code          IS 'steps.input_type と一致する 9 値。';
COMMENT ON COLUMN step_type_definitions.schema_definition  IS 'このステップ型の payload JSON Schema。StepEngine がバリデーションに使用する。';
COMMENT ON COLUMN step_type_definitions.deleted_at         IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-028: work_patterns（外部 ERP ロット → 内部 SOP マッピング）
-- =====================================================
-- DDL-028: TBL-028 work_patterns
-- EN-025 WorkPattern — external_key_bindings の解決先。外部 ID と SOP の中間エンティティ。
-- NOTE: factory_id は予約フィールド。ver1.0.0 では定数 UUID '00000000-0000-7000-8000-000000000001' を使用する。
-- NOTE: factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。
CREATE TABLE IF NOT EXISTS work_patterns (
    work_pattern_id  UUID        NOT NULL DEFAULT gen_random_uuid(),
    sop_id           UUID        NOT NULL,
    operation_id     UUID        NOT NULL,
    pattern_name     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    factory_id       UUID        NOT NULL,
    deleted_at       TIMESTAMPTZ NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_patterns PRIMARY KEY (work_pattern_id),
    CONSTRAINT fk_work_patterns_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    CONSTRAINT fk_work_patterns_operation FOREIGN KEY (operation_id)
        REFERENCES operations (operation_id) ON DELETE RESTRICT,
    CONSTRAINT ck_work_patterns_name_is_object CHECK (
        jsonb_typeof(pattern_name) = 'object'
    ),
    CONSTRAINT ck_work_patterns_name_has_ja CHECK (
        jsonb_typeof(pattern_name -> 'ja') = 'string'
        AND length(pattern_name ->> 'ja') > 0
    )
);

COMMENT ON TABLE  work_patterns IS 'EN-025 WorkPattern — external_key_bindings（TBL-027）の解決先。外部 ERP ロット → 内部 SOP を仲介する。';
COMMENT ON COLUMN work_patterns.factory_id IS 'ver1.0.0 では定数 UUID（シングルファクトリー運用）。将来のマルチファクトリー拡張時に使用する。factories テーブルは ver1.0.0 では作成しない（04_概要設計/99 §2-5 準拠）。';
COMMENT ON COLUMN work_patterns.deleted_at IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-034: device_sync_states（デバイス同期状態管理）
-- =====================================================
-- DDL-034: TBL-034 device_sync_states
-- EN-024 DeviceSyncState — デバイスのマスタ同期状態管理（更新可）
CREATE TABLE IF NOT EXISTS device_sync_states (
    device_id               UUID        NOT NULL,
    last_sync_at            TIMESTAMPTZ NULL,
    last_master_version_id  UUID        NULL,
    sync_status             VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_device_sync_states PRIMARY KEY (device_id),
    CONSTRAINT fk_device_sync_states_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE CASCADE,
    CONSTRAINT fk_device_sync_states_version FOREIGN KEY (last_master_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT,
    CONSTRAINT ck_device_sync_states_status CHECK (
        sync_status IN ('SYNCED', 'PENDING', 'CONFLICT')
    )
);

COMMENT ON TABLE  device_sync_states IS 'EN-024 DeviceSyncState — デバイスごとのマスタ同期状態。1 デバイス 1 レコード（device_id が PK）。CONFLICT 時は UI でオペレーター確認が必要。';
COMMENT ON COLUMN device_sync_states.last_master_version_id IS '最後に同期した master_version_id。NULL は初回同期前。';
COMMENT ON COLUMN device_sync_states.sync_status            IS 'SYNCED: 最新版同期済み / PENDING: 同期待ち / CONFLICT: 競合（手動解決必要）。';

-- =====================================================
-- TBL-039: sampling_plans（AQL サンプリング計画マスタ）
-- =====================================================
-- DDL-039: TBL-039 sampling_plans
-- EN-031 SamplingPlan — AQL サンプリング計画マスタ（版管理・時点固定 JSONB）
CREATE TABLE IF NOT EXISTS sampling_plans (
    plan_id                 UUID            NOT NULL DEFAULT gen_random_uuid(),
    material_id             UUID            NOT NULL,
    supplier_id             UUID            NOT NULL,
    aql                     NUMERIC(5,2)    NOT NULL,
    inspection_level        TEXT            NOT NULL DEFAULT 'II',
    aql_table_snapshot      JSONB           NOT NULL DEFAULT '{}'::jsonb,
    version                 INTEGER         NOT NULL DEFAULT 1,
    is_active               BOOLEAN         NOT NULL DEFAULT TRUE,
    deleted_at              TIMESTAMPTZ     NULL,
    created_by              UUID            NOT NULL,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_sampling_plans PRIMARY KEY (plan_id),
    CONSTRAINT fk_sampling_plans_material FOREIGN KEY (material_id) REFERENCES materials (material_id),
    CONSTRAINT fk_sampling_plans_supplier FOREIGN KEY (supplier_id) REFERENCES suppliers (supplier_id),
    CONSTRAINT fk_sampling_plans_creator  FOREIGN KEY (created_by)  REFERENCES users (user_id),
    CONSTRAINT ck_sampling_plans_aql CHECK (aql > 0),
    CONSTRAINT ck_sampling_plans_level CHECK (
        inspection_level IN ('S-1', 'S-2', 'S-3', 'S-4', 'I', 'II', 'III')
    ),
    CONSTRAINT ck_sampling_plans_version CHECK (version >= 1),
    CONSTRAINT ck_sampling_plans_snapshot_is_object CHECK (
        jsonb_typeof(aql_table_snapshot) = 'object'
    )
);

COMMENT ON TABLE  sampling_plans IS 'EN-031 SamplingPlan — AQL 計画マスタ。aql_table_snapshot に JIS Z 9015-1 の n/Ac/Re 表を時点固定で JSONB スナップショットとして保存する。';
COMMENT ON COLUMN sampling_plans.aql_table_snapshot IS 'JIS Z 9015-1 サンプル文字表 + AQL マスタ表の時点固定スナップショット（JSONB）。作成後に変更しない。';
COMMENT ON COLUMN sampling_plans.inspection_level   IS 'JIS Z 9015-1 の検査水準（S-1/S-2/S-3/S-4/I/II/III）。';
COMMENT ON COLUMN sampling_plans.deleted_at         IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';

-- =====================================================
-- TBL-046: rework_sop_mapping（不適合カテゴリ×リワーク種別→SOP マッピング）
-- =====================================================
-- DDL-046: TBL-046 rework_sop_mapping
-- EN-035 ReworkSopMapping — 不適合カテゴリ×リワーク種別→対象SOP のマッピングマスタ
CREATE TABLE IF NOT EXISTS rework_sop_mapping (
    mapping_id               UUID            NOT NULL DEFAULT gen_random_uuid(),
    nonconformity_category   VARCHAR(128)    NOT NULL,
    source_sop_id            UUID            NULL,
    source_step_id           UUID            NULL,
    target_rework_sop_id     UUID            NOT NULL,
    rework_type              TEXT            NOT NULL,
    version                  INTEGER         NOT NULL DEFAULT 1,
    is_active                BOOLEAN         NOT NULL DEFAULT TRUE,
    deleted_at               TIMESTAMPTZ     NULL,
    created_at               TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_rework_sop_mapping PRIMARY KEY (mapping_id),
    CONSTRAINT fk_rework_sop_mapping_sop FOREIGN KEY (target_rework_sop_id) REFERENCES sops (sop_id),
    CONSTRAINT fk_rework_sop_mapping_source_sop FOREIGN KEY (source_sop_id) REFERENCES sops (sop_id),
    CONSTRAINT fk_rework_sop_mapping_source_step FOREIGN KEY (source_step_id) REFERENCES steps (step_id),
    CONSTRAINT ck_rework_sop_mapping_type CHECK (
        rework_type IN ('TOUCH_UP', 'REWORK_FULL', 'SORTING', 'SCRAP', 'RETURN')
    )
);

COMMENT ON TABLE  rework_sop_mapping IS 'EN-035 ReworkSopMapping — 不適合カテゴリ×リワーク種別から適用 SOP を決定するマッピングマスタ。';
COMMENT ON COLUMN rework_sop_mapping.deleted_at IS '論理削除時刻。NULL が現役、NON-NULL が削除済み。';
