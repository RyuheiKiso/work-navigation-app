# 03 SopFlowEditor 詳細設計

本章は MOD-FE-MA-003 SopFlowEditor の責務・TypeScript 型定義・Zustand ストア設計・アルゴリズム仕様・コンポーネント Props・react-query フック・アクセシビリティ要件・パフォーマンス要件を確定する。本章の実装によって FR-MA-016（Step-DAG ビジュアルフロー編集）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-003 |
| 物理名 | SopFlowEditor |
| ファイルパス | `src/features/sop-editor/flow/` |
| 関連 FR | FR-MA-016（主）、FR-MA-007（連携：DslConditionBuilder 呼出し） |
| 関連 SCR | SCR-MA-004（DAG フローモード） |
| アクセスロール | master_admin, quality_admin |
| 連携モジュール | MOD-FE-MA-001 SopEditor（Step 存在・並び順の単一権威）、MOD-FE-MA-002 DslConditionBuilder（エッジ条件式編集） |

**責務境界:**
- MOD-FE-MA-001 SopEditor: Step 単体の属性（instructionText / inputType / evidenceRequired 等）を管理する
- MOD-FE-MA-002 DslConditionBuilder: JSON Logic 条件式の構文ツリー編集を管理する
- **本モジュール MOD-FE-MA-003 SopFlowEditor**: Step 間の有向エッジ（skip / goto / insert）と DAG 全体のビジュアル編集・検証・シミュレーションを管理する

---

## 2. 状態定義（Zustand ストア）

### 2-1. 型定義

```typescript
import type { JsonLogicRule } from '../dsl/types';

// Step ノード（レイアウト座標はクライアント側のみ）
export interface FlowNode {
  stepId: string;          // TBL-008 steps.step_id
  stepNumber: number;      // 表示順序（SopEditor と同期）
  layoutPos: { x: number; y: number };  // 自動レイアウト計算結果
}

// エッジアクション
export type FlowEdgeAction =
  | { action: 'skip' }
  | { action: 'goto' }
  | { action: 'insert'; insertPosition: 'before_target' | 'after_source' };

// DAG エッジ（TBL-030 step_flow_rules の 1 行に対応）
export interface FlowEdge {
  edgeId: string;                          // UUID v7
  sourceStepId: string;
  targetStepId: string;
  actionType: FlowEdgeAction;
  condition: JsonLogicRule | null;         // null = 無条件実行
  priority: number;                        // 1〜100
  isOrphan?: boolean;                      // 参照先 Step 削除時に true
}

// シミュレーションコンテキスト
export interface SimContext {
  sampleValues: Record<string, unknown>;  // stepId → サンプル値
  executedPath: string[];                  // 実行パス stepId 列
  appliedEdgeIds: string[];                // 適用されたエッジ ID 列
}

// ストア状態
export interface SopFlowEditorState {
  sopId: string;
  versionId: string;
  nodes: FlowNode[];
  edges: FlowEdge[];
  selectedEdgeId: string | null;
  simulationContext: SimContext | null;
  mode: 'compose' | 'simulate' | 'legend';
  isDirty: boolean;
  lastAutoSaveAt: Date | null;
}

// ストアアクション（FNC-FE-017）
export interface SopFlowEditorActions {
  addEdge(source: string, target: string, action: FlowEdgeAction): void;
  removeEdge(edgeId: string): void;
  updateEdgeCondition(edgeId: string, rule: JsonLogicRule | null): void;
  updateEdgePriority(edgeId: string, priority: number): void;
  reorderNodes(orderedStepIds: string[]): void;  // SopEditor.reorderSteps と同期
  setMode(mode: SopFlowEditorState['mode']): void;
  enterSimulation(sampleValues: Record<string, unknown>): void;
  clearSimulation(): void;
  markClean(): void;
}
```

### 2-2. ストア実装方針

- ストアは SopEditor の `useSopEditorStore` と共通の `EditorTimeMachine`（50 ステップ undo/redo、FNC-FE-002 の拡張）を使用する
- `nodes` は `useSopEditorStore.steps` のセレクタから派生させる（SopEditor が単一権威）
- `edges` は本ストアが所有し、TBL-030 `step_flow_rules` に永続化する
- Auto-Save は FNC-FE-002 `useAutoSave` の追加 subscriber として 30 秒デバウンスで登録する

---

## 3. コンポーネント Props 定義

