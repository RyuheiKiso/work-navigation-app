// 対応 §: ロードマップ §3.4.1 §3.4.2 §10.2.1（バリデーション）§10.2.4
// HSM 形式検証（到達可能性／デッドロック検出）の TypeScript 実装。
// React Flow 上で「保存前に検証」できるよう、純粋関数として提供する。

import type { Flow, FlowEdge, FlowNode } from './flow';

/** 検証結果 */
export interface FlowValidationResult {
  // 不到達ノード（start から到達できない）
  readonly unreachable: ReadonlyArray<string>;
  // デッドロック（end ノードに到達できない start／step ノード）
  readonly deadlocked: ReadonlyArray<string>;
  // 無限ループ（self-loop または閉じた循環で end に到達できないもの）
  readonly cycles: ReadonlyArray<ReadonlyArray<string>>;
  // 不到達 end ノード（誰も指していない）
  readonly orphanedEnds: ReadonlyArray<string>;
  // 重複辺（同じ from→to が複数）
  readonly duplicateEdges: ReadonlyArray<readonly [string, string]>;
  // §3.6.4 アンチパターン: 同 from から条件無し辺が複数
  readonly nondeterministicChoices: ReadonlyArray<string>;
  // 検証通過？（全カテゴリ空）
  readonly valid: boolean;
}

/** Flow 全体を検証する */
export function validateFlow(flow: Flow): FlowValidationResult {
  // ノード／辺を取り出す
  const nodes = flow.nodes;
  const edges = flow.edges;
  // ノード ID → ノード
  const byId = new Map<string, FlowNode>(nodes.map((n) => [n.id, n]));
  // start ノード集合
  const starts = nodes.filter((n) => n.kind === 'start').map((n) => n.id);
  // end ノード集合
  const ends = nodes.filter((n) => n.kind === 'end').map((n) => n.id);

  // 隣接リスト（順方向／逆方向）
  const fwd = adjacency(edges, false);
  const rev = adjacency(edges, true);

  // 不到達: BFS from start union
  const reachableFromStart = bfsAll(starts, fwd);
  const unreachable = nodes
    .filter((n) => !reachableFromStart.has(n.id) && n.kind !== 'start')
    .map((n) => n.id);

  // 各 end に到達できないノード（BFS from ends backwards）
  const reachableToEnd = bfsAll(ends, rev);
  // step／start／decision／parallel から end に届かないものをデッドロックとする
  const deadlocked = nodes
    .filter((n) => !reachableToEnd.has(n.id) && n.kind !== 'end')
    .map((n) => n.id);

  // サイクル検出（DFS, white/gray/black）
  const cycles = findCycles(nodes, fwd);

  // 孤児 end（誰も指していない）
  const incoming = new Map<string, number>();
  for (const e of edges) {
    incoming.set(e.to, (incoming.get(e.to) ?? 0) + 1);
  }
  const orphanedEnds = ends.filter((id) => (incoming.get(id) ?? 0) === 0);

  // 重複辺
  const seen = new Set<string>();
  const duplicates: Array<readonly [string, string]> = [];
  for (const e of edges) {
    const key = `${e.from}->${e.to}`;
    if (seen.has(key)) duplicates.push([e.from, e.to] as const);
    seen.add(key);
  }

  // 非決定的分岐: decision 以外で条件無し辺が 2 本以上
  const nondeterministic: string[] = [];
  for (const n of nodes) {
    if (n.kind === 'decision' || n.kind === 'parallel') continue;
    const out = edges.filter((e) => e.from === n.id);
    const noCondCount = out.filter((e) => !e.condition).length;
    if (noCondCount >= 2) nondeterministic.push(n.id);
  }

  // 結果
  const result: FlowValidationResult = {
    unreachable,
    deadlocked,
    cycles,
    orphanedEnds,
    duplicateEdges: duplicates,
    nondeterministicChoices: nondeterministic,
    valid:
      unreachable.length === 0 &&
      deadlocked.length === 0 &&
      cycles.length === 0 &&
      orphanedEnds.length === 0 &&
      duplicates.length === 0 &&
      nondeterministic.length === 0
  };
  // byId をテストで検査できるよう全ノードを返却対象に含めない（純粋結果のみ）
  void byId;
  return result;
}

/** 隣接リストを構築する */
function adjacency(
  edges: ReadonlyArray<FlowEdge>,
  reverse: boolean
): ReadonlyMap<string, ReadonlyArray<string>> {
  const map = new Map<string, string[]>();
  for (const e of edges) {
    const from = reverse ? e.to : e.from;
    const to = reverse ? e.from : e.to;
    if (!map.has(from)) map.set(from, []);
    map.get(from)!.push(to);
  }
  return map;
}

/** 複数ソースからの BFS で到達可能集合を返す */
function bfsAll(
  sources: ReadonlyArray<string>,
  adj: ReadonlyMap<string, ReadonlyArray<string>>
): Set<string> {
  const seen = new Set<string>();
  const queue: string[] = [...sources];
  while (queue.length > 0) {
    const cur = queue.shift()!;
    if (seen.has(cur)) continue;
    seen.add(cur);
    for (const nx of adj.get(cur) ?? []) {
      if (!seen.has(nx)) queue.push(nx);
    }
  }
  return seen;
}

/** サイクル検出（Tarjan 風 DFS） */
function findCycles(
  nodes: ReadonlyArray<FlowNode>,
  adj: ReadonlyMap<string, ReadonlyArray<string>>
): ReadonlyArray<ReadonlyArray<string>> {
  const cycles: string[][] = [];
  const color = new Map<string, 'white' | 'gray' | 'black'>();
  for (const n of nodes) color.set(n.id, 'white');

  const path: string[] = [];

  function dfs(u: string): void {
    color.set(u, 'gray');
    path.push(u);
    for (const v of adj.get(u) ?? []) {
      const c = color.get(v);
      if (c === 'gray') {
        // バック辺 → サイクル
        const idx = path.indexOf(v);
        if (idx >= 0) {
          cycles.push(path.slice(idx));
        }
      } else if (c === 'white') {
        dfs(v);
      }
    }
    path.pop();
    color.set(u, 'black');
  }

  for (const n of nodes) {
    if (color.get(n.id) === 'white') dfs(n.id);
  }
  return cycles;
}
