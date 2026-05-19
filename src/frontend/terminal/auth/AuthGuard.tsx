// 未認証時に /(auth)/login へリダイレクトするルートガード
import React, { useEffect } from 'react';
import { Redirect } from 'expo-router';
import { useAuth } from '../contexts/AuthContext';

export function AuthGuard({ children }: { children: React.ReactNode }): JSX.Element {
  const { state } = useAuth();

  useEffect(() => {
    // isLoading 中は redirect を遅延する（Splash 表示時の競合回避）
  }, [state.isAuthenticated, state.isLoading]);

  if (!state.isAuthenticated) {
    return <Redirect href="/(auth)/login" />;
  }
  return <>{children}</>;
}
