// 対応 §: ロードマップ §20.1 §3.6.4 §11.2
// API 失敗時のリカバリ導線をひとつに集約する。
// 既に i18n 済みの ApiError 文言を受け取り、再試行ボタンと閉じるボタンを提供する。

import { useState } from 'react';
import { t } from '../../i18n';
import { palette, fontSize, fontWeight, radius, space } from '../../tokens/access';
import { Icon } from '../components/icon/icon';

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
        padding: space[3],
        background: palette.danger.subtle,
        color: palette.danger.strong,
        borderRadius: radius.medium,
        border: `1px solid ${palette.danger.default}`,
        display: 'grid',
        gap: space[2]
      }}
    >
      <div style={{ display: 'flex', gap: space[2], alignItems: 'flex-start' }}>
        <Icon name="warning-triangle" size={20} color={palette.danger.default} />
        <span style={{ flex: 1, fontSize: props.inline === true ? fontSize.caption : fontSize.body }}>
          {props.message}
        </span>
      </div>
      <div style={{ display: 'flex', gap: space[2], justifyContent: 'flex-end', flexWrap: 'wrap' }}>
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
            padding: space[2],
            background: palette.white,
            color: palette.neutral[700],
            fontSize: fontSize.caption,
            overflow: 'auto',
            borderRadius: radius.small
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
  padding: `${space[1]} ${space[3]}`,
  background: 'transparent',
  color: palette.danger.strong,
  border: `1px solid ${palette.danger.strong}`,
  borderRadius: radius.medium,
  cursor: 'pointer',
  fontSize: fontSize.caption
};

const primaryButtonStyle: React.CSSProperties = {
  minHeight: 32,
  padding: `${space[1]} ${space[3]}`,
  background: palette.danger.strong,
  color: palette.white,
  border: 'none',
  borderRadius: radius.medium,
  cursor: 'pointer',
  fontSize: fontSize.caption,
  fontWeight: fontWeight.bold
};
