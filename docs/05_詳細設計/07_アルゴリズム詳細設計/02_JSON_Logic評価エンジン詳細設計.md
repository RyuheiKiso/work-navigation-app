# 02 JSON Logic 評価エンジン詳細設計

本章は SOP ステップの条件分岐判定に使用する JSON Logic 安全評価エンジン（ALG-004/005, FNC-FE-005〜008）の完全仕様を確定する。`eval()` および動的コード実行を一切使用しない安全なホワイトリスト評価・DAG 深度検証・タイムアウトガードを定義する。対応する業務ルールは BR-BUS-022（DSL ネスト深度 5 以内）・BR-BUS-024（評価タイムアウト 1 秒）である。

---

## 1. 許可オペレータ・ホワイトリスト（ALG-004）

JSON Logic 評価エンジンは以下のオペレータのみを許可する。ホワイトリストに存在しないオペレータは `ERR-VAL-003: Forbidden operator` を送出し処理を中断する。

```typescript
// 許可オペレータの完全リスト
const ALLOWED_OPERATORS = new Set<string>([
  // 比較演算子
  '==', '!=', '<', '>', '<=', '>=',
  // 論理演算子
  'and', 'or', '!',
  // 文字列・配列
  'in', 'cat',
  // 変数参照（コンテキストから取得。任意の JS アクセスは不可）
  'var',
  // 制御フロー
  'if',
  // 算術（4 則のみ）
  '+', '-', '*', '/',
  // 集計
  'min', 'max',
]);
```

### 1-1. 禁止オペレータの根拠

| 禁止カテゴリ | 例 | 禁止理由 |
|---|---|---|
| 動的評価 | `eval`, `function` | 任意コード実行に直結（BR-BUS-022 明示禁止）|
| ファイルアクセス | `missing`, `missing_some`（独自拡張版）| 予期しないコンテキスト探索 |
| 高階関数 | `filter`, `map`, `reduce`, `all`, `some`, `none` | 反復処理で無限ループのリスク |
| 型変換 | `!!`, `%` | 実装済み算術演算子で代替可能であり、副作用を避けるため除外 |

---

## 2. evaluateRule アルゴリズム（ALG-004, FNC-FE-005）

```typescript
type JsonLogicPrimitive = string | number | boolean | null;
type JsonLogicRule = JsonLogicPrimitive | { [operator: string]: JsonLogicRule[] };

/**
 * FNC-FE-005: JSON Logic ルール評価（eval() 不使用）
 *
 * @throws DslEvaluationError  - 禁止オペレータ使用時 (ERR-VAL-003)
 * @throws DslEvaluationError  - ゼロ除算時 (ERR-VAL-003)
 * @throws DslEvaluationError  - 型不一致時 (ERR-VAL-003)
 */
function evaluateRule(
  rule: JsonLogicRule,
  context: Record<string, unknown>,
): unknown {
  // プリミティブ値はそのまま返す
  if (typeof rule !== 'object' || rule === null) {
    return rule;
  }

  const entries = Object.entries(rule);
  if (entries.length !== 1) {
    throw new DslEvaluationError('ERR-VAL-003: Rule must have exactly one operator key');
  }

  const [operator, rawArgs] = entries[0];
  const args = Array.isArray(rawArgs) ? rawArgs : [rawArgs];

  if (!ALLOWED_OPERATORS.has(operator)) {
    throw new DslEvaluationError(`ERR-VAL-003: Forbidden operator: ${operator}`);
  }

  switch (operator) {
    // 変数参照
    case 'var':
      return resolveVar(args[0] as string, context);

    // 比較演算子
    case '==':
      return evaluateRule(args[0], context) === evaluateRule(args[1], context);
    case '!=':
      return evaluateRule(args[0], context) !== evaluateRule(args[1], context);
    case '<':
      return (evaluateRule(args[0], context) as number) < (evaluateRule(args[1], context) as number);
    case '>':
      return (evaluateRule(args[0], context) as number) > (evaluateRule(args[1], context) as number);
    case '<=':
      return (evaluateRule(args[0], context) as number) <= (evaluateRule(args[1], context) as number);
    case '>=':
      return (evaluateRule(args[0], context) as number) >= (evaluateRule(args[1], context) as number);

    // 論理演算子
    case 'and':
      return args.every(a => !!evaluateRule(a, context));
    case 'or':
      return args.some(a => !!evaluateRule(a, context));
    case '!':
      return !evaluateRule(args[0], context);

    // 文字列・配列
    case 'in': {
      const needle = evaluateRule(args[0], context);
      const haystack = evaluateRule(args[1], context);
      if (Array.isArray(haystack)) return haystack.includes(needle);
      if (typeof haystack === 'string') return haystack.includes(String(needle));
      throw new DslEvaluationError('ERR-VAL-003: "in" requires array or string as second argument');
    }
    case 'cat':
      return args.map(a => String(evaluateRule(a, context))).join('');

    // 制御フロー
    case 'if': {
      // args: [condition, trueBranch, falseBranch?, ...]
      for (let i = 0; i < args.length - 1; i += 2) {
        if (!!evaluateRule(args[i], context)) {
          return evaluateRule(args[i + 1], context);
        }
      }
      return args.length % 2 === 1
        ? evaluateRule(args[args.length - 1], context)
        : null;
    }

    // 算術
    case '+': {
      const vals = args.map(a => evaluateRule(a, context) as number);
      return vals.reduce((acc, v) => acc + v, 0);
    }
    case '-':
      return (evaluateRule(args[0], context) as number) - (evaluateRule(args[1], context) as number);
    case '*':
      return (evaluateRule(args[0], context) as number) * (evaluateRule(args[1], context) as number);
    case '/': {
      const divisor = evaluateRule(args[1], context) as number;
      if (divisor === 0) throw new DslEvaluationError('ERR-VAL-003: Division by zero');
      return (evaluateRule(args[0], context) as number) / divisor;
    }

    // 集計
    case 'min':
      return Math.min(...args.map(a => evaluateRule(a, context) as number));
    case 'max':
      return Math.max(...args.map(a => evaluateRule(a, context) as number));

    default:
      // ALLOWED_OPERATORS チェック後のため到達しない
      throw new DslEvaluationError(`ERR-VAL-003: Unreachable operator: ${operator}`);
  }
}
```

