// 対応 §: ロードマップ §9.5.1 §6.3 §11.2.2
// 設計トークンを CSS カスタムプロパティ (--wna-*) として `:root` に注入する。
// `[data-theme="outdoor"]` で 1000nit 屋外用の高コントラスト版（§6.3）に切替、
// `[data-theme="dark"]` で暗所・夜間用に neutral を反転する。
// テーマ切替は `documentElement.dataset.theme` を上書きするだけで全画面に伝搬する。

import { TOKENS } from './tokens';

const c = TOKENS.color.color;
const e = TOKENS.elevation.shadow;

// 既定 (standard) — 既存の値をそのまま CSS 変数として宣言する。
// 名前を短く保つため `var(--wna-c-success-default)` のように圧縮命名。
function buildBaseVars(): string {
  return [
    // base
    `--wna-c-white: ${c.base.white.value}`,
    `--wna-c-black: ${c.base.black.value}`,
    // neutrals
    `--wna-c-n-0: ${c.neutral['0'].value}`,
    `--wna-c-n-50: ${c.neutral['50'].value}`,
    `--wna-c-n-100: ${c.neutral['100'].value}`,
    `--wna-c-n-200: ${c.neutral['200'].value}`,
    `--wna-c-n-300: ${c.neutral['300'].value}`,
    `--wna-c-n-400: ${c.neutral['400'].value}`,
    `--wna-c-n-500: ${c.neutral['500'].value}`,
    `--wna-c-n-600: ${c.neutral['600'].value}`,
    `--wna-c-n-700: ${c.neutral['700'].value}`,
    `--wna-c-n-800: ${c.neutral['800'].value}`,
    `--wna-c-n-900: ${c.neutral['900'].value}`,
    // semantics
    `--wna-c-success-subtle: ${c.semantic.success.subtle.value}`,
    `--wna-c-success-default: ${c.semantic.success.default.value}`,
    `--wna-c-success-strong: ${c.semantic.success.strong.value}`,
    `--wna-c-warning-subtle: ${c.semantic.warning.subtle.value}`,
    `--wna-c-warning-default: ${c.semantic.warning.default.value}`,
    `--wna-c-warning-strong: ${c.semantic.warning.strong.value}`,
    `--wna-c-danger-subtle: ${c.semantic.danger.subtle.value}`,
    `--wna-c-danger-default: ${c.semantic.danger.default.value}`,
    `--wna-c-danger-strong: ${c.semantic.danger.strong.value}`,
    `--wna-c-info-subtle: ${c.semantic.info.subtle.value}`,
    `--wna-c-info-default: ${c.semantic.info.default.value}`,
    `--wna-c-info-strong: ${c.semantic.info.strong.value}`,
    // andon 5 段階
    `--wna-c-andon-1: ${c.andon.level1_normal.value}`,
    `--wna-c-andon-2: ${c.andon.level2_attention.value}`,
    `--wna-c-andon-3: ${c.andon.level3_warning.value}`,
    `--wna-c-andon-4: ${c.andon.level4_alert.value}`,
    `--wna-c-andon-5: ${c.andon.level5_critical.value}`,
    // elevation
    `--wna-shadow-0: ${e.elevation['0'].value}`,
    `--wna-shadow-1: ${e.elevation['1'].value}`,
    `--wna-shadow-2: ${e.elevation['2'].value}`,
    `--wna-shadow-3: ${e.elevation['3'].value}`,
    `--wna-shadow-4: ${e.elevation['4'].value}`,
    // 画面背景／前景の意味付き別名（テーマで反転する対象）
    `--wna-c-bg: ${c.neutral['50'].value}`,
    `--wna-c-surface: ${c.neutral['0'].value}`,
    `--wna-c-surface-alt: ${c.neutral['100'].value}`,
    `--wna-c-fg: ${c.neutral['900'].value}`,
    `--wna-c-fg-muted: ${c.neutral['600'].value}`,
    `--wna-c-border: ${c.neutral['200'].value}`,
    `--wna-c-border-strong: ${c.neutral['400'].value}`
  ].join(';\n  ') + ';';
}

