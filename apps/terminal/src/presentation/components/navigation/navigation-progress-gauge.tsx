// 対応 §: ロードマップ §3.6.2 §9.5.2 §21 注 2
// 残ステップ数と「経過 vs 標準時間」を 1 つの円形ゲージで直感化する。
// 文字列ベタ書きを避け、進捗の色相変化（success → warning → danger）で
// 超過リスクを 100ms 以下の知覚同期で伝える。

import { palette, fontSize, fontWeight, motion, space } from '../../../tokens/access';

export interface NavigationProgressGaugeProps {
  // 0..1
  progress: number;
  remaining: number;
  elapsedSec: number;
  stdSec: number;
  overrun: boolean;
  ariaLabel: string;
  // 表示サイズ（直径 px）。フォーカス領域では大きく、サイドでは小さく。
  diameter?: number;
}

function ringColor(progress: number, overrun: boolean): string {
  if (overrun) return palette.danger.default;
  if (progress >= 0.85) return palette.success.default;
  if (progress >= 0.5) return palette.info.default;
  return palette.warning.default;
}

export function NavigationProgressGauge(props: NavigationProgressGaugeProps): JSX.Element {
  const d = props.diameter ?? 160;
  const stroke = Math.max(8, Math.round(d / 14));
  const r = (d - stroke) / 2;
  const cx = d / 2;
  const cy = d / 2;
  const circumference = 2 * Math.PI * r;
  const clamped = Math.min(1, Math.max(0, props.progress));
  const dashOffset = circumference * (1 - clamped);
  const color = ringColor(clamped, props.overrun);
  const ratio = props.stdSec > 0 ? props.elapsedSec / props.stdSec : 0;
  const ratioPct = Math.round(ratio * 100);

  return (
    <div
      role="progressbar"
      aria-label={props.ariaLabel}
      aria-valuenow={Math.round(clamped * 100)}
      aria-valuemin={0}
      aria-valuemax={100}
      style={{
        position: 'relative',
        width: d,
        height: d,
        display: 'inline-grid',
        placeItems: 'center'
      }}
    >
      <svg width={d} height={d} viewBox={`0 0 ${d} ${d}`} aria-hidden="true">
        <circle cx={cx} cy={cy} r={r} fill="none" stroke={palette.neutral[200]} strokeWidth={stroke} />
        <circle
          cx={cx}
          cy={cy}
          r={r}
          fill="none"
          stroke={color}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={dashOffset}
          transform={`rotate(-90 ${cx} ${cy})`}
          style={{ transition: `stroke-dashoffset ${motion.durationLong} ${motion.easeEmphasized}, stroke ${motion.durationStandard} ${motion.easeStandard}` }}
        />
      </svg>
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: space[1]
        }}
      >
        <strong style={{ fontSize: fontSize.display, fontWeight: fontWeight.bold, color: palette.neutral[900], lineHeight: 1 }}>
          {props.remaining}
        </strong>
        <span style={{ fontSize: fontSize.caption, color: palette.neutral[600] }}>
          {props.elapsedSec}s / {props.stdSec}s
        </span>
        <span style={{ fontSize: fontSize.caption, color: props.overrun ? palette.danger.strong : palette.neutral[500], fontWeight: fontWeight.medium }}>
          {ratioPct}%
        </span>
      </div>
    </div>
  );
}
