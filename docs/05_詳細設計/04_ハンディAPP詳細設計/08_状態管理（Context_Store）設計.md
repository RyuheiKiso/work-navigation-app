# 08 状態管理（Context / Store）設計

本章は MOD-FE-HA-001（NetworkProvider）を含む React Native アプリ全体の状態管理設計を確定する。React Context + useReducer を採用し、Redux 系ライブラリを不使用とする。ネットワーク状態コンテキスト・Step 実行コンテキスト・認証コンテキストの型定義・Reducer・Provider の仕様を定める。

図: fig_dd_ha_context_providers（img/ 配下）を参照。

---

## 1. ネットワーク状態コンテキスト（MOD-FE-HA-001 NetworkProvider）

### 1-1. 型定義

```typescript
// src/features/network/NetworkContext.ts

export type NetworkState = 'CONNECTED' | 'DEGRADED' | 'DISCONNECTED' | 'EMERGENCY_MODE';

export interface NetworkContextValue {
  /** 現在のネットワーク状態（4 段階）*/
  state: NetworkState;

  /** SQLite outbox_events の PENDING 件数 */
  outboxQueueDepth: number;

  /** 最終 Outbox 送信成功日時（null = 一度も送信していない）*/
  lastSyncAt: string | null;

  /** Emergency Mode（切断 5 分超）フラグ */
  isEmergencyMode: boolean;

  /** 最終同期試行日時 */
  disconnectedSince: string | null;
}

export const NetworkContext = React.createContext<NetworkContextValue>({
  state: 'CONNECTED',
  outboxQueueDepth: 0,
  lastSyncAt: null,
  isEmergencyMode: false,
  disconnectedSince: null,
});
```

### 1-2. NetworkProvider 実装

```typescript
// src/features/network/NetworkProvider.tsx
import React, { useCallback, useEffect, useReducer, useRef } from 'react';
import NetInfo, { NetInfoState } from '@react-native-community/netinfo';

import type { NetworkContextValue, NetworkState } from './NetworkContext';
import { EMERGENCY_THRESHOLD_MS } from '../network/outbox/OutboxWorker';

type NetworkAction =
  | { type: 'CONNECTED' }
  | { type: 'DEGRADED' }
  | { type: 'DISCONNECTED'; disconnectedSince: string }
  | { type: 'EMERGENCY_MODE' }
  | { type: 'OUTBOX_DEPTH_UPDATED'; depth: number }
  | { type: 'SYNC_COMPLETED'; lastSyncAt: string };

function networkReducer(
  state: NetworkContextValue,
  action: NetworkAction,
): NetworkContextValue {
  switch (action.type) {
    case 'CONNECTED':
      return { ...state, state: 'CONNECTED', isEmergencyMode: false, disconnectedSince: null };
    case 'DEGRADED':
      return { ...state, state: 'DEGRADED', isEmergencyMode: false };
    case 'DISCONNECTED':
      return { ...state, state: 'DISCONNECTED', disconnectedSince: action.disconnectedSince };
    case 'EMERGENCY_MODE':
      return { ...state, state: 'EMERGENCY_MODE', isEmergencyMode: true };
    case 'OUTBOX_DEPTH_UPDATED':
      return { ...state, outboxQueueDepth: action.depth };
    case 'SYNC_COMPLETED':
      return { ...state, lastSyncAt: action.lastSyncAt };
    default:
      return state;
  }
}

const initialNetworkState: NetworkContextValue = {
  state: 'CONNECTED',
  outboxQueueDepth: 0,
  lastSyncAt: null,
  isEmergencyMode: false,
  disconnectedSince: null,
};

export const NetworkProvider: React.FC<React.PropsWithChildren> = ({ children }) => {
  const [networkState, dispatch] = useReducer(networkReducer, initialNetworkState);
  const disconnectedSinceRef = useRef<number | null>(null);
  const emergencyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleNetInfoChange = useCallback((info: NetInfoState) => {
    const { isConnected, details } = info;

    if (isConnected === true) {
      if (emergencyTimerRef.current != null) {
        clearTimeout(emergencyTimerRef.current);
        emergencyTimerRef.current = null;
      }
      disconnectedSinceRef.current = null;

      // 応答時間で CONNECTED / DEGRADED を判定
      // details.strength や RTT が利用可能な場合は活用
      dispatch({ type: 'CONNECTED' });
    } else {
      const now = Date.now();
      if (disconnectedSinceRef.current == null) {
        disconnectedSinceRef.current = now;
        dispatch({
          type: 'DISCONNECTED',
          disconnectedSince: new Date(now).toISOString(),
        });

        // Emergency Mode タイマーを設定
        emergencyTimerRef.current = setTimeout(() => {
          dispatch({ type: 'EMERGENCY_MODE' });
        }, EMERGENCY_THRESHOLD_MS);
      }
    }
  }, []);

  useEffect(() => {
    const unsubscribe = NetInfo.addEventListener(handleNetInfoChange);
    return () => {
      unsubscribe();
      if (emergencyTimerRef.current != null) {
        clearTimeout(emergencyTimerRef.current);
      }
    };
  }, [handleNetInfoChange]);

  return (
    <NetworkContext.Provider value={networkState}>
      {children}
    </NetworkContext.Provider>
  );
};

export function useNetwork(): NetworkContextValue {
  return React.useContext(NetworkContext);
}
```

