// 対応 §: ロードマップ §11.3 §11.3.1 §3.1.5.2 §28
// 端末アプリの i18n エントリ。最小実装としてロケール辞書のロードと t() 関数を提供する。
// react-i18next 等の本格ライブラリへの差し替えは将来 ADR で記録する（§30 Type 1 候補）。

// 同梱ロケール（§11.3.1 段階的拡張、初版〜+24ヶ月＋RTL レーン）
import { ja } from './ja';
import { en } from './en';
import { zh } from './zh';
import { ko } from './ko';
import { de } from './de';
import { es } from './es';
import { vi } from './vi';
import { th } from './th';
import { id as idLocale } from './id';
import { fr } from './fr';
import { pt } from './pt';
import { ar } from './ar';
import { he } from './he';

/**
 * 同梱ロケール一覧（§11.3.1 拡張計画準拠）
 *
 * - 初版: ja, en
 * - +6ヶ月: zh（簡体）, ko
 * - +12ヶ月: de, es
 * - +18ヶ月: vi, th, id（インドネシア語）
 * - +24ヶ月: fr, pt
 * - RTL レーン: ar, he（先行投入で RTL レイアウトの早期検証、§11.3.1）
 *
 * `id` は globalThis.id 等とのキー名衝突を避けるため `idLocale` で import し、key として `id` で公開する。
 */
export const LOCALES = {
  ja,
  en,
  zh,
  ko,
  de,
  es,
  vi,
  th,
  id: idLocale,
  fr,
  pt,
  ar,
  he
} as const;

/**
 * RTL ロケール一覧（§11.3.1 RTL レーン）
 *
 * UI 側で `dir="rtl"` を設定する際の判定に使う。
 */
export const RTL_LOCALES: ReadonlyArray<keyof typeof LOCALES> = ['ar', 'he'];

/** 現在ロケールが RTL かどうかを判定する */
export function isRtl(locale?: keyof typeof LOCALES): boolean {
  // 引数省略時は現在ロケールを採用
  const target = locale ?? getLocale();
  // RTL リストに含まれるか
  return (RTL_LOCALES as ReadonlyArray<string>).includes(target);
}

/** 利用可能なロケールキー */
export type LocaleKey = keyof typeof LOCALES;

/** 翻訳辞書の型（ja を正とする） */
export type Dictionary = typeof ja;

/** 現在のロケール（デフォルトは ja） */
let currentLocale: LocaleKey = 'ja';

/** ロケール変更を購読するコールバック */
type LocaleListener = (locale: LocaleKey) => void;
const localeListeners: Set<LocaleListener> = new Set();

/** ロケールを切り替える */
export function setLocale(locale: LocaleKey): void {
  // 同値変更で購読側を不要に走らせない
  if (currentLocale === locale) return;
  currentLocale = locale;
  // §11.3.1 RTL レーン含むレイアウト副作用 (例: document.documentElement.dir) を購読側で同期する
  localeListeners.forEach((l) => l(locale));
}

/** 現在のロケールを取得 */
export function getLocale(): LocaleKey {
  // 内部状態を返す
  return currentLocale;
}

/**
 * ロケール変更の購読。返り値の関数で購読解除する。
 * `<html lang|dir>` 属性同期や数値・日付フォーマッタの差し替えに用いる。
 */
export function subscribeLocale(listener: LocaleListener): () => void {
  localeListeners.add(listener);
  return () => {
    localeListeners.delete(listener);
  };
}

/**
 * 翻訳関数。
 *
 * `t('task.start_button')` のようにドット区切りのキーで参照する。
 * 未登録キーはキー文字列をそのまま返す（§3.6.4 過剰提示／不足提示禁止のため
 * 「キーが見えてしまう」失敗パターンを開発時に検出しやすくする）。
 */
export function t(key: string): string {
  // 現在ロケールの辞書を取得
  const dict = LOCALES[currentLocale] as unknown as Record<string, string>;
  // ドット区切りで再帰的に解決する
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
