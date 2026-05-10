// 対応 §: ロードマップ §13.1 §9.2.2 §11.2
// ConfirmDialog の単体テスト。a11y 役割と主要インタラクションを検証する。

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ConfirmDialog } from './confirm-dialog';

const baseProps = {
  title: 'アンドンを発火しますか？',
  description: '班長への応援要請が即時送信されます。取り消せません。',
  confirmLabel: '発火する',
  cancelLabel: 'やめる',
  onConfirm: () => undefined,
  onCancel: () => undefined
};

describe('ConfirmDialog', () => {
  it('renders nothing when closed', () => {
    const { container } = render(<ConfirmDialog open={false} {...baseProps} />);
    expect(container.firstChild).toBeNull();
  });

  it('renders alertdialog role with title and description', () => {
    render(<ConfirmDialog open={true} {...baseProps} />);
    const dialog = screen.getByRole('alertdialog');
    expect(dialog).toHaveAccessibleName('アンドンを発火しますか？');
    expect(dialog).toHaveAccessibleDescription(/応援要請/);
  });

  it('focuses cancel button on open', () => {
    render(<ConfirmDialog open={true} {...baseProps} />);
    expect(screen.getByText('やめる')).toBe(document.activeElement);
  });

  it('invokes onConfirm when confirm button is clicked', () => {
    const onConfirm = vi.fn();
    render(<ConfirmDialog open={true} {...baseProps} onConfirm={onConfirm} />);
    fireEvent.click(screen.getByText('発火する'));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('invokes onCancel when cancel button is clicked', () => {
    const onCancel = vi.fn();
    render(<ConfirmDialog open={true} {...baseProps} onCancel={onCancel} />);
    fireEvent.click(screen.getByText('やめる'));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('invokes onCancel on Escape key', () => {
    const onCancel = vi.fn();
    render(<ConfirmDialog open={true} {...baseProps} onCancel={onCancel} />);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