```typescript
// SopFlowEditor（ルートコンポーネント）
export interface SopFlowEditorProps {
  sopId: string;
  versionId: string;
  readOnly?: boolean;
  onChange: (edges: FlowEdge[]) => void;
}

// SopFlowCanvas（CMP-MA-005）
export interface SopFlowCanvasProps {
  nodes: FlowNode[];
  edges: FlowEdge[];
  selectedEdgeId: string | null;
  cycleEdgeIds: string[];          // ERR-VAL-024 対象エッジ（赤強調）
  orphanEdgeIds: string[];         // ERR-VAL-026 対象エッジ（グレーアウト）
  readOnly?: boolean;
  onAddEdge: (source: string, target: string) => void;
  onSelectEdge: (edgeId: string) => void;
  onReorderNodes: (orderedStepIds: string[]) => void;
}

// EdgeConditionPopover（DslConditionBuilder を内包）
export interface EdgeConditionPopoverProps {
  edgeId: string;
  initialCondition: JsonLogicRule | null;
  sourceStep: StepSummary;          // availableVariables の算出元
  onSave: (edgeId: string, rule: JsonLogicRule | null) => void;
  onCancel: () => void;
}

// FlowSimulationPanel
export interface FlowSimulationPanelProps {
  nodes: FlowNode[];
  edges: FlowEdge[];
  onEnterSimulation: (sampleValues: Record<string, unknown>) => void;
  onClearSimulation: () => void;
  simulationContext: SimContext | null;
}
```

---

## 4. react-query フック定義

```typescript
// フロールール取得（FNC-FE-017 の react-query 部分）
export declare function useStepFlowRules(
  sopId: string,
  versionId: string
): UseQueryResult<FlowEdge[]>;

// フロールール保存 Mutation
export declare function useSaveStepFlowRulesMutation(
  sopId: string,
  versionId: string
): UseMutationResult<void, Error, FlowEdge[]>;
// onMutate で FNC-FE-018 validateDagAcyclic をクライアント側プレフライトとして実行し、
// 循環参照が検出された場合は mutation を発行しない（ERR-VAL-024）。
// サーバ側 API-master-007 dry-run でも同型の検証を行い、二重バリアとする。
```

---

## 5. DAG 検証アルゴリズム（FNC-FE-018 validateDagAcyclic）

サーバ側 DAG DFS 実装（`04_概要設計/02_ソフトウェア方式設計/04_拡張Stepエンジン設計（プラグイン機構）.md` §3-3）と同型の白/灰/黒色付け深さ優先探索をクライアントで実施する。

```typescript
export type DagColor = 'white' | 'gray' | 'black';

export function validateDagAcyclic(
  nodes: FlowNode[],
  edges: FlowEdge[]
): { isAcyclic: boolean; cyclePath: string[] | null } {
  const color = new Map<string, DagColor>(
    nodes.map(n => [n.stepId, 'white'])
  );
  const cyclePath: string[] = [];

  function dfs(nodeId: string, path: string[]): boolean {
    color.set(nodeId, 'gray');
    const outEdges = edges.filter(e => e.sourceStepId === nodeId && !e.isOrphan);
    for (const edge of outEdges) {
      const c = color.get(edge.targetStepId);
      if (c === 'gray') {
        cyclePath.push(...path, nodeId, edge.targetStepId);
        return false;  // 循環検出
      }
      if (c === 'white') {
        if (!dfs(edge.targetStepId, [...path, nodeId])) return false;
      }
    }
    color.set(nodeId, 'black');
    return true;
  }

  for (const node of nodes) {
    if (color.get(node.stepId) === 'white') {
      if (!dfs(node.stepId, [])) {
        return { isAcyclic: false, cyclePath };
      }
    }
  }
  return { isAcyclic: true, cyclePath: null };
}
```

サイクル検出時は `SopFlowCanvas` の対象エッジを `ERR-VAL-024` として赤強調し、保存ボタンを disabled にする。

---

## 6. 自動レイアウト（FNC-FE-021 useStepFlowAutoLayout）

Sugiyama 法による階層レイアウトを採用する。決定論的アルゴリズムとすることで `computeStepFlowDiff`（FNC-FE-020）の diff 安定性を保証する。

- レイヤー割当: トポロジカルソートに基づく rank 計算
- 頂点順序: 重心法（Barycentric heuristic）によるエッジ交差最小化
- 座標割当: 最小エッジ長制約を満たす整数座標

