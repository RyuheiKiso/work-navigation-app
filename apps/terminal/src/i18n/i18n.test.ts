// 対応 §: ロードマップ §11.3 §13.1
// i18n 単体テスト。

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setLocale, t, getLocale, subscribeLocale } from './index';

describe('i18n', () => {
  // 各テスト前にデフォルトロケールへ戻す
  beforeEach(() => {
    setLocale('ja');
  });

  // ja で「作業」を返す
  it('returns Japanese term for task in ja locale', () => {
    expect(t('term.task')).toBe('作業');
  });

  // en で「Task」を返す
  it('returns English term for task in en locale', () => {
    setLocale('en');
    expect(t('term.task')).toBe('Task');
  });

  // ロケール切替が即時反映
  it('switches locale immediately', () => {
    expect(getLocale()).toBe('ja');
    setLocale('en');
    expect(getLocale()).toBe('en');
  });

  // 未登録キーはそのまま返る
  it('returns key as-is when missing', () => {
    expect(t('nonexistent.key')).toBe('nonexistent.key');
  });

  // 状態名 HSM の翻訳
  it('translates HSM state names', () => {
    expect(t('state.Running')).toBe('実行中');
    setLocale('en');
    expect(t('state.Running')).toBe('Running');
  });

  // §11.3.1 拡張ロケール: zh は「作业（操作）」（誤読回避）
  it('uses combined Chinese translation to avoid academic homework reading', () => {
    setLocale('zh');
    expect(t('term.task')).toBe('作业（操作）');
  });

  // ko の作業
  it('returns Korean term for task', () => {
    setLocale('ko');
    expect(t('term.task')).toBe('작업');
  });

  // de の作業
  it('returns German term for task', () => {
    setLocale('de');
    expect(t('term.task')).toBe('Aufgabe');
  });

  // es の作業
  it('returns Spanish term for task', () => {
    setLocale('es');
    expect(t('term.task')).toBe('Tarea');
  });

  // vi/th/id/fr/pt の作業
  it('returns ASEAN/Latin terms for task', () => {
    setLocale('vi');
    expect(t('term.task')).toBe('Công việc');
    setLocale('th');
    expect(t('term.task')).toBe('งาน');
    setLocale('id');
    expect(t('term.task')).toBe('Tugas');
    setLocale('fr');
    expect(t('term.task')).toBe('Tâche');
    setLocale('pt');
    expect(t('term.task')).toBe('Tarefa');
  });

  // ロケール変更の購読
  it('notifies subscribers on locale change', () => {
    const listener = vi.fn();
    const unsubscribe = subscribeLocale(listener);
    setLocale('en');
    setLocale('ar');
    expect(listener).toHaveBeenCalledTimes(2);
    expect(listener).toHaveBeenNthCalledWith(1, 'en');
    expect(listener).toHaveBeenNthCalledWith(2, 'ar');
    unsubscribe();
  });

  // 同値変更は subscribers を起こさない
  it('skips notifying subscribers when locale does not change', () => {
    setLocale('ja');
    const listener = vi.fn();
    const unsubscribe = subscribeLocale(listener);
    setLocale('ja');
    expect(listener).not.toHaveBeenCalled();
    unsubscribe();
  });

  // 解除後はもう呼ばれない
  it('stops notifying after unsubscribe', () => {
    const listener = vi.fn();
    const unsubscribe = subscribeLocale(listener);
    setLocale('en');
    unsubscribe();
    setLocale('zh');
    expect(listener).toHaveBeenCalledTimes(1);
  });

  // ar/he の作業＋ RTL 判定
  it('returns RTL terms and flags isRtl', async () => {
    const { isRtl } = await import('./index');
    setLocale('ar');
    expect(t('term.task')).toBe('مهمة');
    expect(isRtl()).toBe(true);
    setLocale('he');
    expect(t('term.task')).toBe('משימה');
    expect(isRtl()).toBe(true);
    // 非 RTL ロケール
    setLocale('ja');
    expect(isRtl()).toBe(false);
  });
});
