// 対応 §: ロードマップ §10.6 §3.6.4 §14.2
// ブラウザのオンライン/オフライン状態を購読する Hook。
// 設定担当者が「保存できているのか」をひと目で判断できるよう、
// イベント駆動で navigator.onLine の真値を提供する。

import { useEffect, useState } from 'react';

export function useOnlineStatus(): boolean {
  const [online, setOnline] = useState<boolean>(() => readInitial());

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const onOnline = (): void => setOnline(true);
    const onOffline = (): void => setOnline(false);
    window.addEventListener('online', onOnline);
    window.addEventListener('offline', onOffline);
    setOnline(readInitial());
    return () => {
      window.removeEventListener('online', onOnline);
      window.removeEventListener('offline', onOffline);
    };
  }, []);

  return online;
}

function readInitial(): boolean {
  if (typeof navigator === 'undefined') return true;
  return navigator.onLine;
}