処理は Web Worker で実行し、メインスレッドの 16ms フレーム制約を遵守する。ノード数 200（CFG-016）での完了時間を 100ms 以内に収める。

```typescript
export declare function useStepFlowAutoLayout(
  nodes: FlowNode[],
  edges: FlowEdge[]
): {
  layoutedNodes: FlowNode[];
  isComputing: boolean;
};
```

---

## 7. シミュレーション（FNC-FE-019 useFlowSimulation）

サンプル値を入力として DAG のフロー全体を実行し、どのエッジが適用されるかをトレースする。

```typescript
export declare function useFlowSimulation(
  nodes: FlowNode[],
  edges: FlowEdge[],
  sampleValues: Record<string, unknown>
): SimContext;
```

**アルゴリズム:**
1. 最初の Step（stepNumber 最小）からスタートする
2. 現在の Step の全 outgoing エッジを priority 昇順でソートする
3. 各エッジの condition を `evaluateJsonLogic`（FNC-FE-004 / MOD-FE-MA-002）で評価する
4. 最初に condition が true になったエッジのアクション（skip / goto / insert）を適用する
5. 対象 Step に移動し、2 に戻る
6. 全 Step を消化するか、無限ループ防止のため最大 steps.length × 2 ステップで打ち切る

UI は各 Step へのホップを 200ms のアニメーションで表現する（reduce-motion 設定時は即時遷移）。

---

## 8. 差分計算（FNC-FE-020 computeStepFlowDiff）

SOP バージョン間のエッジ集合差分を計算し、`VersionDiffViewer`（MOD-FE-MA-005）の右ペインに表示する。

```typescript
export interface FlowEdgeDiff {
  added: FlowEdge[];
  removed: FlowEdge[];
  conditionChanged: Array<{
    edgeId: string;
    before: JsonLogicRule | null;
    after: JsonLogicRule | null;
  }>;
  priorityChanged: Array<{
    edgeId: string;
    before: number;
    after: number;
  }>;
}

export declare function computeStepFlowDiff(
  before: FlowEdge[],
  after: FlowEdge[]
): FlowEdgeDiff;
```

安定エッジキー `(sourceStepId, targetStepId, actionType.action)` の LCS で照合する。条件式差分は JSON Patch 形式で表現する。

---

## 9. シリアライズ（FNC-FE-022 serializeFlowToStepFlowRules）

```typescript
export declare function serializeFlowToStepFlowRules(
  edges: FlowEdge[]
): StepFlowRuleRow[];

export interface StepFlowRuleRow {
  rule_id: string;           // edgeId（UUID v7）
  source_step_id: string;
  target_step_id: string;
  condition: JsonLogicRule | null;
  action: 'skip' | 'goto' | 'insert';
  insert_position: 'before_target' | 'after_source' | null;
  priority: number;
}
// 1 FlowEdge = 1 StepFlowRuleRow（TBL-030 の 1 行）
// 新規エッジの rule_id は MOD-SH-002 の UUID v7 生成ユーティリティで発番する
```

---

## 10. SopEditor（MOD-FE-MA-001）との連携

- `useSopEditorStore.steps` が Step の存在・番号順序の単一権威である
- `SopFlowEditor` は `useSopEditorStore` をセレクタで購読し、`FlowNode` 配列を派生させる
- SopEditor で Step が追加/削除された場合:
  - 追加: 対応 FlowNode を自動生成し、孤立しないよう自動レイアウトを再実行する
  - 削除: 参照エッジを `isOrphan: true` でマークし、ERR-VAL-026 としてグレーアウト表示する（自動削除しない）
- DAG キャンバスでの Step ドラッグ並べ替えは `useSopEditorStore.reorderSteps` を呼び出して `stepNumber` を同期する

---

## 11. DslConditionBuilder（MOD-FE-MA-002）との連携

- `EdgeConditionPopover` は `<DslConditionBuilder value={edge.condition} onChange={...} availableVariables={vars} />` として DslConditionBuilder をマウントする
- `availableVariables` は source Step の `inputType` と `judgmentCondition`（USL / LSL / unit）から算出する
- MOD-FE-MA-002 が定義するホワイトリスト演算子（ALLOWED_OPERATORS）と MAX_RULE_DEPTH=5 の制約は DAG エッジ条件式にも適用する

---

## 12. Auto-Save & Undo/Redo

