// 対応 §: ロードマップ §3.6.4 §11.2 §11.2.2
// 0 件状態を「初期化中」と区別する。アイコン + 主文 + 補助文 + 任意のアクション。
// 形状つき SVG (`iconName`) を推奨。後方互換のため文字列 `icon` も受け付ける（テスト互換）。

import type { ReactNode } from 'react';
import { palette, fontSize, fontWeight, space } from '../../tokens/access';
import { Icon } from '../components/icon/icon';
import type { IconName } from '../components/icon/glyphs';

export interface EmptyStateProps {
  // 形状で識別可能な SVG 名（推奨）
  iconName?: IconName;
  // 後方互換: 既存の絵文字／任意文字列
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
        gap: space[2],
        padding: props.inline === true ? space[4] : space[6],
        color: palette.neutral[600],
        textAlign: 'center'
      }}
    >
      {props.iconName !== undefined ? (
        <Icon name={props.iconName} size={props.inline === true ? 24 : 40} color={palette.neutral[500]} />
      ) : props.icon !== undefined ? (
        <div aria-hidden="true" style={{ fontSize: props.inline === true ? 24 : 36 }}>
          {props.icon}
        </div>
      ) : null}
      <strong
        style={{
          fontSize: props.inline === true ? fontSize.caption : fontSize.body,
          color: palette.neutral[900],
          fontWeight: fontWeight.medium
        }}
      >
        {props.title}
      </strong>
      {props.description !== undefined && (
        <span style={{ fontSize: fontSize.caption, maxWidth: 320, lineHeight: 1.5 }}>{props.description}</span>
      )}
      {props.action !== undefined && <div style={{ marginTop: space[2] }}>{props.action}</div>}
    </div>
  );
}
