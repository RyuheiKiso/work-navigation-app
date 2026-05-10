// 対応 §: ロードマップ §10.4.4 §13.1
// ストレージライフサイクル判定の単体テスト。

import { describe, it, expect } from 'vitest';
import { canCapture, evaluate, BLOCK_RATIO, WARNING_RATIO } from './storage-lifecycle';

describe('storage lifecycle', () => {
  // 通常状態（50%）
  it('returns normal status under warning threshold', () => {
    const r = evaluate({ totalBytes: 1000, usedBytes: 500 });
    expect(r.status).toBe('normal');
    expect(r.utilization).toBeCloseTo(0.5);
  });

  // 警告状態（80%）
  it('returns warning at 80% utilization', () => {
    const r = evaluate({ totalBytes: 1000, usedBytes: 800 });
    expect(r.status).toBe('warning');
    expect(r.utilization).toBeCloseTo(WARNING_RATIO);
  });

  // ブロック状態（90%）
  it('blocks at 90% utilization', () => {
    const r = evaluate({ totalBytes: 1000, usedBytes: 900 });
    expect(r.status).toBe('blocked');
    expect(r.utilization).toBeCloseTo(BLOCK_RATIO);
    // 撮影不可
    expect(canCapture({ totalBytes: 1000, usedBytes: 900 })).toBe(false);
  });

  // 警告でも撮影は可能
  it('allows capture in warning state', () => {
    expect(canCapture({ totalBytes: 1000, usedBytes: 800 })).toBe(true);
  });

  // 不正な totalBytes
  it('throws on non-positive totalBytes', () => {
    expect(() => evaluate({ totalBytes: 0, usedBytes: 0 })).toThrow(/正の整数/);
  });

  // §20.1 「人を責めない」: メッセージにユーザを責める表現を含めない
  it('uses non-blaming language', () => {
    const r = evaluate({ totalBytes: 1000, usedBytes: 950 });
    // 「あなたが」「ユーザが」等を含まない
    expect(r.message).not.toMatch(/あなた|ユーザが/);
  });
});
