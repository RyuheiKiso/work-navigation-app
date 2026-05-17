# 05 JSONB スキーマ定義（TBL-030 step_flow_rules 等）

本章の責務は、PostgreSQL テーブル内の JSONB 列に保存されるデータ構造を JSON Schema（Draft 2020-12 準拠）で確定することである。これにより Rust 側の `serde_json` デシリアライズ型・TypeScript 側の型定義・フロントエンドバリデーションの実装根拠が確定し、JSONB 列への不正データ投入を構造レベルで防止する。

---

## 1. TBL-030 step_flow_rules.rule_definition（JSON Logic 条件式）

### 1-1. 用途

StepEngine が現在ステップの入力結果を評価し、次ステップを決定する条件分岐ルール。JSON Logic（https://jsonlogic.com/）仕様に準拠する。

### 1-2. JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "urn:wnav:schema:step_flow_rule_definition:v1",
  "title": "StepFlowRuleDefinition",
  "description": "JSON Logic 形式の条件式。StepEngine が next_step を決定するために評価する。",
  "type": "object",
  "properties": {
    "condition": {
      "description": "JSON Logic 演算子ツリー。評価結果が true の場合、このルールが適用される。",
      "oneOf": [
        {
          "description": "比較演算子（==, !=, >, >=, <, <=）",
          "type": "object",
          "properties": {
            "==":  { "type": "array", "minItems": 2, "maxItems": 2 },
            "!=":  { "type": "array", "minItems": 2, "maxItems": 2 },
            ">":   { "type": "array", "minItems": 2, "maxItems": 2 },
            ">=":  { "type": "array", "minItems": 2, "maxItems": 2 },
            "<":   { "type": "array", "minItems": 2, "maxItems": 2 },
            "<=":  { "type": "array", "minItems": 2, "maxItems": 2 }
          },
          "maxProperties": 1
        },
        {
          "description": "論理演算子（and, or, !）",
          "type": "object",
          "properties": {
            "and": { "type": "array", "minItems": 2 },
            "or":  { "type": "array", "minItems": 2 },
            "!":   {}
          },
          "maxProperties": 1
        },
        {
          "description": "変数参照（var）— payload.value 等を参照する",
          "type": "object",
          "properties": {
            "var": { "type": "string", "description": "ドット区切りのパス。例: 'payload.value', 'payload.judgment'" }
          },
          "required": ["var"],
          "maxProperties": 1
        },
        {
          "description": "定数値（null は always-true ルールを表す）",
          "type": ["null", "boolean", "number", "string"]
        }
      ]
    },
    "label": {
      "description": "このルールの人間可読ラベル。マスタ管理 UI での表示に使用する。",
      "type": "object",
      "properties": {
        "ja": { "type": "string", "minLength": 1 },
        "en": { "type": "string" }
      },
      "required": ["ja"]
    }
  },
  "required": ["condition"],
  "additionalProperties": false,
  "examples": [
    {
      "condition": { ">": [{ "var": "payload.value" }, 10.5] },
      "label": { "ja": "測定値が上限超過", "en": "Value exceeds USL" }
    },
    {
      "condition": {
        "and": [
          { ">=": [{ "var": "payload.value" }, 9.5] },
          { "<=": [{ "var": "payload.value" }, 10.5] }
        ]
      },
      "label": { "ja": "測定値が規格内（OK ルート）", "en": "Value within spec" }
    },
    {
      "condition": null,
      "label": { "ja": "常に適用（デフォルト遷移）", "en": "Always apply (default)" }
    }
  ]
}
```

---

## 2. TBL-008 steps.instruction_text（多言語作業指示）

### 2-1. 用途

ハンディ端末の作業ナビゲーション画面に表示する作業指示文。日本語・英語・やさしい日本語（外国人労働者対応）の 3 言語をサポートする。

### 2-2. JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "urn:wnav:schema:instruction_text:v1",
  "title": "InstructionText",
  "description": "多言語作業指示テキスト。ja は必須。",
  "type": "object",
  "properties": {
    "ja": {
      "type": "string",
      "minLength": 1,
      "maxLength": 2000,
      "description": "日本語作業指示（必須）"
    },
    "en": {
      "type": "string",
      "maxLength": 2000,
      "description": "英語作業指示（任意）"
    },
    "ja-simple": {
      "type": "string",
      "maxLength": 2000,
      "description": "やさしい日本語（外国人労働者向け、任意）"
    }
  },
  "required": ["ja"],
  "additionalProperties": false,
  "examples": [
    {
      "ja": "トルクレンチでボルトを 25 N·m で締め付けてください。",
      "en": "Tighten the bolt to 25 N·m using a torque wrench.",
      "ja-simple": "トルクレンチで ボルトを しめてください。25ニュートンメートルです。"
    }
  ]
}
```

