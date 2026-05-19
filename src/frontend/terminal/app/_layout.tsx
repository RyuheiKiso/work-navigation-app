// TypeORM のデコレータは reflect-metadata に依存するため最初にインポートする
import 'reflect-metadata';
import React, { useEffect, useState } from 'react';
import { Slot } from 'expo-router';
import * as SplashScreen from 'expo-splash-screen';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { I18nextProvider } from 'react-i18next';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { ErrorBoundary } from '../src/components/ErrorBoundary';
import { initDatabase } from '../db/data-source';
import { i18n, initI18n } from '../i18n';
// 開発環境でのみ MSW を起動する（CLAUDE.md §ADR: MSW msw/native 使用）
import { server } from '@wnav/shared/mocks/native';
if (__DEV__) {
  server.listen({ onUnhandledRequest: 'bypass' });
}
import { AuthProvider } from '../contexts/AuthContext';
import { NetworkProvider } from '../contexts/NetworkContext';
import { LocaleProvider } from '../contexts/LocaleContext';
import { OutboxProvider } from '../contexts/OutboxContext';
import { WorkExecutionProvider } from '../contexts/WorkExecutionContext';

// スプラッシュ表示中に DB/i18n を確実に初期化したいので自動非表示を一時停止する
SplashScreen.preventAutoHideAsync().catch(() => undefined);

// TanStack Query のリトライ・キャッシュ方針は端末アプリの Offline-First を前提に設定する
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 3,
      staleTime: 1000 * 60 * 5,
      refetchOnWindowFocus: false,
    },
  },
});

export default function RootLayout(): JSX.Element {
  const [ready, setReady] = useState<boolean>(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        await initDatabase();
        await initI18n();
      } finally {
        if (!cancelled) {
          setReady(true);
          await SplashScreen.hideAsync().catch(() => undefined);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (!ready) {
    return <></>;
  }

  return (
    <ErrorBoundary>
      <SafeAreaProvider>
        <QueryClientProvider client={queryClient}>
          <I18nextProvider i18n={i18n}>
            <LocaleProvider>
              <AuthProvider>
                <NetworkProvider>
                  <OutboxProvider>
                    <WorkExecutionProvider>
                      <Slot />
                    </WorkExecutionProvider>
                  </OutboxProvider>
                </NetworkProvider>
              </AuthProvider>
            </LocaleProvider>
          </I18nextProvider>
        </QueryClientProvider>
      </SafeAreaProvider>
    </ErrorBoundary>
  );
}
