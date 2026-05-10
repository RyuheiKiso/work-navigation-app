// 対応 §: ロードマップ §11.3 §13.1
// 設定 UI i18n の単体テスト。

import { describe, it, expect, beforeEach } from 'vitest';
import { setLocale, t } from './index';

describe('config-ui i18n', () => {
  // 各テスト前にデフォルトへ戻す
  beforeEach(() => {
    localStorage.removeItem('wna.config-ui.locale');
    setLocale('ja');
  });

  it('returns Japanese for fall-back locale', () => {
    expect(t('flow.publish_trial_button')).toBe('試行版を発行する');
  });

  it('switches to English on demand', () => {
    setLocale('en');
    expect(t('flow.publish_trial_button')).toBe('Publish trial');
  });

  it('returns key as-is when missing', () => {
    expect(t('does.not.exist')).toBe('does.not.exist');
  });

  it('persists the chosen locale to localStorage', () => {
    setLocale('en');
    expect(localStorage.getItem('wna.config-ui.locale')).toBe('en');
  });
});
