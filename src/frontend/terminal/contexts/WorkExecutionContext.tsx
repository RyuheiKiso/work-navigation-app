// 作業実行コンテキスト。caseId・sopVersionId・currentStepIndex・caseLock 状態を共有する
import React, { createContext, useContext, useMemo, useReducer } from 'react';

export type CaseLockStatus = 'NONE' | 'ACQUIRING' | 'ACTIVE' | 'EXPIRED' | 'CONFLICT';

export interface WorkExecutionState {
  caseId: string | null;
  workExecutionId: string | null;
  sopVersionId: string | null;
  currentStepIndex: number;
  currentStepId: string | null;
  totalSteps: number;
  caseLockStatus: CaseLockStatus;
}

export type WorkExecutionAction =
  | {
      type: 'START_EXECUTION';
      payload: { caseId: string; workExecutionId: string; sopVersionId: string; totalSteps: number };
    }
  | { type: 'ADVANCE_STEP' }
  | { type: 'SET_STEP_INDEX'; payload: { index: number } }
  | { type: 'SET_CURRENT_STEP'; payload: { index: number; stepId: string } }
  | { type: 'SET_LOCK_STATUS'; payload: { status: CaseLockStatus } }
  | { type: 'CLEAR' };

const initialState: WorkExecutionState = {
  caseId: null,
  workExecutionId: null,
  sopVersionId: null,
  currentStepIndex: 0,
  currentStepId: null,
  totalSteps: 0,
  caseLockStatus: 'NONE',
};

export const workExecutionReducer = (
  state: WorkExecutionState,
  action: WorkExecutionAction,
): WorkExecutionState => {
  switch (action.type) {
    case 'START_EXECUTION':
      return {
        ...state,
        caseId: action.payload.caseId,
        workExecutionId: action.payload.workExecutionId,
        sopVersionId: action.payload.sopVersionId,
        totalSteps: action.payload.totalSteps,
        currentStepIndex: 0,
        currentStepId: null,
      };
    case 'ADVANCE_STEP':
      return { ...state, currentStepIndex: Math.min(state.totalSteps, state.currentStepIndex + 1), currentStepId: null };
    case 'SET_STEP_INDEX':
      return { ...state, currentStepIndex: action.payload.index, currentStepId: null };
    // SET_CURRENT_STEP: ステップ画面が表示された時点で SOP 定義から解決した stepId を登録する
    case 'SET_CURRENT_STEP':
      return { ...state, currentStepIndex: action.payload.index, currentStepId: action.payload.stepId };
    case 'SET_LOCK_STATUS':
      return { ...state, caseLockStatus: action.payload.status };
    case 'CLEAR':
      return initialState;
  }
};

interface WorkExecutionContextValue {
  state: WorkExecutionState;
  dispatch: React.Dispatch<WorkExecutionAction>;
}

const WorkExecutionContext = createContext<WorkExecutionContextValue | null>(null);

export function WorkExecutionProvider({ children }: { children: React.ReactNode }): JSX.Element {
  const [state, dispatch] = useReducer(workExecutionReducer, initialState);
  const value = useMemo(() => ({ state, dispatch }), [state]);
  return <WorkExecutionContext.Provider value={value}>{children}</WorkExecutionContext.Provider>;
}

export function useWorkExecution(): WorkExecutionContextValue {
  const ctx = useContext(WorkExecutionContext);
  if (ctx === null) throw new Error('useWorkExecution must be used within WorkExecutionProvider');
  return ctx;
}
