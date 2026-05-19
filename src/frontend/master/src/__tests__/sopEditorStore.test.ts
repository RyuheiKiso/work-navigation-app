import { describe, it, expect, beforeEach } from 'vitest';
import { useSopEditorStore } from '@/stores/sopEditorStore';
import type { Step } from '@wnav/shared/types';

function makeStep(id: string, n: number): Step {
  return {
    id,
    sopVersionId: 'sv-1',
    stepNumber: n,
    stepType: 'check',
    titleJson: { ja: `Step ${n}`, en: '', zh: '' },
    instructionJson: { ja: '', en: '', zh: '' },
    payload: '{}',
    isMandatory: false,
    requiresEvidence: false,
    requiresSign: false,
    skillLevelRequired: 1,
    estimatedSeconds: 30,
    fallbackType: 'manual',
    flowRules: { onComplete: 'next', onSkip: 'next' },
    deletedAt: null,
  };
}

describe('sopEditorStore', () => {
  beforeEach(() => {
    useSopEditorStore.getState().clear();
  });

  it('push で undo スタックに前状態を保存し isDirty を立てる', () => {
    const s = useSopEditorStore.getState();
    s.push([makeStep('s1', 1)]);
    s.push([makeStep('s1', 1), makeStep('s2', 2)]);
    expect(useSopEditorStore.getState().steps).toHaveLength(2);
    expect(useSopEditorStore.getState().undoStack).toHaveLength(2);
    expect(useSopEditorStore.getState().isDirty).toBe(true);
  });

  it('undo は直前状態に戻し redo スタックに現状態を退避する', () => {
    const s = useSopEditorStore.getState();
    s.push([makeStep('s1', 1)]);
    s.push([makeStep('s1', 1), makeStep('s2', 2)]);
    s.undo();
    expect(useSopEditorStore.getState().steps).toHaveLength(1);
    expect(useSopEditorStore.getState().redoStack).toHaveLength(1);
    s.redo();
    expect(useSopEditorStore.getState().steps).toHaveLength(2);
  });

  it('履歴は最大 50 件で先頭を捨てる', () => {
    const s = useSopEditorStore.getState();
    for (let i = 0; i < 60; i += 1) {
      s.push([makeStep(`s${i}`, i)]);
    }
    expect(useSopEditorStore.getState().undoStack.length).toBeLessThanOrEqual(50);
  });

  it('markSaved で isDirty を解除する', () => {
    const s = useSopEditorStore.getState();
    s.push([makeStep('s1', 1)]);
    s.markSaved('2026-05-19T00:00:00Z');
    expect(useSopEditorStore.getState().isDirty).toBe(false);
    expect(useSopEditorStore.getState().lastSavedAt).toBe('2026-05-19T00:00:00Z');
  });
});
