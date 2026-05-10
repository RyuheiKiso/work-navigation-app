// 対応 §: ロードマップ §11.2.2 §3.6.4
// 「色のみで情報を伝えない」ため、トーン色＋アイコン形＋テキストを 3 重化したチップ。
// §11.2.2 SC 1.4.1 を最小単位で担保する基本部品。

import type { ReactNode } from 'react';
import { palette, fontSize, fontWeight, radius, space } from '../../../tokens/access';
import { tone, type SemanticTone } from '../../../tokens/access';
import { Icon, type IconProps } from '../icon/icon';

export interface StatusPillProps {
  toneName: SemanticTone;
  iconName?: IconProps['name'];
  children: ReactNode;
  // role を渡したい場合（`status` / `alert`）
  role?: 'status' | 'alert' | 'note';
  ariaLive?: 'polite' | 'assertive' | 'off';
  ariaLabel?: string;
  size?: 'sm' | 'md';
}

export function StatusPill(props: StatusPillProps): JSX.Element {
  const t = tone(props.toneName);
  const dim =
    props.size === 'sm'
      ? { padding: `${space[1]} ${space[3]}`, fontSize: fontSize.caption, iconSize: 16 as const }
      : { padding: `${space[2]} ${space[4]}`, fontSize: fontSize.body, iconSize: 20 as const };
  const role = props.role ?? 'status';
  return (
    <span
      role={role}
      aria-live={props.ariaLive}
      aria-label={props.ariaLabel}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: space[2],
        padding: dim.padding,
        background: t.bg,
        color: t.fg,
        border: `1px solid ${t.border}`,
        borderRadius: radius.pill,
        fontSize: dim.fontSize,
        fontWeight: fontWeight.medium,
        // 透明色を吸い込みやすい場面用に発光ドットを保証
        whiteSpace: 'nowrap'
      }}
    >
      {props.iconName !== undefined && <Icon name={props.iconName} size={dim.iconSize} />}
      {props.iconName === undefined && (
        <span
          aria-hidden="true"
          style={{
            width: 8,
            height: 8,
            borderRadius: radius.pill,
            background: t.border
          }}
        />
      )}
      <span>{props.children}</span>
    </span>
  );
}

// 互換: 一部呼び出し側が tone と分離して palette 直参照していた場合の橋渡し
export const tonePalette = palette;
