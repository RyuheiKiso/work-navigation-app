// 対応 §: ロードマップ §11.2 §3.6.4
// キーボードショートカット一覧モーダル。? もしくは Header の Keyboard ボタンで開く。
// 単純なリストだが視覚等価物として `<kbd>` を併用し、見た目で慣れさせる。

import { useEffect } from 'react';
import { palette, fontSize, fontWeight, radius, space, elevation } from '../../../tokens/access';
import { Icon } from '../icon/icon';
import { formatShortcut, type ShortcutSpec } from '../../hooks/use-keyboard-shortcuts';

export interface NavigationShortcutsHelpProps {
  open: boolean;
  shortcuts: ReadonlyArray<ShortcutSpec>;
  title: string;
  closeLabel: string;
  onClose(): void;
}

export function NavigationShortcutsHelp(props: NavigationShortcutsHelpProps): JSX.Element | null {
  useEffect(() => {
    if (!props.open) return;
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') props.onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [props.open, props.onClose]);

  if (!props.open) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(13,17,23,0.55)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
        padding: space[4]
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label={props.title}
        style={{
          background: palette.surface,
          color: palette.fg,
          borderRadius: radius.large,
          boxShadow: elevation[4],
          minWidth: 360,
          maxWidth: 560,
          width: '100%',
          padding: space[5]
        }}
      >
        <header
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: space[2],
            marginBottom: space[4]
          }}
        >
          <Icon name="keyboard" size={24} color={palette.info.default} />
          <h2 style={{ margin: 0, fontSize: fontSize.subtitle, fontWeight: fontWeight.bold }}>
            {props.title}
          </h2>
          <button
            type="button"
            onClick={props.onClose}
            aria-label={props.closeLabel}
            style={{
              marginLeft: 'auto',
              padding: space[2],
              background: 'transparent',
              color: palette.fgMuted,
              border: 'none',
              borderRadius: radius.medium,
              cursor: 'pointer'
            }}
          >
            <Icon name="close" size={20} />
          </button>
        </header>
        <ul style={{ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: space[3] }}>
          {props.shortcuts.map((s) => (
            <li
              key={s.key + s.description}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: space[4],
                padding: `${space[2]} ${space[3]}`,
                background: palette.surfaceAlt,
                borderRadius: radius.medium
              }}
            >
              <span style={{ fontSize: fontSize.body, color: palette.fg }}>{s.description}</span>
              <kbd>{formatShortcut(s)}</kbd>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
