// WorkExecutionContext reducer の全アクションを検証する
import { workExecutionReducer } from '../../contexts/WorkExecutionContext';
import type { WorkExecutionState, WorkExecutionAction } from '../../contexts/WorkExecutionContext';

const initialState: WorkExecutionState = {
  caseId: null,
  workExecutionId: null,
  sopVersionId: null,
  currentStepIndex: 0,
  currentStepId: null,
  totalSteps: 0,
  caseLockStatus: 'NONE',
};

const startedState: WorkExecutionState = {
  caseId: 'case-001',
  workExecutionId: 'exec-001',
  sopVersionId: 'sop-v1',
  currentStepIndex: 0,
  currentStepId: null,
  totalSteps: 5,
  caseLockStatus: 'NONE',
};

describe('workExecutionReducer — START_EXECUTION', () => {
  it('作業開始時に全フィールドを初期化する', () => {
    const action: WorkExecutionAction = {
      type: 'START_EXECUTION',
      payload: { caseId: 'case-001', workExecutionId: 'exec-001', sopVersionId: 'sop-v1', totalSteps: 5 },
    };
    const next = workExecutionReducer(initialState, action);
    expect(next.caseId).toBe('case-001');
    expect(next.workExecutionId).toBe('exec-001');
    expect(next.sopVersionId).toBe('sop-v1');
    expect(next.totalSteps).toBe(5);
    expect(next.currentStepIndex).toBe(0);
    expect(next.currentStepId).toBeNull();
  });

  it('再起動時に currentStepId をリセットする', () => {
    const stateWithStep: WorkExecutionState = { ...startedState, currentStepId: 'step-x' };
    const action: WorkExecutionAction = {
      type: 'START_EXECUTION',
      payload: { caseId: 'case-002', workExecutionId: 'exec-002', sopVersionId: 'sop-v2', totalSteps: 3 },
    };
    const next = workExecutionReducer(stateWithStep, action);
    expect(next.currentStepId).toBeNull();
  });
});

describe('workExecutionReducer — SET_CURRENT_STEP', () => {
  it('stepId と stepIndex を同時に更新する', () => {
    const action: WorkExecutionAction = {
      type: 'SET_CURRENT_STEP',
      payload: { index: 2, stepId: 'step-abc' },
    };
    const next = workExecutionReducer(startedState, action);
    expect(next.currentStepIndex).toBe(2);
    expect(next.currentStepId).toBe('step-abc');
  });

  it('他のフィールドは変更しない', () => {
    const action: WorkExecutionAction = {
      type: 'SET_CURRENT_STEP',
      payload: { index: 1, stepId: 'step-001' },
    };
    const next = workExecutionReducer(startedState, action);
    expect(next.caseId).toBe(startedState.caseId);
    expect(next.sopVersionId).toBe(startedState.sopVersionId);
    expect(next.totalSteps).toBe(startedState.totalSteps);
  });
});

describe('workExecutionReducer — ADVANCE_STEP', () => {
  it('currentStepIndex を 1 増加し currentStepId をリセットする', () => {
    const state: WorkExecutionState = { ...startedState, currentStepIndex: 1, currentStepId: 'step-001' };
    const next = workExecutionReducer(state, { type: 'ADVANCE_STEP' });
    expect(next.currentStepIndex).toBe(2);
    expect(next.currentStepId).toBeNull();
  });

  it('totalSteps を超えない（上限クランプ）', () => {
    const state: WorkExecutionState = { ...startedState, currentStepIndex: 5, currentStepId: 'step-last' };
    const next = workExecutionReducer(state, { type: 'ADVANCE_STEP' });
    expect(next.currentStepIndex).toBe(5);
    expect(next.currentStepId).toBeNull();
  });
});

describe('workExecutionReducer — SET_STEP_INDEX', () => {
  it('任意のインデックスにジャンプし currentStepId をリセットする', () => {
    const state: WorkExecutionState = { ...startedState, currentStepIndex: 3, currentStepId: 'step-003' };
    const next = workExecutionReducer(state, { type: 'SET_STEP_INDEX', payload: { index: 0 } });
    expect(next.currentStepIndex).toBe(0);
    expect(next.currentStepId).toBeNull();
  });
});

describe('workExecutionReducer — SET_LOCK_STATUS', () => {
  it('caseLockStatus のみ更新する', () => {
    const next = workExecutionReducer(startedState, { type: 'SET_LOCK_STATUS', payload: { status: 'ACTIVE' } });
    expect(next.caseLockStatus).toBe('ACTIVE');
    expect(next.currentStepId).toBe(startedState.currentStepId);
  });
});

describe('workExecutionReducer — CLEAR', () => {
  it('initialState に戻す', () => {
    const state: WorkExecutionState = {
      caseId: 'case-001',
      workExecutionId: 'exec-001',
      sopVersionId: 'sop-v1',
      currentStepIndex: 3,
      currentStepId: 'step-003',
      totalSteps: 10,
      caseLockStatus: 'ACTIVE',
    };
    const next = workExecutionReducer(state, { type: 'CLEAR' });
    expect(next.caseId).toBeNull();
    expect(next.currentStepId).toBeNull();
    expect(next.currentStepIndex).toBe(0);
    expect(next.caseLockStatus).toBe('NONE');
  });
});
