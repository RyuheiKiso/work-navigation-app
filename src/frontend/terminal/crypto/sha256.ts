// SHA-256 ラッパ。canonical JSON を生成してから 256bit ハッシュを計算する
import { sha256 as nobleSha256 } from '@noble/hashes/sha2';
import { bytesToHex } from '@noble/hashes/utils';
import { toCanonicalJson } from '@wnav/shared/domain/hash-chain';

export function sha256Hex(input: Uint8Array | string): string {
  const bytes = typeof input === 'string' ? new TextEncoder().encode(input) : input;
  return bytesToHex(nobleSha256(bytes));
}

// 任意のオブジェクトを canonical JSON 化して SHA-256 を計算する
export function sha256OfJson(value: unknown): string {
  const canonical = toCanonicalJson(value);
  return sha256Hex(canonical);
}
