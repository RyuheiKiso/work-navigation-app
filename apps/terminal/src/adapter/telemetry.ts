// 対応 §: ロードマップ §31.1 §31.2 §11.4 §14.2 ／ ADR-0012
// フロントエンドのテレメトリ送信レイヤ（v1）。
// - Web Vitals (LCP/INP/CLS/FCP/TTFB) を 1 イベントずつ受け取り収集パイプへ流す。
// - ADR-0012 の PII マスキング規約に従い、user_id を SHA-256 短縮ハッシュに置換する。
// - v1 ではエンドポイントが未稼働のため console.info にフォールバックする。
//   将来 backend POST /telemetry/vitals を有効化したら ENDPOINT を切り替える。

export interface VitalEvent {
  name: 'LCP' | 'INP' | 'CLS' | 'FCP' | 'TTFB';
  value: number;
  rating?: 'good' | 'needs-improvement' | 'poor';
  /** マスク済み user_id（生 ID は決して含めない） */
  hashed_user_id?: string;
  navigation_id: string;
  ts: number;
}

const ENDPOINT: string | null = null;

/**
 * Web Vitals を 1 件送る。
 * 失敗してもフロントの動作を止めない（telemetry は best-effort）。
 */
export async function reportVital(event: VitalEvent): Promise<void> {
  if (ENDPOINT === null) {
    // 開発時の可視化と SLO ベースライン取得のためにコンソールへも出す
    // eslint-disable-next-line no-console
    console.info('[telemetry/vital]', event);
    return;
  }
  try {
    await fetch(ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(event),
      keepalive: true
    });
  } catch {
    // best-effort: 失敗時は捨てる
  }
}

/** ADR-0012 に従い user_id を SHA-256 で短縮する */
export async function hashUserId(userId: string): Promise<string> {
  if (typeof crypto === 'undefined' || !crypto.subtle) return 'na';
  const data = new TextEncoder().encode(userId);
  const buf = await crypto.subtle.digest('SHA-256', data);
  const hex = Array.from(new Uint8Array(buf))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  return hex.slice(0, 16);
}
