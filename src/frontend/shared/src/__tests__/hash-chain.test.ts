import { describe, expect, it } from 'vitest';
import {
  GENESIS_HASH,
  computeContentHash,
  toCanonicalJson,
  verifyChain,
} from '../domain/hash-chain';

describe('hash-chain', () => {
  it('GENESIS_HASH は 64 文字の 0 で構成される', () => {
    expect(GENESIS_HASH).toHaveLength(64);
    expect(GENESIS_HASH).toBe('0'.repeat(64));
  });

  it('toCanonicalJson はキー順序を辞書順に揃える', () => {
    const a = toCanonicalJson({ b: 1, a: 2 });
    const b = toCanonicalJson({ a: 2, b: 1 });
    expect(a).toBe(b);
    expect(a).toBe('{"a":2,"b":1}');
  });

  it('toCanonicalJson はネストオブジェクトの順序も再帰的に揃える', () => {
    const a = toCanonicalJson({ outer: { z: 1, a: 2 }, key: [{ y: 3, x: 4 }] });
    expect(a).toBe('{"key":[{"x":4,"y":3}],"outer":{"a":2,"z":1}}');
  });

  it('computeContentHash は同じ入力に対して決定論的にハッシュを返す', () => {
    const payload = { activity: 'step_completed', stepId: 'step-1' };
    const h1 = computeContentHash(GENESIS_HASH, payload);
    const h2 = computeContentHash(GENESIS_HASH, payload);
    expect(h1).toBe(h2);
    expect(h1).toMatch(/^[0-9a-f]{64}$/);
  });

  it('computeContentHash は prevHash が異なれば結果も異なる', () => {
    const payload = { activity: 'step_completed' };
    const h1 = computeContentHash(GENESIS_HASH, payload);
    const h2 = computeContentHash('a'.repeat(64), payload);
    expect(h1).not.toBe(h2);
  });

  it('verifyChain は空配列を許容する', () => {
    expect(verifyChain([])).toBe(true);
  });

  it('verifyChain は正常なチェーンを true として検証する', () => {
    const payload1 = { activity: 'work_started' };
    const hash1 = computeContentHash(GENESIS_HASH, payload1);
    const payload2 = { activity: 'step_completed' };
    const hash2 = computeContentHash(hash1, payload2);
    const events = [
      { prevHash: GENESIS_HASH, contentHash: hash1, payload: payload1 },
      { prevHash: hash1, contentHash: hash2, payload: payload2 },
    ];
    expect(verifyChain(events)).toBe(true);
  });

  it('verifyChain は prevHash の改竄を検出する', () => {
    const payload1 = { activity: 'work_started' };
    const hash1 = computeContentHash(GENESIS_HASH, payload1);
    const payload2 = { activity: 'step_completed' };
    const hash2 = computeContentHash(hash1, payload2);
    const events = [
      { prevHash: GENESIS_HASH, contentHash: hash1, payload: payload1 },
      { prevHash: 'f'.repeat(64), contentHash: hash2, payload: payload2 },
    ];
    expect(verifyChain(events)).toBe(false);
  });

  it('verifyChain は contentHash の改竄を検出する', () => {
    const payload1 = { activity: 'work_started' };
    const hash1 = computeContentHash(GENESIS_HASH, payload1);
    const events = [
      { prevHash: GENESIS_HASH, contentHash: 'f'.repeat(64), payload: payload1 },
    ];
    expect(verifyChain(events)).toBe(false);
  });

  it('verifyChain は payload の改竄を検出する（contentHash と不整合）', () => {
    const payload1 = { activity: 'work_started' };
    const hash1 = computeContentHash(GENESIS_HASH, payload1);
    const events = [
      { prevHash: GENESIS_HASH, contentHash: hash1, payload: { activity: 'tampered' } },
    ];
    expect(verifyChain(events)).toBe(false);
  });
});
