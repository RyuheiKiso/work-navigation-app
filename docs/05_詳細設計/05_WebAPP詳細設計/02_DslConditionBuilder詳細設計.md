# 02 DslConditionBuilder 詳細設計

本章は MOD-FE-MA-002（DslConditionBuilder）の TypeScript インターフェース・JSON Logic AST 型定義・ホワイトリスト演算子一覧・DAG 検証ロジック・ビジュアルエディタ Props を確定する。DslConditionBuilder は FR-MA-004/007 で要求されるビジュアル DSL エディタであり、JSON Logic ルールを安全に構築するためのセキュリティ境界を担う。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-002 |
| 物理名 | DslConditionBuilder |
| ファイルパス | `src/features/sop-editor/dsl/` |
| 関連 FR | FR-MA-004（判定条件設定）・FR-MA-007（DSL 検証）|
| 関連 SCR | SCR-MA-006（SOP プレビュー・条件確認）・SCR-MA-004（DAG フローモードのエッジ条件式 popover として EdgeConditionPopover 内に mount される）|
| アクセスロール | master_admin |

---

## 2. JSON Logic AST 型定義

ホワイトリストに含まれる安全な演算子のみを型として定義する。`eval`・関数呼び出し・動的プロパティアクセスは型レベルで表現不可能な構造とする。

```typescript
// ホワイトリスト済み変数参照（var 演算子）
export type JsonLogicVar = { var: string };

// プリミティブ値（変数または定数）
export type JsonLogicValue = JsonLogicVar | number | string | boolean | null;

// 比較演算子（2 項）
export type JsonLogicComparison =
  | { '==': [JsonLogicValue, JsonLogicValue] }
  | { '!=': [JsonLogicValue, JsonLogicValue] }
  | { '>':  [JsonLogicValue, JsonLogicValue] }
  | { '<':  [JsonLogicValue, JsonLogicValue] }
  | { '>=': [JsonLogicValue, JsonLogicValue] }
  | { '<=': [JsonLogicValue, JsonLogicValue] };

// 論理演算子
export type JsonLogicLogical =
  | { and: JsonLogicRule[] }
  | { or:  JsonLogicRule[] }
  | { '!': JsonLogicRule };

// ルートノード（再帰型）
export type JsonLogicRule =
  | JsonLogicComparison
  | JsonLogicLogical;

// 使用可能変数定義
export interface StepVariable {
  /** 変数名（var 演算子の値として使用）*/
  name: string;
  /** 表示ラベル（多言語）*/
  label: import('../types').MultilingualText;
  /** 変数の型（バリデーション用）*/
  valueType: 'number' | 'string' | 'boolean';
  /** 単位（例: 'mm', '°C'）*/
  unit?: string;
}
```

---

## 3. コンポーネント Props 定義

```typescript
// DslConditionBuilder コンポーネント（SCR-MA-006 内に埋め込み）
export interface DslConditionBuilderProps {
  /** 現在のルール（null = 条件なし）*/
  value: JsonLogicRule | null;
  /** ルール変更コールバック */
  onChange: (rule: JsonLogicRule | null) => void;
  /** ビジュアルエディタで選択可能な測定変数一覧 */
  availableVariables: StepVariable[];
  /** quality_admin レビュー時は true（変更不可）*/
  readOnly?: boolean;
}
```

---

## 4. ホワイトリスト検証（FNC-FE-005）

```typescript
/** ホワイトリスト演算子集合 */
const ALLOWED_OPERATORS = new Set([
  '==', '!=', '>', '<', '>=', '<=',
  'and', 'or', '!',
  'var',
] as const);

/**
 * FNC-FE-005: JSON Logic AST がホワイトリスト演算子のみで構成されているか検証する
 *
 * @param rule - 検証対象のルール
 * @param allowedVarNames - 許可変数名セット（StepVariable.name の集合）
 * @returns バリデーション結果
 *
 * 禁止事項:
 * - eval / Function / dynamic access などの演算子
 * - allowedVarNames に含まれない変数名
 * - ネスト深さ > MAX_DEPTH（= 10）
 */
export declare function validateDslAst(
  rule: JsonLogicRule,
  allowedVarNames: ReadonlySet<string>,
): DslValidationResult;

export interface DslValidationResult {
  isValid: boolean;
  errors: DslValidationError[];
}

export interface DslValidationError {
  path: string;          // JSONPath 形式（例: '$.and[0].>'）
  code: DslErrorCode;
  message: string;
}

export type DslErrorCode =
  | 'UNKNOWN_OPERATOR'     // ホワイトリスト外の演算子
  | 'UNKNOWN_VARIABLE'     // allowedVarNames 外の変数参照
  | 'MAX_DEPTH_EXCEEDED'   // ネスト深さ超過
  | 'EMPTY_AND_OR'         // and/or の配列が空
  | 'TYPE_MISMATCH';       // 変数型と比較値の型不一致

/** AST の最大ネスト深さ */
const MAX_DEPTH = 10 as const;
```

