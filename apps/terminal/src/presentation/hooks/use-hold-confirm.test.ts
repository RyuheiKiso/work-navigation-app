// 対応 §: ロードマップ §9.2.2 §13.1 ／ ルート CLAUDE.md（不変条件は型または property test で守る）
// useHoldConfirm の状態機械不変条件を fast-check で網羅する。
// - 閾値未満で離した時 onComplete は呼ばれない
// - 閾値以上保持した時 onComplete はちょうど 1 回呼ばれる
// - 任意のイベント列でも onComplete は最大 1 回 (連続発火防止)

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import fc from 'fast-check';
import { useHoldConfirm } from './use-hold-confirm';

describe('useHoldConfirm', () => {
  beforeEach(() => { vi.useFakeTimers(); });
  afterEach(() => { vi.useRealTimers(); });

  it('閾値未満で離した時 onComplete は呼ばれず onCancel が呼ばれる', () => {
    const onComplete = vi.fn();
    const onCancel = vi.fn();
    const { result } = renderHook(() =>
      useHoldConfirm({ holdMs: 800, onComplete, onCancel, hapticEnabled: false })
    );
    act(() => result.current.pointerHandlers.onPointerDown());
    act(() => { vi.advanceTimersByTime(500); });
    act(() => result.current.pointerHandlers.onPointerUp());

    expect(onComplete).not.toHaveBeenCalled();
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(result.current.state).toBe('cancelled');
    expect(result.current.progress).toBe(0);
  });

  it('閾値以上保持した時 onComplete が呼ばれ state が completed になる', () => {
    const onComplete = vi.fn();
    const { result } = renderHook(() =>
      useHoldConfirm({ holdMs: 800, onComplete, hapticEnabled: false })
    );
    act(() => result.current.pointerHandlers.onPointerDown());
    act(() => { vi.advanceTimersByTime(900); });

    expect(onComplete).toHaveBeenCalledTimes(1);
    expect(result.current.state).toBe('completed');
    expect(result.current.progress).toBe(1);
  });

  it('完了後の pointerUp は onCancel を呼ばない', () => {
    const onComplete = vi.fn();
    const onCancel = vi.fn();
    const { result } = renderHook(() =>
      useHoldConfirm({ holdMs: 500, onComplete, onCancel, hapticEnabled: false })
    );
    act(() => result.current.pointerHandlers.onPointerDown());
    act(() => { vi.advanceTimersByTime(600); });
    act(() => result.current.pointerHandlers.onPointerUp());

    expect(onComplete).toHaveBeenCalledTimes(1);
    expect(onCancel).not.toHaveBeenCalled();
  });

  it('reset で idle / progress=0 に戻る', () => {
    const onComplete = vi.fn();
    const { result } = renderHook(() =>
      useHoldConfirm({ holdMs: 500, onComplete, hapticEnabled: false })
    );
    act(() => result.current.pointerHandlers.onPointerDown());
    act(() => { vi.advanceTimersByTime(600); });
    act(() => result.current.reset());

    expect(result.current.state).toBe('idle');
    expect(result.current.progress).toBe(0);
  });

  it('property: 経過時間が holdMs 以上の場合のみ onComplete がちょうど 1 回呼ばれる', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 100, max: 2000 }),
        fc.integer({ min: 0, max: 3000 }),
        (holdMs, elapsed) => {
          vi.useFakeTimers();
          const onComplete = vi.fn();
          const { result, unmount } = renderHook(() =>
            useHoldConfirm({ holdMs, onComplete, hapticEnabled: false })
          );
          act(() => result.current.pointerHandlers.onPointerDown());
          act(() => { vi.advanceTimersByTime(elapsed); });

          const expected = elapsed >= holdMs ? 1 : 0;
          expect(onComplete).toHaveBeenCalledTimes(expected);
          unmount();
          vi.useRealTimers();
        }
      ),
      { numRuns: 30 }
    );
  });

  it('property: 任意の advance シーケンスでも onComplete は最大 1 回', () => {
    fc.assert(
      fc.property(
        fc.array(fc.integer({ min: 50, max: 1000 }), { minLength: 1, maxLength: 6 }),
        (advances) => {
          vi.useFakeTimers();
          const onComplete = vi.fn();
          const { result, unmount } = renderHook(() =>
            useHoldConfirm({ holdMs: 400, onComplete, hapticEnabled: false })
          );
          act(() => result.current.pointerHandlers.onPointerDown());
          for (const a of advances) {
            act(() => { vi.advanceTimersByTime(a); });
          }
          expect(onComplete.mock.calls.length).toBeLessThanOrEqual(1);
          unmount();
          vi.useRealTimers();
        }
      ),
      { numRuns: 30 }
    );
  });
});
