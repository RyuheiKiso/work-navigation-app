// 対応 §: ロードマップ §9.5.1 §11.2 §9.2.2
// アクションのトーン・サイズを語彙に閉じ、CTA の階層が崩れない単一実装。
// `variant`: primary（完了等の正方向）／danger（破壊・アンドン）／ghost（取消・閉じる）／warning（中断）。
// `size`: lg は底面 CTA、md は通常、sm はチップ／インライン。

import type { ButtonHTMLAttributes, CSSProperties, ReactNode } from 'react';
import { palette, radius, fontSize, fontWeight, motion, space, elevation } from '../../../tokens/access';

export type ButtonVariant = 'primary' | 'danger' | 'warning' | 'ghost';
export type ButtonSize = 'sm' | 'md' | 'lg' | 'xl';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  block?: boolean;
  leadingIcon?: ReactNode;
  trailingIcon?: ReactNode;
}

function paint(variant: ButtonVariant, disabled: boolean): { bg: string; fg: string; border: string } {
  if (disabled) return { bg: palette.neutral[200], fg: palette.neutral[500], border: palette.neutral[200] };
  switch (variant) {
    case 'primary':
      return { bg: palette.success.default, fg: palette.white, border: palette.success.default };
    case 'danger':
      return { bg: palette.danger.default, fg: palette.white, border: palette.danger.default };
    case 'warning':
      return { bg: palette.warning.default, fg: palette.neutral[900], border: palette.warning.default };
    case 'ghost':
      return { bg: 'transparent', fg: palette.neutral[800], border: palette.neutral[400] };
  }
}

function dimensions(size: ButtonSize): { minHeight: string; padding: string; fontSize: string; gap: string } {
  switch (size) {
    case 'xl':
      return { minHeight: '88px', padding: `${space[4]} ${space[6]}`, fontSize: fontSize.title, gap: space[3] };
    case 'lg':
      return { minHeight: space.touchRecommended, padding: `${space[3]} ${space[5]}`, fontSize: fontSize.subtitle, gap: space[2] };
    case 'md':
      return { minHeight: space.touchMin, padding: `${space[2]} ${space[4]}`, fontSize: fontSize.body, gap: space[2] };
    case 'sm':
      return { minHeight: '32px', padding: `${space[1]} ${space[3]}`, fontSize: fontSize.caption, gap: space[1] };
  }
}

export function Button(props: ButtonProps): JSX.Element {
  const { variant = 'primary', size = 'md', block, leadingIcon, trailingIcon, style, children, ...rest } = props;
  const disabled = props.disabled === true;
  const c = paint(variant, disabled);
  const d = dimensions(size);
  const danger = variant === 'danger';
  const merged: CSSProperties = {
    minHeight: d.minHeight,
    padding: d.padding,
    fontSize: d.fontSize,
    gap: d.gap,
    width: block === true ? '100%' : undefined,
    background: c.bg,
    color: c.fg,
    border: `1px solid ${c.border}`,
    // §9.2.2 危険操作は剛性表現（角を立てる）で誤タップ抑止
    borderRadius: danger ? radius.none : radius.medium,
    fontWeight: fontWeight.medium,
    cursor: disabled ? 'not-allowed' : 'pointer',
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: `background ${motion.durationShort} ${motion.easeStandard}, box-shadow ${motion.durationShort} ${motion.easeStandard}, transform ${motion.durationShort} ${motion.easeStandard}`,
    boxShadow: variant === 'ghost' || disabled ? 'none' : elevation[1],
    ...style
  };
  return (
    <button {...rest} disabled={disabled} style={merged}>
      {leadingIcon}
      {children}
      {trailingIcon}
    </button>
  );
}
