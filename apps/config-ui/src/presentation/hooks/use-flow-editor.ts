// 対応 §: ロードマップ §10.2.1 §10.2.2 §3.4.1 §14.2
// フロー編集の状態 (nodes / edges / 各種属性マップ) と HSM 検証結果を集約するフック。
// 表示層 (flow-canvas.tsx) はこのフックの戻り値を介してドメインに触れる。
// 中断耐性 (§14.2) のため、編集 state を localStorage に debounce 永続化する。

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
import {
  clearFlowDraft,
  loadFlowDraft,
  saveFlowDraft,
  type FlowDraft
} from '../utils/flow-draft';
import { useAutosave, type AutosaveStatus } from './use-autosave';

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
  autosaveStatus: AutosaveStatus;
  lastSavedAt: number | null;
  restoredFromDraft: boolean;
  discardDraft(): void;
}

interface InitialState {
  nodes: Node[];
  edges: Edge[];
  nodeKinds: Map<string, FlowNodeKind>;
  nodeLabels: Map<string, string>;
  nodeCompletion: Map<string, 'manual' | 'photo' | undefined>;
  flowName: string;
  restored: boolean;
}

function buildInitialState(flow: Flow): InitialState {
  const draft = loadFlowDraft(flow.id);
  if (draft !== null) return fromDraft(draft);
  return {
    nodes: toRfNodes(flow),
    edges: toRfEdges(flow),
    nodeKinds: new Map(flow.nodes.map((n) => [n.id, n.kind])),
    nodeLabels: new Map(flow.nodes.map((n) => [n.id, n.label])),
    nodeCompletion: new Map(flow.nodes.map((n) => [n.id, n.completion_criteria])),
    flowName: flow.name,
    restored: false
  };
}

function fromDraft(d: FlowDraft): InitialState {
  return {
    nodes: [...d.nodes],
    edges: [...d.edges],
    nodeKinds: new Map(d.nodeKinds),
    nodeLabels: new Map(d.nodeLabels),
    nodeCompletion: new Map(d.nodeCompletion),
    flowName: d.flowName,
    restored: true
  };
}

export function useFlowEditor(initialFlow: Flow): FlowEditor {
  const initial = useMemo(() => buildInitialState(initialFlow), [initialFlow]);
  const [nodes, setNodes] = useState<Node[]>(initial.nodes);
  const [edges, setEdges] = useState<Edge[]>(initial.edges);
  const [nodeKinds, setNodeKinds] = useState<Map<string, FlowNodeKind>>(initial.nodeKinds);
  const [nodeLabels, setNodeLabels] = useState<Map<string, string>>(initial.nodeLabels);
  const [nodeCompletion, setNodeCompletion] = useState<
    Map<string, 'manual' | 'photo' | undefined>
  >(initial.nodeCompletion);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [flowId] = useState(initialFlow.id);
  const [flowName, setFlowName] = useState(initial.flowName);
  const [restoredFromDraft, setRestoredFromDraft] = useState(initial.restored);

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

  // 編集 state 全体をひとつの値に畳んで autosave Hook に渡す
  const draftSnapshot = useMemo(
    () => ({
      flowId,
      flowName,
      nodes,
      edges,
      nodeKinds: Array.from(nodeKinds.entries()),
      nodeLabels: Array.from(nodeLabels.entries()),
      nodeCompletion: Array.from(nodeCompletion.entries())
    }),
    [flowId, flowName, nodes, edges, nodeKinds, nodeLabels, nodeCompletion]
  );

  const writeDraft = useCallback(
    (snap: typeof draftSnapshot) => saveFlowDraft({ ...snap, savedAt: Date.now() }),
    []
  );
  const autosave = useAutosave({ value: draftSnapshot, write: writeDraft });

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
    // テンプレ読込はユーザー意図の上書きなので復元バナーを下げる
    setRestoredFromDraft(false);
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

  const discardDraft = useCallback(() => {
    clearFlowDraft(initialFlow.id);
    setNodes(toRfNodes(initialFlow));
    setEdges(toRfEdges(initialFlow));
    setNodeKinds(new Map(initialFlow.nodes.map((n) => [n.id, n.kind])));
    setNodeLabels(new Map(initialFlow.nodes.map((n) => [n.id, n.label])));
    setNodeCompletion(new Map(initialFlow.nodes.map((n) => [n.id, n.completion_criteria])));
    setFlowName(initialFlow.name);
    setRestoredFromDraft(false);
  }, [initialFlow]);

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
    setNodeCompletion: setNodeCompletionValue,
    autosaveStatus: autosave.status,
    lastSavedAt: autosave.lastSavedAt,
    restoredFromDraft,
    discardDraft
  };
}
