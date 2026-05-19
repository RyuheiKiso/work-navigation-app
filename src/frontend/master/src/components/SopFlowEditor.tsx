import type React from 'react';
import { useMemo, useCallback } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  type Node,
  type Edge,
  type Connection,
  addEdge,
  applyNodeChanges,
  applyEdgeChanges,
  type NodeChange,
  type EdgeChange,
  MarkerType,
} from 'reactflow';
import type { Step } from '@wnav/shared/types';

interface FlowState {
  nodes: Node[];
  edges: Edge[];
}

// SopFlowCanvas（CMP-MA-005）。Step を ReactFlow ノードに、flowRules を Edge にマッピングする。
// DAG 編集の本格機能はタスク #10 で実装し、ここでは表示と接続の基本機能のみ。
export function SopFlowEditor({
  steps,
  flow,
  onFlowChange,
  readOnly = false,
}: {
  steps: Step[];
  flow: FlowState;
  onFlowChange: (next: FlowState) => void;
  readOnly?: boolean;
}): React.ReactElement {
  const computedNodes = useMemo<Node[]>(() => {
    if (flow.nodes.length > 0) return flow.nodes;
    return steps.map((s, idx) => ({
      id: s.id,
      data: { label: `${s.stepNumber}: ${s.titleJson.ja || s.id}` },
      position: { x: (idx % 4) * 220, y: Math.floor(idx / 4) * 120 },
      type: 'default',
    }));
  }, [steps, flow.nodes]);

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => {
      if (readOnly) return;
      onFlowChange({ nodes: applyNodeChanges(changes, computedNodes), edges: flow.edges });
    },
    [readOnly, computedNodes, flow.edges, onFlowChange],
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      if (readOnly) return;
      onFlowChange({ nodes: computedNodes, edges: applyEdgeChanges(changes, flow.edges) });
    },
    [readOnly, computedNodes, flow.edges, onFlowChange],
  );

  const onConnect = useCallback(
    (params: Connection) => {
      if (readOnly) return;
      onFlowChange({
        nodes: computedNodes,
        edges: addEdge({ ...params, markerEnd: { type: MarkerType.ArrowClosed } }, flow.edges),
      });
    },
    [readOnly, computedNodes, flow.edges, onFlowChange],
  );

  return (
    <div style={{ height: 600 }} aria-label="SOP フローキャンバス">
      <ReactFlow
        nodes={computedNodes}
        edges={flow.edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodesDraggable={!readOnly}
        edgesUpdatable={!readOnly}
        fitView
      >
        <Background />
        <Controls />
        <MiniMap pannable zoomable />
      </ReactFlow>
    </div>
  );
}
