// Outbox の保留件数・同期状況を共有するコンテキスト
import React, { createContext, useContext, useMemo, useReducer } from 'react';

export interface OutboxState {
  pendingCount: number;
  isSyncing: boolean;
}

export type OutboxAction =
  | { type: 'SET_PENDING'; payload: { count: number } }
  | { type: 'SYNC_START' }
  | { type: 'SYNC_END' };

const initialState: OutboxState = { pendingCount: 0, isSyncing: false };

export const outboxReducer = (state: OutboxState, action: OutboxAction): OutboxState => {
  switch (action.type) {
    case 'SET_PENDING':
      return { ...state, pendingCount: action.payload.count };
    case 'SYNC_START':
      return { ...state, isSyncing: true };
    case 'SYNC_END':
      return { ...state, isSyncing: false };
  }
};

interface OutboxContextValue {
  state: OutboxState;
  dispatch: React.Dispatch<OutboxAction>;
}

const OutboxContext = createContext<OutboxContextValue | null>(null);

export function OutboxProvider({ children }: { children: React.ReactNode }): JSX.Element {
  const [state, dispatch] = useReducer(outboxReducer, initialState);
  const value = useMemo(() => ({ state, dispatch }), [state]);
  return <OutboxContext.Provider value={value}>{children}</OutboxContext.Provider>;
}

export function useOutbox(): OutboxContextValue {
  const ctx = useContext(OutboxContext);
  if (ctx === null) throw new Error('useOutbox must be used within OutboxProvider');
  return ctx;
}
