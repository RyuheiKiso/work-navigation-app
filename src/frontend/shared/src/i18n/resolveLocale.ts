import type { Locale, LocalizedText } from '../types';

// JSONB 多言語テキストをユーザーロケールで解決する（フォールバック: ロケール → ja → 空文字）
export function resolveLocale(
  text: LocalizedText | null | undefined,
  locale: Locale = 'ja',
): string {
  if (!text) return '';
  const candidate = text[locale];
  if (candidate && candidate.length > 0) return candidate;
  if (text.ja && text.ja.length > 0) return text.ja;
  return '';
}
