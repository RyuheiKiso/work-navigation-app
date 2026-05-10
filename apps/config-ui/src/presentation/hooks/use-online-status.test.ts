// 対応 §: ロードマップ §13.1 §10.6
// useOnlineStatus Hook のテスト（config-ui）。

import { describe, it, expect } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useOnlineStatus } from './use-online-status';

describe('useOnlineStatus', () => {
  it('reflects navigator.onLine on mount', () => {
    Object.defineProperty(navigator, 'onLine', { value: true, configurable: true });
    const { result } = renderHook(() => useOnlineStatus());
    expect(result.current).toBe(true);
  });

  it('flips on offline / online events', () => {
    Object.defineProperty(navigator, 'onLine', { value: true, configurable: true });
    const { result } = renderHook(() => useOnlineStatus());
    act(() => {
      Object.defineProperty(navigator, 'onLine', { value: false, configurable: true });
      window.dispatchEvent(new Event('offline'));
    });
    expect(result.current).toBe(false);
    act(() => {
      Object.defineProperty(navigator, 'onLine', { value: true, configurable: true });
      window.dispatchEvent(new Event('online'));
    });
    expect(result.current).toBe(true);
  });
});
