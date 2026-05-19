import { create } from 'zustand';
import type { Step } from '@wnav/shared/types';

// SOP 編集の Undo/Redo スタック（最大 50 件）。即時公開禁止のため公開フラグは持たない（保存→レビュー依頼 UI で別管理）。
const MAX_HISTORY = 50;

interface SopEditorState {
  steps: Step[];
  undoStack: Step[][];
  redoStack: Step[][];
  isDirty: boolean;
  lastSavedAt: string | null;
  setSteps: (next: Step[]) => void;
  // 編集操作: 現状を undo スタックに push してから新状態を反映
  push: (next: Step[]) => void;
  undo: () => void;
  redo: () => void;
  markSaved: (at: string) => void;
  clear: () => void;
}

export const useSopEditorStore = create<SopEditorState>((set, get) => ({
  steps: [],
  undoStack: [],
  redoStack: [],
  isDirty: false,
  lastSavedAt: null,
  setSteps: (next) => set({ steps: next, isDirty: true }),
  push: (next) => {
    const { steps, undoStack } = get();
    const newUndo = [...undoStack, steps].slice(-MAX_HISTORY);
    set({ steps: next, undoStack: newUndo, redoStack: [], isDirty: true });
  },
  undo: () => {
    const { undoStack, redoStack, steps } = get();
    const last = undoStack[undoStack.length - 1];
    if (!last) return;
    set({
      steps: last,
      undoStack: undoStack.slice(0, -1),
      redoStack: [...redoStack, steps].slice(-MAX_HISTORY),
      isDirty: true,
    });
  },
  redo: () => {
    const { undoStack, redoStack, steps } = get();
    const last = redoStack[redoStack.length - 1];
    if (!last) return;
    set({
      steps: last,
      undoStack: [...undoStack, steps].slice(-MAX_HISTORY),
      redoStack: redoStack.slice(0, -1),
      isDirty: true,
    });
  },
  markSaved: (at) => set({ isDirty: false, lastSavedAt: at }),
  clear: () => set({ steps: [], undoStack: [], redoStack: [], isDirty: false, lastSavedAt: null }),
}));
