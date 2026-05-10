// 対応 §: ロードマップ §10.5 §11.2 §10.2.2
// 設定 UI のログイン画面。

import { useState } from 'react';
import { login, getBackendUrl, setBackendUrl } from '../../adapter/api-client';
import { toApiError } from '../../adapter/api-error';
import { t } from '../../i18n';

export interface LoginScreenProps {
  onLoggedIn(user: { user_id: string; display_name: string }): void;
}

export function LoginScreen(props: LoginScreenProps): JSX.Element {
  const [userId, setUserId] = useState('charlie');
  const [password, setPassword] = useState('hello-world');
  const [backend, setBackend] = useState(getBackendUrl());
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function handleSubmit(e: React.FormEvent): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      setBackendUrl(backend);
      const u = await login(userId, password);
      props.onLoggedIn({ user_id: u.user_id, display_name: u.display_name });
    } catch (err) {
      setError(t(toApiError(err).i18nKey()));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main
      style={{
        display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: '100vh',
        background: '#F8F9FA', fontFamily: 'Inter, "Noto Sans JP", system-ui, sans-serif'
      }}
    >
      <form
        onSubmit={(e) => void handleSubmit(e)}
        style={{
          width: 420, padding: 32, background: '#FFFFFF', borderRadius: 16,
          boxShadow: '0 10px 25px rgba(13,17,23,0.10)'
        }}
      >
        <h1 style={{ fontSize: 24, marginTop: 0 }}>⚙️ 設定 UI ログイン</h1>
        <p style={{ color: '#6C757D', fontSize: 13 }}>work-navigation-app 設定 Web UI</p>
        <label style={{ display: 'block', marginTop: 16, fontSize: 14 }}>
          バックエンド URL
          <input value={backend} onChange={(e) => setBackend(e.target.value)}
            style={{ width: '100%', padding: 10, fontSize: 14, marginTop: 4 }} />
        </label>
        <label style={{ display: 'block', marginTop: 12, fontSize: 14 }}>
          ユーザ ID
          <input value={userId} onChange={(e) => setUserId(e.target.value)}
            autoComplete="username"
            style={{ width: '100%', padding: 10, fontSize: 16, marginTop: 4 }} />
        </label>
        <label style={{ display: 'block', marginTop: 12, fontSize: 14 }}>
          パスワード
          <input type="password" value={password} onChange={(e) => setPassword(e.target.value)}
            autoComplete="current-password"
            style={{ width: '100%', padding: 10, fontSize: 16, marginTop: 4 }} />
        </label>
        {error && (
          <div style={{ marginTop: 12, padding: 10, background: '#F8D7DA', color: '#721C24', borderRadius: 6, fontSize: 13 }} role="alert">
            {error}
          </div>
        )}
        <button type="submit" disabled={busy}
          style={{
            width: '100%', minHeight: 48, marginTop: 16, padding: 12, fontSize: 16,
            background: busy ? '#ADB5BD' : '#28A745', color: '#FFFFFF',
            border: 'none', borderRadius: 8, cursor: busy ? 'wait' : 'pointer'
          }}>
          {busy ? '...' : 'ログイン'}
        </button>
        <p style={{ marginTop: 16, fontSize: 12, color: '#6C757D' }}>
          デモ: alice / bob / charlie （パスワード: hello-world）
        </p>
      </form>
    </main>
  );
}
