import { describe, expect, it } from 'vitest';
import {
  aqlValueSchema,
  pinSchema,
  passwordSchema,
  measurementValueSchema,
  receivedQuantitySchema,
  sopCodeSchema,
  validateJsonLogicDsl,
  uuidV7Schema,
} from '../validators';

describe('validators', () => {
  it('uuidV7Schema は UUID v7 を許容する', () => {
    expect(uuidV7Schema.safeParse('019682ab-7c1f-7000-a1b2-3c4d5e6f7890').success).toBe(true);
    expect(uuidV7Schema.safeParse('invalid').success).toBe(false);
  });

  it('pinSchema は 4〜8 桁の数字のみ許容する', () => {
    expect(pinSchema.safeParse('1234').success).toBe(true);
    expect(pinSchema.safeParse('12345678').success).toBe(true);
    expect(pinSchema.safeParse('123').success).toBe(false);
    expect(pinSchema.safeParse('123456789').success).toBe(false);
    expect(pinSchema.safeParse('abcd').success).toBe(false);
  });

  it('passwordSchema は 8〜128 文字を許容する', () => {
    expect(passwordSchema.safeParse('Pass1234').success).toBe(true);
    expect(passwordSchema.safeParse('short').success).toBe(false);
  });

  it('aqlValueSchema は ANSI/ASQ Z1.4 の標準値のみ許容する', () => {
    expect(aqlValueSchema.safeParse(1.0).success).toBe(true);
    expect(aqlValueSchema.safeParse(2.5).success).toBe(true);
    expect(aqlValueSchema.safeParse(3.0).success).toBe(false);
  });

  it('receivedQuantitySchema は正の整数のみ許容する', () => {
    expect(receivedQuantitySchema.safeParse(100).success).toBe(true);
    expect(receivedQuantitySchema.safeParse(0).success).toBe(false);
    expect(receivedQuantitySchema.safeParse(-1).success).toBe(false);
    expect(receivedQuantitySchema.safeParse(1.5).success).toBe(false);
  });

  it('measurementValueSchema は範囲チェックを動的生成する', () => {
    const schema = measurementValueSchema(10, 25);
    expect(schema.safeParse(15).success).toBe(true);
    expect(schema.safeParse(10).success).toBe(true);
    expect(schema.safeParse(25).success).toBe(true);
    expect(schema.safeParse(9).success).toBe(false);
    expect(schema.safeParse(26).success).toBe(false);
  });

  it('sopCodeSchema は英数字とハイフン・アンダースコアのみ許容する', () => {
    expect(sopCodeSchema.safeParse('SOP-001').success).toBe(true);
    expect(sopCodeSchema.safeParse('SOP_002').success).toBe(true);
    expect(sopCodeSchema.safeParse('SOP 003').success).toBe(false);
  });

  describe('validateJsonLogicDsl', () => {
    it('許容演算子のみで構成された DSL を許容する', () => {
      expect(validateJsonLogicDsl({ '>': [{ var: 'a' }, 10] })).toBe(true);
      expect(validateJsonLogicDsl({ and: [{ '>': [{ var: 'a' }, 0] }, { '<': [{ var: 'a' }, 100] }] })).toBe(true);
    });
    it('未許容演算子を拒否する', () => {
      expect(validateJsonLogicDsl({ evil_op: [1, 2] })).toBe(false);
    });
    it('深いネスト（>5）を拒否する', () => {
      const deeply = { and: [{ and: [{ and: [{ and: [{ and: [{ and: [{ '>': [1, 0] }] }] }] }] }] }] };
      expect(validateJsonLogicDsl(deeply)).toBe(false);
    });
  });
});
