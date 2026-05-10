// 対応 §: ロードマップ §11.2 §3.6.4
// 物理キーボード／外付けバーコードリーダ／PoE 端末を併用する現場で、
// マウス／タッチ無しで主要アクションを完遂できるショートカットを提供する。
// `data-shortcut-target="0|1"` の属性を持つ要素にフォーカスがあるときは
// テキスト入力中とみなし、グローバルショートカットを発火しない。

import { useEffect } from 'react';

export interface ShortcutSpec {
  key: string;
  // 修飾キーの要求。指定なしは「修飾なしのみ」を許容する。
  shift?: boolean;
  ctrl?: boolean;
  alt?: boolean;
  meta?: boolean;
  // 説明文。ヘルプモーダルで表示する。
  description: string;
  handler(e: KeyboardEvent): void;
}

function isEditing(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName.toUpperCase();
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  return false;
}

function matches(e: KeyboardEvent, spec: ShortcutSpec): boolean {
  if (e.key !== spec.key && e.key.toLowerCase() !== spec.key.toLowerCase()) return false;
  if ((spec.shift ?? false) !== e.shiftKey) return false;
  if ((spec.ctrl ?? false) !== e.ctrlKey) return false;
  if ((spec.alt ?? false) !== e.altKey) return false;
  if ((spec.meta ?? false) !== e.metaKey) return false;
  return true;
}

export function useKeyboardShortcuts(specs: ReadonlyArray<ShortcutSpec>, enabled = true): void {
  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;
    const onKey = (e: KeyboardEvent): void => {
      if (isEditing(e.target)) return;
      for (const spec of specs) {
        if (matches(e, spec)) {
          e.preventDefault();
          spec.handler(e);
          return;
        }
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [specs, enabled]);
}

// 表示用: `Ctrl+Enter` のような文字列表現に整形
export function formatShortcut(spec: Pick<ShortcutSpec, 'key' | 'shift' | 'ctrl' | 'alt' | 'meta'>): string {
  const parts: string[] = [];
  if (spec.ctrl === true) parts.push('Ctrl');
  if (spec.alt === true) parts.push('Alt');
  if (spec.shift === true) parts.push('Shift');
  if (spec.meta === true) parts.push('⌘');
  parts.push(prettyKey(spec.key));
  return parts.join('+');
}

function prettyKey(key: string): string {
  if (key === ' ') return 'Space';
  if (key === 'Enter') return 'Enter';
  if (key === 'Escape') return 'Esc';
  if (key === 'ArrowLeft') return '←';
  if (key === 'ArrowRight') return '→';
  if (key === 'ArrowUp') return '↑';
  if (key === 'ArrowDown') return '↓';
  if (key.length === 1) return key.toUpperCase();
  return key;
}
