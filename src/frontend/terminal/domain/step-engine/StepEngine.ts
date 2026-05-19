// 端末側 StepEngine。shared の純粋関数を再利用しつつローカル WorkEvent + Outbox 連携を担当する
import {
  evaluateCondition,
  resolveNextStep,
  validateStep,
  applyFallback,
} from '@wnav/shared/domain/step-engine';
import type {
  BranchingStepPayload,
  CanAdvanceResult,
  ConditionResult,
  FallbackType,
  QrScanStepPayload,
  RequiredScan,
  StepContext,
  StepPayload,
  ValidationResult,
} from '@wnav/shared/domain/step-engine';
import { computeContentHash, GENESIS_HASH } from '@wnav/shared/domain/hash-chain';
import { generateId } from '@wnav/shared/domain/id';
import type { ActivityType } from '@wnav/shared';
import { WorkEventRepository } from '../../db/repositories/WorkEventRepository';
import { OutboxRepository } from '../../db/repositories/OutboxRepository';
import { SopRepository } from '../../db/repositories/SopRepository';

export interface CompleteStepParams {
  caseId: string;
  stepId: string;
  stepIndex: number;
  sopVersionId: string;
  workerId: string;
  terminalId: string;
  payload: StepPayload;
  inputData: Record<string, unknown>;
  activity?: ActivityType;
}

export class StepEngine {
  constructor(
    private readonly workEventRepo: WorkEventRepository,
    private readonly outboxRepo: OutboxRepository,
    private readonly sopRepository: SopRepository,
  ) {}

  // FNC-FE-001: BR-BUS-001（ロックステップ）・BR-BUS-002（証拠必須）・FR-AU-001（サイン必須）・FR-EV-013（ポカヨケ照合）の全ゲートを検証する
  async canAdvanceToStep(
    caseId: string,
    targetStepIndex: number,
    sopVersionId: string,
  ): Promise<CanAdvanceResult> {
    const [events, steps] = await Promise.all([
      this.workEventRepo.findByCaseId(caseId),
      this.sopRepository.findStepsBySopVersionId(sopVersionId),
    ]);

    // BR-BUS-001: targetStepIndex 手前のすべての Step が step_completed であることを確認する
    for (let i = 0; i < targetStepIndex; i++) {
      const step = steps[i];
      if (step == null) {
        return { canAdvance: false, blockedReason: 'PREVIOUS_STEP_NOT_COMPLETED' };
      }
      const completed = events.some(
        (e) => e.stepId === step.id && e.activity === 'step_completed',
      );
      if (!completed) {
        return { canAdvance: false, blockedReason: 'PREVIOUS_STEP_NOT_COMPLETED' };
      }
    }

    const targetStep = steps[targetStepIndex];
    if (targetStep == null) {
      return { canAdvance: false, blockedReason: 'PREVIOUS_STEP_NOT_COMPLETED' };
    }

    // ES2022 互換: findLast の代替として末尾から線形スキャンする
    const lastEventForStep = (stepId: string) => {
      for (let i = events.length - 1; i >= 0; i--) {
        const e = events[i];
        if (e != null && e.stepId === stepId) return e;
      }
      return null;
    };

    // BR-BUS-002: 証拠必須ゲート - photo_capture または qr_scan イベントが存在することを確認する
    if (targetStep.requiresEvidence) {
      const lastEvent = lastEventForStep(targetStep.id);
      if (lastEvent == null) {
        return { canAdvance: false, blockedReason: 'EVIDENCE_REQUIRED' };
      }
      const evtPayload = JSON.parse(lastEvent.payload) as StepPayload;
      if (evtPayload.inputType !== 'photo_capture' && evtPayload.inputType !== 'qr_scan') {
        return { canAdvance: false, blockedReason: 'EVIDENCE_REQUIRED' };
      }
    }

    // FR-AU-001: 電子サイン必須ゲート - signature_pad イベントが存在することを確認する
    if (targetStep.requiresSign) {
      const lastEvent = lastEventForStep(targetStep.id);
      if (lastEvent == null) {
        return { canAdvance: false, blockedReason: 'SIGN_REQUIRED' };
      }
      const evtPayload = JSON.parse(lastEvent.payload) as StepPayload;
      if (evtPayload.inputType !== 'signature_pad') {
        return { canAdvance: false, blockedReason: 'SIGN_REQUIRED' };
      }
    }

    // FR-EV-013: ポカヨケ照合ゲート - required_scans のすべての required エントリが verified: true であることを確認する
    type StepPayloadConfig = { requiredScans?: RequiredScan[] };
    const stepConfig = JSON.parse(targetStep.payload) as StepPayloadConfig;
    const requiredScans = stepConfig.requiredScans ?? [];
    if (requiredScans.some((s) => s.required)) {
      // step_completed イベントの中で最後の qr_scan を取得する
      let lastQrEvent = null;
      for (let i = events.length - 1; i >= 0; i--) {
        const e = events[i];
        if (e != null && e.stepId === targetStep.id && e.activity === 'step_completed') {
          lastQrEvent = e;
          break;
        }
      }
      if (lastQrEvent == null) {
        return { canAdvance: false, blockedReason: 'WRONG_TOOL_SCAN' };
      }
      const qrPayload = JSON.parse(lastQrEvent.payload) as StepPayload;
      if (qrPayload.inputType !== 'qr_scan') {
        return { canAdvance: false, blockedReason: 'WRONG_TOOL_SCAN' };
      }
      // qr_scan payload は QrScanStepPayload 型に絞り込まれているが型アサーションで明示する
      const typedQr = qrPayload as QrScanStepPayload;
      const verifications = typedQr.scanVerifications;
      for (const entry of requiredScans) {
        if (!entry.required) continue;
        const match = verifications.find(
          (v) =>
            v.target === entry.target &&
            (entry.refId == null || v.expectedRefId === entry.refId),
        );
        // BR-BUS-007: target: 'instrument' は EvidenceCaptureModule で校正期限確認済みのため
        // ここでは verified フラグのみを信頼して二重確認する
        if (match == null || !match.verified) {
          return { canAdvance: false, blockedReason: 'WRONG_TOOL_SCAN' };
        }
      }
    }

    return { canAdvance: true };
  }

