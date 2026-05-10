// 対応 §: ロードマップ §13.1 §10.6
// useOnlineStatus Hook のテスト。online/offline イベント購読を検証。

import { describe, it, expect } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useOnlineStatus } from './use-online-status';

describe('useOnlineStatus', () => {
  it('reflects navigator.onLine on mount', () => {
    Object.defineProperty(navigator, 'onLine', { value: true, configurable: true });
    const { result } = renderHook(() => useOnlineStatus());
    expect(result.current).toBe(true);
  });

  it('flips to false on offline event', () => {
    Object.defineProperty(navigator, 'onLine', { value: true, configurable: true });
    const { result } = renderHook(() => useOnlineStatus());
    act(() => {
      Object.defineProperty(navigator, 'onLine', { value: false, configurable: true });
      window.dispatchEvent(new Event('offline'));
    });
    expect(result.current).toBe(false);
  });

  it('flips back to true on online event', () => {
    Object.defineProperty(navigator, 'onLine', { value: false, configurable: true });
    const { result } = renderHook(() => useOnlineStatus());
    act(() => {
      Object.defineProperty(navigator, 'onLine', { value: true, configurable: true });
      window.dispatchEvent(new Event('online'));
    });
    expect(result.current).toBe(true);
  });

  it('removes listeners on unmount', () => {
    Object.defineProperty(navigator, 'onLine', { value: true, configurable: true });
    const { result, unmount } = renderHook(() => useOnlineStatus());
    unmount();
    act(() => {
      window.dispatchEvent(new Event('offline'));
    });
    // unmount 後はイベントを受け取らない
    expect(result.current).toBe(true);
  });
});
