// 対応 §: ロードマップ §20.1 §3.6.4 §11.2
// API 失敗時のリカバリ導線をひとつに集約する。
// 既に i18n 済みの ApiError 文言を受け取り、再試行ボタンと閉じるボタンを提供する。

import { useState } from 'react';
import { t } from '../../i18n';

export interface ErrorPanelProps {
  /** すでに t() 済みのユーザー向けメッセージ */
  message: string;
  /** 任意。技術詳細（status 等）。折りたたみで表示する */
  detail?: string;
  /** 任意。再試行可能な場合に渡す */
  onRetry?: () => void;
  /** 任意。閉じるボタンを出す */
  onDismiss?: () => void;
  inline?: boolean;
}

export function ErrorPanel(props: ErrorPanelProps): JSX.Element {
  const [showDetail, setShowDetail] = useState(false);
  return (
    <div
      role="alert"
      aria-live="assertive"
      style={{
        padding: 12,
        background: '#F8D7DA',
        color: '#721C24',
        borderRadius: 8,
        border: '1px solid #F5C6CB',
        display: 'grid',
        gap: 8
      }}
    >
      <div style={{ display: 'flex', gap: 8, alignItems: 'flex-start' }}>
        <span aria-hidden="true">⚠️</span>
        <span style={{ flex: 1, fontSize: props.inline ? 13 : 14 }}>{props.message}</span>
      </div>
      <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', flexWrap: 'wrap' }}>
        {props.detail !== undefined && (
          <button
            type="button"
            onClick={() => setShowDetail((v) => !v)}
            style={detailButtonStyle}
            aria-expanded={showDetail}
          >
            {showDetail ? t('error.hide_detail') : t('error.show_detail')}
          </button>
        )}
        {props.onDismiss !== undefined && (
          <button type="button" onClick={props.onDismiss} style={detailButtonStyle}>
            {t('error.dismiss')}
          </button>
        )}
        {props.onRetry !== undefined && (
          <button type="button" onClick={props.onRetry} style={primaryButtonStyle}>
            {t('error.retry')}
          </button>
        )}
      </div>
      {showDetail && props.detail !== undefined && (
        <pre
          style={{
            margin: 0,
            padding: 8,
            background: '#FFFFFF',
            color: '#495057',
            fontSize: 11,
            overflow: 'auto',
            borderRadius: 4
          }}
        >
          {props.detail}
        </pre>
      )}
    </div>
  );
}

const detailButtonStyle: React.CSSProperties = {
  minHeight: 32,
  padding: '4px 10px',
  background: 'transparent',
  color: '#721C24',
  border: '1px solid #721C24',
  borderRadius: 6,
  cursor: 'pointer',
  fontSize: 12
};

const primaryButtonStyle: React.CSSProperties = {
  minHeight: 32,
  padding: '4px 12px',
  background: '#721C24',
  color: '#FFFFFF',
  border: 'none',
  borderRadius: 6,
  cursor: 'pointer',
  fontSize: 12,
  fontWeight: 600
};