- Auto-Save: FNC-FE-002 `useAutoSave` の追加 subscriber として本モジュールを登録する。30 秒デバウンスで `useSaveStepFlowRulesMutation` を発火する
- Undo/Redo: SopEditor と共有する `EditorTimeMachine` が 50 ステップの複合スナップショット `{ steps, edges }` を管理する。単一タイムマシンとすることで、Step 削除 → エッジ孤立 → Undo の一貫性を保証する

---

## 13. エラーハンドリング

| ERR コード | 条件 | UI 対応 |
|---|---|---|
| ERR-VAL-024 | DAG 循環参照検出（validateDagAcyclic が false を返す） | 循環エッジを赤強調、保存ボタン disabled、バナー「循環参照を検出しました。エッジを削除してください。」 |
| ERR-VAL-025 | エッジ条件式が空（action: skip 以外で condition = null） | EdgeConditionPopover 内の保存ボタンを disabled、赤バナー「条件式を入力してください。」 |
| ERR-VAL-026 | 削除済 Step を参照する孤立エッジ | 孤立エッジをグレーアウト、各エッジにワンクリック削除ボタンを表示 |
| ERR-BIZ-013 | DAG 保存時に Draft 状態でない | バナー「下書き状態でないため保存できません。」、保存ボタン disabled |

---

## 14. アクセシビリティ

- キャンバスは `role="application"` + `aria-roledescription="DAGエディタ"` とする
- **キーボード操作セット**:
  - Tab: ノード巡回
  - Enter / Space: ノード選択
  - 矢印キー: 選択ノードから隣接ノードへ移動
  - E: エッジ作画モードのトグル
  - Delete / Backspace: 選択エッジを削除
  - Escape: モード解除
- 各ノードには `aria-label={`Step ${step.stepNumber}: ${step.instructionText.ja}`}` を必須とする
- エッジ種別は色 + ダッシュパターン + アイコンの 3 つの手がかりで識別する（色のみに依存しない）:
  - skip: 灰色破線 + 二重矢印アイコン
  - goto: 青色実線 + 矢印アイコン
  - insert: 緑色実線 + ＋アイコン
- reduce-motion メディアクエリが有効な場合、シミュレーションアニメーションを即時遷移に置換する

---

## 15. パフォーマンス要件

| 要件 | 制約値 | 対応 |
|---|---|---|
| DAG ノード数上限 | 200（CFG-016） | 超過時は保存ボタン disabled + 警告バナー |
| DAG エッジ数上限 | 500（CFG-017） | 超過時は保存ボタン disabled + 警告バナー |
| 自動レイアウト完了時間 | ノード 200 で 100ms 以内 | Web Worker で実行しメインスレッドをブロックしない |
| シミュレーション応答時間 | 200ms 以内（クライアント完結） | evaluateJsonLogic の 1s タイムアウトを Step 数で按分 |
| ビューポートカリング | ノード 50 超で viewport 外 DOM を削除 | react-virtual または同等の仮想化 |

---

**本節で確定した方針**
- **状態分離**: SopEditor が Step 属性（単体属性・番号順序）の単一権威であり、SopFlowEditor はエッジ（DAG フロー）のみを所有する。両モジュールは 50 ステップ undo/redo（EditorTimeMachine）と Auto-Save トリガを共有する。
- **二重 DAG 検証**: クライアント側 validateDagAcyclic（FNC-FE-018）で保存前にプレフライト検証し、サーバ側 API-master-007 dry-run で最終確認する二重バリアを設けることで循環参照を確実に排除する。
- **自動レイアウトの決定論性**: Sugiyama 法の実装を決定論的とし、同一エッジ集合に対して常に同一座標を返すことで computeStepFlowDiff（FNC-FE-020）の diff 安定性を保証する。
- **条件式 DSL の委譲**: エッジ条件式の編集は MOD-FE-MA-002 DslConditionBuilder に全面委譲し、演算子ホワイトリスト・MAX_RULE_DEPTH 制約を継承する。SopFlowEditor は DAG トポロジの管理のみを担当する。

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)
- [`90_業界分析/19_電子チェックリストと手順遵守の科学.md`](../../90_業界分析/19_電子チェックリストと手順遵守の科学.md)

### 関連
- [`90_業界分析/29_競合製品と作業ナビ・MES・eBR市場.md`](../../90_業界分析/29_競合製品と作業ナビ・MES・eBR市場.md)
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
