// 対応 §: ロードマップ §11.3.1 §11.2 §3.6.4
// `<html>` 要素の lang / dir 属性を現在ロケールと同期させる Hook。
// アプリ最上位 (App.tsx) で 1 度だけ呼ぶ。
// 副作用は document に対するものだけで、コンポーネント state は持たない。

import { useEffect } from 'react';
import { getLocale, isRtl, subscribeLocale, type LocaleKey } from '../../i18n';

export function useDocumentLocale(): void {
  useEffect(() => {
    // jsdom 含む SSR-like 環境でも安全に動作させる
    const root = typeof document !== 'undefined' ? document.documentElement : null;
    if (root === null) return;
    const apply = (l: LocaleKey): void => {
      root.lang = l;
      root.dir = isRtl(l) ? 'rtl' : 'ltr';
    };
    apply(getLocale());
    return subscribeLocale(apply);
  }, []);
}
