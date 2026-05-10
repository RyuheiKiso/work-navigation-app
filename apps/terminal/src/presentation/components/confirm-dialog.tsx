// 対応 §: ロードマップ §9.2.2 §11.2 §3.6.4
// 共通の確認ダイアログ。手袋・濡れ手・騒音下での誤タップが致命的にならないよう、
// 破壊的操作 (アンドン発火、タスク完了、削除など) の前に必ず差し挟む。
// `window.confirm()` は a11y 不全（フォーカストラップ無し、screenreader 対応不安定）のため使わない。

import { useEffect, useRef } from 'react';
import { palette, fontSize, fontWeight, radius, space, elevation } from '../../tokens/access';

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
  const accent = variant === 'danger' ? palette.danger.default : palette.success.default;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(13,17,23,0.55)',
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
          background: palette.white,
          padding: space[5],
          borderRadius: radius.large,
          minWidth: 320,
          maxWidth: 480,
          boxShadow: elevation[4]
        }}
      >
        <h2
          id="confirm-dialog-title"
          style={{ marginTop: 0, fontSize: fontSize.subtitle, fontWeight: fontWeight.bold, color: accent }}
        >
          {props.title}
        </h2>
        <p
          id="confirm-dialog-desc"
          style={{ fontSize: fontSize.body, lineHeight: 1.6, color: palette.neutral[800] }}
        >
          {props.description}
        </p>
        <div style={{ display: 'flex', gap: space[3], marginTop: space[5], justifyContent: 'flex-end' }}>
          <button
            ref={cancelRef}
            type="button"
            onClick={props.onCancel}
            style={{
              minHeight: 48,
              minWidth: 96,
              padding: `${space[2]} ${space[4]}`,
              background: palette.white,
              color: palette.neutral[800],
              border: `1px solid ${palette.neutral[500]}`,
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
              minHeight: 48,
              minWidth: 96,
              padding: `${space[2]} ${space[4]}`,
              background: accent,
              color: palette.white,
              border: 'none',
              // §9.2.2 危険操作の角丸 0
              borderRadius: variant === 'danger' ? radius.none : radius.medium,
              cursor: 'pointer',
              fontSize: fontSize.body,
              fontWeight: fontWeight.bold
            }}
          >
            {props.confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
