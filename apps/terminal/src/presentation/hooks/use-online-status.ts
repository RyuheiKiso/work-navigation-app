// 対応 §: ロードマップ §10.6 §3.6.4 §14.2
// ブラウザのオンライン/オフライン状態を購読する Hook。
// 製造現場で「なぜタスクが進まないか」を即座に把握させるため、
// UI 側に navigator.onLine の連続 polling ではなくイベント駆動の真値を提供する。

import { useEffect, useState } from 'react';

export function useOnlineStatus(): boolean {
  const [online, setOnline] = useState<boolean>(() => readInitial());

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const onOnline = (): void => setOnline(true);
    const onOffline = (): void => setOnline(false);
    window.addEventListener('online', onOnline);
    window.addEventListener('offline', onOffline);
    // 初回マウント時に navigator.onLine を再評価する（hydration 後の差分対策）
    setOnline(readInitial());
    return () => {
      window.removeEventListener('online', onOnline);
      window.removeEventListener('offline', onOffline);
    };
  }, []);

  return online;
}

function readInitial(): boolean {
  // navigator が無い実行環境ではオンライン扱い (機能制限を作らない)
  if (typeof navigator === 'undefined') return true;
  return navigator.onLine;
}
