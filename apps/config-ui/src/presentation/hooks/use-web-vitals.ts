// 対応 §: ロードマップ §31.1 §31.2 §14.2 ／ ADR-0012 P2-15
// Web Vitals を購読し、telemetry レイヤへ送る Hook。
// アプリのルート (App.tsx) で 1 度だけ呼ぶ。
// onLCP/onINP/onCLS/onFCP/onTTFB はそれぞれ 1 度（CLS は更新あり）発火する。

import { useEffect } from 'react';
import { onLCP, onINP, onCLS, onFCP, onTTFB, type Metric } from 'web-vitals';
import { reportVital } from '../../adapter/telemetry';

export function useWebVitals(): void {
  useEffect(() => {
    if (typeof window === 'undefined') return;
    const navId = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const send = (m: Metric): void => {
      void reportVital({
        name: m.name as 'LCP' | 'INP' | 'CLS' | 'FCP' | 'TTFB',
        value: m.value,
        rating: m.rating,
        navigation_id: navId,
        ts: Date.now()
      });
    };
    onLCP(send);
    onINP(send);
    onCLS(send);
    onFCP(send);
    onTTFB(send);
  }, []);
}