---

## 3. resolveVar アルゴリズム（FNC-FE-007）

`var` オペレータは評価コンテキストの許可キーからのみ値を取得する。任意の JavaScript プロパティチェーン（プロトタイプ汚染につながる `__proto__` 等）へのアクセスを禁止する。

```typescript
// 評価コンテキストの許可キー一覧（コンテキスト変数ホワイトリスト）
const ALLOWED_CONTEXT_KEYS = new Set<string>([
  'step.input',
  'step.input.value',
  'step.input.unit',
  'execution.worker_id',
  'execution.started_at',
  'execution.elapsed_seconds',
  'sop.version',
  'env.current_time',
  'env.is_emergency_mode',
]);

/**
 * FNC-FE-007: コンテキスト変数の安全解決
 */
function resolveVar(path: string, context: Record<string, unknown>): unknown {
  if (path === '' || path === null) return context;

  if (!ALLOWED_CONTEXT_KEYS.has(path)) {
    throw new DslEvaluationError(
      `ERR-VAL-003: Forbidden context variable: ${path}`,
    );
  }

  // ドット記法でのネストアクセス（プロトタイプチェーン除外）
  const parts = path.split('.');
  let current: unknown = context;
  for (const part of parts) {
    if (
      current === null ||
      current === undefined ||
      typeof current !== 'object' ||
      !Object.prototype.hasOwnProperty.call(current, part)
    ) {
      return null;  // 存在しないキーは null を返す
    }
    current = (current as Record<string, unknown>)[part];
  }
  return current;
}
```

---

## 4. DAG 深度検証アルゴリズム（ALG-005, FNC-FE-006）

SOP 保存時にルール定義の妥当性を静的検証する。ネスト深度が 5 を超えるルールは BR-BUS-022 違反として拒否する。

