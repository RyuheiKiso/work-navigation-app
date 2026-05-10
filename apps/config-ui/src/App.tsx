// 対応 §: ロードマップ §7.2 §10.2 §10.5
// 設定 UI ルート。ログイン → AppShell。

import { useEffect, useState } from 'react';
import { LoginScreen } from './presentation/components/login-screen';
import { AppShell } from './presentation/components/app-shell';
import { getCurrentUser } from './adapter/api-client';
import { useWebVitals } from './presentation/hooks/use-web-vitals';

export function App(): JSX.Element {
  const [user, setUser] = useState<{ user_id: string; display_name: string } | null>(null);
  // §31.1 SLO ベースライン取得：LCP/INP/CLS/FCP/TTFB を計測する
  useWebVitals();
  useEffect(() => {
    const u = getCurrentUser();
    if (u) setUser(u);
  }, []);
  if (!user) return <LoginScreen onLoggedIn={(u) => setUser(u)} />;
  return <AppShell user={user} onLogout={() => setUser(null)} />;
}