---

## 5. JSON Logic 評価（FNC-FE-004）

```typescript
/**
 * FNC-FE-004: ホワイトリスト検証済みの JSON Logic ルールをデータに適用して評価する
 *
 * @param rule - validateDslAst で isValid === true を確認済みのルール
 * @param data - 変数名 → 値のマッピング
 * @returns 評価結果（true/false）
 *
 * 制約:
 * - eval・Function コンストラクタは使用しない
 * - 再帰的ツリー走査で純粋に評価する
 */
export declare function evaluateJsonLogic(
  rule: JsonLogicRule,
  data: Readonly<Record<string, number | string | boolean | null>>,
): boolean;
```

---

## 6. ビジュアルエディタ内部コンポーネントツリー

```
DslConditionBuilder (MOD-FE-MA-002)
  ConditionGroupNode (and/or グループ)
    ConditionLeafNode (比較式 1 件)
      VariableSelector (availableVariables から選択)
      OperatorSelector ('==' | '!=' | '>' | '<' | '>=' | '<=')
      ValueInput (valueType に応じて number/text/boolean 入力)
    AddLeafButton
    AddGroupButton
    RemoveNodeButton
  ClearButton
  JsonPreviewPanel (現在の AST を JSON として表示・コピー可）
```

---

## 7. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-020 | UNKNOWN_OPERATOR 検出 | 該当ノードを赤枠表示・保存ブロック |
| ERR-VAL-021 | UNKNOWN_VARIABLE 検出 | 変数名をハイライト・保存ブロック |
| ERR-VAL-022 | MAX_DEPTH_EXCEEDED | 追加ボタン非活性・ツールチップ表示 |
| ERR-VAL-023 | EMPTY_AND_OR | and/or ノードに「最低 1 件追加」バナー |

---

## 8. SopFlowEditor（MOD-FE-MA-003）との境界

本モジュール DslConditionBuilder は **JSON Logic 条件式（単一の boolean 式ツリー）の編集のみ** を担当する。

- **対象内**: 演算子パレット・変数ピッカー・条件ツリー表示・RAW JSON モード・プレビュー実行
- **対象外**: Step 間の DAG トポロジ（skip / goto / insert エッジの追加・削除・並べ替え） → MOD-FE-MA-003 が担当する

SopFlowEditor の EdgeConditionPopover は `<DslConditionBuilder value={edge.condition} onChange={...} availableVariables={vars} />` として本コンポーネントをマウントする。`availableVariables` は呼び出し側（EdgeConditionPopover）が source Step の inputType / USL / LSL から組み立てる。

演算子ホワイトリスト（ALLOWED_OPERATORS）と MAX_RULE_DEPTH=5 の制約は、DAG エッジ条件式に対しても適用する。

---

**本節で確定した方針**
- **JSON Logic の演算子ホワイトリストを `'=='・'!='・'>'・'<'・'>='・'<='・'and'・'or'・'!'・'var'` の 10 種に限定し、型定義・validateDslAst の両層でブロックすることを確定した。**
- **eval・Function コンストラクタ・動的プロパティアクセス（`[]` 演算子）は型レベルで表現不可能とし、evaluateJsonLogic は純粋な再帰ツリー走査のみで実装することを確定した。**
- **AST ネスト深さ上限を 10 とし、MAX_DEPTH_EXCEEDED 時は追加ボタン非活性にして UI からそれ以上の深さへの到達を防止することを確定した。**
- **DslConditionBuilder は条件式 DSL 単体の編集に責務を限定し、Step-DAG フロー編集は MOD-FE-MA-003 SopFlowEditor に分離することを確定した（FR-MA-016）。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
