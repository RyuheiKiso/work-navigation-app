// 対応 §: ロードマップ §10.2.1 §9.5
// Flow Aggregate と React Flow の Node/Edge 表現を相互変換するユーティリティ。
// ノード kind ごとの色は §9.5.1 design-tokens の semantic カラーに対応する。

import type { Edge, Node } from 'reactflow';
import type { Flow, FlowNodeKind } from '../../domain/flow';

export function toRfNodes(flow: Flow): Node[] {
  return flow.nodes.map((n, i) => ({
    id: n.id,
    type: 'default',
    data: {
      label: `${n.label}\n[${n.kind}${n.completion_criteria ? '/' + n.completion_criteria : ''}]`
    },
    position: { x: 100 + (i % 3) * 220, y: 80 + Math.floor(i / 3) * 160 },
    style: nodeStyle(n.kind)
  }));
}

export function toRfEdges(flow: Flow): Edge[] {
  return flow.edges.map((e, i) => ({
    id: `e-${i}`,
    source: e.from,
    target: e.to,
    label: e.condition,
    animated: !!e.condition
  }));
}

export function nodeStyle(kind: FlowNodeKind): React.CSSProperties {
  switch (kind) {
    case 'start':
      return { background: '#D4EDDA', border: '2px solid #28A745', borderRadius: 8 };
    case 'end':
      return { background: '#F8D7DA', border: '2px solid #DC3545', borderRadius: 8 };
    case 'decision':
      return { background: '#FFF3CD', border: '2px solid #FFC107', borderRadius: 0 };
    case 'parallel':
      return { background: '#D1ECF1', border: '2px solid #17A2B8', borderRadius: 8 };
    default:
      return { background: '#FFFFFF', border: '1px solid #6C757D', borderRadius: 8 };
  }
}
