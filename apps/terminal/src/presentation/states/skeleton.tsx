// 対応 §: ロードマップ §3.6.4 §11.2 §14.2
// シマー付きスケルトン。`global.css` の `.wna-skeleton` keyframe を利用する。
// ローディング時にもレイアウトが保持されることで「次に何が出るか」を予告する。
// 装飾モーション扱いとし、`data-motion="decoration"` で reduced-motion 時は静止する。

import type { CSSProperties } from 'react';
import { radius, space } from '../../tokens/access';

export interface SkeletonProps {
  width?: number | string;
  height?: number | string;
  // 角丸スタイル: pill は完全に丸く、small/medium は token と一致
  shape?: 'pill' | 'small' | 'medium' | 'large';
  style?: CSSProperties;
  ariaLabel?: string;
}

function radiusFor(shape: SkeletonProps['shape']): string {
  if (shape === 'pill') return radius.pill;
  if (shape === 'medium') return radius.medium;
  if (shape === 'large') return radius.large;
  return radius.small;
}

export function Skeleton(props: SkeletonProps): JSX.Element {
  return (
    <div
      className="wna-skeleton"
      data-motion="decoration"
      role="presentation"
      aria-hidden={props.ariaLabel === undefined ? true : undefined}
      aria-label={props.ariaLabel}
      style={{
        width: props.width ?? '100%',
        height: props.height ?? 16,
        borderRadius: radiusFor(props.shape),
        ...props.style
      }}
    />
  );
}

// ありがちなパターン: 縦並びのテキスト行をモック
export interface SkeletonTextProps {
  lines?: number;
  lineHeight?: number;
  lastLineWidth?: number | string;
}

export function SkeletonText(props: SkeletonTextProps): JSX.Element {
  const lines = props.lines ?? 3;
  const lh = props.lineHeight ?? 14;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: space[2] }}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          height={lh}
          width={i === lines - 1 ? props.lastLineWidth ?? '60%' : '100%'}
        />
      ))}
    </div>
  );
}

// タスクドロワー用: カード形を保ったまま list 件数を埋める
export function SkeletonTaskCard(): JSX.Element {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: space[2],
        padding: space[3],
        border: '1px solid var(--wna-c-border)',
        borderRadius: radius.medium
      }}
    >
      <Skeleton height={14} width="70%" />
      <Skeleton height={10} width="40%" />
    </div>
  );
}
