// 対応 §: ロードマップ §10.2.2 §14.2
// フロー編集の中断耐性を確保するためのドラフト永続化ユーティリティ。
// Flow.create が不変条件違反で例外を投げる中間状態でも復元できるよう、
// ドメイン Aggregate ではなく React Flow の生 state を JSON として保持する。

import type { Edge, Node } from 'reactflow';
import type { FlowNodeKind } from '../../domain/flow';

const STORAGE_PREFIX = 'wna.config-ui.flow-draft.v1.';

const DRAFT_SCHEMA_VERSION = 1 as const;

type CompletionValue = 'manual' | 'photo' | undefined;

export interface FlowDraft {
  readonly schemaVersion: typeof DRAFT_SCHEMA_VERSION;
  readonly flowId: string;
  readonly flowName: string;
  readonly nodes: ReadonlyArray<Node>;
  readonly edges: ReadonlyArray<Edge>;
  readonly nodeKinds: ReadonlyArray<readonly [string, FlowNodeKind]>;
  readonly nodeLabels: ReadonlyArray<readonly [string, string]>;
  readonly nodeCompletion: ReadonlyArray<readonly [string, CompletionValue]>;
  readonly savedAt: number;
}

export function flowDraftKey(flowId: string): string {
  return STORAGE_PREFIX + flowId;
}

export function loadFlowDraft(
  flowId: string,
  storage: Storage = globalThis.localStorage
): FlowDraft | null {
  const raw = safeGet(storage, flowDraftKey(flowId));
  if (raw === null) return null;
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!isFlowDraft(parsed)) return null;
    if (parsed.flowId !== flowId) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function saveFlowDraft(
  draft: Omit<FlowDraft, 'schemaVersion'>,
  storage: Storage = globalThis.localStorage
): void {
  const payload: FlowDraft = { schemaVersion: DRAFT_SCHEMA_VERSION, ...draft };
  safeSet(storage, flowDraftKey(draft.flowId), JSON.stringify(payload));
}

export function clearFlowDraft(
  flowId: string,
  storage: Storage = globalThis.localStorage
): void {
  safeRemove(storage, flowDraftKey(flowId));
}

function isFlowDraft(v: unknown): v is FlowDraft {
  if (typeof v !== 'object' || v === null) return false;
  const o = v as Record<string, unknown>;
  return (
    o['schemaVersion'] === DRAFT_SCHEMA_VERSION &&
    typeof o['flowId'] === 'string' &&
    typeof o['flowName'] === 'string' &&
    Array.isArray(o['nodes']) &&
    Array.isArray(o['edges']) &&
    Array.isArray(o['nodeKinds']) &&
    Array.isArray(o['nodeLabels']) &&
    Array.isArray(o['nodeCompletion']) &&
    typeof o['savedAt'] === 'number'
  );
}

// QuotaExceeded やプライベートブラウジング無効化に耐えるラッパ
function safeGet(storage: Storage, key: string): string | null {
  try {
    return storage.getItem(key);
  } catch {
    return null;
  }
}

function safeSet(storage: Storage, key: string, value: string): void {
  try {
    storage.setItem(key, value);
  } catch {
    // 保存失敗は黙って諦める。呼び出し側の autosave 状態が error を表示する。
    throw new Error('storage_write_failed');
  }
}

function safeRemove(storage: Storage, key: string): void {
  try {
    storage.removeItem(key);
  } catch {
    // noop
  }
}
