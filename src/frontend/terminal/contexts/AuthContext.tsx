// AuthContext。JWT トークンとユーザー情報を保持し React Context + useReducer で管理する
import React, { createContext, useCallback, useContext, useMemo, useReducer } from 'react';
import type { UserRole, Locale } from '@wnav/shared';

export interface AuthUser {
  userId: string;
  displayName: string;
  role: UserRole;
  roles: UserRole[];
  locale: Locale;
  factoryId: string;
}

export interface AuthState {
  token: string | null;
  refreshToken: string | null;
  user: AuthUser | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export type AuthAction =
  | { type: 'LOGIN_START' }
  | { type: 'LOGIN_SUCCESS'; payload: { token: string; refreshToken: string; user: AuthUser } }
  | { type: 'LOGIN_FAILURE'; payload: { errCode: string; message: string } }
  | { type: 'REFRESH_SUCCESS'; payload: { token: string } }
  | { type: 'LOGOUT' };

const initialState: AuthState = {
  token: null,
  refreshToken: null,
  user: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,
};

// useReducer の純粋関数として副作用なくステート遷移を表現する
export const authReducer = (state: AuthState, action: AuthAction): AuthState => {
  switch (action.type) {
    case 'LOGIN_START':
      return { ...state, isLoading: true, error: null };
    case 'LOGIN_SUCCESS':
      return {
        ...state,
        isLoading: false,
        token: action.payload.token,
        refreshToken: action.payload.refreshToken,
        user: action.payload.user,
        isAuthenticated: true,
        error: null,
      };
    case 'LOGIN_FAILURE':
      return {
        ...state,
        isLoading: false,
        error: action.payload.message,
        isAuthenticated: false,
      };
    case 'REFRESH_SUCCESS':
      return { ...state, token: action.payload.token };
    case 'LOGOUT':
      return initialState;
  }
};

interface AuthContextValue {
  state: AuthState;
  dispatch: React.Dispatch<AuthAction>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }): JSX.Element {
  const [state, dispatch] = useReducer(authReducer, initialState);
  const logout = useCallback(() => dispatch({ type: 'LOGOUT' }), []);
  const value = useMemo(() => ({ state, dispatch, logout }), [state, logout]);
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (ctx === null) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
