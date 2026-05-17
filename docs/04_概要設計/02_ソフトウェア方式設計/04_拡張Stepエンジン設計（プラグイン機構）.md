# 04 拡張 Step エンジン設計（プラグイン機構）

本章の責務は、計画 18 章（拡張可能 Step エンジン）で確定した Step エンジンの概要設計レベルでの具体化を確定することである。標準 Step タイプの実装・拡張 Step の追加機構・JSON Logic 条件分岐・DAG 検証の設計を定める。

図: fig_des_arch_plugin_step_engine（img/ 配下）を参照。

---

## 1. Step タイプ体系

### 1-1. 標準 4 タイプ（変更不可）

| タイプ名 | input_type 値 | 入力 UI | 出力データ |
|---|---|---|---|
| boolean_check | `boolean_check` | チェックボックス（OK/NG）| `{value: bool}` |
| numeric_input | `numeric_input` | 数値入力フィールド + 単位 | `{value: number, unit: string}` |
| photo_capture | `photo_capture` | カメラ起動 | `{evidence_id: UUID, hash: string}` |
| text_input | `text_input` | テキストエリア | `{value: string}` |

標準 4 タイプは変更・削除不可。コードを直接ハードコードし、プラグインシステムの外に置く。

### 1-2. 拡張タイプ（TBL-029 step_type_definitions から動的読み込み）

| タイプ名 | 概要 | バリデーション |
|---|---|---|
| slider_range | スライダーで数値範囲を選択 | USL/LSL の範囲内 |
| composite_check | 複数チェックボックスの AND 条件 | 全件 check が必要な場合 |
| color_chart | 色票選択（RAL/Munsell 等）| 許容色域の JSON Logic 評価 |
| signature | 電子署名パッド入力 | ElectronicSign 4 要素必須 |
| custom_list | マスタ定義の選択リスト | マスタ存在チェック |
| barcode_scan | GS1 バーコードスキャン | GS1 形式チェック（BR-BUS-034）|

### 1-3. 拡張タイプの削除ポリシー

計画 18 章確定: 削除時は `text_input` への縮退動作で過去記録の参照可能性を保証する。

```
1. TBL-029 の当該レコードを `is_active = FALSE` に論理削除
2. 当該タイプを参照する Step の `input_type` を `text_input` に変更（マイグレーション）
3. 過去の WorkEvent の payload は JSON のまま保持（縮退後も readable）
```

---

## 2. Step 実行エンジン（MOD-FE-HA StepEngine）のインターフェース

### 2-1. Step レンダラのインターフェース（TypeScript）

```typescript
// 全 Step タイプが実装すべきインターフェース
interface StepRenderer {
  inputType: string;
  render(step: StepDefinition, onComplete: (payload: StepPayload) => void): ReactElement;
  validate(payload: StepPayload, step: StepDefinition): ValidationResult;
}

// 標準タイプは static import、拡張タイプは TBL-029 の schema から動的生成
const stepRenderers: Map<string, StepRenderer> = new Map([
  ['boolean_check', new BooleanCheckRenderer()],
  ['numeric_input', new NumericInputRenderer()],
  ['photo_capture', new PhotoCaptureRenderer()],
  ['text_input', new TextInputRenderer()],
  // 拡張タイプは起動時に TBL-029 から読み込み登録
]);
```

### 2-2. ロックステップ進行の強制（BR-BUS-001）

```typescript
// Step 完了ゲート（StepEngine内部）
function canAdvanceToNextStep(
  currentStepIndex: number,
  steps: StepDefinition[],
  completedEvents: WorkEvent[]
): boolean {
  const currentStep = steps[currentStepIndex];
  const lastEvent = completedEvents.at(-1);
  
  // 直前 Step が完了していない場合は進行禁止
  if (!lastEvent || lastEvent.step_id !== currentStep.step_id) {
    throw new DomainError('ERR-BIZ-001', 'Lock step violation');
  }
  
  // 証拠必須の Step で証拠が未取得の場合も禁止
  if (currentStep.evidence_required && !lastEvent.payload.evidence_id) {
    throw new DomainError('ERR-BIZ-002', 'Evidence required');
  }
  
  return true;
}
```

---

## 3. JSON Logic 条件分岐（BR-BUS-020〜025）

### 3-1. JSON Logic 評価エンジン

- 採用ライブラリ: `json-logic-js`（Apache 2.0、npm）
- Rust 側: `json-logic-rust` crate（Apache 2.0）
- eval / Function constructor は禁止

### 3-2. 条件分岐ルール（TBL-030 step_flow_rules）

```typescript
// TBL-030 の condition 列の例
// "measured_value が USL 超過時は品質差し戻し Step に goto"
{
  "condition": {">": [{"var": "measured_value"}, {"var": "usl"}]},
  "action": "goto",
  "target_step_id": "STEP-QUALITY-HOLD"
}

// 評価（JSON Logic API）
const result = jsonLogic.apply(rule.condition, payload);
```

### 3-3. DAG 検証（循環参照防止）

TBL-030（step_flow_rules）の登録時に API-master-007（dry-run）で DAG 検証を実施する。

```
DAG 検証アルゴリズム（Rust / wnav_domain）:
1. step_flow_rules を隣接リストとして表現
2. 深さ優先探索（DFS）で後退辺を検出
3. 後退辺が存在する場合 → ERR-BIZ-007 を発生させ登録を拒否
```

---

**本節で確定した方針**
- **標準 4 タイプ（boolean_check/numeric_input/photo_capture/text_input）を変更・削除不可とし、拡張タイプを TBL-029 から動的読み込みするプラグイン機構を確定した。**
- **JSON Logic（Apache 2.0）のみを条件評価に使用し、eval / Function constructor の使用を禁止した（P7 原則の遵守）。**
- **DAG 検証（DFS アルゴリズム）をマスタ登録 API（API-master-007 dry-run）で実行し、循環参照による無限ループを設計レベルで防止する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/19_電子チェックリストと手順遵守の科学.md`](../../90_業界分析/19_電子チェックリストと手順遵守の科学.md)
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/12_認知工学と状況認識.md`](../../90_業界分析/12_認知工学と状況認識.md)