---

## 3. TBL-008 steps.judgment_condition（合否判定条件）

### 3-1. 用途

`input_type = 'numeric_input'` のステップで使用する合否判定の基準値。測定値が \[lsl, usl\] の範囲内であれば OK、外れれば NG と判定する。

### 3-2. JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "urn:wnav:schema:judgment_condition:v1",
  "title": "JudgmentCondition",
  "description": "数値入力ステップの合否判定条件。input_type=numeric_input 時は必須。",
  "type": "object",
  "properties": {
    "usl": {
      "type": "number",
      "description": "上限規格値（Upper Specification Limit）"
    },
    "lsl": {
      "type": "number",
      "description": "下限規格値（Lower Specification Limit）"
    },
    "target": {
      "type": "number",
      "description": "目標値（任意）。SPC 管理図の中心線として使用する。"
    },
    "unit": {
      "type": "string",
      "minLength": 1,
      "maxLength": 32,
      "description": "UCUM コード。例: mm, Cel, kPa, kg"
    },
    "tolerance": {
      "type": "number",
      "exclusiveMinimum": 0,
      "description": "許容差（target ± tolerance が OK 範囲）。usl/lsl と併用時は AND 条件。"
    },
    "decimal_places": {
      "type": "integer",
      "minimum": 0,
      "maximum": 6,
      "description": "表示桁数（小数点以下）。UI の表示フォーマットに使用する。"
    }
  },
  "required": ["unit"],
  "anyOf": [
    { "required": ["usl"] },
    { "required": ["lsl"] },
    { "required": ["tolerance"] }
  ],
  "additionalProperties": false,
  "examples": [
    {
      "usl": 10.5,
      "lsl": 9.5,
      "target": 10.0,
      "unit": "mm",
      "tolerance": 0.5,
      "decimal_places": 2
    },
    {
      "usl": 30.0,
      "lsl": 20.0,
      "unit": "Cel",
      "decimal_places": 1
    }
  ]
}
```

---

## 4. TBL-001 work_events.payload（activity 別 JSONB）

### 4-1. 用途

work_events の `activity` 列の値に応じて payload の構造が変わる。9 種の activity それぞれに個別 JSON Schema を定義する。Rust の `serde_json::Value` を `#[serde(tag = "activity")]` で直列化する際の型定義根拠となる。

### 4-2. work_started payload

```json
{
  "$id": "urn:wnav:schema:payload:work_started:v1",
  "title": "WorkStartedPayload",
  "type": "object",
  "properties": {
    "lot_id": {
      "type": ["string", "null"],
      "description": "開始時に紐付けたロット ID（UUID 文字列）"
    },
    "work_order_id": {
      "type": ["string", "null"],
      "description": "開始時に紐付けたワークオーダー ID（UUID 文字列）"
    },
    "initial_step_index": {
      "type": "integer",
      "minimum": 0,
      "description": "開始ステップインデックス（再開でない場合は 0）"
    }
  },
  "required": ["initial_step_index"],
  "additionalProperties": false
}
```

### 4-3. step_completed payload

