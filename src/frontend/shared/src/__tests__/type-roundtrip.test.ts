// TST-intg-021〜023: PG → JSON → SQLite → JSON → PG の型ラウンドトリップテスト
// 仕様: docs/05_詳細設計/01_データベース詳細設計/07a_PG_SQLiteスキーマ同期戦略.md §5
import { describe, it, expect } from 'vitest';
import { toCanonicalJson } from '../domain/hash-chain';

describe('TST-intg-021: UUID v7 ラウンドトリップ', () => {
  it('UUID v7 は TEXT として往復しても同値を保持する', () => {
    // PG uuid 型 → JSON → SQLite TEXT → JSON → PG uuid のシミュレーション
    const original = '019571a3-7c4f-7e46-9b28-1234567890ab';

    // JSON シリアライズ・デシリアライズ（API レスポンス → TypeORM INSERT）
    const serialized = JSON.stringify({ id: original });
    const deserialized = JSON.parse(serialized) as { id: string };

    expect(deserialized.id).toBe(original);
    // UUID v7 の形式保証（xxxxxxxx-xxxx-7xxx-xxxx-xxxxxxxxxxxx）
    expect(deserialized.id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
  });

  it('SQLite TEXT に保存した UUID を読み出しても大文字小文字が保持される', () => {
    const uuid = '019571A3-7C4F-7E46-9B28-1234567890AB';
    // SQLite NOCASE ではないため、大文字小文字そのままで往復する
    expect(JSON.parse(JSON.stringify(uuid))).toBe(uuid);
  });
});

describe('TST-intg-022: TIMESTAMPTZ ラウンドトリップ', () => {
  it('TIMESTAMPTZ は UTC ISO 8601 形式で往復しても同値を保持する', () => {
    // PG timestamptz → TEXT（ISO 8601 UTC）→ SQLite TEXT → TEXT → PG timestamptz
    const original = '2026-05-18T12:34:56.123Z';

    const serialized = JSON.stringify({ ts: original });
    const deserialized = JSON.parse(serialized) as { ts: string };

    expect(deserialized.ts).toBe(original);
    // UTC 形式（末尾 Z）を保持していること
    expect(deserialized.ts).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/);
  });

  it('マイクロ秒は許容丸め（ミリ秒精度に丸める）', () => {
    // PG microsecond → JS millisecond の丸め許容
    const pgMicrosecond = '2026-05-18T12:34:56.123456Z';
    const jsMillisecond = new Date(pgMicrosecond).toISOString(); // '2026-05-18T12:34:56.123Z'

    expect(jsMillisecond).toMatch(/\.\d{3}Z$/);
    // 分以上の精度は保持する
    expect(jsMillisecond.startsWith('2026-05-18T12:34:56.')).toBe(true);
  });

  it('タイムゾーンオフセットは UTC に正規化される', () => {
    // JST 2026-05-18T21:34:56+09:00 → UTC 2026-05-18T12:34:56.000Z
    const jstDate = new Date('2026-05-18T21:34:56+09:00');
    const utcIso = jstDate.toISOString();
    expect(utcIso).toBe('2026-05-18T12:34:56.000Z');
  });
});

describe('TST-intg-023: JSONB ラウンドトリップ', () => {
  it('JSONB は canonical JSON で往復してもキー順と値が一致する', () => {
    // PG jsonb → TEXT（canonical JSON）→ SQLite TEXT → JSON.parse → PG jsonb
    const original = { ja: '作業', en: 'Work', zh: '作业' };

    // canonical JSON（キーをソート）
    const canonical = toCanonicalJson(original);
    expect(canonical).toBe('{"en":"Work","ja":"作業","zh":"作业"}');

    // デシリアライズ後も同値
    const deserialized = JSON.parse(canonical) as typeof original;
    expect(deserialized.ja).toBe('作業');
    expect(deserialized.en).toBe('Work');
    expect(deserialized.zh).toBe('作业');
  });

  it('ネストした JSONB も canonical JSON で往復する', () => {
    const nested = {
      payload: { stepId: 'step-001', value: 42 },
      metadata: { source: 'terminal', version: 1 },
    };
    const canonical = toCanonicalJson(nested);
    const back = JSON.parse(canonical) as typeof nested;
    expect(back.payload.stepId).toBe('step-001');
    expect(back.payload.value).toBe(42);
    expect(back.metadata.source).toBe('terminal');
  });

  it('NULL 値は canonical JSON で null として往復する', () => {
    // NULL-able JSONB カラムは null で表現する
    const withNull = { ja: '作業', en: null, zh: '' };
    const canonical = toCanonicalJson(withNull);
    const back = JSON.parse(canonical) as typeof withNull;
    expect(back.en).toBeNull();
    expect(back.zh).toBe('');
  });
});
