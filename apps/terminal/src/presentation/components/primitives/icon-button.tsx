// 対応 §: ロードマップ §9.5.1 §11.2 §11.2.2
// アイコンのみのボタン。`tokens.icon.policy.label_paired = true` のため
// 視覚ラベルは無いが必ず `aria-label` を要求する。

import type { ButtonHTMLAttributes } from 'react';
import { palette, radius, motion, space } from '../../../tokens/access';
import { Icon, type IconProps } from '../icon/icon';

export interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'aria-label'> {
  iconName: IconProps['name'];
  iconSize?: IconProps['size'];
  // 必須。アイコン単独提示の禁止に対応。
  ariaLabel: string;
  selected?: boolean;
  tone?: 'neutral' | 'danger' | 'success';
}

export function IconButton(props: IconButtonProps): JSX.Element {
  const { iconName, iconSize = 24, ariaLabel, selected, tone = 'neutral', style, ...rest } = props;
  const fg =
    tone === 'danger' ? palette.danger.default
    : tone === 'success' ? palette.success.default
    : palette.neutral[700];
  const bg = selected === true ? palette.neutral[100] : 'transparent';
  return (
    <button
      type="button"
      aria-label={ariaLabel}
      aria-pressed={selected === true ? true : undefined}
      {...rest}
      style={{
        minWidth: space.touchMin,
        minHeight: space.touchMin,
        padding: space[2],
        background: bg,
        color: fg,
        border: 'none',
        borderRadius: radius.medium,
        cursor: 'pointer',
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        transition: `background ${motion.durationShort} ${motion.easeStandard}`,
        ...style
      }}
    >
      <Icon name={iconName} size={iconSize} />
    </button>
  );
}
