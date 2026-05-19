// NetworkContext。4 段階ネットワーク品質 + Emergency Mode の遷移管理
import React, { createContext, useContext, useEffect, useMemo, useReducer } from 'react';
import type { NetworkQuality } from '@wnav/shared';
import { NetworkMonitor } from '../network/NetworkMonitor';

const EMERGENCY_THRESHOLD_MS = 5 * 60 * 1000;

export interface NetworkState {
  quality: NetworkQuality;
  isEmergencyMode: boolean;
  lastOnlineAt: string | null;
  lastSyncedAt: string | null;
  pendingSyncCount: number;
}

export type NetworkAction =
  | { type: 'QUALITY_CHANGED'; payload: { quality: NetworkQuality } }
  | { type: 'INCREMENT_PENDING' }
  | { type: 'SYNC_COMPLETE'; payload: { syncedCount: number; syncedAt: string } }
  | { type: 'ENTER_EMERGENCY' };

const initialState: NetworkState = {
  quality: 'disconnected',
  isEmergencyMode: false,
  lastOnlineAt: null,
  lastSyncedAt: null,
  pendingSyncCount: 0,
};

export const networkReducer = (state: NetworkState, action: NetworkAction): NetworkState => {
  switch (action.type) {
    case 'QUALITY_CHANGED': {
      const online = action.payload.quality === 'high' || action.payload.quality === 'low';
      return {
        ...state,
        quality: action.payload.quality,
        isEmergencyMode: action.payload.quality === 'emergency',
        lastOnlineAt: online ? new Date().toISOString() : state.lastOnlineAt,
      };
    }
    case 'INCREMENT_PENDING':
      return { ...state, pendingSyncCount: state.pendingSyncCount + 1 };
    case 'SYNC_COMPLETE':
      return {
        ...state,
        pendingSyncCount: Math.max(0, state.pendingSyncCount - action.payload.syncedCount),
        lastSyncedAt: action.payload.syncedAt,
      };
    case 'ENTER_EMERGENCY':
      return { ...state, isEmergencyMode: true, quality: 'emergency' };
  }
};

interface NetworkContextValue {
  state: NetworkState;
  dispatch: React.Dispatch<NetworkAction>;
}

const NetworkContext = createContext<NetworkContextValue | null>(null);

export function NetworkProvider({ children }: { children: React.ReactNode }): JSX.Element {
  const [state, dispatch] = useReducer(networkReducer, initialState);

  useEffect(() => {
    const monitor = new NetworkMonitor();
    const unsubscribe = monitor.subscribe((quality) => {
      dispatch({ type: 'QUALITY_CHANGED', payload: { quality } });
    });
    monitor.start();
    return () => {
      unsubscribe();
      monitor.stop();
    };
  }, []);

  // 5 分以上切断状態が続いたら Emergency Mode に遷移する
  useEffect(() => {
    if (state.quality !== 'disconnected') return;
    if (state.lastOnlineAt === null) return;
    const timer = setTimeout(() => {
      const last = Date.parse(state.lastOnlineAt ?? '');
      if (Number.isFinite(last) && Date.now() - last >= EMERGENCY_THRESHOLD_MS) {
        dispatch({ type: 'ENTER_EMERGENCY' });
      }
    }, EMERGENCY_THRESHOLD_MS);
    return () => clearTimeout(timer);
  }, [state.quality, state.lastOnlineAt]);

  const value = useMemo(() => ({ state, dispatch }), [state]);
  return <NetworkContext.Provider value={value}>{children}</NetworkContext.Provider>;
}

export function useNetwork(): NetworkContextValue {
  const ctx = useContext(NetworkContext);
  if (ctx === null) throw new Error('useNetwork must be used within NetworkProvider');
  return ctx;
}
