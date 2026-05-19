-- V20260519130001__create_sqlite_mirror_tables.sql
--
-- SQLite ミラーテーブル — バックエンド PG テーブルのハンディ端末ローカル複製
--
-- ペア・マイグレーション規則（ADR-006）:
--   PG 対応: src/backend/migrations/ 配下の各テーブル定義マイグレーション
--   SQLite: 本ファイル（TypeORM migration として管理する）
--
-- PG → SQLite 型変換規約（07a_PG_SQLiteスキーマ同期戦略.md §2）:
--   UUID            → TEXT            （UUID v7 はアプリ側で生成、DEFAULT 句なし）
--   TIMESTAMPTZ     → TEXT            （ISO 8601 UTC 形式 'YYYY-MM-DDTHH:MM:SSZ'）
--   JSONB           → TEXT            （CHECK (json_valid(column)) で整合性保証）
--   BOOLEAN         → INTEGER         （0/1; TypeORM が変換）
--   SMALLINT/INT    → INTEGER
--   BIGSERIAL       → 端末非対応       （PG-only; 端末では UUID を使用）
--   TEXT/VARCHAR    → TEXT
--   NUMERIC         → REAL
--   CHAR(64)        → TEXT            （SHA-256 ハッシュ値 64 hex chars）
--   INET            → TEXT            （IP アドレスを文字列として保存）
--
-- PG-only テーブル（SQLite 非同期対象）:
--   TBL-002/004/006/010/014〜015/016〜023/025/027〜050
--   ※ ただし users/roles/user_roles/skills/user_skills/terminals/sites/production_lines/
--     step_flow_rules/disposition_rules は本ファイルで端末動作に必要なため追加する
--
-- 注意: SQLite は gen_random_uuid() をサポートしないため UUID の DEFAULT 句は設けない
--       アプリケーション層（TypeORM エンティティ）で UUID v7 を生成して INSERT する

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ============================================================
-- 1. sops — SOP マスタ（TBL-007 ミラー）
--    PG 権威: work_navigations テーブル群の中核マスタ
--    同期方向: サーバー → 端末（端末からは更新しない）
-- ============================================================
CREATE TABLE IF NOT EXISTS sops (
    -- UUID v7 はアプリ側で生成する（PG の gen_random_uuid() は端末では使用不可）
    id                  TEXT PRIMARY KEY,
    -- SOP コード（例: 'SOP-001'）
    code                TEXT NOT NULL UNIQUE,
    -- 多言語名称（PG: JSONB → SQLite: TEXT、形式: {"ja":"...","en":"...","zh":"..."}）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- SOP バージョン番号
    version             INTEGER NOT NULL DEFAULT 1,
    -- 論理削除タイムスタンプ（PG: TIMESTAMPTZ → SQLite: TEXT ISO8601）
    deleted_at          TEXT,
    -- 作成日時（PG: TIMESTAMPTZ → SQLite: TEXT ISO8601）
    created_at          TEXT NOT NULL,
    -- 更新日時（PG: TIMESTAMPTZ → SQLite: TEXT ISO8601）
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sops_code ON sops (code);
CREATE INDEX IF NOT EXISTS idx_sops_deleted_at ON sops (deleted_at);

-- ============================================================
-- 2. steps — SOP ステップ（TBL-008 ミラー）
--    PG 権威: sops.id を外部キーとする子テーブル
--    同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS steps (
    id                  TEXT PRIMARY KEY,
    -- 所属 SOP の ID（ミラー FK）
    sop_id              TEXT NOT NULL REFERENCES sops(id),
    -- ステップ内連番（表示順序）
    step_order          INTEGER NOT NULL,
    -- ステップ種別（例: 'inspection' | 'assembly' | 'measurement'）
    step_type           TEXT NOT NULL,
    -- 多言語ステップ名称（PG: JSONB → SQLite: TEXT）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- ステップ実行ロジック定義（JSON Logic; PG: JSONB → SQLite: TEXT）
    logic_json          TEXT CHECK (logic_json IS NULL OR json_valid(logic_json)),
    -- 必須入力項目定義（PG: JSONB → SQLite: TEXT）
    required_inputs_json TEXT CHECK (required_inputs_json IS NULL OR json_valid(required_inputs_json)),
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_steps_sop_id ON steps (sop_id);
CREATE INDEX IF NOT EXISTS idx_steps_sop_order ON steps (sop_id, step_order);

-- ============================================================
-- 3. users — ユーザー（TBL-016 ミラー）
--    PG 権威: ユーザー認証・権限管理テーブル
--    同期方向: サーバー → 端末
--    重要: password_hash は機密情報のためミラー対象外（同期しない）
-- ============================================================
CREATE TABLE IF NOT EXISTS users (
    id                  TEXT PRIMARY KEY,
    -- ログイン用ユーザー名
    username            TEXT NOT NULL UNIQUE,
    -- 表示名（PG: JSONB → SQLite: TEXT）
    display_name_json   TEXT NOT NULL CHECK (json_valid(display_name_json)),
    -- ユーザー状態（'active' | 'inactive' | 'locked'）
    status              TEXT NOT NULL DEFAULT 'active',
    -- 所属拠点 ID（ミラー FK）
    site_id             TEXT,
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
    -- password_hash は機密情報のためミラーしない（端末認証は JWT トークンで行う）
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_users_site_id ON users (site_id);

-- ============================================================
-- 4. roles — ロール（TBL-017 ミラー）
--    PG 権威: RBAC ロール定義テーブル
--    同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS roles (
    id                  TEXT PRIMARY KEY,
    -- ロールコード（例: 'operator' | 'supervisor' | 'auditor'）
    code                TEXT NOT NULL UNIQUE,
    -- 多言語ロール名称（PG: JSONB → SQLite: TEXT）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_roles_code ON roles (code);

-- ============================================================
-- 5. user_roles — ユーザー・ロール紐付（TBL-018 ミラー）
--    PG 権威: users と roles の多対多中間テーブル
--    同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS user_roles (
    -- 複合主キー
    user_id             TEXT NOT NULL REFERENCES users(id),
    role_id             TEXT NOT NULL REFERENCES roles(id),
    created_at          TEXT NOT NULL,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles (user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles (role_id);

-- ============================================================
-- 6. skills — スキル（TBL-019 ミラー）
--    PG 権威: 作業スキル定義テーブル
--    同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS skills (
    id                  TEXT PRIMARY KEY,
    -- スキルコード（例: 'welding-basic'）
    code                TEXT NOT NULL UNIQUE,
    -- 多言語スキル名称（PG: JSONB → SQLite: TEXT）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_skills_code ON skills (code);

-- ============================================================
-- 7. user_skills — ユーザー・スキル（TBL-020 ミラー）
--    PG 権威: users と skills の多対多中間テーブル
--    同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS user_skills (
    user_id             TEXT NOT NULL REFERENCES users(id),
    skill_id            TEXT NOT NULL REFERENCES skills(id),
    -- スキルレベル（例: 1=見習い, 2=一人前, 3=指導者）
    level               INTEGER NOT NULL DEFAULT 1,
    -- 資格取得日（PG: DATE → SQLite: TEXT ISO 8601 'YYYY-MM-DD'）
    acquired_at         TEXT,
    created_at          TEXT NOT NULL,
    PRIMARY KEY (user_id, skill_id)
);

CREATE INDEX IF NOT EXISTS idx_user_skills_user_id ON user_skills (user_id);
CREATE INDEX IF NOT EXISTS idx_user_skills_skill_id ON user_skills (skill_id);

-- ============================================================
-- 8. terminals — 端末（TBL-021 ミラー）
--    PG 権威: 登録済みハンディ端末の管理テーブル
--    同期方向: サーバー → 端末（自端末情報の確認に使用）
-- ============================================================
CREATE TABLE IF NOT EXISTS terminals (
    id                  TEXT PRIMARY KEY,
    -- 端末識別コード（例: 'TRM-LINE1-001'）
    code                TEXT NOT NULL UNIQUE,
    -- 表示名（PG: JSONB → SQLite: TEXT）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- 端末 OS 種別（'android' | 'ios' | 'windows'）
    os_type             TEXT NOT NULL,
    -- 配置拠点 ID（ミラー FK）
    site_id             TEXT,
    -- 配置製造ライン ID（ミラー FK）
    production_line_id  TEXT,
    -- 端末状態（'active' | 'maintenance' | 'retired'）
    status              TEXT NOT NULL DEFAULT 'active',
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_terminals_code ON terminals (code);
CREATE INDEX IF NOT EXISTS idx_terminals_site_id ON terminals (site_id);

-- ============================================================
-- 9. sites — 拠点（TBL-022 ミラー）
--    PG 権威: 工場・事業所などの拠点マスタ
--    同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS sites (
    id                  TEXT PRIMARY KEY,
    -- 拠点コード（例: 'SITE-TOKYO-001'）
    code                TEXT NOT NULL UNIQUE,
    -- 多言語拠点名称（PG: JSONB → SQLite: TEXT）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sites_code ON sites (code);

-- ============================================================
-- 10. production_lines — 製造ライン（TBL-023 ミラー）
--     PG 権威: 拠点内の製造ライン定義テーブル
--     同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS production_lines (
    id                  TEXT PRIMARY KEY,
    -- 製造ラインコード（例: 'LINE-A'）
    code                TEXT NOT NULL UNIQUE,
    -- 多言語ライン名称（PG: JSONB → SQLite: TEXT）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- 所属拠点 ID（ミラー FK）
    site_id             TEXT NOT NULL REFERENCES sites(id),
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_production_lines_code ON production_lines (code);
CREATE INDEX IF NOT EXISTS idx_production_lines_site_id ON production_lines (site_id);

-- ============================================================
-- 11. step_flow_rules — JSON Logic フロールール（TBL-030 ミラー）
--     PG 権威: SOP ステップの分岐・条件制御ルール定義テーブル
--     同期方向: サーバー → 端末
--     重要: ルール評価は端末ローカルで実行する（Offline-First 原則）
-- ============================================================
CREATE TABLE IF NOT EXISTS step_flow_rules (
    id                  TEXT PRIMARY KEY,
    -- 対象ステップ ID（ミラー FK）
    step_id             TEXT NOT NULL REFERENCES steps(id),
    -- ルール種別（例: 'condition' | 'skip' | 'branch'）
    rule_type           TEXT NOT NULL,
    -- JSON Logic ルール定義（PG: JSONB → SQLite: TEXT）
    -- eval/new Function 禁止: JSON Logic で宣言的に表現する（src/CLAUDE.md）
    rule_json           TEXT NOT NULL CHECK (json_valid(rule_json)),
    -- ルール適用優先度
    priority            INTEGER NOT NULL DEFAULT 0,
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_step_flow_rules_step_id ON step_flow_rules (step_id);
CREATE INDEX IF NOT EXISTS idx_step_flow_rules_priority ON step_flow_rules (step_id, priority);

-- ============================================================
-- 12. disposition_rules — 処置ルール（TBL-046 ミラー）
--     PG 権威: 不適合品・異常時の処置方法定義テーブル
--     同期方向: サーバー → 端末
-- ============================================================
CREATE TABLE IF NOT EXISTS disposition_rules (
    id                  TEXT PRIMARY KEY,
    -- 処置コード（例: 'DISP-REWORK-001'）
    code                TEXT NOT NULL UNIQUE,
    -- 多言語処置名称（PG: JSONB → SQLite: TEXT）
    name_json           TEXT NOT NULL CHECK (json_valid(name_json)),
    -- 処置条件（JSON Logic; PG: JSONB → SQLite: TEXT）
    condition_json      TEXT CHECK (condition_json IS NULL OR json_valid(condition_json)),
    -- 処置手順（PG: JSONB → SQLite: TEXT）
    procedure_json      TEXT CHECK (procedure_json IS NULL OR json_valid(procedure_json)),
    -- 論理削除タイムスタンプ
    deleted_at          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_disposition_rules_code ON disposition_rules (code);
