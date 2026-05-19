-- V20260519120010__seed_minimum.sql
-- 最低限のシードデータ（完全な初期状態）
-- ON CONFLICT DO NOTHING で冪等な再実行を保証する
-- =====================================================
-- 投入データ:
--   1. 初期 organization（開発用）
--      → organizations テーブルは ver1.0.0 では作成しないため、
--        COMMENT で代替する（TBL-004 organizations は上流 DDL カタログにも未定義）
--   2. 6 RBAC ロールレコード（roles テーブル）
--   3. 初期 admin ユーザー（login_id='admin@wnav.local'）
--   4. JWT キーペア雛形レコード（TBL-033 は devices テーブル・JWT キーは別途管理）
--      → JWT キーは jwt_keys テーブルが未定義のため、
--        devices テーブルへのシードと admin user_roles を設定する
-- =====================================================

-- =====================================================
-- 1. 開発用初期デバイス（シードデータ）
-- =====================================================
-- 開発・テスト環境用の管理端末デバイスを登録する
INSERT INTO devices (
    device_id,
    serial_number,
    device_type,
    is_active,
    registered_at,
    updated_at
) VALUES (
    '00000000-0000-7000-8000-000000000010',  -- 固定 UUID（開発用管理端末）
    'DEV-ADMIN-WINDOWS-001',
    'windows',
    TRUE,
    NOW(),
    NOW()
)
ON CONFLICT (device_id) DO NOTHING;

-- =====================================================
-- 2. 6 RBAC ロールレコード
-- role_id は固定 UUID（変更不可の定数値）
-- roles テーブルの CHECK 制約に定義された 6 値に対応する
-- =====================================================
INSERT INTO roles (role_id, role_name, description, created_at) VALUES
    (
        '00000000-0000-7000-8000-000000000001',
        'operator',
        '作業員。作業ナビゲーション実行権限。ハンディ端末で SOP に沿った作業記録を行う。',
        '2026-01-01 00:00:00+00'
    ),
    (
        '00000000-0000-7000-8000-000000000002',
        'supervisor',
        '現場監督。作業承認・アンドン対応・スキル認定権限。operator の作業を承認し、スキル認定を行う。',
        '2026-01-01 00:00:00+00'
    ),
    (
        '00000000-0000-7000-8000-000000000003',
        'master_admin',
        'マスタ管理者。SOP/Step の作成・版数公開権限。マスタメンテナンス画面からすべてのマスタデータを管理する。',
        '2026-01-01 00:00:00+00'
    ),
    (
        '00000000-0000-7000-8000-000000000004',
        'quality_admin',
        '品質担当。不適合・CAPA・改善提案の管理権限。IQC・リワーク・ディスポジションの品質管理を行う。',
        '2026-01-01 00:00:00+00'
    ),
    (
        '00000000-0000-7000-8000-000000000005',
        'system_admin',
        'IT 担当。ユーザー管理・デバイス登録・設定変更権限。システム全体の管理者。初期シードで作成される唯一のユーザーのロール。',
        '2026-01-01 00:00:00+00'
    ),
    (
        '00000000-0000-7000-8000-000000000006',
        'executive',
        '経営層。全データ参照（書き込み権限なし）。ダッシュボード・レポートの閲覧のみ。',
        '2026-01-01 00:00:00+00'
    )
ON CONFLICT (role_id) DO NOTHING;

-- =====================================================
-- 3. 初期 admin ユーザー
-- login_id='admin@wnav.local'
-- パスワードハッシュはプレースホルダー（実環境では環境変数から設定すること）
-- 初期パスワード: 'CHANGEME_ON_FIRST_LOGIN' のハッシュ（bcrypt コスト 12）
-- =====================================================
INSERT INTO users (
    user_id,
    login_id,
    display_name,
    is_active,
    anonymized_at,
    created_at,
    updated_at
) VALUES (
    '00000000-0000-7000-8000-000000000099',  -- 固定 UUID（初期 admin ユーザー）
    'admin@wnav.local',
    'システム管理者（初期）',
    TRUE,
    NULL,
    '2026-01-01 00:00:00+00',
    '2026-01-01 00:00:00+00'
)
ON CONFLICT (user_id) DO NOTHING;

