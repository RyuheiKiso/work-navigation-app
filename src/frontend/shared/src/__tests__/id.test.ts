import { describe, expect, it } from 'vitest';
import { generateId, isValidUuidV7, extractTimestampFromUuidV7 } from '../domain/id';

describe('id', () => {
  it('generateId は UUID v7 形式を返す', () => {
    const id = generateId();
    expect(isValidUuidV7(id)).toBe(true);
  });

  it('連続生成した ID はタイムスタンプ順に並ぶ', () => {
    const ids = Array.from({ length: 5 }, () => generateId());
    for (let i = 1; i < ids.length; i++) {
      const prev = extractTimestampFromUuidV7(ids[i - 1]!);
      const curr = extractTimestampFromUuidV7(ids[i]!);
      expect(prev).not.toBeNull();
      expect(curr).not.toBeNull();
      expect(curr!).toBeGreaterThanOrEqual(prev!);
    }
  });

  it('isValidUuidV7 は invalid 形式を拒否する', () => {
    expect(isValidUuidV7('not-a-uuid')).toBe(false);
    expect(isValidUuidV7('00000000-0000-1000-0000-000000000000')).toBe(false);
  });

  it('isValidUuidV7 は version 7 のみを許容する', () => {
    expect(isValidUuidV7('019682ab-7c1f-7000-a1b2-3c4d5e6f7890')).toBe(true);
    expect(isValidUuidV7('019682ab-7c1f-4000-a1b2-3c4d5e6f7890')).toBe(false);
  });
});
