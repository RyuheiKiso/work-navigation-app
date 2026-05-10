// 対応 §: ロードマップ §11.3 §13.1 §14.2
// i18n 単体テスト。setLocale は dynamic import を伴うため await する。

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setLocale, t, getLocale, subscribeLocale, isRtl } from './index';

describe('i18n', () => {
  beforeEach(async () => {
    localStorage.removeItem('wna.terminal.locale');
    await setLocale('ja');
  });

  it('returns Japanese term for task in ja locale', () => {
    expect(t('term.task')).toBe('作業');
  });

  it('returns English term for task in en locale', async () => {
    await setLocale('en');
    expect(t('term.task')).toBe('Task');
  });

  it('switches locale immediately', async () => {
    expect(getLocale()).toBe('ja');
    await setLocale('en');
    expect(getLocale()).toBe('en');
  });

  it('returns key as-is when missing', () => {
    expect(t('nonexistent.key')).toBe('nonexistent.key');
  });

  it('translates HSM state names', async () => {
    expect(t('state.Running')).toBe('実行中');
    await setLocale('en');
    expect(t('state.Running')).toBe('Running');
  });

  // §11.3.1 拡張ロケール: zh は「作业（操作）」（誤読回避）
  it('uses combined Chinese translation to avoid academic homework reading', async () => {
    await setLocale('zh');
    expect(t('term.task')).toBe('作业（操作）');
  });

  it('returns Korean term for task', async () => {
    await setLocale('ko');
    expect(t('term.task')).toBe('작업');
  });

  it('returns German term for task', async () => {
    await setLocale('de');
    expect(t('term.task')).toBe('Aufgabe');
  });

  it('returns Spanish term for task', async () => {
    await setLocale('es');
    expect(t('term.task')).toBe('Tarea');
  });

  it('returns ASEAN/Latin terms for task', async () => {
    await setLocale('vi');
    expect(t('term.task')).toBe('Công việc');
    await setLocale('th');
    expect(t('term.task')).toBe('งาน');
    await setLocale('id');
    expect(t('term.task')).toBe('Tugas');
    await setLocale('fr');
    expect(t('term.task')).toBe('Tâche');
    await setLocale('pt');
    expect(t('term.task')).toBe('Tarefa');
  });

  it('persists the chosen locale to localStorage', async () => {
    await setLocale('ar');
    expect(localStorage.getItem('wna.terminal.locale')).toBe('ar');
  });

  it('notifies subscribers on locale change', async () => {
    const listener = vi.fn();
    const unsubscribe = subscribeLocale(listener);
    await setLocale('en');
    await setLocale('ar');
    expect(listener).toHaveBeenCalledTimes(2);
    // 通知は最終 currentLocale を渡す（ロード遅延中の連続切替を考慮）
    expect(listener.mock.calls[0]?.[0]).toBe('en');
    expect(listener.mock.calls[1]?.[0]).toBe('ar');
    unsubscribe();
  });

  it('skips notifying subscribers when locale does not change', async () => {
    await setLocale('ja');
    const listener = vi.fn();
    const unsubscribe = subscribeLocale(listener);
    await setLocale('ja');
    expect(listener).not.toHaveBeenCalled();
    unsubscribe();
  });

  it('stops notifying after unsubscribe', async () => {
    const listener = vi.fn();
    const unsubscribe = subscribeLocale(listener);
    await setLocale('en');
    unsubscribe();
    await setLocale('zh');
    expect(listener).toHaveBeenCalledTimes(1);
  });

  it('returns RTL terms and flags isRtl', async () => {
    await setLocale('ar');
    expect(t('term.task')).toBe('مهمة');
    expect(isRtl()).toBe(true);
    await setLocale('he');
    expect(t('term.task')).toBe('משימה');
    expect(isRtl()).toBe(true);
    await setLocale('ja');
    expect(isRtl()).toBe(false);
  });

});
