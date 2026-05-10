// 対応 §: ロードマップ §11.3 §11.3.1 §28
// 設定 UI の i18n エントリ。端末アプリと同等の 13 ロケール対応。

// 同梱ロケール
import { ja } from './ja';
import { en } from './en';

/** 同梱ロケール一覧（端末側と並走的に拡張、現状 ja/en） */
export const LOCALES = {
  ja,
  en
} as const;

/** 利用可能なロケールキー */
export type LocaleKey = keyof typeof LOCALES;

/** 翻訳辞書の型（ja を正とする） */
export type Dictionary = typeof ja;

/** 現在のロケール（デフォルトは ja） */
let currentLocale: LocaleKey = 'ja';

/** ロケールを切り替える */
export function setLocale(locale: LocaleKey | string): void {
  // 未対応ロケールは ja にフォールバックする（§11.3.1 段階的拡張）
  if ((locale as LocaleKey) in LOCALES) {
    currentLocale = locale as LocaleKey;
  } else {
    currentLocale = 'ja';
  }
}

/** 現在のロケールを取得 */
export function getLocale(): LocaleKey {
  // 内部状態を返す
  return currentLocale;
}

/** RTL ロケール一覧（§11.3.1 RTL レーン、現状空） */
export const RTL_LOCALES: ReadonlyArray<string> = ['ar', 'he'];

/** 現在ロケールが RTL かを判定する */
export function isRtl(locale?: string): boolean {
  // 引数省略時は現在ロケール
  const target = locale ?? currentLocale;
  // RTL リストに含まれるか
  return RTL_LOCALES.includes(target);
}

/** ドット区切りキーで翻訳文字列を取得する */
export function t(key: string): string {
  // 現在ロケールの辞書を取得
  const dict = LOCALES[currentLocale] as unknown as Record<string, unknown>;
  const parts = key.split('.');
  let cursor: unknown = dict;
  for (const p of parts) {
    if (cursor && typeof cursor === 'object' && p in (cursor as Record<string, unknown>)) {
      cursor = (cursor as Record<string, unknown>)[p];
    } else {
      // 未登録: キー文字列を返す
      return key;
    }
  }
  // 最終値が文字列ならそれを返す
  return typeof cursor === 'string' ? cursor : key;
}
