// react-i18next 初期化。UI 固定文言を ja/en/zh で持ち、マスタは resolveLocale で動的解決する
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

const resources = {
  ja: {
    translation: {
      common: {
        next: '次へ',
        back: '戻る',
        cancel: 'キャンセル',
        confirm: '確定',
        save: '保存',
        loading: '読み込み中…',
        error: 'エラー',
        retry: '再試行',
      },
      auth: {
        loginTitle: 'ログイン',
        loginId: 'ログイン ID',
        password: 'パスワード',
        login: 'ログイン',
        loginFailed: 'ログインに失敗しました',
      },
      home: { title: '作業一覧', emptyAssignments: '未着手の作業はありません' },
      step: { complete: 'ステップ完了', skip: 'スキップ' },
      suspend: { title: '作業中断', reason: '中断理由', detail: '詳細' },
      resume: { title: '作業再開' },
      andon: { title: 'アンドン発報' },
      nonconformity: { title: '不適合登録' },
      iqc: { receive: '受入登録', measure: '測定値入力', verdict: '判定' },
      rework: {
        execute: '修正作業',
        reInspection: '再検査',
        scrap: '廃却',
        return: '仕入先返却',
      },
      settings: { title: '設定', language: '言語', network: 'ネットワーク' },
      network: { emergency: '緊急モード（オフライン）', lastSync: '最終同期' },
    },
  },
  en: {
    translation: {
      common: {
        next: 'Next',
        back: 'Back',
        cancel: 'Cancel',
        confirm: 'Confirm',
        save: 'Save',
        loading: 'Loading…',
        error: 'Error',
        retry: 'Retry',
      },
      auth: {
        loginTitle: 'Sign In',
        loginId: 'Login ID',
        password: 'Password',
        login: 'Sign In',
        loginFailed: 'Sign-in failed',
      },
      home: { title: 'Work Orders', emptyAssignments: 'No pending work' },
      step: { complete: 'Complete Step', skip: 'Skip' },
      suspend: { title: 'Suspend Work', reason: 'Reason', detail: 'Detail' },
      resume: { title: 'Resume Work' },
      andon: { title: 'Raise Andon' },
      nonconformity: { title: 'Register Nonconformity' },
      iqc: { receive: 'Receive', measure: 'Measure', verdict: 'Verdict' },
      rework: {
        execute: 'Rework',
        reInspection: 'Re-Inspection',
        scrap: 'Scrap',
        return: 'Return to Vendor',
      },
      settings: { title: 'Settings', language: 'Language', network: 'Network' },
      network: { emergency: 'Emergency Mode (Offline)', lastSync: 'Last Sync' },
    },
  },
  zh: {
    translation: {
      common: {
        next: '下一步',
        back: '返回',
        cancel: '取消',
        confirm: '确定',
        save: '保存',
        loading: '加载中…',
        error: '错误',
        retry: '重试',
      },
      auth: {
        loginTitle: '登录',
        loginId: '登录ID',
        password: '密码',
        login: '登录',
        loginFailed: '登录失败',
      },
      home: { title: '作业列表', emptyAssignments: '没有待办作业' },
      step: { complete: '完成步骤', skip: '跳过' },
      suspend: { title: '中断作业', reason: '中断原因', detail: '详细' },
      resume: { title: '恢复作业' },
      andon: { title: '安灯报警' },
      nonconformity: { title: '不合格登记' },
      iqc: { receive: '收货', measure: '测量', verdict: '判定' },
      rework: {
        execute: '返工',
        reInspection: '复检',
        scrap: '报废',
        return: '退货',
      },
      settings: { title: '设置', language: '语言', network: '网络' },
      network: { emergency: '紧急模式（离线）', lastSync: '最后同步' },
    },
  },
} as const;

// シングルトン instance。RootLayout で initI18n() を呼び出してから使用する
export { i18n };

let initialized = false;

export async function initI18n(): Promise<void> {
  if (initialized) return;
  await i18n.use(initReactI18next).init({
    resources,
    lng: 'ja',
    fallbackLng: 'ja',
    interpolation: { escapeValue: false },
    compatibilityJSON: 'v4',
  });
  initialized = true;
}