```typescript
const MAX_RULE_DEPTH = 5; // BR-BUS-022

/**
 * FNC-FE-006: ルール深度検証（SOP 保存時に静的実行）
 *
 * @throws DslValidationError  ネスト深度 5 超過 (ERR-VAL-003)
 */
function validateRuleDepth(rule: JsonLogicRule, depth: number = 0): void {
  if (depth > MAX_RULE_DEPTH) {
    throw new DslValidationError(
      `ERR-VAL-003: Max nesting depth ${MAX_RULE_DEPTH} exceeded (BR-BUS-022)`,
    );
  }

  if (typeof rule !== 'object' || rule === null) {
    return; // プリミティブ：深度カウント不要
  }

  const entries = Object.entries(rule);
  if (entries.length !== 1) {
    throw new DslValidationError('ERR-VAL-003: Rule must have exactly one operator key');
  }

  const [operator, rawArgs] = entries[0];
  if (!ALLOWED_OPERATORS.has(operator)) {
    throw new DslValidationError(`ERR-VAL-003: Forbidden operator: ${operator}`);
  }

  const args = Array.isArray(rawArgs) ? rawArgs : [rawArgs];
  for (const arg of args) {
    validateRuleDepth(arg, depth + 1);
  }
}

/**
 * 循環参照（DAG 違反）検出
 * JSON シリアライズで循環を検出し、エラーを送出する。
 */
function assertAcyclic(rule: JsonLogicRule): void {
  try {
    JSON.stringify(rule);
  } catch {
    throw new DslValidationError('ERR-VAL-003: Circular reference detected in rule (BR-BUS-022)');
  }
}
```

---

## 5. タイムアウトガード（FNC-FE-008）

評価は BR-BUS-024 により 1 秒以内に完了しなければならない。`Promise.race` と `AbortController` を使用してタイムアウトを強制する。

```typescript
const EVALUATION_TIMEOUT_MS = 1000; // BR-BUS-024

/**
 * FNC-FE-008: タイムアウトガード付き評価
 *
 * @throws DslTimeoutError  評価が 1 秒以内に完了しない場合 (ERR-VAL-003)
 */
async function evaluateWithTimeout(
  rule: JsonLogicRule,
  context: Record<string, unknown>,
): Promise<unknown> {
  const controller = new AbortController();

  const timeoutPromise = new Promise<never>((_, reject) => {
    const id = setTimeout(() => {
      controller.abort();
      reject(new DslTimeoutError(
        `ERR-VAL-003: Rule evaluation exceeded ${EVALUATION_TIMEOUT_MS}ms (BR-BUS-024)`,
      ));
    }, EVALUATION_TIMEOUT_MS);

    // GC 防止のためタイマー ID を保持
    return id;
  });

  const evaluationPromise = Promise.resolve(evaluateRule(rule, context));

  return Promise.race([evaluationPromise, timeoutPromise]);
}
```

---

## 6. エラー型定義

```typescript
class DslEvaluationError extends Error {
  readonly code = 'ERR-VAL-003';
  constructor(message: string) {
    super(message);
    this.name = 'DslEvaluationError';
  }
}

class DslValidationError extends Error {
  readonly code = 'ERR-VAL-003';
  constructor(message: string) {
    super(message);
    this.name = 'DslValidationError';
  }
}

class DslTimeoutError extends Error {
  readonly code = 'ERR-VAL-003';
  constructor(message: string) {
    super(message);
    this.name = 'DslTimeoutError';
  }
}
```

---

## 7. 評価フロー全体

SOP 実行時の JSON Logic 評価は以下の順序で実行する。

```
[SOP 保存時（静的検証）]
  1. assertAcyclic(rule)           -- 循環参照チェック
  2. validateRuleDepth(rule, 0)    -- 深度 5 以内チェック (BR-BUS-022)

[ステップ実行時（動的評価）]
  1. コンテキスト組み立て（step.input, execution.*, env.*）
  2. evaluateWithTimeout(rule, context)  -- 1 秒タイムアウト (BR-BUS-024)
     └─ evaluateRule(rule, context)
           └─ resolveVar(path, context)  -- ホワイトリスト変数のみ
  3. 結果が true → 条件分岐先ステップへ
     結果が false → 次ステップへ（通常進行）
```

---

**本節で確定した方針**
- **JSON Logic 評価エンジンは `eval()` および動的コード生成を一切使用せず、switch-case による完全静的ディスパッチで実装することを確定した。許可オペレータを 16 種に限定し、未知オペレータは即座に ERR-VAL-003 を送出する。**
- **DAG 深度検証（validateRuleDepth）はネスト深度 5 超過を静的に検出して BR-BUS-022 を強制し、SOP 保存時に必ず実行することを確定した。**
- **タイムアウトガード（evaluateWithTimeout）は Promise.race と AbortController で 1 秒の上限を強制し、BR-BUS-024 を実装することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)

### 関連
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)