// outdoor: §6.3 1000nit 屋外想定。subtle 系は廃し、strong 文字 × 純白背景で最大コントラストを確保する。
// 影は outdoor 系列で濃度を上げる。
function buildOutdoorVars(): string {
  return [
    `--wna-c-success-default: ${c.outdoor.success_high.value}`,
    `--wna-c-warning-default: ${c.outdoor.warning_high.value}`,
    `--wna-c-danger-default: ${c.outdoor.danger_high.value}`,
    `--wna-c-success-strong: ${c.outdoor.success_high.value}`,
    `--wna-c-warning-strong: ${c.outdoor.warning_high.value}`,
    `--wna-c-danger-strong: ${c.outdoor.danger_high.value}`,
    `--wna-c-info-strong: ${c.semantic.info.strong.value}`,
    `--wna-shadow-1: ${e.outdoor['1'].value}`,
    `--wna-shadow-2: ${e.outdoor['2'].value}`,
    // 屋外は反射でグレーが白く飛ぶため背景を真白に固定
    `--wna-c-bg: ${c.base.white.value}`,
    `--wna-c-surface: ${c.base.white.value}`,
    `--wna-c-fg: ${c.neutral['900'].value}`,
    `--wna-c-border: ${c.neutral['400'].value}`,
    `--wna-c-border-strong: ${c.neutral['700'].value}`
  ].join(';\n  ') + ';';
}

// dark: 夜勤・暗所想定。neutral を反転、危険／警告／成功は色相を保ったまま彩度を上げる。
function buildDarkVars(): string {
  return [
    `--wna-c-bg: ${c.neutral['900'].value}`,
    `--wna-c-surface: ${c.neutral['800'].value}`,
    `--wna-c-surface-alt: ${c.neutral['700'].value}`,
    `--wna-c-fg: ${c.neutral['50'].value}`,
    `--wna-c-fg-muted: ${c.neutral['400'].value}`,
    `--wna-c-border: ${c.neutral['700'].value}`,
    `--wna-c-border-strong: ${c.neutral['500'].value}`,
    // 暗背景上で読みやすい subtle/strong に反転
    `--wna-c-success-subtle: ${c.semantic.success.strong.value}`,
    `--wna-c-success-strong: ${c.semantic.success.subtle.value}`,
    `--wna-c-warning-subtle: ${c.semantic.warning.strong.value}`,
    `--wna-c-warning-strong: ${c.semantic.warning.subtle.value}`,
    `--wna-c-danger-subtle: ${c.semantic.danger.strong.value}`,
    `--wna-c-danger-strong: ${c.semantic.danger.subtle.value}`,
    `--wna-c-info-subtle: ${c.semantic.info.strong.value}`,
    `--wna-c-info-strong: ${c.semantic.info.subtle.value}`,
    // 暗背景では neutral 階調も役割逆転
    `--wna-c-n-0: ${c.neutral['900'].value}`,
    `--wna-c-n-50: ${c.neutral['800'].value}`,
    `--wna-c-n-100: ${c.neutral['700'].value}`,
    `--wna-c-n-200: ${c.neutral['600'].value}`,
    `--wna-c-n-300: ${c.neutral['500'].value}`,
    `--wna-c-n-400: ${c.neutral['400'].value}`,
    `--wna-c-n-500: ${c.neutral['300'].value}`,
    `--wna-c-n-600: ${c.neutral['200'].value}`,
    `--wna-c-n-700: ${c.neutral['100'].value}`,
    `--wna-c-n-800: ${c.neutral['50'].value}`,
    `--wna-c-n-900: ${c.neutral['0'].value}`
  ].join(';\n  ') + ';';
}

export function buildTokenStylesheet(): string {
  return `:root {\n  ${buildBaseVars()}\n}\n` +
    `[data-theme="outdoor"] {\n  ${buildOutdoorVars()}\n}\n` +
    `[data-theme="dark"] {\n  ${buildDarkVars()}\n}\n`;
}

const STYLE_ELEMENT_ID = 'wna-tokens';

/** モジュールロード時に一度だけスタイルシートを差し込む。SSR 環境では何もしない */
export function injectTokensStyle(): void {
  if (typeof document === 'undefined') return;
  if (document.getElementById(STYLE_ELEMENT_ID)) return;
  const style = document.createElement('style');
  style.id = STYLE_ELEMENT_ID;
  style.textContent = buildTokenStylesheet();
  document.head.appendChild(style);
}
