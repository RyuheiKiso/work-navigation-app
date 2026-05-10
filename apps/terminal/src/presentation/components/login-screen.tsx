// 対応 §: ロードマップ §10.5 §11.2 §9.5
// 端末ログイン画面: ID＋パスワード（§10.5.0）+ QR ペアリング URL 表示。

import { useState } from 'react';
import { login, getBackendUrl, setBackendUrl } from '../../adapter/api-client';
import { toApiError } from '../../adapter/api-error';
import { t } from '../../i18n';
import { palette, fontSize, fontWeight, radius, space, elevation, fontStack } from '../../tokens/access';
import { Icon } from './icon/icon';

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

  const inputStyle: React.CSSProperties = {
    width: '100%',
    padding: space[3],
    fontSize: fontSize.body,
    marginTop: space[1],
    border: `1px solid ${palette.neutral[300]}`,
    borderRadius: radius.medium,
    background: palette.white,
    color: palette.neutral[900]
  };

  return (
    <main
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '100vh',
        background: palette.neutral[50],
        fontFamily: fontStack
      }}
    >
      <form
        onSubmit={(e) => void handleSubmit(e)}
        style={{
          width: 400,
          padding: space[6],
          background: palette.white,
          borderRadius: radius.large,
          boxShadow: elevation[3]
        }}
      >
        <h1
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[2],
            margin: 0,
            fontSize: fontSize.title,
            fontWeight: fontWeight.bold,
            color: palette.neutral[900]
          }}
        >
          <Icon name="lock-closed" size={28} color={palette.info.default} />
          {t('login.title')}
        </h1>
        <p style={{ color: palette.neutral[600], fontSize: fontSize.caption, marginTop: space[1] }}>
          {t('login.subtitle')}
        </p>
        {demo.isDemo && (
          <div
            role="status"
            style={{
              marginTop: space[2],
              padding: `${space[2]} ${space[3]}`,
              borderRadius: radius.medium,
              background: palette.warning.subtle,
              color: palette.warning.strong,
              border: `1px solid ${palette.warning.default}`,
              fontSize: fontSize.caption,
              fontWeight: fontWeight.bold,
              display: 'inline-flex',
              alignItems: 'center',
              gap: space[2]
            }}
          >
            <Icon name="warning-triangle" size={16} color={palette.warning.strong} />
            {t('login.demo_banner')}
          </div>
        )}
        <label style={{ display: 'block', marginTop: space[4], fontSize: fontSize.caption, color: palette.neutral[700] }}>
          {t('login.backend_url_label')}
          <input value={backend} onChange={(e) => setBackend(e.target.value)} style={inputStyle} />
        </label>
        <label style={{ display: 'block', marginTop: space[3], fontSize: fontSize.caption, color: palette.neutral[700] }}>
          {t('login.user_id_label')}
          <input
            value={userId}
            onChange={(e) => setUserId(e.target.value)}
            autoComplete="username"
            style={inputStyle}
          />
        </label>
        <label style={{ display: 'block', marginTop: space[3], fontSize: fontSize.caption, color: palette.neutral[700] }}>
          {t('login.password_label')}
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="current-password"
            style={inputStyle}
          />
        </label>
        {error && (
          <div
            role="alert"
            style={{
              marginTop: space[3],
              padding: space[3],
              background: palette.danger.subtle,
              color: palette.danger.strong,
              borderRadius: radius.medium,
              fontSize: fontSize.caption,
              border: `1px solid ${palette.danger.default}`
            }}
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
            marginTop: space[4],
            padding: space[3],
            fontSize: fontSize.body,
            background: busy ? palette.neutral[400] : palette.success.default,
            color: palette.white,
            border: 'none',
            borderRadius: radius.medium,
            cursor: busy ? 'wait' : 'pointer',
            fontWeight: fontWeight.medium
          }}
        >
          {busy ? t('login.submit_busy') : t('login.submit')}
        </button>
        {demo.isDemo && (
          <p style={{ marginTop: space[4], fontSize: fontSize.caption, color: palette.neutral[600] }}>
            {t('login.demo_users')}
          </p>
        )}
      </form>
    </main>
  );
}