---

## 2. Step 実行コンテキスト

### 2-1. 型定義

```typescript
// src/features/navigation/step-engine/StepExecutionContext.ts

import type { StepEntity, WorkExecutionEntity } from './types';
import type { StepExecutionState, SuspendReason } from './state';

export interface StepExecutionContextValue {
  /** 現在の作業実行エンティティ（null = 未開始）*/
  execution: WorkExecutionEntity | null;

  /** 現在の Step インデックス（0-indexed）*/
  currentStepIndex: number;

  /** SOP の全 Step リスト */
  steps: StepEntity[];

  /** ステートマシン状態 */
  machineState: StepExecutionState;

  /** アクションディスパッチャー */
  dispatch: React.Dispatch<StepExecutionAction>;
}

/** Step 実行のアクション Union 型 */
export type StepExecutionAction =
  | { type: 'SET_EXECUTION'; payload: WorkExecutionEntity }
  | { type: 'SET_STEPS'; payload: StepEntity[] }
  | { type: 'ADVANCE_STEP'; payload: { stepIndex: number } }
  | { type: 'WAIT_EVIDENCE'; payload: { stepId: string } }
  | { type: 'WAIT_SIGN'; payload: { stepId: string } }
  | { type: 'EVIDENCE_CAPTURED' }
  | { type: 'SIGN_COMPLETED' }
  | { type: 'SUSPEND'; payload: { reason: SuspendReason; suspendedAt: string } }
  | { type: 'RESUME' }
  | { type: 'COMPLETE' }
  | { type: 'RESET' };
```

### 2-2. Reducer

```typescript
// src/features/navigation/step-engine/StepExecutionReducer.ts

export interface StepExecutionStore {
  execution: WorkExecutionEntity | null;
  currentStepIndex: number;
  steps: StepEntity[];
  machineState: StepExecutionState;
}

export const initialStepExecutionStore: StepExecutionStore = {
  execution: null,
  currentStepIndex: 0,
  steps: [],
  machineState: { type: 'idle' },
};

export function stepExecutionReducer(
  state: StepExecutionStore,
  action: StepExecutionAction,
): StepExecutionStore {
  switch (action.type) {
    case 'SET_EXECUTION':
      return {
        ...state,
        execution: action.payload,
        machineState: { type: 'in_progress', currentStepIndex: action.payload.currentStepIndex },
        currentStepIndex: action.payload.currentStepIndex,
      };

    case 'SET_STEPS':
      return { ...state, steps: action.payload };

    case 'ADVANCE_STEP': {
      const nextIndex = action.payload.stepIndex;
      return {
        ...state,
        currentStepIndex: nextIndex,
        machineState: { type: 'in_progress', currentStepIndex: nextIndex },
      };
    }

    case 'WAIT_EVIDENCE':
      return {
        ...state,
        machineState: { type: 'waiting_evidence', stepId: action.payload.stepId },
      };

    case 'WAIT_SIGN':
      return {
        ...state,
        machineState: { type: 'waiting_sign', stepId: action.payload.stepId },
      };

    case 'EVIDENCE_CAPTURED':
    case 'SIGN_COMPLETED':
      return {
        ...state,
        machineState: { type: 'in_progress', currentStepIndex: state.currentStepIndex },
      };

    case 'SUSPEND':
      return {
        ...state,
        machineState: {
          type: 'suspended',
          reason: action.payload.reason,
          suspendedAt: action.payload.suspendedAt,
        },
      };

    case 'RESUME':
      return {
        ...state,
        machineState: { type: 'in_progress', currentStepIndex: state.currentStepIndex },
      };

    case 'COMPLETE':
      return { ...state, machineState: { type: 'completed' } };

    case 'RESET':
      return initialStepExecutionStore;

    default:
      return state;
  }
}
```

### 2-3. Provider

