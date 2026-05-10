// 対応 §: ロードマップ §11.2 ／ docs/02_設計/設定UI監査.md §1.2
// ToastHost / showToast の挙動を検証。
// - showToast 後にメッセージが表示される
// - 4 秒で自動消滅
// - × ボタンで手動消滅
// - danger / warning は role="alert"、それ以外は role="status"

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { ToastHost, showToast, __resetToastForTests } from './toast';

describe('Toast', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    __resetToastForTests();
  });
  afterEach(() => {
    vi.useRealTimers();
    __resetToastForTests();
  });

  it('showToast でメッセージが表示される', () => {
    render(<ToastHost />);
    act(() => { showToast('success', '保存しました'); });
    expect(screen.getByText('保存しました')).toBeInTheDocument();
  });

  it('4 秒経過で自動消滅する', () => {
    render(<ToastHost />);
    act(() => { showToast('success', '一時表示'); });
    expect(screen.getByText('一時表示')).toBeInTheDocument();
    act(() => { vi.advanceTimersByTime(4001); });
    expect(screen.queryByText('一時表示')).not.toBeInTheDocument();
  });

  it('× ボタンで手動消滅する', () => {
    render(<ToastHost />);
    act(() => { showToast('info', 'ヒント'); });
    fireEvent.click(screen.getByRole('button', { name: '閉じる' }));
    expect(screen.queryByText('ヒント')).not.toBeInTheDocument();
  });

  it('danger は role="alert" で発行される', () => {
    render(<ToastHost />);
    act(() => { showToast('danger', '失敗しました'); });
    const node = screen.getByText('失敗しました').closest('[role]');
    expect(node?.getAttribute('role')).toBe('alert');
  });

  it('success は role="status" で発行される', () => {
    render(<ToastHost />);
    act(() => { showToast('success', '完了'); });
    const node = screen.getByText('完了').closest('[role]');
    expect(node?.getAttribute('role')).toBe('status');
  });
});
