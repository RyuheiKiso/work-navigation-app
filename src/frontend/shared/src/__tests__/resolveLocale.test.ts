import { describe, expect, it } from 'vitest';
import { resolveLocale } from '../i18n/resolveLocale';

describe('resolveLocale', () => {
  const text = { ja: '溶接工程', en: 'Welding Process', zh: '焊接工序' };

  it('ja ロケールで ja を返す', () => {
    expect(resolveLocale(text, 'ja')).toBe('溶接工程');
  });

  it('en ロケールで en を返す', () => {
    expect(resolveLocale(text, 'en')).toBe('Welding Process');
  });

  it('zh ロケールで zh を返す', () => {
    expect(resolveLocale(text, 'zh')).toBe('焊接工序');
  });

  it('未設定ロケールでも ja にフォールバックする', () => {
    expect(resolveLocale({ ja: '溶接', en: '', zh: '' }, 'en')).toBe('溶接');
  });

  it('null/undefined 入力は空文字を返す', () => {
    expect(resolveLocale(null)).toBe('');
    expect(resolveLocale(undefined)).toBe('');
  });

  it('ロケール省略時は ja を返す', () => {
    expect(resolveLocale(text)).toBe('溶接工程');
  });
});
