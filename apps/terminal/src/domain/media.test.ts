// 対応 §: ロードマップ §10.4 §13.1
// メディアドメインの単体テスト。

import { describe, it, expect } from 'vitest';
import { createMediaRef, isValidSha256, type MediaKind } from './media';

describe('MediaRef factory', () => {
  // 有効なハッシュ形式
  it('accepts valid SHA-256 hex', () => {
    expect(isValidSha256('a'.repeat(64))).toBe(true);
    expect(isValidSha256('A'.repeat(64))).toBe(true);
  });

  // 無効なハッシュ形式
  it('rejects invalid SHA-256 hex', () => {
    expect(isValidSha256('a'.repeat(63))).toBe(false);
    expect(isValidSha256('z'.repeat(64))).toBe(false);
  });

  // 妥当な構築
  it('creates frozen MediaRef', () => {
    const ref = createMediaRef({
      id: 'm-1',
      kind: 'photo' as MediaKind,
      sha256: 'a'.repeat(64),
      path: '/encrypted/store/m-1.jpg',
      bytes: 12345,
      capturedAt: '2026-05-10T00:00:00Z'
    });
    expect(ref.id).toBe('m-1');
    expect(ref.sha256).toBe('a'.repeat(64));
    expect(Object.isFrozen(ref)).toBe(true);
  });

  // ID 空は拒否
  it('rejects empty id', () => {
    expect(() =>
      createMediaRef({
        id: '',
        kind: 'photo',
        sha256: 'a'.repeat(64),
        path: '/x',
        bytes: 0,
        capturedAt: '2026-05-10T00:00:00Z'
      })
    ).toThrow(/id/);
  });

  // 不正ハッシュは拒否
  it('rejects invalid hash', () => {
    expect(() =>
      createMediaRef({
        id: 'm-2',
        kind: 'photo',
        sha256: 'invalid',
        path: '/x',
        bytes: 0,
        capturedAt: '2026-05-10T00:00:00Z'
      })
    ).toThrow(/SHA-256/);
  });

  // 負バイトは拒否
  it('rejects negative bytes', () => {
    expect(() =>
      createMediaRef({
        id: 'm-3',
        kind: 'photo',
        sha256: 'a'.repeat(64),
        path: '/x',
        bytes: -1,
        capturedAt: '2026-05-10T00:00:00Z'
      })
    ).toThrow(/非負/);
  });
});
