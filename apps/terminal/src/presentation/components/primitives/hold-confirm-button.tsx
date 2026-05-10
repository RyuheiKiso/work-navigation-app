// 対応 §: ロードマップ §9.2.2 §11.2 §11.2.3 §6.3
// 破壊的・不可逆操作 (完了確定／アンドン発報) 用の押し続けボタン。
// `<Button>` 互換 API に holdMs を加え、内部で useHoldConfirm を使う。
// タッチ／マウス押下は holdMs ms 連続押下で初めて確定し、キーボード
// (Enter / Space) は意図性が高いため即発火 (アクセシビリティ整合)。
// 進捗は底部の細いバーと押下時の彩度変化で表現し、屋外でも視認可能。

import type { ButtonHTMLAttributes, CSSProperties, KeyboardEvent, ReactNode } from 'react';
import { palette, motion, radius, fontWeight, elevation } from '../../../tokens/access';
import { buttonPaint, buttonDimensions, type ButtonVariant, type ButtonSize } from './button';
import { useHoldConfirm } from '../../hooks/use-hold-confirm';

export interface HoldConfirmButtonProps
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'onClick'> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  block?: boolean;
  leadingIcon?: ReactNode;
  trailingIcon?: ReactNode;
  /** 確定までの保持時間 (ms)。屋外運用 800〜1200ms 推奨 */
  holdMs?: number;
  /** 閾値到達時に発火 (Button.onClick の代わり) */
  onHoldComplete: () => void;
  /** 閾値未満で離した時に呼ばれる (任意) */
  onHoldCancel?: () => void;
  /** ハプティック発火の有無。テスト・低スペック端末で抑止可能 */
  hapticEnabled?: boolean;
}

export function HoldConfirmButton(props: HoldConfirmButtonProps): JSX.Element {
  const {
    variant = 'primary', size = 'xl', block, leadingIcon, trailingIcon,
    holdMs = 800, onHoldComplete, onHoldCancel, hapticEnabled,
    style, children, disabled, onKeyDown: extOnKeyDown, ...rest
  } = props;
  const isDisabled = disabled === true;
  const { state, progress, pointerHandlers } = useHoldConfirm({
    holdMs, onComplete: onHoldComplete, onCancel: onHoldCancel, hapticEnabled
  });
  const c = buttonPaint(variant, isDisabled);
  const d = buttonDimensions(size);
  const danger = variant === 'danger';
  const pressed = state === 'pressing';
  const completed = state === 'completed';

  const buttonStyle: CSSProperties = {
    minHeight: d.minHeight,
    padding: d.padding,
    fontSize: d.fontSize,
    gap: d.gap,
    width: block === true ? '100%' : undefined,
    background: c.bg,
    color: c.fg,
    border: `1px solid ${c.border}`,
    // §9.2.2 危険操作は剛性表現 (角を立てる) で誤タップ抑止
    borderRadius: danger ? radius.none : radius.medium,
    fontWeight: fontWeight.medium,
    cursor: isDisabled ? 'not-allowed' : 'pointer',
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition:
      `background ${motion.durationShort} ${motion.easeStandard},` +
      ` box-shadow ${motion.durationShort} ${motion.easeStandard},` +
      ` filter ${motion.durationShort} ${motion.easeStandard}`,
    boxShadow: isDisabled ? 'none' : pressed ? elevation[2] : elevation[1],
    // 押下中は彩度を下げて「進行中」、完了で一瞬明るく反転
    filter: completed ? 'brightness(1.1)' : pressed ? 'brightness(0.92)' : 'none',
    position: 'relative',
    overflow: 'hidden',
    userSelect: 'none',
    // 二重タップでズーム等の OS ジェスチャを抑止
    touchAction: 'manipulation',
    ...style
  };

  // キーボード (Enter / Space) は意図性が高いため即発火。
  // pointerdown/up の hold ロジックとは独立に動かす。
  const onKeyDown = (e: KeyboardEvent<HTMLButtonElement>): void => {
    extOnKeyDown?.(e);
    if (e.defaultPrevented || isDisabled) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onHoldComplete();
    }
  };

  // 進捗バーの色は前景に同調して屋外でも視認可能に
  const progressColor = variant === 'warning' ? palette.neutral[900] : palette.white;

  // disabled 中は pointer 経路を物理的に止める。React の disabled は
  // ハンドラ呼出を保証しないため hook の onPointerDown が動いてしまう
  const handlersWhenEnabled = isDisabled ? undefined : pointerHandlers;

  return (
    <button
      {...rest}
      {...handlersWhenEnabled}
      disabled={isDisabled}
      aria-busy={pressed}
      style={buttonStyle}
      onKeyDown={onKeyDown}
    >
      {leadingIcon}
      {children}
      {trailingIcon}
      <span
        aria-hidden="true"
        style={{
          position: 'absolute',
          left: 0,
          bottom: 0,
          height: '4px',
          width: `${progress * 100}%`,
          background: progressColor,
          opacity: pressed ? 0.85 : 0,
          transition: `opacity ${motion.durationShort} ${motion.easeStandard}`,
          pointerEvents: 'none'
        }}
      />
    </button>
  );
}
