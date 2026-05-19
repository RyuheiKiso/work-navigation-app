import type { LocalizedText } from '../../types';

// StepEngine の入力タイプは標準4 + 拡張5を網羅する（docs/05/04 §1-1）
export type StepInputType =
  | 'boolean_check'
  | 'numeric_input'
  | 'photo_capture'
  | 'text_input'
  | 'slider_range'
  | 'qr_scan'
  | 'signature_pad'
  | 'condition_branch'
  | 'custom';

export type FallbackType = 'skip' | 'manual' | 'halt';

// JSON Logic ルールは json-logic-js で評価可能なオブジェクト構造のみを許容する
export type JsonLogicRule = Record<string, unknown>;

// 判定結果に応じた遷移先 stepId を保持する（null は次ステップへ進行を意味する）
export interface JudgmentCondition {
  rule: JsonLogicRule;
  failStepId: string | null;
  passStepId: string | null;
}

// FR-EV-013 ポカヨケ照合エントリ
export interface RequiredScan {
  target: 'material' | 'tool' | 'instrument';
  refId?: string;
  refScanCode?: string;
  required: boolean;
  label?: LocalizedText;
  gs1Ai?: string;
}

// ポカヨケ照合の検証結果（target 別に成否を保持する）
export interface ScanVerification {
  target: 'material' | 'tool' | 'instrument';
  expectedRefId?: string;
  actualScanValue: string;
  verified: boolean;
  verifiedAt: string;
  isManualInput: boolean;
}

export interface BaseStepPayload {
  inputType: StepInputType;
  stepId: string;
  stepNumber: number;
}

export interface StandardStepPayload extends BaseStepPayload {
  inputType: 'boolean_check' | 'text_input';
  value: boolean | string;
}

export interface BranchingStepPayload extends BaseStepPayload {
  inputType: 'condition_branch';
  branchResult: boolean;
  judgmentCondition: JudgmentCondition;
}

export interface CustomStepPayload extends BaseStepPayload {
  inputType: 'custom';
  value: unknown;
  rendererKey: string;
}

export interface PhotoStepPayload extends BaseStepPayload {
  inputType: 'photo_capture';
  evidenceId: string;
  fileHash: string;
}

export interface MeasurementStepPayload extends BaseStepPayload {
  inputType: 'numeric_input';
  value: number;
  unit: string;
  usl?: number;
  lsl?: number;
}

export interface SignatureStepPayload extends BaseStepPayload {
  inputType: 'signature_pad';
  evidenceId: string;
  signedAt: string;
  pinHash: string;
}

export interface SliderStepPayload extends BaseStepPayload {
  inputType: 'slider_range';
  value: number;
}

export interface QrScanStepPayload extends BaseStepPayload {
  inputType: 'qr_scan';
  qrValue: string;
  scanVerifications: ScanVerification[];
}

export type StepPayload =
  | StandardStepPayload
  | BranchingStepPayload
  | CustomStepPayload
  | PhotoStepPayload
  | MeasurementStepPayload
  | SignatureStepPayload
  | SliderStepPayload
  | QrScanStepPayload;

// JSON Logic 評価時に渡す変数コンテキスト
export interface StepContext {
  measuredValue?: number;
  booleanValue?: boolean;
  textValue?: string;
  qrValue?: string;
  evidenceId?: string;
  usl?: number;
  lsl?: number;
  [key: string]: unknown;
}

// StepEngine の評価結果（次に遷移すべき stepId と pass/fail）
export interface ConditionResult {
  passed: boolean;
  nextStepId: string | null;
  evaluatedRule: JsonLogicRule;
}

export interface ValidationResult {
  valid: boolean;
  errorCode?: string;
  errorMessage?: LocalizedText;
}

export interface FallbackResult {
  fallback: FallbackType;
  blocked: boolean;
  message: LocalizedText;
}

export type BlockedReason =
  | 'PREVIOUS_STEP_NOT_COMPLETED'
  | 'EVIDENCE_REQUIRED'
  | 'SIGN_REQUIRED'
  | 'SKILL_LEVEL_INSUFFICIENT'
  | 'CONDITION_BRANCH_UNRESOLVED'
  | 'WRONG_TOOL_SCAN'
  | 'OUT_OF_SPEC';

export interface CanAdvanceResult {
  canAdvance: boolean;
  blockedReason?: BlockedReason;
}
