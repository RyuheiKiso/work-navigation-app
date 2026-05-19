-- V20260517120060__seed_step_type_definitions.sql
-- TBL-029 step_type_definitions の初期 9 種入力型定義を挿入する。
-- schema_definition は JSON Schema Draft 2020-12 形式で各入力型の検証ルールを記述する。
-- ON CONFLICT (type_code) DO NOTHING で冪等な再実行を保証する。
--
-- 対象ドキュメント: docs/05_詳細設計/01_データベース詳細設計/08_シードデータ・テストフィクスチャ設計.md §1-3

INSERT INTO step_type_definitions (
    step_type_def_id,
    type_code,
    display_name,
    schema_definition,
    is_active,
    created_at
) VALUES

    -- -------------------------------------------------------------------------
    -- boolean_check: チェックボックス形式の確認ステップ
    -- ハンディ端末の大型チェックボックスで OK/NG を確認する。
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'boolean_check',
        '{"ja": "チェックボックス", "en": "Boolean Check"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:boolean_check:v1",
            "title": "BooleanCheckInput",
            "description": "チェックボックス形式の確認ステップ入力値。checked が true の場合のみ step_completed を受け付ける。",
            "type": "object",
            "properties": {
                "checked": {
                    "type": "boolean",
                    "description": "チェック状態（true = 確認済み）"
                }
            },
            "required": ["checked"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- numeric_input: 数値入力ステップ（測定値・トルク値・温度等）
    -- judgment_condition の usl/lsl/tolerance と組み合わせて合否判定を行う。
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'numeric_input',
        '{"ja": "数値入力", "en": "Numeric Input"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:numeric_input:v1",
            "title": "NumericInput",
            "description": "数値入力ステップの入力値。測定値・トルク値・温度等に使用する。judgment_condition と組み合わせて合否判定を行う。",
            "type": "object",
            "properties": {
                "value": {
                    "type": "number",
                    "description": "入力された数値（単位は steps.judgment_condition.unit で定義）"
                },
                "unit": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 32,
                    "description": "UCUM コード。例: mm, Cel, kPa, kg"
                }
            },
            "required": ["value"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- photo_capture: 写真撮影による証拠収集ステップ
    -- evidence_files テーブルへの参照を記録する。
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'photo_capture',
        '{"ja": "写真撮影", "en": "Photo Capture"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:photo_capture:v1",
            "title": "PhotoCaptureInput",
            "description": "写真撮影ステップの入力値。evidence_files テーブルの evidence_id を記録する。",
            "type": "object",
            "properties": {
                "evidence_id": {
                    "type": "string",
                    "description": "evidence_files.evidence_id（UUID 文字列）"
                },
                "file_hash": {
                    "type": "string",
                    "minLength": 64,
                    "maxLength": 64,
                    "description": "SHA-256 ハッシュ（64 文字 hex）。改ざん検知に使用する。"
                },
                "thumbnail_data_url": {
                    "type": "string",
                    "description": "Base64 エンコードのサムネイル（オフライン同期前の一時表示用）"
                }
            },
            "required": ["evidence_id", "file_hash"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- text_input: テキスト入力ステップ（フリーテキスト確認・コメント等）
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'text_input',
        '{"ja": "テキスト入力", "en": "Text Input"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:text_input:v1",
            "title": "TextInput",
            "description": "テキスト入力ステップの入力値。フリーテキスト確認・コメント記入に使用する。",
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 2000,
                    "description": "入力されたテキスト"
                }
            },
            "required": ["text"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- slider_range: スライダー入力ステップ（段階評価・割合入力等）
    -- min/max の範囲内で連続値を入力する。
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'slider_range',
        '{"ja": "スライダー入力", "en": "Slider Range"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:slider_range:v1",
            "title": "SliderRangeInput",
            "description": "スライダー入力ステップの入力値。段階評価・割合入力（0〜100）等に使用する。",
            "type": "object",
            "properties": {
                "value": {
                    "type": "number",
                    "description": "スライダーで選択された値"
                },
                "min": {
                    "type": "number",
                    "description": "スライダーの最小値（ステップ定義から引き継ぐ）"
                },
                "max": {
                    "type": "number",
                    "description": "スライダーの最大値（ステップ定義から引き継ぐ）"
                },
                "step": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "スライダーの刻み幅"
                }
            },
            "required": ["value"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- multi_select: 複数選択ステップ（チェックリスト・部品選択等）
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'multi_select',
        '{"ja": "複数選択", "en": "Multi Select"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:multi_select:v1",
            "title": "MultiSelectInput",
            "description": "複数選択ステップの入力値。チェックリスト・部品種別選択等に使用する。",
            "type": "object",
            "properties": {
                "selected": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "minLength": 1
                    },
                    "minItems": 0,
                    "description": "選択された選択肢の値リスト"
                },
                "available_options": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "value": { "type": "string" },
                            "label": {
                                "type": "object",
                                "properties": {
                                    "ja": { "type": "string" },
                                    "en": { "type": "string" }
                                },
                                "required": ["ja"]
                            }
                        },
                        "required": ["value", "label"]
                    },
                    "description": "利用可能な選択肢一覧（記録用スナップショット）"
                }
            },
            "required": ["selected"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- signature: 電子署名ステップ（supervisor 承認・品質確認等）
    -- electronic_signs テーブルへの参照を記録する。
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'signature',
        '{"ja": "電子サイン", "en": "Signature"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:signature:v1",
            "title": "SignatureInput",
            "description": "電子署名ステップの入力値。electronic_signs テーブルの sign_id を記録する。supervisor 承認・品質確認等に使用する。",
            "type": "object",
            "properties": {
                "sign_id": {
                    "type": "string",
                    "description": "electronic_signs.sign_id（UUID 文字列）"
                },
                "signed_by": {
                    "type": "string",
                    "description": "署名者の user_id（UUID 文字列）"
                },
                "signed_at": {
                    "type": "string",
                    "format": "date-time",
                    "description": "署名日時（ISO 8601 UTC）"
                }
            },
            "required": ["sign_id", "signed_by", "signed_at"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- barcode_scan: バーコード・QR コードスキャンステップ
    -- ポカヨケ照合（FR-EV-013）と組み合わせて使用する。
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'barcode_scan',
        '{"ja": "バーコードスキャン", "en": "Barcode Scan"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:barcode_scan:v1",
            "title": "BarcodeScanInput",
            "description": "バーコード・QR コードスキャンステップの入力値。ポカヨケ照合（FR-EV-013）と組み合わせる。",
            "type": "object",
            "properties": {
                "barcode": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 512,
                    "description": "スキャンされたバーコード値（raw）"
                },
                "barcode_format": {
                    "type": "string",
                    "enum": ["QR_CODE", "CODE_128", "CODE_39", "EAN_13", "EAN_8", "GS1_128", "DATA_MATRIX", "PDF_417", "UNKNOWN"],
                    "description": "バーコードフォーマット種別"
                },
                "gs1_ai_data": {
                    "type": "object",
                    "description": "GS1 Application Identifier デコード結果（例: {\"01\": \"04006381333931\", \"17\": \"221231\"}）",
                    "additionalProperties": {
                        "type": "string"
                    }
                }
            },
            "required": ["barcode"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    ),

    -- -------------------------------------------------------------------------
    -- nfc_read: NFC タグ読み取りステップ（工具・機器の確認照合）
    -- -------------------------------------------------------------------------
    (
        gen_random_uuid(),
        'nfc_read',
        '{"ja": "NFC 読み取り", "en": "NFC Read"}'::jsonb,
        '{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:wnav:schema:step_input:nfc_read:v1",
            "title": "NfcReadInput",
            "description": "NFC タグ読み取りステップの入力値。工具・治具・機器の識別と照合に使用する。",
            "type": "object",
            "properties": {
                "nfc_data": {
                    "type": "string",
                    "minLength": 1,
                    "description": "NFC タグから読み取ったデータ（raw hex 文字列）"
                },
                "nfc_id": {
                    "type": "string",
                    "description": "NFC タグ ID（UID、16 進数文字列）"
                },
                "ndef_records": {
                    "type": "array",
                    "description": "NDEF レコードの配列（NDEF 対応タグの場合）",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": { "type": "string" },
                            "payload": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["nfc_data"],
            "additionalProperties": false
        }'::jsonb,
        TRUE,
        NOW()
    )

ON CONFLICT (type_code) DO NOTHING;
