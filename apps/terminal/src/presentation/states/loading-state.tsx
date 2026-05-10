// 対応 §: ロードマップ §3.6.4 §11.2 §14.2
// 「待ってるのか壊れてるのか分からない」体感低下を防ぐローディング表示。
// aria-live で screenreader 利用者にも進行中であることを通知する。

export interface LoadingStateProps {
  label: string;
  /** 控えめ表示 (リスト内など) */
  inline?: boolean;
}

export function LoadingState(props: LoadingStateProps): JSX.Element {
  const size = props.inline ? 16 : 32;
  return (
    <div
      role="status"
      aria-live="polite"
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: props.inline ? 'flex-start' : 'center',
        gap: 12,
        padding: props.inline ? 8 : 24,
        color: '#6C757D'
      }}
    >
      <span
        aria-hidden="true"
        style={{
          width: size,
          height: size,
          border: '3px solid #DEE2E6',
          borderTopColor: '#28A745',
          borderRadius: '50%',
          animation: 'wna-spin 0.9s linear infinite',
          display: 'inline-block'
        }}
      />
      <span style={{ fontSize: props.inline ? 12 : 14 }}>{props.label}</span>
      <style>{`@keyframes wna-spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}