```json
{
  "$id": "urn:wnav:schema:payload:step_completed:v1",
  "title": "StepCompletedPayload",
  "type": "object",
  "properties": {
    "value": {
      "description": "入力値。input_type により型が異なる（boolean/number/string/array）"
    },
    "judgment": {
      "type": "string",
      "enum": ["OK", "NG", "WARNING", "N/A"],
      "description": "合否判定結果"
    },
    "duration_seconds": {
      "type": "integer",
      "minimum": 0,
      "description": "このステップに要した秒数（ステップ開始〜完了）"
    },
    "sign_id": {
      "type": ["string", "null"],
      "description": "電子サイン UUID（evidence_required=TRUE 時は必須）"
    }
  },
  "required": ["judgment", "duration_seconds"],
  "additionalProperties": false
}
```

### 4-4. step_skipped payload

```json
{
  "$id": "urn:wnav:schema:payload:step_skipped:v1",
  "title": "StepSkippedPayload",
  "type": "object",
  "properties": {
    "skip_reason": {
      "type": "string",
      "minLength": 1,
      "maxLength": 512,
      "description": "スキップ理由（監督承認必須時に必須、アプリ層で制御）"
    },
    "approved_by": {
      "type": ["string", "null"],
      "description": "スキップ承認者の user_id（UUID 文字列）"
    },
    "sign_id": {
      "type": ["string", "null"],
      "description": "承認電子サイン UUID"
    }
  },
  "required": ["skip_reason"],
  "additionalProperties": false
}
```

### 4-5. step_rejected payload

```json
{
  "$id": "urn:wnav:schema:payload:step_rejected:v1",
  "title": "StepRejectedPayload",
  "description": "訂正イベント方式（第2層保証）。誤記録の修正はこのイベントで開始する。",
  "type": "object",
  "properties": {
    "rejection_reason": {
      "type": "string",
      "minLength": 1,
      "maxLength": 1024,
      "description": "棄却理由（例: 誤入力: 数値 XXX → 正しくは YYY）"
    },
    "original_event_id": {
      "type": "string",
      "description": "修正対象の event_id（UUID 文字列）"
    },
    "rejected_by": {
      "type": "string",
      "description": "棄却実施者の user_id（UUID 文字列）"
    },
    "sign_id": {
      "type": "string",
      "description": "棄却承認の電子サイン UUID（必須）"
    }
  },
  "required": ["rejection_reason", "original_event_id", "rejected_by", "sign_id"],
  "additionalProperties": false
}
```

### 4-6. work_suspended payload

```json
{
  "$id": "urn:wnav:schema:payload:work_suspended:v1",
  "title": "WorkSuspendedPayload",
  "type": "object",
  "properties": {
    "suspension_id": {
      "type": "string",
      "description": "suspensions テーブルの suspension_id（UUID 文字列）"
    },
    "reason_category": {
      "type": "string",
      "enum": ["MATERIAL_WAIT", "EQUIPMENT_FAILURE", "QUALITY_HOLD", "WORKER_ABSENCE", "SAFETY_ISSUE", "PROCESS_CHANGE", "OTHER"]
    }
  },
  "required": ["suspension_id", "reason_category"],
  "additionalProperties": false
}
```

### 4-7. work_resumed payload

```json
{
  "$id": "urn:wnav:schema:payload:work_resumed:v1",
  "title": "WorkResumedPayload",
  "type": "object",
  "properties": {
    "suspension_id": {
      "type": "string",
      "description": "対応する suspension_id（UUID 文字列）"
    },
    "resumed_step_index": {
      "type": "integer",
      "minimum": 0,
      "description": "再開ステップインデックス"
    }
  },
  "required": ["suspension_id", "resumed_step_index"],
  "additionalProperties": false
}
```

### 4-8. work_completed payload

```json
{
  "$id": "urn:wnav:schema:payload:work_completed:v1",
  "title": "WorkCompletedPayload",
  "type": "object",
  "properties": {
    "total_duration_seconds": {
      "type": "integer",
      "minimum": 0,
      "description": "合計作業時間（中断時間を除く）"
    },
    "sign_id": {
      "type": ["string", "null"],
      "description": "作業完了時の電子サイン UUID"
    },
    "completion_note": {
      "type": ["string", "null"],
      "maxLength": 1024
    }
  },
  "required": ["total_duration_seconds"],
  "additionalProperties": false
}
```

### 4-9. work_cancelled payload

