// 対応 §: ロードマップ §3.6.4 §11.2
// 0 件状態を「初期化中」と区別する。アイコン + 主文 + 補助文 + 任意のアクション。

import type { ReactNode } from 'react';

export interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
  action?: ReactNode;
  inline?: boolean;
}

export function EmptyState(props: EmptyStateProps): JSX.Element {
  return (
    <div
      role="status"
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 8,
        padding: props.inline ? 16 : 32,
        color: '#6C757D',
        textAlign: 'center'
      }}
    >
      {props.icon !== undefined && (
        <div aria-hidden="true" style={{ fontSize: props.inline ? 24 : 36 }}>
          {props.icon}
        </div>
      )}
      <strong style={{ fontSize: props.inline ? 13 : 16, color: '#212529' }}>{props.title}</strong>
      {props.description !== undefined && (
        <span style={{ fontSize: 12, maxWidth: 320 }}>{props.description}</span>
      )}
      {props.action !== undefined && <div style={{ marginTop: 8 }}>{props.action}</div>}
    </div>
  );
}
