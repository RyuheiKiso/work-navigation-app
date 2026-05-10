// 対応 §: ロードマップ §9.2.2 §13.1
// HoldConfirmButton の振る舞いを RTL で検証する。
// - タッチ/マウスは閾値以上の押下で初めて発火
// - 閾値未満で離すと cancel
// - キーボード (Enter/Space) は即発火 (意図性が高い)
// - disabled は全経路で発火しない

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { HoldConfirmButton } from './hold-confirm-button';

describe('<HoldConfirmButton>', () => {
  beforeEach(() => { vi.useFakeTimers(); });
  afterEach(() => { vi.useRealTimers(); });

  it('閾値未満の押下では onHoldComplete を呼ばない', () => {
    const onComplete = vi.fn();
    const onCancel = vi.fn();
    render(
      <HoldConfirmButton holdMs={800} onHoldComplete={onComplete} onHoldCancel={onCancel} hapticEnabled={false}>
        完了
      </HoldConfirmButton>
    );
    const btn = screen.getByRole('button', { name: '完了' });
    act(() => { fireEvent.pointerDown(btn); });
    act(() => { vi.advanceTimersByTime(400); });
    act(() => { fireEvent.pointerUp(btn); });

    expect(onComplete).not.toHaveBeenCalled();
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('閾値以上の押下で onHoldComplete を呼ぶ', () => {
    const onComplete = vi.fn();
    render(
      <HoldConfirmButton holdMs={500} onHoldComplete={onComplete} hapticEnabled={false}>
        完了
      </HoldConfirmButton>
    );
    const btn = screen.getByRole('button', { name: '完了' });
    act(() => { fireEvent.pointerDown(btn); });
    act(() => { vi.advanceTimersByTime(600); });

    expect(onComplete).toHaveBeenCalledTimes(1);
    expect(btn.getAttribute('aria-busy')).toBe('false');
  });

  it('Enter キーで即発火する (意図性が高いため閾値不要)', () => {
    const onComplete = vi.fn();
    render(
      <HoldConfirmButton holdMs={5000} onHoldComplete={onComplete} hapticEnabled={false}>
        完了
      </HoldConfirmButton>
    );
    const btn = screen.getByRole('button', { name: '完了' });
    fireEvent.keyDown(btn, { key: 'Enter' });
    expect(onComplete).toHaveBeenCalledTimes(1);
  });

  it('Space キーで即発火する', () => {
    const onComplete = vi.fn();
    render(
      <HoldConfirmButton holdMs={5000} onHoldComplete={onComplete} hapticEnabled={false}>
        完了
      </HoldConfirmButton>
    );
    const btn = screen.getByRole('button', { name: '完了' });
    fireEvent.keyDown(btn, { key: ' ' });
    expect(onComplete).toHaveBeenCalledTimes(1);
  });

  it('disabled では押下/キーボードどちらでも発火しない', () => {
    const onComplete = vi.fn();
    render(
      <HoldConfirmButton holdMs={300} onHoldComplete={onComplete} disabled hapticEnabled={false}>
        完了
      </HoldConfirmButton>
    );
    const btn = screen.getByRole('button', { name: '完了' });
    act(() => { fireEvent.pointerDown(btn); });
    act(() => { vi.advanceTimersByTime(500); });
    fireEvent.keyDown(btn, { key: 'Enter' });
    expect(onComplete).not.toHaveBeenCalled();
  });
});
