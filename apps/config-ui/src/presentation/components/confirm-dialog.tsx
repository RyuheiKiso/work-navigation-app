// 対応 §: ロードマップ §9.2.2 §11.2 §3.6.4
// 共通の確認ダイアログ。手袋・濡れ手・騒音下での誤タップが致命的にならないよう、
// 破壊的操作 (アンドン発火、タスク完了、削除など) の前に必ず差し挟む。
// `window.confirm()` は a11y 不全（フォーカストラップ無し、screenreader 対応不安定）のため使わない。
// 配色は tokens/access 経由でテーマ可変。danger は palette.danger、normal は palette.brand を accent に。

import { useEffect, useRef } from 'react';
import { palette, radius, fontSize, fontWeight, lineHeight, space, elevation } from '../../tokens/access';

export type ConfirmVariant = 'danger' | 'normal';

export interface ConfirmDialogProps {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  cancelLabel: string;
  variant?: ConfirmVariant;
  onConfirm(): void;
  onCancel(): void;
}

export function ConfirmDialog(props: ConfirmDialogProps): JSX.Element | null {
  const cancelRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!props.open) return;
    // Escape で取消、Enter で実行も可能だが Enter は誤実行リスクが高いため Escape のみに絞る
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        e.preventDefault();
        props.onCancel();
      }
    };
    window.addEventListener('keydown', onKey);
    // 初期フォーカスは「取消」側に置く（現場での誤タップ確率を下げる）
    cancelRef.current?.focus();
    return () => window.removeEventListener('keydown', onKey);
  }, [props.open, props.onCancel]);

  if (!props.open) return null;

  const variant: ConfirmVariant = props.variant ?? 'normal';
  const accent = variant === 'danger' ? palette.danger.default : palette.brand.default;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        // scrim — 操作対象を奥に退ける半透明オーバーレイ。トークン化対象外（CSS 変数化は別途）
        background: 'rgba(13, 17, 23, 0.55)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000
      }}
      onClick={(e) => {
        // 背景クリックは取消扱い
        if (e.target === e.currentTarget) props.onCancel();
      }}
    >
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
        aria-describedby="confirm-dialog-desc"
        style={{
          background: palette.surface,
          color: palette.fg,
          padding: space[5],
          borderRadius: radius.large,
          minWidth: '320px',
          maxWidth: '480px',
          boxShadow: elevation[3]
        }}
      >
        <h2
          id="confirm-dialog-title"
          style={{ marginTop: 0, fontSize: fontSize.subtitle, color: accent, fontWeight: fontWeight.semibold }}
        >
          {props.title}
        </h2>
        <p
          id="confirm-dialog-desc"
          style={{ fontSize: fontSize.body, lineHeight: lineHeight.relaxed, color: palette.fg }}
        >
          {props.description}
        </p>
        <div style={{ display: 'flex', gap: space[3], marginTop: space[5], justifyContent: 'flex-end' }}>
          <button
            ref={cancelRef}
            type="button"
            onClick={props.onCancel}
            style={{
              minHeight: '48px',
              minWidth: '96px',
              padding: `${space[2]} ${space[4]}`,
              background: palette.surface,
              color: palette.fg,
              border: `1px solid ${palette.borderStrong}`,
              borderRadius: radius.medium,
              cursor: 'pointer',
              fontSize: fontSize.body
            }}
          >
            {props.cancelLabel}
          </button>
          <button
            type="button"
            onClick={props.onConfirm}
            style={{
              minHeight: '48px',
              minWidth: '96px',
              padding: `${space[2]} ${space[4]}`,
              background: accent,
              color: palette.white,
              border: 'none',
              borderRadius: radius.medium,
              cursor: 'pointer',
              fontSize: fontSize.body,
              fontWeight: fontWeight.semibold
            }}
          >
            {props.confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
