// 対応 §: ロードマップ §7.1 §10.5 §11.3.1
// 端末作業ナビアプリのルート: ログイン状態を管理し、ナビシェルへ。

import { useEffect, useState } from 'react';
import { LoginScreen } from './presentation/components/login-screen';
import { NavigationShell } from './presentation/components/navigation-shell';
import { getCurrentUser } from './adapter/api-client';
import { useDocumentLocale } from './presentation/hooks/use-document-locale';
import { useWebVitals } from './presentation/hooks/use-web-vitals';

/** ルート */
export function App(): JSX.Element {
  const [user, setUser] = useState<{ user_id: string; display_name: string } | null>(null);
  // ログイン画面段階から RTL/フォント選択を正しく動作させる
  useDocumentLocale();
  // §31.1 SLO ベースライン取得：LCP/INP/CLS/FCP/TTFB を計測する
  useWebVitals();

  useEffect(() => {
    const u = getCurrentUser();
    if (u) setUser(u);
  }, []);

  if (!user) {
    return <LoginScreen onLoggedIn={(u) => setUser(u)} />;
  }
  return <NavigationShell user={user} onLogout={() => setUser(null)} />;
}
