// 対応 §: ロードマップ §11.3 §11.3.1 §3.1.5.2 §28 §14.2
// 端末アプリの i18n エントリ。
// ja は static import（必ず必要・フォールバック先）。
// 他 12 言語は dynamic import で必要時にだけロードし、初期 LCP を改善する。
// 切替直後はまだ ja で描画されるが、ロード完了後 subscribe 経路で UI が再描画される。

// ja は常に必要（フォールバック / 初期描画 / 翻訳テーブルの型源）
import { ja } from './ja';

/** 翻訳辞書の型（ja を正とする） */
export type Dictionary = typeof ja;

/** 利用可能なロケールキー（§11.3.1 段階的拡張） */
export type LocaleKey =
  | 'ja'
  | 'en'
  | 'zh'
  | 'ko'
  | 'de'
  | 'es'
  | 'vi'
  | 'th'
  | 'id'
  | 'fr'
  | 'pt'
  | 'ar'
  | 'he';

/** 表示用の全ロケールキー */
export const LOCALE_KEYS: ReadonlyArray<LocaleKey> = [
  'ja', 'en', 'zh', 'ko', 'de', 'es', 'vi', 'th', 'id', 'fr', 'pt', 'ar', 'he'
];

/** RTL ロケール一覧（§11.3.1 RTL レーン） */
export const RTL_LOCALES: ReadonlyArray<LocaleKey> = ['ar', 'he'];

/** 現在ロケールが RTL かどうかを判定する */
export function isRtl(locale?: LocaleKey): boolean {
  const target = locale ?? getLocale();
  return RTL_LOCALES.includes(target);
}

/** 動的ローダ。Vite が自動で言語ごとの chunk を切る */
const LOADERS: Record<Exclude<LocaleKey, 'ja'>, () => Promise<Dictionary>> = {
  en: () => import('./en').then((m) => m.en as unknown as Dictionary),
  zh: () => import('./zh').then((m) => m.zh as unknown as Dictionary),
  ko: () => import('./ko').then((m) => m.ko as unknown as Dictionary),
  de: () => import('./de').then((m) => m.de as unknown as Dictionary),
  es: () => import('./es').then((m) => m.es as unknown as Dictionary),
  vi: () => import('./vi').then((m) => m.vi as unknown as Dictionary),
  th: () => import('./th').then((m) => m.th as unknown as Dictionary),
  id: () => import('./id').then((m) => m.id as unknown as Dictionary),
  fr: () => import('./fr').then((m) => m.fr as unknown as Dictionary),
  pt: () => import('./pt').then((m) => m.pt as unknown as Dictionary),
  ar: () => import('./ar').then((m) => m.ar as unknown as Dictionary),
  he: () => import('./he').then((m) => m.he as unknown as Dictionary)
};

/** 既ロード済み辞書のキャッシュ。ja は常駐 */
const LOADED: Partial<Record<LocaleKey, Dictionary>> = { ja };

/** localStorage の保存キー */
const LOCALE_STORAGE_KEY = 'wna.terminal.locale';

function readPersistedLocale(): LocaleKey {
  try {
    const v = globalThis.localStorage?.getItem(LOCALE_STORAGE_KEY);
    if (v && (LOCALE_KEYS as ReadonlyArray<string>).includes(v)) return v as LocaleKey;
  } catch {
    // localStorage アクセス失敗は無視
  }
  return 'ja';
}

/** 現在のロケール（永続化された値、または ja） */
let currentLocale: LocaleKey = readPersistedLocale();

type LocaleListener = (locale: LocaleKey) => void;
const localeListeners: Set<LocaleListener> = new Set();

/** 現在のロケールを取得 */
export function getLocale(): LocaleKey {
  return currentLocale;
}

/**
 * ロケールを切り替える。
 *
 * 同期 API を維持するため、ロード完了前に呼び出した場合は
 * currentLocale を即座に書き替えるが描画は ja フォールバックで継続される。
 * 該当辞書がロードされ次第、購読者へ通知して UI が再描画される。
 *
 * Promise を await すれば、ロード完了後に解決する。
 */
export function setLocale(locale: LocaleKey): Promise<void> {
  if (currentLocale === locale && LOADED[locale]) return Promise.resolve();
  // localStorage への保存は即時
  try {
    globalThis.localStorage?.setItem(LOCALE_STORAGE_KEY, locale);
  } catch {
    // 失敗しても UI は止めない
  }
  currentLocale = locale;

  if (LOADED[locale]) {
    localeListeners.forEach((l) => l(locale));
    return Promise.resolve();
  }
  // 動的ロード
  const loader = (LOADERS as Record<string, () => Promise<Dictionary>>)[locale];
  if (!loader) {
    // 未定義キー (起こり得ないが防御的)
    localeListeners.forEach((l) => l(locale));
    return Promise.resolve();
  }
  return loader().then((dict) => {
    LOADED[locale] = dict;
    // 切替が複数回連続した場合、現在ロケールが変わっていても全購読者に通知（最新値を渡す）
    localeListeners.forEach((l) => l(currentLocale));
  });
}

/**
 * ロケール変更の購読。返り値の関数で購読解除する。
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
 * 現在ロケールの辞書がまだロードされていない場合は ja にフォールバックする。
 */
export function t(key: string): string {
  const dict = LOADED[currentLocale] ?? LOADED.ja!;
  const v = lookup(dict, key);
  if (v !== undefined) return v;
  if (dict !== LOADED.ja) {
    const fb = lookup(LOADED.ja!, key);
    if (fb !== undefined) return fb;
  }
  return key;
}

function lookup(dict: unknown, key: string): string | undefined {
  const parts = key.split('.');
  let cursor: unknown = dict;
  for (const p of parts) {
    if (cursor && typeof cursor === 'object' && p in (cursor as Record<string, unknown>)) {
      cursor = (cursor as Record<string, unknown>)[p];
    } else {
      return undefined;
    }
  }
  return typeof cursor === 'string' ? cursor : undefined;
}
