// 対応 §: ロードマップ §10.2.1 §10.2.2 §3.4.1
// フロー編集の状態 (nodes / edges / 各種属性マップ) と HSM 検証結果を集約するフック。
// 表示層 (flow-canvas.tsx) はこのフックの戻り値を介してドメインに触れる。

import { useCallback, useMemo, useState } from 'react';
import {
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  type Connection,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange
} from 'reactflow';
import {
  Flow,
  type FlowEdge,
  type FlowNode,
  type FlowNodeKind
} from '../../domain/flow';
import { validateFlow } from '../../domain/flow-validation';
import { flowFromYaml } from '../../adapter/yaml-flow-parser';
import { nodeStyle, toRfEdges, toRfNodes } from '../utils/flow-rf-mapping';

export interface SelectedNode {
  id: string;
  kind: FlowNodeKind;
  label: string;
  completion: 'manual' | 'photo' | undefined;
}

export type ValidationResult = ReturnType<typeof validateFlow>;

export interface FlowEditor {
  nodes: Node[];
  edges: Edge[];
  flowId: string;
  flowName: string;
  setFlowName(name: string): void;
  selectedNode: SelectedNode | null;
  setSelectedNodeId(id: string | null): void;
  currentFlow: Flow | null;
  validation: ValidationResult | null;
  onNodesChange(changes: NodeChange[]): void;
  onEdgesChange(changes: EdgeChange[]): void;
  onConnect(conn: Connection): void;
  addNode(kind: FlowNodeKind): void;
  loadTemplate(path: string): Promise<void>;
  setNodeLabel(id: string, label: string): void;
  setNodeCompletion(id: string, value: 'manual' | 'photo' | undefined): void;
}

export function useFlowEditor(initialFlow: Flow): FlowEditor {
  const [nodes, setNodes] = useState<Node[]>(() => toRfNodes(initialFlow));
  const [edges, setEdges] = useState<Edge[]>(() => toRfEdges(initialFlow));
  const [nodeKinds, setNodeKinds] = useState<Map<string, FlowNodeKind>>(
    () => new Map(initialFlow.nodes.map((n) => [n.id, n.kind]))
  );
  const [nodeLabels, setNodeLabels] = useState<Map<string, string>>(
    () => new Map(initialFlow.nodes.map((n) => [n.id, n.label]))
  );
  const [nodeCompletion, setNodeCompletion] = useState<
    Map<string, 'manual' | 'photo' | undefined>
  >(() => new Map(initialFlow.nodes.map((n) => [n.id, n.completion_criteria])));
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [flowId] = useState(initialFlow.id);
  const [flowName, setFlowName] = useState(initialFlow.name);

  const currentFlow = useMemo<Flow | null>(() => {
    try {
      const fNodes: FlowNode[] = nodes.map((rn) => {
        const completion = nodeCompletion.get(rn.id);
        return {
          id: rn.id,
          kind: nodeKinds.get(rn.id) ?? 'step',
          label: nodeLabels.get(rn.id) ?? rn.id,
          ...(completion !== undefined ? { completion_criteria: completion } : {})
        };
      });
      const fEdges: FlowEdge[] = edges.map((re) => ({
        from: re.source,
        to: re.target,
        ...(re.label !== undefined && re.label !== '' ? { condition: String(re.label) } : {})
      }));
      return Flow.create(flowId, flowName, fNodes, fEdges, undefined, 1);
    } catch {
      // 不変条件違反でも UI は止めない
      return null;
    }
  }, [nodes, edges, nodeKinds, nodeLabels, nodeCompletion, flowId, flowName]);

  const validation = useMemo(() => {
    if (!currentFlow) return null;
    return validateFlow(currentFlow);
  }, [currentFlow]);

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => setNodes((nds) => applyNodeChanges(changes, nds)),
    []
  );
  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    []
  );
  const onConnect = useCallback(
    (conn: Connection) =>
      setEdges((eds) => addEdge({ ...conn, id: `e-${Date.now()}`, animated: false }, eds)),
    []
  );

  const addNode = useCallback((kind: FlowNodeKind) => {
    const id = `${kind}-${Date.now()}`;
    const label = kind === 'start' ? '開始' : kind === 'end' ? '終了' : `新規${kind}`;
    setNodeKinds((m) => new Map(m).set(id, kind));
    setNodeLabels((m) => new Map(m).set(id, label));
    setNodes((nds) => [
      ...nds,
      {
        id,
        type: 'default',
        data: { label: `${label}\n[${kind}]` },
        position: { x: 50 + Math.random() * 400, y: 50 + Math.random() * 200 },
        style: nodeStyle(kind)
      }
    ]);
  }, []);

  const loadTemplate = useCallback(async (path: string) => {
    const res = await fetch(path);
    if (!res.ok) throw new Error('template fetch failed');
    const yaml = await res.text();
    const flow = flowFromYaml(yaml);
    setNodes(toRfNodes(flow));
    setEdges(toRfEdges(flow));
    setNodeKinds(new Map(flow.nodes.map((n) => [n.id, n.kind])));
    setNodeLabels(new Map(flow.nodes.map((n) => [n.id, n.label])));
    setNodeCompletion(new Map(flow.nodes.map((n) => [n.id, n.completion_criteria])));
    setFlowName(flow.name);
  }, []);

  const setNodeLabel = useCallback((id: string, label: string) => {
    setNodeLabels((m) => new Map(m).set(id, label));
  }, []);

  const setNodeCompletionValue = useCallback(
    (id: string, value: 'manual' | 'photo' | undefined) => {
      setNodeCompletion((m) => {
        const nm = new Map(m);
        if (value === undefined) nm.delete(id);
        else nm.set(id, value);
        return nm;
      });
    },
    []
  );

  const selectedNode = useMemo<SelectedNode | null>(() => {
    if (!selectedNodeId) return null;
    return {
      id: selectedNodeId,
      kind: nodeKinds.get(selectedNodeId) ?? 'step',
      label: nodeLabels.get(selectedNodeId) ?? selectedNodeId,
      completion: nodeCompletion.get(selectedNodeId)
    };
  }, [selectedNodeId, nodeKinds, nodeLabels, nodeCompletion]);

  return {
    nodes,
    edges,
    flowId,
    flowName,
    setFlowName,
    selectedNode,
    setSelectedNodeId,
    currentFlow,
    validation,
    onNodesChange,
    onEdgesChange,
    onConnect,
    addNode,
    loadTemplate,
    setNodeLabel,
    setNodeCompletion: setNodeCompletionValue
  };
}
