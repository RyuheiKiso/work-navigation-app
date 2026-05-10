// 対応 §: ロードマップ §9.5.1 §11.2 §11.3 §6.3
// `tokens.ts` (Style Dictionary 出力) を UI 層が直接参照しなくて済むよう平坦化する単一窓口。
// 配色と影は CSS カスタムプロパティ参照に変換し、`<html data-theme="outdoor">`
// で 1000nit 屋外モード、`data-theme="dark"` で暗所モードに即時切替できるようにする。
// 数値系（spacing / radius / typography / motion）はテーマで変わらないため静的値のまま返す。

import { TOKENS } from './tokens';

const s = TOKENS.spacing.space;
const f = TOKENS.typography.font;
const r = TOKENS.radius.radius;
const m = TOKENS.motion.motion;
const d = TOKENS.density.density;

const cv = (name: string): string => `var(--wna-${name})`;

// 配色: テーマ可変。`tokens/css-vars.ts` で定義された CSS 変数を参照する。
export const palette = {
  white: cv('c-white'),
  black: cv('c-black'),
  neutral: {
    0: cv('c-n-0'),
    50: cv('c-n-50'),
    100: cv('c-n-100'),
    200: cv('c-n-200'),
    300: cv('c-n-300'),
    400: cv('c-n-400'),
    500: cv('c-n-500'),
    600: cv('c-n-600'),
    700: cv('c-n-700'),
    800: cv('c-n-800'),
    900: cv('c-n-900')
  },
  // ブランドアクセント（CTA／リンク／フォーカス誘導）— success と区別する
  brand: {
    subtle: cv('c-brand-subtle'),
    default: cv('c-brand-default'),
    strong: cv('c-brand-strong')
  },
  success: {
    subtle: cv('c-success-subtle'),
    default: cv('c-success-default'),
    strong: cv('c-success-strong')
  },
  warning: {
    subtle: cv('c-warning-subtle'),
    default: cv('c-warning-default'),
    strong: cv('c-warning-strong')
  },
  danger: {
    subtle: cv('c-danger-subtle'),
    default: cv('c-danger-default'),
    strong: cv('c-danger-strong')
  },
  info: {
    subtle: cv('c-info-subtle'),
    default: cv('c-info-default'),
    strong: cv('c-info-strong')
  },
  // §17 Andon 5 段階 — シグナルタワー比喩で使う
  andon: {
    1: cv('c-andon-1'),
    2: cv('c-andon-2'),
    3: cv('c-andon-3'),
    4: cv('c-andon-4'),
    5: cv('c-andon-5')
  },
  // テーマ可変の意味別名（背景／前景／枠）
  bg: cv('c-bg'),
  surface: cv('c-surface'),
  surfaceAlt: cv('c-surface-alt'),
  fg: cv('c-fg'),
  fgMuted: cv('c-fg-muted'),
  border: cv('c-border'),
  borderStrong: cv('c-border-strong')
} as const;

export const space = {
  0: s.unit['0'].value,
  1: s.unit['1'].value,
  2: s.unit['2'].value,
  3: s.unit['3'].value,
  4: s.unit['4'].value,
  5: s.unit['5'].value,
  6: s.unit['6'].value,
  7: s.unit['7'].value,
  8: s.unit['8'].value,
  // §21 注 6 タッチターゲット下限／推奨／確定（破壊的操作）
  touchMin: s.touchTarget.minimum.value,
  touchRecommended: s.touchTarget.recommended.value,
  touchCritical: s.touchTarget.critical.value
} as const;

export const radius = {
  none: r.surface.none.value,
  small: r.surface.small.value,
  medium: r.surface.medium.value,
  large: r.surface.large.value,
  pill: r.pill.value
} as const;

export const fontSize = {
  caption: f.size.caption.value,
  body: f.size.body.value,
  subtitle: f.size.subtitle.value,
  title: f.size.title.value,
  display: f.size.display.value
} as const;

export const fontWeight = {
  regular: f.weight.regular.value,
  medium: f.weight.medium.value,
  semibold: f.weight.semibold.value,
  bold: f.weight.bold.value
} as const;

export const lineHeight = {
  tight: f.lineHeight.tight.value,
  snug: f.lineHeight.snug.value,
  normal: f.lineHeight.normal.value,
  relaxed: f.lineHeight.relaxed.value,
  loose: f.lineHeight.loose.value
} as const;

// CJK・RTL を含むフォールバック連鎖を 1 文字列で配信する
export const fontStack = f.family.sans.value.join(', ');

// elevation はテーマ（屋外／暗所）で陰影濃度が変わる
export const elevation = {
  0: cv('shadow-0'),
  1: cv('shadow-1'),
  2: cv('shadow-2'),
  3: cv('shadow-3'),
  4: cv('shadow-4'),
  // 後方互換: 直接屋外シャドウを参照する経路（テーマ未適用環境用）
  outdoor1: cv('shadow-1'),
  outdoor2: cv('shadow-2')
} as const;

// 配置文脈ごとの情報密度（compact=班長俯瞰／cozy=Web編集／comfortable=作業者画面）
export type DensityVariant = 'compact' | 'cozy' | 'comfortable';

export interface DensityValues {
  touchTarget: string;
  gap: string;
  fontScale: number;
}

export const density: Record<DensityVariant, DensityValues> = {
  compact: {
    touchTarget: d.compact.touchTarget.value,
    gap: d.compact.gap.value,
    fontScale: Number(d.compact.fontScale.value)
  },
  cozy: {
    touchTarget: d.cozy.touchTarget.value,
    gap: d.cozy.gap.value,
    fontScale: Number(d.cozy.fontScale.value)
  },
  comfortable: {
    touchTarget: d.comfortable.touchTarget.value,
    gap: d.comfortable.gap.value,
    fontScale: Number(d.comfortable.fontScale.value)
  }
};

// §11.2.2 SC 2.4.7 — フォーカスリングはテーマ別に色が変わるため CSS 変数経由
export const focus = {
  ringWidth: cv('focus-ring-width'),
  ringOffset: cv('focus-ring-offset'),
  ringStyle: cv('focus-ring-style'),
  ringRadius: cv('focus-ring-radius'),
  ringColor: cv('c-focus-ring')
} as const;

export const motion = {
  // §21 注 2 100ms 以下＝知覚同期、240ms 以上＝離脱
  durationInstant: m.duration.instant.value,
  durationShort: m.duration.short.value,
  durationStandard: m.duration.standard.value,
  durationLong: m.duration.long.value,
  easeStandard: m.ease.standard.value,
  easeDecelerated: m.ease.decelerated.value,
  easeAccelerated: m.ease.accelerated.value,
  easeEmphasized: m.ease.emphasized.value
} as const;

export type SemanticTone = 'success' | 'warning' | 'danger' | 'info' | 'neutral';

export interface TonePair {
  bg: string;
  fg: string;
  border: string;
}

// 「色のみ」を避けるため、状態ペアの背景／前景／枠を一括で返す（§11.2.2 SC 1.4.1）
export function tone(name: SemanticTone): TonePair {
  switch (name) {
    case 'success':
      return { bg: palette.success.subtle, fg: palette.success.strong, border: palette.success.default };
    case 'warning':
      return { bg: palette.warning.subtle, fg: palette.warning.strong, border: palette.warning.default };
    case 'danger':
      return { bg: palette.danger.subtle, fg: palette.danger.strong, border: palette.danger.default };
    case 'info':
      return { bg: palette.info.subtle, fg: palette.info.strong, border: palette.info.default };
    case 'neutral':
      return { bg: palette.surfaceAlt, fg: palette.fg, border: palette.border };
  }
}