```json
{
  "$id": "urn:wnav:schema:payload:work_cancelled:v1",
  "title": "WorkCancelledPayload",
  "type": "object",
  "properties": {
    "cancel_reason": {
      "type": "string",
      "minLength": 1,
      "maxLength": 1024
    },
    "cancelled_by": {
      "type": "string",
      "description": "キャンセル実施者の user_id（UUID 文字列）"
    }
  },
  "required": ["cancel_reason", "cancelled_by"],
  "additionalProperties": false
}
```

### 4-10. evidence_attached payload

```json
{
  "$id": "urn:wnav:schema:payload:evidence_attached:v1",
  "title": "EvidenceAttachedPayload",
  "type": "object",
  "properties": {
    "evidence_id": {
      "type": "string",
      "description": "evidence_files テーブルの evidence_id（UUID 文字列）"
    },
    "file_type": {
      "type": "string",
      "enum": ["PHOTO", "AUDIO", "DOCUMENT", "VIDEO"]
    },
    "file_hash": {
      "type": "string",
      "minLength": 64,
      "maxLength": 64,
      "description": "SHA-256 ハッシュ（64 文字 hex）"
    }
  },
  "required": ["evidence_id", "file_type", "file_hash"],
  "additionalProperties": false
}
```

---

## 5. TBL-021 processes.name / TBL-022 operations.name 等（多言語 JSONB）

### 5-1. 用途

processes・operations・products・sops・steps・equipments・instruments・work_patterns の `name` 列に共通して使用する多言語名称スキーマ。

### 5-2. JSON Schema（再利用可能定義）

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "urn:wnav:schema:multilingual_name:v1",
  "title": "MultilingualName",
  "description": "多言語名称。ja は必須。en は任意。他言語は additionalProperties で追加可能。",
  "type": "object",
  "properties": {
    "ja": {
      "type": "string",
      "minLength": 1,
      "maxLength": 256,
      "description": "日本語名称（必須）"
    },
    "en": {
      "type": "string",
      "maxLength": 256,
      "description": "英語名称（任意）"
    }
  },
  "required": ["ja"],
  "additionalProperties": {
    "type": "string",
    "maxLength": 256
  },
  "examples": [
    { "ja": "組立工程", "en": "Assembly Process" },
    { "ja": "トルク締め付け", "en": "Torque Fastening" }
  ]
}
```

---

## 6. TBL-027 external_key_bindings.external_key（外部キーマッピング JSONB）

### 6-1. 用途

親機 ERP/MES から受信する外部識別子のキーバリューマップ。どのキーを持つかは外部システムに依存するため、スキーマは「値は全て文字列」という制約のみを課す。

### 6-2. JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "urn:wnav:schema:external_key:v1",
  "title": "ExternalKey",
  "description": "外部システム識別子のキーバリューマップ。キー名は外部システムに依存する（例: lot_id, product_code, work_order_no）。",
  "type": "object",
  "minProperties": 1,
  "additionalProperties": {
    "type": "string",
    "minLength": 1,
    "maxLength": 256
  },
  "examples": [
    { "lot_id": "L001", "product_code": "P-A001-REV2" },
    { "work_order_no": "WO-2026-001234" },
    { "lot_id": "L002", "serial_no": "SN-00123" }
  ]
}
```

---

**本節で確定した方針**
- **全 JSONB 列の JSON Schema（Draft 2020-12 準拠）を確定し、Rust の serde 型定義・TypeScript の型定義・フロントエンドバリデーションの実装根拠を提供した。**
- **work_events.payload は activity 別に 9 種の個別 JSON Schema を定義し、step_rejected には `original_event_id` と `sign_id` を必須とすることで訂正イベント方式の ALCOA+ Original 要件を JSON 構造レベルで確保した。**
- **steps.judgment_condition の `anyOf` 制約により usl・lsl・tolerance のいずれか 1 つ以上の必須化を強制し、合否判定基準が全く設定されない numeric_input ステップの登録を JSON Schema レベルで防止した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../../90_業界分析/21_電子記録の法規制とALCOA+.md)

### 関連
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../../90_業界分析/27_オフライン同期とデータ整合性.md)
