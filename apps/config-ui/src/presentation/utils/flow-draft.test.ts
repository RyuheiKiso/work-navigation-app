// 対応 §: ロードマップ §13.1 §10.2.2 §14.2
// FlowDraft 永続化ユーティリティの単体テスト。

import { describe, it, expect, beforeEach } from 'vitest';
import {
  flowDraftKey,
  loadFlowDraft,
  saveFlowDraft,
  clearFlowDraft,
  type FlowDraft
} from './flow-draft';

function makeDraft(flowId: string): Omit<FlowDraft, 'schemaVersion'> {
  return {
    flowId,
    flowName: 'サンプル',
    nodes: [{ id: 'start', position: { x: 0, y: 0 }, data: { label: '開始' } }],
    edges: [],
    nodeKinds: [['start', 'start']],
    nodeLabels: [['start', '開始']],
    nodeCompletion: [['start', undefined]],
    savedAt: 1_700_000_000_000
  };
}

describe('flow-draft persistence', () => {
  beforeEach(() => localStorage.clear());

  it('returns null when no draft exists', () => {
    expect(loadFlowDraft('nonexistent')).toBeNull();
  });

  it('round-trips a saved draft', () => {
    saveFlowDraft(makeDraft('f1'));
    const loaded = loadFlowDraft('f1');
    expect(loaded).not.toBeNull();
    expect(loaded?.flowId).toBe('f1');
    expect(loaded?.flowName).toBe('サンプル');
    expect(loaded?.nodes).toHaveLength(1);
  });

  it('clears a saved draft', () => {
    saveFlowDraft(makeDraft('f2'));
    clearFlowDraft('f2');
    expect(loadFlowDraft('f2')).toBeNull();
  });

  it('isolates drafts by flowId', () => {
    saveFlowDraft(makeDraft('f1'));
    saveFlowDraft({ ...makeDraft('f2'), flowName: '別フロー' });
    expect(loadFlowDraft('f1')?.flowName).toBe('サンプル');
    expect(loadFlowDraft('f2')?.flowName).toBe('別フロー');
  });

  it('rejects malformed JSON gracefully', () => {
    localStorage.setItem(flowDraftKey('broken'), 'this is not json');
    expect(loadFlowDraft('broken')).toBeNull();
  });

  it('rejects mismatched flowId in stored payload', () => {
    // 故意に flowId を書き換えた壊れた payload を直接書き込む
    localStorage.setItem(
      flowDraftKey('expected'),
      JSON.stringify({ ...makeDraft('expected'), schemaVersion: 1, flowId: 'other' })
    );
    expect(loadFlowDraft('expected')).toBeNull();
  });

  it('rejects payload missing required fields', () => {
    localStorage.setItem(flowDraftKey('partial'), JSON.stringify({ schemaVersion: 1 }));
    expect(loadFlowDraft('partial')).toBeNull();
  });
});
