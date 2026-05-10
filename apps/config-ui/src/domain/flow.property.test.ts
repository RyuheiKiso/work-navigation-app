// 対応 §: ロードマップ §13.1 §10.2.1 ／ ルート CLAUDE.md（不変条件は型または property test で守る）
// Flow Aggregate の不変条件を fast-check で網羅する。
// - ノード ID の一意性
// - 開始ノードの存在
// - 辺の参照整合性

import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import { Flow, type FlowEdge, type FlowNode } from './flow';

// ASCII 識別子の任意モデル（重複を生み出しやすくするため短い）
const idArb = fc.stringMatching(/^[a-z][a-z0-9]{0,4}$/);

const startNodeArb: fc.Arbitrary<FlowNode> = fc.record({
  id: idArb,
  kind: fc.constant<'start'>('start'),
  label: fc.string({ minLength: 1, maxLength: 20 })
});

const stepNodeArb: fc.Arbitrary<FlowNode> = fc.record({
  id: idArb,
  kind: fc.constantFrom<'step' | 'decision' | 'parallel' | 'end'>(
    'step', 'decision', 'parallel', 'end'
  ),
  label: fc.string({ minLength: 1, maxLength: 20 })
});

describe('Flow invariants (property-based)', () => {
  it('rejects any node array without a start node', () => {
    fc.assert(
      fc.property(fc.array(stepNodeArb, { minLength: 1, maxLength: 8 }), (nodes) => {
        const dedup = uniqueById(nodes);
        if (dedup.some((n) => n.kind === 'start')) return; // start を含む場合はスキップ
        expect(() => Flow.create('f', 'F', dedup, [])).toThrow(/開始ノード/);
      })
    );
  });

  it('rejects any node array with duplicate ids', () => {
    fc.assert(
      fc.property(idArb, (sharedId) => {
        const nodes: FlowNode[] = [
          { id: sharedId, kind: 'start', label: 'A' },
          { id: sharedId, kind: 'step', label: 'B' }
        ];
        expect(() => Flow.create('f', 'F', nodes, [])).toThrow(/重複/);
      })
    );
  });

  it('rejects edges that point to a non-existent node', () => {
    fc.assert(
      fc.property(idArb, idArb, idArb, (startId, otherId, missingId) => {
        // start と other は存在し、辺は missing を指す
        if (startId === missingId || otherId === missingId || startId === otherId) return;
        const nodes: FlowNode[] = [
          { id: startId, kind: 'start', label: 'S' },
          { id: otherId, kind: 'step', label: 'O' }
        ];
        const edges: FlowEdge[] = [{ from: startId, to: missingId }];
        expect(() => Flow.create('f', 'F', nodes, edges)).toThrow(/to ノード/);
      })
    );
  });

  it('accepts well-formed flows for any valid arrangement', () => {
    fc.assert(
      fc.property(
        fc.array(stepNodeArb, { minLength: 0, maxLength: 6 }),
        idArb,
        (steps, startId) => {
          const dedupSteps = uniqueById(steps).filter((s) => s.id !== startId);
          const nodes: FlowNode[] = [
            { id: startId, kind: 'start', label: 'start' },
            ...dedupSteps
          ];
          // 全辺を start → 各 step とする（有効な参照のみ）
          const edges: FlowEdge[] = dedupSteps.map((s) => ({ from: startId, to: s.id }));
          const flow = Flow.create('f', 'F', nodes, edges);
          expect(flow.nodeCount).toBe(nodes.length);
          expect(flow.edgeCount).toBe(edges.length);
        }
      )
    );
  });
});

function uniqueById(nodes: FlowNode[]): FlowNode[] {
  const seen = new Set<string>();
  const out: FlowNode[] = [];
  for (const n of nodes) {
    if (seen.has(n.id)) continue;
    seen.add(n.id);
    out.push(n);
  }
  return out;
}
