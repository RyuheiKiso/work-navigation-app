// 対応 §: ロードマップ §11.4 §3.6.4 §20.1
// React のレンダリング中に発生した非同期外の例外を捕捉し、現場ユーザーに
// 「画面が真っ白で何もできない」状態を作らない最後の防波堤。
// children のレンダ失敗時は title/description と再読込ボタンを描画する。

import { Component, type ErrorInfo, type ReactNode } from 'react';
import { t } from '../../i18n';

export interface ErrorBoundaryProps {
  children: ReactNode;
  /** ログ出力先 (Sentry 等) を差し替えるためのフック。未指定なら開発時のみ console.error */
  onError?: (error: Error, info: ErrorInfo) => void;
}

interface ErrorBoundaryState {
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  override state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  override componentDidCatch(error: Error, info: ErrorInfo): void {
    if (this.props.onError) {
      this.props.onError(error, info);
      return;
    }
    // 既定では console に出す。Phase 2 で Sentry/RUM へ差し替える前提のフック点
    // eslint-disable-next-line no-console
    console.error('[ErrorBoundary]', error, info.componentStack);
  }

  private handleReload = (): void => {
    if (typeof window !== 'undefined') window.location.reload();
  };

  override render(): ReactNode {
    if (!this.state.error) return this.props.children;
    return (
      <div
        role="alert"
        aria-live="assertive"
        style={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: 24,
          background: '#F8D7DA',
          color: '#721C24',
          fontFamily: 'Inter, "Noto Sans JP", system-ui, sans-serif'
        }}
      >
        <div style={{ maxWidth: 480, textAlign: 'center' }}>
          <h1 style={{ fontSize: 24, marginTop: 0 }}>{t('error.boundary_title')}</h1>
          <p style={{ fontSize: 16, lineHeight: 1.6 }}>{t('error.boundary_description')}</p>
          <button
            type="button"
            onClick={this.handleReload}
            style={{
              minHeight: 48,
              minWidth: 160,
              marginTop: 24,
              padding: '12px 24px',
              background: '#721C24',
              color: '#FFFFFF',
              border: 'none',
              borderRadius: 8,
              cursor: 'pointer',
              fontSize: 16
            }}
          >
            {t('error.reload')}
          </button>
        </div>
      </div>
    );
  }
}
