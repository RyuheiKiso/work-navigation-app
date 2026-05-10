// 対応 §: ロードマップ §10.5 §11.2 §9.5
// 端末ログイン画面: ID＋パスワード（§10.5.0）+ QR ペアリング URL 表示。

import { useState } from 'react';
import { login, getBackendUrl, setBackendUrl } from '../../adapter/api-client';
import { toApiError } from '../../adapter/api-error';
import { t } from '../../i18n';

export interface LoginScreenProps {
  onLoggedIn(user: { user_id: string; display_name: string }): void;
}

/**
 * Vite の `import.meta.env` から既定値を読む。
 * - VITE_DEMO_MODE=true のときだけ user_id/password の既定値を埋める
 * - 本番ビルドではデモ値が出ない（誤って alice/hello-world で出荷する事故を防ぐ）
 */
function readDemoDefaults(): { userId: string; password: string; isDemo: boolean } {
  // import.meta.env は Vite ビルド時に静的置換される
  const env = (import.meta as { env?: Record<string, string | undefined> }).env ?? {};
  const isDemo = env.VITE_DEMO_MODE === 'true';
  return {
    userId: isDemo ? env.VITE_DEMO_USER_ID ?? 'alice' : '',
    password: isDemo ? env.VITE_DEMO_PASSWORD ?? 'hello-world' : '',
    isDemo
  };
}

export function LoginScreen(props: LoginScreenProps): JSX.Element {
  const demo = readDemoDefaults();
  const [userId, setUserId] = useState(demo.userId);
  const [password, setPassword] = useState(demo.password);
  const [backend, setBackend] = useState(getBackendUrl());
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function handleSubmit(e: React.FormEvent): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      setBackendUrl(backend);
      const user = await login(userId, password);
      props.onLoggedIn({ user_id: user.user_id, display_name: user.display_name });
    } catch (err) {
      // §20.1 「人を責めない」表現: ApiError 分類経由でユーザー文言を引く
      setError(t(toApiError(err).i18nKey()));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '100vh',
        background: '#F8F9FA',
        fontFamily: 'Inter, "Noto Sans JP", system-ui, sans-serif'
      }}
    >
      <form
        onSubmit={(e) => void handleSubmit(e)}
        style={{
          width: 400,
          padding: 32,
          background: '#FFFFFF',
          borderRadius: 16,
          boxShadow: '0 10px 25px rgba(13,17,23,0.10)'
        }}
      >
        <h1 style={{ fontSize: 24, marginTop: 0 }}>🔐 {t('login.title')}</h1>
        <p style={{ color: '#6C757D', fontSize: 13 }}>{t('login.subtitle')}</p>
        {demo.isDemo && (
          <div
            role="status"
            style={{
              marginTop: 8,
              padding: '6px 10px',
              borderRadius: 6,
              background: '#FFF3CD',
              color: '#856404',
              border: '1px solid #FFEEBA',
              fontSize: 12,
              fontWeight: 600
            }}
          >
            ⚠ {t('login.demo_banner')}
          </div>
        )}
        <label style={{ display: 'block', marginTop: 16, fontSize: 14 }}>
          {t('login.backend_url_label')}
          <input
            value={backend}
            onChange={(e) => setBackend(e.target.value)}
            style={{ width: '100%', padding: 10, fontSize: 14, marginTop: 4 }}
          />
        </label>
        <label style={{ display: 'block', marginTop: 12, fontSize: 14 }}>
          {t('login.user_id_label')}
          <input
            value={userId}
            onChange={(e) => setUserId(e.target.value)}
            autoComplete="username"
            style={{ width: '100%', padding: 10, fontSize: 16, marginTop: 4 }}
          />
        </label>
        <label style={{ display: 'block', marginTop: 12, fontSize: 14 }}>
          {t('login.password_label')}
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="current-password"
            style={{ width: '100%', padding: 10, fontSize: 16, marginTop: 4 }}
          />
        </label>
        {error && (
          <div
            style={{
              marginTop: 12,
              padding: 10,
              background: '#F8D7DA',
              color: '#721C24',
              borderRadius: 6,
              fontSize: 13
            }}
            role="alert"
          >
            {error}
          </div>
        )}
        <button
          type="submit"
          disabled={busy}
          style={{
            width: '100%',
            minHeight: 48,
            marginTop: 16,
            padding: 12,
            fontSize: 16,
            background: busy ? '#ADB5BD' : '#28A745',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: 8,
            cursor: busy ? 'wait' : 'pointer'
          }}
        >
          {busy ? t('login.submit_busy') : t('login.submit')}
        </button>
        {demo.isDemo && (
          <p style={{ marginTop: 16, fontSize: 12, color: '#6C757D' }}>
            {t('login.demo_users')}
          </p>
        )}
      </form>
    </main>
  );
}