-- admin ユーザーに system_admin ロールを付与する
-- granted_by は自己参照（初期シードのみ許容）
INSERT INTO user_roles (
    user_id,
    role_id,
    granted_at,
    granted_by
) VALUES (
    '00000000-0000-7000-8000-000000000099',  -- admin ユーザー
    '00000000-0000-7000-8000-000000000005',  -- system_admin ロール
    '2026-01-01 00:00:00+00',
    '00000000-0000-7000-8000-000000000099'   -- 自己参照（初期シードのみ許容）
)
ON CONFLICT (user_id, role_id) DO NOTHING;

-- admin ユーザーに master_admin ロールも付与する（初期マスタ設定のため）
INSERT INTO user_roles (
    user_id,
    role_id,
    granted_at,
    granted_by
) VALUES (
    '00000000-0000-7000-8000-000000000099',  -- admin ユーザー
    '00000000-0000-7000-8000-000000000003',  -- master_admin ロール
    '2026-01-01 00:00:00+00',
    '00000000-0000-7000-8000-000000000099'   -- 自己参照（初期シードのみ許容）
)
ON CONFLICT (user_id, role_id) DO NOTHING;

-- =====================================================
-- 4. step_type_definitions シードデータ（9 種の入力型定義）
-- =====================================================
INSERT INTO step_type_definitions (
    step_type_def_id,
    type_code,
    display_name,
    schema_definition,
    is_active,
    created_at,
    updated_at
) VALUES
    (
        gen_random_uuid(),
        'boolean_check',
        '{"ja": "チェックボックス", "en": "Boolean Check"}'::jsonb,
        '{"type": "object", "properties": {"checked": {"type": "boolean"}}, "required": ["checked"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'numeric_input',
        '{"ja": "数値入力", "en": "Numeric Input"}'::jsonb,
        '{"type": "object", "properties": {"value": {"type": "number"}}, "required": ["value"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'photo_capture',
        '{"ja": "写真撮影", "en": "Photo Capture"}'::jsonb,
        '{"type": "object", "properties": {"evidence_id": {"type": "string"}}, "required": ["evidence_id"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'text_input',
        '{"ja": "テキスト入力", "en": "Text Input"}'::jsonb,
        '{"type": "object", "properties": {"text": {"type": "string", "minLength": 1}}, "required": ["text"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'slider_range',
        '{"ja": "スライダー入力", "en": "Slider Range"}'::jsonb,
        '{"type": "object", "properties": {"value": {"type": "number"}}, "required": ["value"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'multi_select',
        '{"ja": "複数選択", "en": "Multi Select"}'::jsonb,
        '{"type": "object", "properties": {"selected": {"type": "array", "items": {"type": "string"}}}, "required": ["selected"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'signature',
        '{"ja": "電子サイン", "en": "Signature"}'::jsonb,
        '{"type": "object", "properties": {"sign_id": {"type": "string"}}, "required": ["sign_id"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'barcode_scan',
        '{"ja": "バーコードスキャン", "en": "Barcode Scan"}'::jsonb,
        '{"type": "object", "properties": {"barcode": {"type": "string", "minLength": 1}}, "required": ["barcode"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    ),
    (
        gen_random_uuid(),
        'nfc_read',
        '{"ja": "NFC 読み取り", "en": "NFC Read"}'::jsonb,
        '{"type": "object", "properties": {"nfc_data": {"type": "string"}}, "required": ["nfc_data"]}'::jsonb,
        TRUE,
        NOW(),
        NOW()
    )
ON CONFLICT (type_code) DO NOTHING;

-- =====================================================
-- 5. JWT キーペア雛形レコードについてのコメント
-- =====================================================
-- TBL-033 は documents では jwt_keys テーブルとして記述されているが、
-- 実際の詳細設計 DDL では devices テーブル（EN-023 Device）として定義されている。
-- JWT キーペアの管理は wnav_auth クレート内の設定（YAML / 環境変数）で行うため、
-- DB に雛形レコードを投入する方式は採用しない。
-- JWT 公開鍵・秘密鍵は以下で管理する:
--   開発環境: config/local.yaml の jwt.secret_key（figment + secret_ref）
--   本番環境: 環境変数 WNAV_JWT__SECRET_KEY
-- （YAML Config Architecture / ADR-006 参照）