  // Step 完了イベントを HashChain に連結し WorkEvent と Outbox の双方に追記する
  // FNC-FE-002: canAdvanceToStep で全ゲートを事前検証し、不合格なら ERR-BIZ-001 で即座に中断する
  async completeStep(params: CompleteStepParams): Promise<{ eventId: string; contentHash: string }> {
    const gate = await this.canAdvanceToStep(params.caseId, params.stepIndex, params.sopVersionId);
    if (!gate.canAdvance) {
      throw new Error(`ERR-BIZ-001: ステップ進行不可 [${gate.blockedReason ?? 'UNKNOWN'}]`);
    }
    const prevEvent = await this.workEventRepo.findLatestByCaseId(params.caseId);
    const prevHash = prevEvent?.contentHash ?? GENESIS_HASH;
    const eventPayload = { stepId: params.stepId, inputData: params.inputData };
    const contentHash = computeContentHash(prevHash, eventPayload);
    const eventId = generateId();
    const now = new Date().toISOString();
    const activity: ActivityType = params.activity ?? 'step_completed';

    await this.workEventRepo.append({
      eventId,
      caseId: params.caseId,
      activity,
      timestampClient: now,
      resource: `${params.workerId}:${params.terminalId}`,
      sopVersionId: params.sopVersionId,
      stepId: params.stepId,
      payload: JSON.stringify(eventPayload),
      prevHash,
      contentHash,
      terminalId: params.terminalId,
      synced: false,
    });

    // Outbox に積んでバックグラウンドの OutboxWorker が created_at 順に送信する
    await this.outboxRepo.enqueue({
      eventId,
      idempotencyKey: generateId(),
      payload: JSON.stringify({ type: 'work_event', eventId, activity, contentHash, prevHash }),
      prevHash,
      createdAt: now,
      sent: false,
      retryCount: 0,
      lastError: null,
      nextRetryAt: now,
    });

    return { eventId, contentHash };
  }

  // 条件分岐 Step の次 stepId を決定する。eval 禁止のため json-logic-js のみ使用
  resolveBranch(payload: BranchingStepPayload, data: StepContext): ConditionResult {
    return resolveNextStep(payload, data);
  }

  // 任意の JSON Logic ルールをローカル評価する
  evaluate(rule: Record<string, unknown>, data: StepContext): boolean {
    return evaluateCondition(rule, data);
  }

  // Step ペイロードのバリデーションを実行する
  validate(payload: StepPayload): ValidationResult {
    return validateStep(payload);
  }

  // fallback_type による縮退動作を解決する
  fallback(type: FallbackType) {
    return applyFallback(type);
  }
}
