// ロケール選択コンテキスト。i18next の言語切替と連動する
import React, { createContext, useCallback, useContext, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Locale } from '@wnav/shared';

interface LocaleContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
}

const LocaleContext = createContext<LocaleContextValue | null>(null);

export function LocaleProvider({ children }: { children: React.ReactNode }): JSX.Element {
  const { i18n } = useTranslation();
  const [locale, setLocaleState] = useState<Locale>((i18n.language as Locale) ?? 'ja');

  const setLocale = useCallback(
    (next: Locale) => {
      setLocaleState(next);
      void i18n.changeLanguage(next);
    },
    [i18n],
  );

  const value = useMemo(() => ({ locale, setLocale }), [locale, setLocale]);
  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>;
}

export function useLocale(): LocaleContextValue {
  const ctx = useContext(LocaleContext);
  if (ctx === null) throw new Error('useLocale must be used within LocaleProvider');
  return ctx;
}
