// 対応 §: ロードマップ §3.6.4 §11.2 §14.2
// 「待ってるのか壊れてるのか分からない」体感低下を防ぐローディング表示。
// aria-live で screenreader 利用者にも進行中であることを通知する。

import { palette, fontSize, space } from '../../tokens/access';

export interface LoadingStateProps {
  label: string;
  /** 控えめ表示 (リスト内など) */
  inline?: boolean;
}

export function LoadingState(props: LoadingStateProps): JSX.Element {
  const size = props.inline === true ? 16 : 32;
  return (
    <div
      role="status"
      aria-live="polite"
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: props.inline === true ? 'flex-start' : 'center',
        gap: space[3],
        padding: props.inline === true ? space[2] : space[5],
        color: palette.neutral[600]
      }}
    >
      <span
        aria-hidden="true"
        style={{
          width: size,
          height: size,
          border: `3px solid ${palette.neutral[200]}`,
          borderTopColor: palette.success.default,
          borderRadius: '50%',
          animation: 'wna-spin 0.9s linear infinite',
          display: 'inline-block'
        }}
      />
      <span style={{ fontSize: props.inline === true ? fontSize.caption : fontSize.body }}>{props.label}</span>
      <style>{`@keyframes wna-spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}