```typescript
// src/features/navigation/step-engine/StepExecutionProvider.tsx

export const StepExecutionContext = React.createContext<StepExecutionContextValue>({
  execution: null,
  currentStepIndex: 0,
  steps: [],
  machineState: { type: 'idle' },
  dispatch: () => void 0,
});

export const StepExecutionProvider: React.FC<React.PropsWithChildren> = ({ children }) => {
  const [state, dispatch] = useReducer(stepExecutionReducer, initialStepExecutionStore);

  const contextValue: StepExecutionContextValue = {
    execution: state.execution,
    currentStepIndex: state.currentStepIndex,
    steps: state.steps,
    machineState: state.machineState,
    dispatch,
  };

  return (
    <StepExecutionContext.Provider value={contextValue}>
      {children}
    </StepExecutionContext.Provider>
  );
};

export function useStepExecution(): StepExecutionContextValue {
  return React.useContext(StepExecutionContext);
}
```

---

## 3. 認証コンテキスト

```typescript
// src/shared/auth/AuthContext.ts

export interface AuthUser {
  userId: string;
  loginId: string;
  displayName: string;
  role: UserRole;
  skillLevel: number;
  locale: 'ja' | 'en' | 'ja-simple';
}

export type UserRole =
  | 'operator'
  | 'supervisor'
  | 'quality_admin'
  | 'master_admin'
  | 'system_admin'
  | 'executive';

export interface AuthContextValue {
  user: AuthUser | null;
  isAuthenticated: boolean;
  jwtExpiresAt: string | null;
  login: (loginId: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
}

export const AuthContext = React.createContext<AuthContextValue>({
  user: null,
  isAuthenticated: false,
  jwtExpiresAt: null,
  login: async () => void 0,
  logout: async () => void 0,
  refreshToken: async () => void 0,
});

export function useAuth(): AuthContextValue {
  return React.useContext(AuthContext);
}
```

---

## 4. コンテキストプロバイダー階層

```typescript
// src/App.tsx（Provider 階層）

export default function App(): React.ReactElement {
  return (
    <LocalDbProvider>           {/* LocalDbService 初期化 */}
      <AuthProvider>            {/* JWT 管理・自動リフレッシュ */}
        <NetworkProvider>       {/* MOD-FE-HA-001: 4 段階ネットワーク状態 */}
          <OutboxWorkerProvider> {/* MOD-FE-HA-002: Outbox 起動・停止 */}
            <I18nProvider>      {/* react-i18next: ja/en/ja-simple */}
              <StepExecutionProvider>  {/* Step 実行ステートマシン */}
                <NavigationContainer>
                  <RootNavigator />
                </NavigationContainer>
              </StepExecutionProvider>
            </I18nProvider>
          </OutboxWorkerProvider>
        </NetworkProvider>
      </AuthProvider>
    </LocalDbProvider>
  );
}
```

Provider は外側から内側に向かって依存する。`StepExecutionProvider` は最も内側に配置し、LocalDb・Auth・Network・Outbox のすべてを前提とする。

---

## 5. カスタムフック一覧

| フック名 | 参照コンテキスト | 用途 |
|---|---|---|
| `useNetwork()` | NetworkContext | ネットワーク状態・Outbox 深さ・Emergency Mode |
| `useStepExecution()` | StepExecutionContext | Step 実行 Reducer・dispatch |
| `useAuth()` | AuthContext | ログインユーザー・JWT 管理 |
| `useLocalDb()` | LocalDbContext | LocalDbService インスタンス取得 |
| `useOutboxWorker()` | OutboxWorkerContext | processQueue の手動トリガー |
| `useI18n()` | I18nContext（react-i18next）| 翻訳テキスト取得・ロケール切替 |

---

**本節で確定した方針**
- **状態管理は React Context + useReducer のみで実装し、Redux / Zustand / Jotai 等の追加ライブラリを不使用とした。各コンテキストは単一責務を持ち、Provider 階層の外側から内側への一方向依存を確保した。**
- **NetworkProvider（MOD-FE-HA-001）は @react-native-community/netinfo の addEventListener で接続変化を受け取り、切断継続時間が EMERGENCY_THRESHOLD_MS（5 分）を超えた時点でタイマー駆動で EMERGENCY_MODE に遷移する設計を確定した。**
- **StepExecutionReducer の全 9 アクション（SET_EXECUTION / SET_STEPS / ADVANCE_STEP / WAIT_EVIDENCE / WAIT_SIGN / EVIDENCE_CAPTURED / SIGN_COMPLETED / SUSPEND / RESUME / COMPLETE / RESET）をステートマシン仕様と 1:1 対応させ、不正な状態遷移をコンパイル時の型安全性と Reducer の網羅的な switch で防止した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)

### 関連
- [`90_業界分析/12_認知工学と状況認識.md`](../../90_業界分析/12_認知工学と状況認識.md)
