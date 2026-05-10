// 対応 §: ロードマップ §13.1 §11.3.1 §14.2
// useDocumentLocale Hook が `<html>` の lang/dir を locale 変更に追随させることを検証する。
// setLocale は dynamic import を伴うため await する。

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useDocumentLocale } from './use-document-locale';
import { setLocale } from '../../i18n';

describe('useDocumentLocale', () => {
  beforeEach(async () => {
    await setLocale('ja');
    document.documentElement.lang = 'ja';
    document.documentElement.dir = '';
  });
  afterEach(async () => {
    await setLocale('ja');
  });

  it('sets lang and dir to current locale on mount', async () => {
    await setLocale('en');
    renderHook(() => useDocumentLocale());
    expect(document.documentElement.lang).toBe('en');
    expect(document.documentElement.dir).toBe('ltr');
  });

  it('switches dir to rtl for arabic locale', async () => {
    renderHook(() => useDocumentLocale());
    await setLocale('ar');
    expect(document.documentElement.lang).toBe('ar');
    expect(document.documentElement.dir).toBe('rtl');
  });

  it('switches back to ltr after leaving rtl locale', async () => {
    renderHook(() => useDocumentLocale());
    await setLocale('he');
    expect(document.documentElement.dir).toBe('rtl');
    await setLocale('ja');
    expect(document.documentElement.dir).toBe('ltr');
    expect(document.documentElement.lang).toBe('ja');
  });

  it('unsubscribes on unmount', async () => {
    const { unmount } = renderHook(() => useDocumentLocale());
    expect(document.documentElement.dir).toBe('ltr');
    unmount();
    await setLocale('ar');
    expect(document.documentElement.dir).toBe('ltr');
    expect(document.documentElement.lang).toBe('ja');
  });
});
