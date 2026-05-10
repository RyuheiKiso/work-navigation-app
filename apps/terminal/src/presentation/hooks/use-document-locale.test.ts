// 対応 §: ロードマップ §13.1 §11.3.1
// useDocumentLocale Hook が `<html>` の lang/dir を locale 変更に追随させることを検証する。

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useDocumentLocale } from './use-document-locale';
import { setLocale } from '../../i18n';

describe('useDocumentLocale', () => {
  beforeEach(() => {
    // テスト独立化のため、ja 既定で初期化する
    setLocale('ja');
    document.documentElement.lang = 'ja';
    document.documentElement.dir = '';
  });
  afterEach(() => {
    setLocale('ja');
  });

  it('sets lang and dir to current locale on mount', () => {
    setLocale('en');
    renderHook(() => useDocumentLocale());
    expect(document.documentElement.lang).toBe('en');
    expect(document.documentElement.dir).toBe('ltr');
  });

  it('switches dir to rtl for arabic locale', () => {
    renderHook(() => useDocumentLocale());
    setLocale('ar');
    expect(document.documentElement.lang).toBe('ar');
    expect(document.documentElement.dir).toBe('rtl');
  });

  it('switches back to ltr after leaving rtl locale', () => {
    renderHook(() => useDocumentLocale());
    setLocale('he');
    expect(document.documentElement.dir).toBe('rtl');
    setLocale('ja');
    expect(document.documentElement.dir).toBe('ltr');
    expect(document.documentElement.lang).toBe('ja');
  });

  it('unsubscribes on unmount', () => {
    const { unmount } = renderHook(() => useDocumentLocale());
    // マウント時は ja を適用しているので一旦 ltr になる
    expect(document.documentElement.dir).toBe('ltr');
    unmount();
    setLocale('ar');
    // 購読解除済みなので、ar に切り替えても dir は ltr のまま
    expect(document.documentElement.dir).toBe('ltr');
    expect(document.documentElement.lang).toBe('ja');
  });
});
