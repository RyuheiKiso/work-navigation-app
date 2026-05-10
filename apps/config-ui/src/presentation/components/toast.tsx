// 対応 §: ロードマップ §11.2 §9.2.1 ／ docs/02_設計/設定UI監査.md §1.2「強制ダイアログ禁止」
// 設定 UI のための非モーダル通知。window.alert は OS モーダルで作業を強制中断し
// アクセシビリティ不全 (focus 奪取・screenreader 不安定) のため使用禁止とする。
// 代わりに role="status" / "alert" を tone で切替えるトーストで通知し、4 秒で自動消滅。
// グローバル emitter で showToast() を component 階層と無関係に発火できる。

import { useEffect, useState } from 'react';
import { radius, fontSize, fontWeight, space, motion, elevation, tone as toneOf, type SemanticTone } from '../../tokens/access';

export type ToastTone = SemanticTone;

export interface ToastMessage {
  id: number;
  tone: ToastTone;
  text: string;
}

const listeners = new Set<(msg: ToastMessage) => void>();
let nextId = 1;

/** 非モーダルでメッセージを表示する。tone は意味色 (success / warning / danger / info / neutral) */
export function showToast(tone: ToastTone, text: string): void {
  const msg: ToastMessage = { id: nextId++, tone, text };
  listeners.forEach((l) => l(msg));
}

const AUTO_DISMISS_MS = 4000;

/** アプリのルート付近に 1 つだけ配置する。複数同時表示はせず、最新で上書き */
export function ToastHost(): JSX.Element | null {
  const [current, setCurrent] = useState<ToastMessage | null>(null);

  useEffect(() => {
    const fn = (msg: ToastMessage): void => setCurrent(msg);
    listeners.add(fn);
    return () => { listeners.delete(fn); };
  }, []);

  useEffect(() => {
    if (current === null) return;
    const id = window.setTimeout(() => setCurrent(null), AUTO_DISMISS_MS);
    return () => window.clearTimeout(id);
  }, [current]);

  if (current === null) return null;
  const t = toneOf(current.tone);
  // danger / warning は assertive (即時読み上げ)、それ以外は polite
  const assertive = current.tone === 'danger' || current.tone === 'warning';

  return (
    <div
      role={assertive ? 'alert' : 'status'}
      aria-live={assertive ? 'assertive' : 'polite'}
      style={{
        position: 'fixed',
        right: space[5],
        bottom: space[5],
        zIndex: 1100,
        maxWidth: '420px',
        padding: `${space[3]} ${space[4]}`,
        borderRadius: radius.medium,
        background: t.bg,
        color: t.fg,
        border: `1px solid ${t.border}`,
        boxShadow: elevation[2],
        fontSize: fontSize.body,
        fontWeight: fontWeight.medium,
        display: 'flex',
        gap: space[3],
        alignItems: 'flex-start',
        animation: `wna-toast-in ${motion.durationShort} ${motion.easeDecelerated}`
      }}
    >
      <span style={{ flex: 1, whiteSpace: 'pre-line' }}>{current.text}</span>
      <button
        type="button"
        onClick={() => setCurrent(null)}
        aria-label="閉じる"
        style={{
          background: 'transparent',
          border: 'none',
          color: t.fg,
          cursor: 'pointer',
          fontSize: fontSize.body,
          padding: 0,
          lineHeight: 1
        }}
      >
        ×
      </button>
    </div>
  );
}

// テスト用: グローバル listener 状態を clean に保つ
export function __resetToastForTests(): void {
  listeners.clear();
  nextId = 1;
}
