import { sha256 } from '@noble/hashes/sha2';
import { bytesToHex } from '@noble/hashes/utils';

// ジェネシス（最初のイベント）の prevHash は 64 文字の 0 で固定する
export const GENESIS_HASH = '0'.repeat(64);

// キー順序を固定してハッシュの再現性を保証する（改竄検知の基盤）
export function toCanonicalJson(obj: unknown): string {
  if (obj === null || obj === undefined) return JSON.stringify(obj ?? null);
  if (typeof obj !== 'object') return JSON.stringify(obj);
  if (Array.isArray(obj)) {
    return '[' + obj.map((v) => toCanonicalJson(v)).join(',') + ']';
  }
  const entries = Object.entries(obj as Record<string, unknown>).sort(([a], [b]) =>
    a < b ? -1 : a > b ? 1 : 0,
  );
  return (
    '{' +
    entries.map(([k, v]) => `${JSON.stringify(k)}:${toCanonicalJson(v)}`).join(',') +
    '}'
  );
}

// SHA-256 チェーン: 各イベントに前イベントのハッシュを含めて改竄を構造的に検出する
export function computeContentHash(prevHash: string, payload: unknown): string {
  const canonical = toCanonicalJson({ prevHash, payload });
  const bytes = new TextEncoder().encode(canonical);
  return bytesToHex(sha256(bytes));
}

// チェーン全体の連続性と各 contentHash の整合性を検証する
export function verifyChain(
  events: ReadonlyArray<{ contentHash: string; prevHash: string; payload: unknown }>,
): boolean {
  if (events.length === 0) return true;
  for (let i = 0; i < events.length; i++) {
    const ev = events[i];
    if (ev === undefined) return false;
    const expectedPrev = i === 0 ? GENESIS_HASH : events[i - 1]!.contentHash;
    if (ev.prevHash !== expectedPrev) return false;
    if (ev.contentHash !== computeContentHash(ev.prevHash, ev.payload)) return false;
  }
  return true;
}
