import { describe, expect, it } from 'vitest';
import {
  applyFallback,
  evaluateCondition,
  resolveNextStep,
  validateStep,
} from '../domain/step-engine';
import type { BranchingStepPayload, MeasurementStepPayload } from '../domain/step-engine';

describe('step-engine', () => {
  describe('evaluateCondition', () => {
    it('比較演算で true/false を返す', () => {
      expect(evaluateCondition({ '>': [{ var: 'measuredValue' }, 10] }, { measuredValue: 15 })).toBe(true);
      expect(evaluateCondition({ '>': [{ var: 'measuredValue' }, 10] }, { measuredValue: 5 })).toBe(false);
    });

    it('and/or 演算を評価する', () => {
      expect(
        evaluateCondition(
          { and: [{ '>': [{ var: 'measuredValue' }, 0] }, { '<': [{ var: 'measuredValue' }, 100] }] },
          { measuredValue: 50 },
        ),
      ).toBe(true);
      expect(
        evaluateCondition(
          { or: [{ '>': [{ var: 'measuredValue' }, 100] }, { '<': [{ var: 'measuredValue' }, 0] }] },
          { measuredValue: 50 },
        ),
      ).toBe(false);
    });
  });

  describe('resolveNextStep', () => {
    const branching: BranchingStepPayload = {
      inputType: 'condition_branch',
      stepId: 'step-3',
      stepNumber: 3,
      branchResult: false,
      judgmentCondition: {
        rule: { '>': [{ var: 'measuredValue' }, 10] },
        passStepId: 'step-4',
        failStepId: 'step-99',
      },
    };

    it('pass 時は passStepId を返す', () => {
      const result = resolveNextStep(branching, { measuredValue: 15 });
      expect(result.passed).toBe(true);
      expect(result.nextStepId).toBe('step-4');
    });

    it('fail 時は failStepId を返す', () => {
      const result = resolveNextStep(branching, { measuredValue: 5 });
      expect(result.passed).toBe(false);
      expect(result.nextStepId).toBe('step-99');
    });
  });

  describe('validateStep', () => {
    it('正常な boolean_check は valid: true', () => {
      const result = validateStep({
        inputType: 'boolean_check',
        stepId: 'step-1',
        stepNumber: 1,
        value: true,
      });
      expect(result.valid).toBe(true);
    });

    it('numeric_input の USL 超過は ERR-VAL-002', () => {
      const measurement: MeasurementStepPayload = {
        inputType: 'numeric_input',
        stepId: 'step-1',
        stepNumber: 1,
        value: 30,
        unit: 'Nm',
        lsl: 10,
        usl: 25,
      };
      const result = validateStep(measurement);
      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('ERR-VAL-002');
    });

    it('numeric_input の LSL 未満は ERR-VAL-002', () => {
      const measurement: MeasurementStepPayload = {
        inputType: 'numeric_input',
        stepId: 'step-1',
        stepNumber: 1,
        value: 5,
        unit: 'Nm',
        lsl: 10,
        usl: 25,
      };
      const result = validateStep(measurement);
      expect(result.valid).toBe(false);
      expect(result.errorCode).toBe('ERR-VAL-002');
    });

    it('範囲内の numeric_input は valid', () => {
      const measurement: MeasurementStepPayload = {
        inputType: 'numeric_input',
        stepId: 'step-1',
        stepNumber: 1,
        value: 20,
        unit: 'Nm',
        lsl: 10,
        usl: 25,
      };
      expect(validateStep(measurement).valid).toBe(true);
    });

    it('photo_capture の fileHash 形式不正は invalid', () => {
      const result = validateStep({
        inputType: 'photo_capture',
        stepId: 'step-1',
        stepNumber: 1,
        evidenceId: 'evidence-1',
        fileHash: 'invalid',
      });
      expect(result.valid).toBe(false);
    });
  });

  describe('applyFallback', () => {
    it('skip は blocked: false', () => {
      expect(applyFallback('skip').blocked).toBe(false);
    });
    it('manual は blocked: false', () => {
      expect(applyFallback('manual').blocked).toBe(false);
    });
    it('halt は blocked: true', () => {
      expect(applyFallback('halt').blocked).toBe(true);
    });
  });
});
