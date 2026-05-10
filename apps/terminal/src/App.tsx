// 対応 §: ロードマップ §7.1 §10.5 §11.3.1
// 端末作業ナビアプリのルート: ログイン状態を管理し、ナビシェルへ。

import { useEffect, useState } from 'react';
import { LoginScreen } from './presentation/components/login-screen';
import { NavigationShell } from './presentation/components/navigation-shell';
import { getCurrentUser } from './adapter/api-client';

/** ルート */
export function App(): JSX.Element {
  const [user, setUser] = useState<{ user_id: string; display_name: string } | null>(null);

  useEffect(() => {
    const u = getCurrentUser();
    if (u) setUser(u);
  }, []);

  if (!user) {
    return <LoginScreen onLoggedIn={(u) => setUser(u)} />;
  }
  return <NavigationShell user={user} onLogout={() => setUser(null)} />;
}
