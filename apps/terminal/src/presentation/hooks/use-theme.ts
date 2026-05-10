// 対応 §: ロードマップ §6.3 §11.2.2 §3.6.4
// 端末の表示モード（standard / outdoor / dark）を管理する。
// 永続化キー: `wna.terminal.theme`。OS ダークモード優先連動は `auto`、明示指定でロック。
// `data-theme` 属性は `<html>` に設定する。CSS 側は `tokens/css-vars.ts` の override が反応する。

import { useCallback, useEffect, useState } from 'react';

export type ThemeMode = 'standard' | 'outdoor' | 'dark' | 'auto';

const STORAGE_KEY = 'wna.terminal.theme';

function readPersisted(): ThemeMode {
  try {
    const v = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (v === 'standard' || v === 'outdoor' || v === 'dark' || v === 'auto') return v;
  } catch {
    /* localStorage 失敗は無視 */
  }
  return 'auto';
}

function osPrefersDark(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false;
}

function resolveAttribute(mode: ThemeMode): 'standard' | 'outdoor' | 'dark' {
  if (mode === 'auto') return osPrefersDark() ? 'dark' : 'standard';
  return mode;
}

function apply(mode: ThemeMode): void {
  if (typeof document === 'undefined') return;
  const resolved = resolveAttribute(mode);
  if (resolved === 'standard') {
    document.documentElement.removeAttribute('data-theme');
  } else {
    document.documentElement.setAttribute('data-theme', resolved);
  }
}

export interface UseThemeResult {
  mode: ThemeMode;
  // OS dark に追従する `auto` を解決した実効値
  resolved: 'standard' | 'outdoor' | 'dark';
  set(mode: ThemeMode): void;
  // standard → outdoor → dark → auto → standard で循環
  cycle(): void;
}

const ORDER: ThemeMode[] = ['standard', 'outdoor', 'dark', 'auto'];

export function useTheme(): UseThemeResult {
  const [mode, setMode] = useState<ThemeMode>(() => readPersisted());

  useEffect(() => {
    apply(mode);
    try {
      globalThis.localStorage?.setItem(STORAGE_KEY, mode);
    } catch {
      /* 書込失敗は無視 */
    }
  }, [mode]);

  // OS のダーク／ライト切替を auto モード時に追従する
  useEffect(() => {
    if (mode !== 'auto' || typeof window === 'undefined') return;
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (): void => apply('auto');
    mq.addEventListener?.('change', handler);
    return () => mq.removeEventListener?.('change', handler);
  }, [mode]);

  const set = useCallback((next: ThemeMode) => setMode(next), []);
  const cycle = useCallback(() => {
    setMode((prev) => {
      const idx = ORDER.indexOf(prev);
      return ORDER[(idx + 1) % ORDER.length] ?? 'standard';
    });
  }, []);

  return { mode, resolved: resolveAttribute(mode), set, cycle };
}
