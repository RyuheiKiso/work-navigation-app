import { describe, expect, it } from 'vitest';
import {
  judgeAql,
  resolveSampleSizeCode,
  resolveSamplingPlan,
  SAMPLE_SIZE_BY_CODE,
} from '../domain/aql';

describe('aql', () => {
  describe('resolveSampleSizeCode (Level II)', () => {
    it('ロット 2〜8 は記号 A', () => {
      expect(resolveSampleSizeCode(2, 'II')).toBe('A');
      expect(resolveSampleSizeCode(8, 'II')).toBe('A');
    });
    it('ロット 91〜150 は記号 F', () => {
      expect(resolveSampleSizeCode(91, 'II')).toBe('F');
      expect(resolveSampleSizeCode(150, 'II')).toBe('F');
    });
    it('ロット 501〜1200 は記号 J（n=80）', () => {
      const code = resolveSampleSizeCode(1000, 'II');
      expect(code).toBe('J');
      expect(SAMPLE_SIZE_BY_CODE[code]).toBe(80);
    });
    it('ロット 1201〜3200 は記号 K（n=125）', () => {
      const code = resolveSampleSizeCode(2500, 'II');
      expect(code).toBe('K');
      expect(SAMPLE_SIZE_BY_CODE[code]).toBe(125);
    });
  });

  describe('resolveSamplingPlan AQL=1.0', () => {
    it('ロット 1000 / AQL 1.0 / Level II → n=80, Ac=2, Re=3', () => {
      const plan = resolveSamplingPlan(1000, 1.0, 'II');
      expect(plan.sampleSizeCode).toBe('J');
      expect(plan.sampleSizeN).toBe(80);
      expect(plan.acceptNumberAc).toBe(2);
      expect(plan.rejectNumberRe).toBe(3);
    });
    it('ロット 2500 / AQL 1.0 / Level II → n=125, Ac=3, Re=4', () => {
      const plan = resolveSamplingPlan(2500, 1.0, 'II');
      expect(plan.sampleSizeCode).toBe('K');
      expect(plan.sampleSizeN).toBe(125);
      expect(plan.acceptNumberAc).toBe(3);
      expect(plan.rejectNumberRe).toBe(4);
    });
  });

  describe('resolveSamplingPlan AQL=2.5', () => {
    it('ロット 100 / AQL 2.5 / Level II → n=20, Ac=1, Re=2', () => {
      const plan = resolveSamplingPlan(100, 2.5, 'II');
      expect(plan.sampleSizeCode).toBe('F');
      expect(plan.sampleSizeN).toBe(20);
      expect(plan.acceptNumberAc).toBe(1);
      expect(plan.rejectNumberRe).toBe(2);
    });
  });

  describe('judgeAql', () => {
    it('不良数 0 / Ac=2 / Re=3 → PASSED', () => {
      expect(judgeAql(0, 2, 3)).toBe('PASSED');
    });
    it('不良数 2 (= Ac) / Ac=2 / Re=3 → PASSED（境界 Ac 以下）', () => {
      expect(judgeAql(2, 2, 3)).toBe('PASSED');
    });
    it('不良数 3 (= Re) / Ac=2 / Re=3 → REJECTED（境界 Re 以上）', () => {
      expect(judgeAql(3, 2, 3)).toBe('REJECTED');
    });
    it('不良数 5 / Ac=2 / Re=3 → REJECTED', () => {
      expect(judgeAql(5, 2, 3)).toBe('REJECTED');
    });
    it('Ac=0 / Re=1 で不良数 0 → PASSED', () => {
      expect(judgeAql(0, 0, 1)).toBe('PASSED');
    });
    it('Ac=0 / Re=1 で不良数 1 → REJECTED', () => {
      expect(judgeAql(1, 0, 1)).toBe('REJECTED');
    });
  });
});
