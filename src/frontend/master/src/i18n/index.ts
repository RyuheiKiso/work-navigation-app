import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import type { Locale } from '@wnav/shared/types';

// UI 固定文言のみ静的リソース。マスタ翻訳は JSONB を resolveLocale で動的解決する（src/CLAUDE.md §多言語対応）。
const resources: Record<Locale, { translation: Record<string, string> }> = {
  ja: {
    translation: {
      'common.save': '保存',
      'common.cancel': 'キャンセル',
      'common.delete': '廃止',
      'common.edit': '編集',
      'common.create': '新規作成',
      'common.search': '検索',
      'common.refresh': '更新',
      'common.export': 'エクスポート',
      'common.import': 'インポート',
      'common.confirm': '確認',
      'common.back': '戻る',
      'common.loading': '読み込み中...',
      'common.no_data': 'データがありません',
      'auth.login': 'ログイン',
      'auth.logout': 'ログアウト',
      'auth.unauthorized': '権限がありません',
    },
  },
  en: {
    translation: {
      'common.save': 'Save',
      'common.cancel': 'Cancel',
      'common.delete': 'Deprecate',
      'common.edit': 'Edit',
      'common.create': 'Create',
      'common.search': 'Search',
      'common.refresh': 'Refresh',
      'common.export': 'Export',
      'common.import': 'Import',
      'common.confirm': 'Confirm',
      'common.back': 'Back',
      'common.loading': 'Loading...',
      'common.no_data': 'No data',
      'auth.login': 'Login',
      'auth.logout': 'Logout',
      'auth.unauthorized': 'Unauthorized',
    },
  },
  zh: {
    translation: {
      'common.save': '保存',
      'common.cancel': '取消',
      'common.delete': '废止',
      'common.edit': '编辑',
      'common.create': '新建',
      'common.search': '搜索',
      'common.refresh': '刷新',
      'common.export': '导出',
      'common.import': '导入',
      'common.confirm': '确认',
      'common.back': '返回',
      'common.loading': '加载中...',
      'common.no_data': '无数据',
      'auth.login': '登录',
      'auth.logout': '注销',
      'auth.unauthorized': '无权限',
    },
  },
};

void i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: 'ja',
    fallbackLng: 'ja',
    interpolation: { escapeValue: false },
    returnNull: false,
  });

export default i18n;
