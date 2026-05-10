// 対応 §: ロードマップ §13.1 §10.2.2 §14.2
// useAutosave Hook のテスト。debounce／状態遷移／cleanup を検証する。

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAutosave } from './use-autosave';

describe('useAutosave', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('does not write on initial mount', async () => {
    const write = vi.fn();
    renderHook(() => useAutosave({ value: { v: 1 }, write, debounceMs: 100 }));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(500);
    });
    expect(write).not.toHaveBeenCalled();
  });

  it('writes after debounce period when value changes', async () => {
    const write = vi.fn();
    const { rerender, result } = renderHook(
      ({ v }: { v: number }) => useAutosave({ value: { v }, write, debounceMs: 100 }),
      { initialProps: { v: 1 } }
    );
    rerender({ v: 2 });
    expect(result.current.status).toBe('saving');
    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });
    expect(write).toHaveBeenCalledTimes(1);
    expect(result.current.status).toBe('saved');
    expect(result.current.lastSavedAt).not.toBeNull();
  });

  it('coalesces rapid changes into a single write', async () => {
    const write = vi.fn();
    const { rerender } = renderHook(
      ({ v }: { v: number }) => useAutosave({ value: { v }, write, debounceMs: 100 }),
      { initialProps: { v: 1 } }
    );
    rerender({ v: 2 });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(50);
    });
    rerender({ v: 3 });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(50);
    });
    rerender({ v: 4 });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });
    expect(write).toHaveBeenCalledTimes(1);
  });

  it('reports error status when write throws', async () => {
    const write = vi.fn(() => {
      throw new Error('quota');
    });
    const { rerender, result } = renderHook(
      ({ v }: { v: number }) => useAutosave({ value: { v }, write, debounceMs: 50 }),
      { initialProps: { v: 1 } }
    );
    rerender({ v: 2 });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(50);
    });
    expect(result.current.status).toBe('error');
    expect(result.current.lastSavedAt).toBeNull();
  });

  it('skips writes when disabled', async () => {
    const write = vi.fn();
    const { rerender } = renderHook(
      ({ v, enabled }: { v: number; enabled: boolean }) =>
        useAutosave({ value: { v }, write, enabled, debounceMs: 50 }),
      { initialProps: { v: 1, enabled: false } }
    );
    rerender({ v: 2, enabled: false });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });
    expect(write).not.toHaveBeenCalled();
  });
});
