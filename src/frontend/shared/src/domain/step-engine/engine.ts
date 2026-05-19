import jsonLogic from 'json-logic-js';
import { z } from 'zod';
import type {
  BranchingStepPayload,
  ConditionResult,
  FallbackResult,
  FallbackType,
  JsonLogicRule,
  MeasurementStepPayload,
  StepContext,
  StepPayload,
  ValidationResult,
} from './types';

// eval/new Function に依存しない json-logic-js の apply のみを評価エンジンとして使用する（src/CLAUDE.md §動的評価禁止）
export function evaluateCondition(rule: JsonLogicRule, data: StepContext): boolean {
  const result = jsonLogic.apply(rule, data);
  return Boolean(result);
}

// 分岐ステップで pass/fail に応じた次の stepId を確定する
export function resolveNextStep(
  payload: BranchingStepPayload,
  data: StepContext,
): ConditionResult {
  const rule = payload.judgmentCondition.rule;
  const passed = evaluateCondition(rule, data);
  const nextStepId = passed
    ? payload.judgmentCondition.passStepId
    : payload.judgmentCondition.failStepId;
  return {
    passed,
    nextStepId,
    evaluatedRule: rule,
  };
}

const booleanCheckSchema = z.object({
  inputType: z.literal('boolean_check'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  value: z.boolean(),
});

const textInputSchema = z.object({
  inputType: z.literal('text_input'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  value: z.string(),
});

const numericInputSchema = z.object({
  inputType: z.literal('numeric_input'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  value: z.number().finite(),
  unit: z.string().min(1).max(20),
  usl: z.number().optional(),
  lsl: z.number().optional(),
});

const photoSchema = z.object({
  inputType: z.literal('photo_capture'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  evidenceId: z.string().min(1),
  fileHash: z.string().regex(/^[0-9a-f]{64}$/),
});

const signatureSchema = z.object({
  inputType: z.literal('signature_pad'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  evidenceId: z.string().min(1),
  signedAt: z.string().min(1),
  pinHash: z.string().min(1),
});

const sliderSchema = z.object({
  inputType: z.literal('slider_range'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  value: z.number().finite(),
});

const qrScanSchema = z.object({
  inputType: z.literal('qr_scan'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  qrValue: z.string().min(1),
  scanVerifications: z.array(
    z.object({
      target: z.enum(['material', 'tool', 'instrument']),
      expectedRefId: z.string().optional(),
      actualScanValue: z.string(),
      verified: z.boolean(),
      verifiedAt: z.string(),
      isManualInput: z.boolean(),
    }),
  ),
});

const branchingSchema = z.object({
  inputType: z.literal('condition_branch'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  branchResult: z.boolean(),
  judgmentCondition: z.object({
    rule: z.record(z.unknown()),
    failStepId: z.string().nullable(),
    passStepId: z.string().nullable(),
  }),
});

const customSchema = z.object({
  inputType: z.literal('custom'),
  stepId: z.string().min(1),
  stepNumber: z.number().int().min(1),
  value: z.unknown(),
  rendererKey: z.string().min(1),
});

// バリデーションでは USL/LSL チェックを numeric_input に限定して個別実施する
export function validateStep(payload: StepPayload): ValidationResult {
  let parseResult: z.SafeParseReturnType<unknown, unknown>;
  switch (payload.inputType) {
    case 'boolean_check':
      parseResult = booleanCheckSchema.safeParse(payload);
      break;
    case 'text_input':
      parseResult = textInputSchema.safeParse(payload);
      break;
    case 'numeric_input':
      parseResult = numericInputSchema.safeParse(payload);
      break;
    case 'photo_capture':
      parseResult = photoSchema.safeParse(payload);
      break;
    case 'signature_pad':
      parseResult = signatureSchema.safeParse(payload);
      break;
    case 'slider_range':
      parseResult = sliderSchema.safeParse(payload);
      break;
    case 'qr_scan':
      parseResult = qrScanSchema.safeParse(payload);
      break;
    case 'condition_branch':
      parseResult = branchingSchema.safeParse(payload);
      break;
    case 'custom':
      parseResult = customSchema.safeParse(payload);
      break;
  }

  if (!parseResult.success) {
    return {
      valid: false,
      errorCode: 'ERR-VAL-001',
      errorMessage: {
        ja: parseResult.error.issues[0]?.message ?? 'バリデーションエラー',
        en: parseResult.error.issues[0]?.message ?? 'Validation error',
        zh: parseResult.error.issues[0]?.message ?? '验证错误',
      },
    };
  }

  if (payload.inputType === 'numeric_input') {
    const measurement = payload as MeasurementStepPayload;
    if (measurement.lsl !== undefined && measurement.value < measurement.lsl) {
      return {
        valid: false,
        errorCode: 'ERR-VAL-002',
        errorMessage: {
          ja: `測定値が下限を下回りました（LSL: ${measurement.lsl}）`,
          en: `Value below lower spec limit (LSL: ${measurement.lsl})`,
          zh: `测量值低于下限（LSL: ${measurement.lsl}）`,
        },
      };
    }
    if (measurement.usl !== undefined && measurement.value > measurement.usl) {
      return {
        valid: false,
        errorCode: 'ERR-VAL-002',
        errorMessage: {
          ja: `測定値が上限を超えました（USL: ${measurement.usl}）`,
          en: `Value above upper spec limit (USL: ${measurement.usl})`,
          zh: `测量值超过上限（USL: ${measurement.usl}）`,
        },
      };
    }
  }

  return { valid: true };
}

// fallback_type ごとの挙動を一意に決める（halt のみ blocked: true）
export function applyFallback(fallback: FallbackType): FallbackResult {
  switch (fallback) {
    case 'skip':
      return {
        fallback,
        blocked: false,
        message: {
          ja: 'スキップして次のステップへ進みます',
          en: 'Skipping to the next step',
          zh: '跳过并进入下一步',
        },
      };
    case 'manual':
      return {
        fallback,
        blocked: false,
        message: {
          ja: '手動操作で対応してください',
          en: 'Please proceed manually',
          zh: '请手动处理',
        },
      };
    case 'halt':
      return {
        fallback,
        blocked: true,
        message: {
          ja: '進行不可: 監督者の対応が必要です',
          en: 'Halted: supervisor intervention required',
          zh: '已停止：需要主管介入',
        },
      };
  }
}
