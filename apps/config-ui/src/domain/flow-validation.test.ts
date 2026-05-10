// 対応 §: ロードマップ §3.4.1 §3.4.2 §13.1 §10.2.4
// HSM 形式検証の単体テスト。

import { describe, it, expect } from 'vitest';
import { Flow, type FlowEdge, type FlowNode } from './flow';
import { validateFlow } from './flow-validation';

function build(nodes: FlowNode[], edges: FlowEdge[]): Flow {
  return Flow.create('f', 'test', nodes, edges, undefined, 1);
}

describe('validateFlow', () => {
  it('passes valid flow', () => {
    const f = build(
      [
        { id: 'start', kind: 'start', label: 'S' },
        { id: 'step', kind: 'step', label: 'A' },
        { id: 'end', kind: 'end', label: 'E' }
      ],
      [
        { from: 'start', to: 'step' },
        { from: 'step', to: 'end' }
      ]
    );
    const r = validateFlow(f);
    expect(r.valid).toBe(true);
  });

  it('detects unreachable node', () => {
    const f = build(
      [
        { id: 'start', kind: 'start', label: 'S' },
        { id: 'lonely', kind: 'step', label: 'L' },
        { id: 'end', kind: 'end', label: 'E' }
      ],
      [{ from: 'start', to: 'end' }]
    );
    const r = validateFlow(f);
    expect(r.unreachable).toContain('lonely');
    expect(r.valid).toBe(false);
  });

  it('detects deadlock', () => {
    const f = build(
      [
        { id: 'start', kind: 'start', label: 'S' },
        { id: 'trap', kind: 'step', label: 'T' },
        { id: 'end', kind: 'end', label: 'E' }
      ],
      [{ from: 'start', to: 'trap' }]
    );
    const r = validateFlow(f);
    expect(r.deadlocked).toContain('trap');
    expect(r.valid).toBe(false);
  });

  it('detects cycle', () => {
    const f = build(
      [
        { id: 'start', kind: 'start', label: 'S' },
        { id: 'a', kind: 'step', label: 'A' },
        { id: 'b', kind: 'step', label: 'B' },
        { id: 'end', kind: 'end', label: 'E' }
      ],
      [
        { from: 'start', to: 'a' },
        { from: 'a', to: 'b' },
        { from: 'b', to: 'a' },
        { from: 'b', to: 'end' }
      ]
    );
    const r = validateFlow(f);
    expect(r.cycles.length).toBeGreaterThan(0);
    expect(r.valid).toBe(false);
  });

  it('detects orphaned end', () => {
    const f = build(
      [
        { id: 'start', kind: 'start', label: 'S' },
        { id: 'end1', kind: 'end', label: 'E1' },
        { id: 'end2', kind: 'end', label: 'E2' }
      ],
      [{ from: 'start', to: 'end1' }]
    );
    const r = validateFlow(f);
    expect(r.orphanedEnds).toContain('end2');
    expect(r.valid).toBe(false);
  });

  it('detects duplicate edges', () => {
    const f = build(
      [
        { id: 'start', kind: 'start', label: 'S' },
        { id: 'end', kind: 'end', label: 'E' }
      ],
      [
        { from: 'start', to: 'end' },
        { from: 'start', to: 'end' }
      ]
    );
    const r = validateFlow(f);
    expect(r.duplicateEdges.length).toBe(1);
    expect(r.valid).toBe(false);
  });

  it('detects nondeterministic choice on non-decision node', () => {
    const f = build(
      [
        { id: 'start', kind: 'start', label: 'S' },
        { id: 'a', kind: 'step', label: 'A' },
        { id: 'b', kind: 'step', label: 'B' },
        { id: 'c', kind: 'step', label: 'C' },
        { id: 'end', kind: 'end', label: 'E' }
      ],
      [
        // a から条件なし辺が 2 本（step なら NG）
        { from: 'start', to: 'a' },
        { from: 'a', to: 'b' },
        { from: 'a', to: 'c' },
        { from: 'b', to: 'end' },
        { from: 'c', to: 'end' }
      ]
    );
    const r = validateFlow(f);
    expect(r.nondeterministicChoices).toContain('a');
    expect(r.valid).toBe(false);
  });
});
