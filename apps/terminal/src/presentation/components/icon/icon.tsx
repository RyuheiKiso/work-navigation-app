// 対応 §: ロードマップ §9.5.1 §11.2.2 §11.2.3
// アイコン単独提示は禁止（`tokens.icon.policy.label_paired = true`）。
// この `Icon` は装飾用途・ボタン併設用途のみで使用し、必ず `aria-label` 付きのテキストと併用する。
// `decorative` を true にすると `aria-hidden` で読み上げから除外する。

import { GLYPHS, type IconName } from './glyphs';

export interface IconProps {
  name: IconName;
  // 16/24/32/48 のいずれか。`tokens.icon.size` 系列に対応。
  size?: 16 | 18 | 20 | 24 | 28 | 32 | 40 | 48;
  // 線幅。stroke 系統のみで適用される（`tokens.icon.stroke.regular = 1.5px` 既定）。
  strokeWidth?: number;
  // 親要素の `color` を継承するため、固定指定が無ければ currentColor で塗る。
  color?: string;
  decorative?: boolean;
  title?: string;
  className?: string;
}

export function Icon(props: IconProps): JSX.Element {
  const size = props.size ?? 24;
  const stroke = props.strokeWidth ?? 1.75;
  const color = props.color ?? 'currentColor';
  const decorative = props.decorative ?? true;
  const glyph = GLYPHS[props.name];
  const a11y = decorative
    ? ({ 'aria-hidden': true } as const)
    : ({ role: 'img', 'aria-label': props.title ?? props.name } as const);

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      style={{ flexShrink: 0, display: 'inline-block', verticalAlign: 'middle' }}
      className={props.className}
      {...a11y}
    >
      {glyph.paths.map((d, i) =>
        glyph.mode === 'fill' ? (
          <path key={i} d={d} fill={color} />
        ) : (
          <path
            key={i}
            d={d}
            stroke={color}
            strokeWidth={stroke}
            strokeLinecap="round"
            strokeLinejoin="round"
            fill="none"
          />
        )
      )}
    </svg>
  );
}
