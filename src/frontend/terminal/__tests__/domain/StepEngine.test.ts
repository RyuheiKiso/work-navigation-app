// StepEngine の純粋関数（resolveBranch・validate・fallback）と
// canAdvanceToStep の全ゲート（BR-BUS-001/002, FR-AU-001, FR-EV-013）を検証する
import { StepEngine } from '../../domain/step-engine/StepEngine';
import type { BranchingStepPayload } from '@wnav/shared/domain/step-engine';
import type { LocalWorkEvent } from '../../db/entities/LocalWorkEvent';
import type { LocalStep } from '../../db/entities/LocalStep';

// =================== Fake Repositories ===================

class FakeWorkEventRepo {
  events: LocalWorkEvent[] = [];
  async findLatestByCaseId(caseId: string): Promise<LocalWorkEvent | null> {
    const found = this.events.filter((e) => e.caseId === caseId);
    return found[found.length - 1] ?? null;
  }
  async append(event: LocalWorkEvent): Promise<LocalWorkEvent> {
    this.events.push(event);
    return event;
  }
  async findByCaseId(caseId: string): Promise<LocalWorkEvent[]> {
    return this.events.filter((e) => e.caseId === caseId);
  }
  async findUnsynced(): Promise<LocalWorkEvent[]> {
    return [];
  }
  async markSynced(): Promise<void> {
    return;
  }
}

class FakeOutboxRepo {
  events: unknown[] = [];
  async enqueue(event: unknown): Promise<unknown> {
    this.events.push(event);
    return event;
  }
  async findOldestPending(): Promise<null> {
    return null;
  }
  async delete(): Promise<void> {
    return;
  }
  async markRetry(): Promise<void> {
    return;
  }
  async pendingCount(): Promise<number> {
    return 0;
  }
}

class FakeSopRepository {
  private steps: LocalStep[];
  constructor(steps: LocalStep[]) {
    this.steps = steps;
  }
  async findStepsBySopVersionId(_sopVersionId: string): Promise<LocalStep[]> {
    return this.steps;
  }
  async findById(): Promise<null> {
    return null;
  }
  async findActiveSops(): Promise<never[]> {
    return [];
  }
  async upsertSop(): Promise<void> {
    return;
  }
  async upsertStep(): Promise<void> {
    return;
  }
}

// =================== Helpers ===================

const makeStep = (overrides: Partial<LocalStep> & { id: string; stepNumber: number }): LocalStep => ({
  sopVersionId: 'sop-v1',
  stepType: 'standard',
  titleJson: '{"ja":"Step","en":"","zh":""}',
  instructionJson: '{"ja":"","en":"","zh":""}',
  payload: '{}',
  isMandatory: true,
  requiresEvidence: false,
  requiresSign: false,
  skillLevelRequired: 1,
  estimatedSeconds: 60,
  fallbackType: 'manual',
  flowRules: '{"onComplete":"next","onSkip":"next"}',
  deletedAt: null,
  ...overrides,
});

const makeEvent = (overrides: Partial<LocalWorkEvent> & { caseId: string; stepId: string; activity: string }): LocalWorkEvent => ({
  eventId: `evt-${Math.random()}`,
  timestampClient: new Date().toISOString(),
  resource: 'worker1:terminal1',
  sopVersionId: 'sop-v1',
  payload: '{"inputType":"boolean_check","stepId":"s","stepNumber":1,"value":true}',
  prevHash: 'genesis',
  contentHash: 'hash1',
  terminalId: 'terminal1',
  synced: false,
  ...overrides,
});

function makeEngine(steps: LocalStep[], events: LocalWorkEvent[] = []) {
  const workEventRepo = new FakeWorkEventRepo();
  workEventRepo.events = events;
  const outboxRepo = new FakeOutboxRepo();
  const sopRepo = new FakeSopRepository(steps);
  return new StepEngine(workEventRepo as never, outboxRepo as never, sopRepo as never);
}

// =================== canAdvanceToStep Tests ===================

describe('StepEngine.canAdvanceToStep — BR-BUS-001 ロックステップ', () => {
  it('全前 Step が step_completed であれば canAdvance: true を返す', async () => {
    const steps = [
      makeStep({ id: 's1', stepNumber: 1 }),
      makeStep({ id: 's2', stepNumber: 2 }),
    ];
    const events = [
      makeEvent({ caseId: 'case1', stepId: 's1', activity: 'step_completed' }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 1, 'sop-v1');
    expect(result.canAdvance).toBe(true);
  });

  it('前 Step が未完了なら PREVIOUS_STEP_NOT_COMPLETED を返す', async () => {
    const steps = [
      makeStep({ id: 's1', stepNumber: 1 }),
      makeStep({ id: 's2', stepNumber: 2 }),
    ];
    const engine = makeEngine(steps, []);
    const result = await engine.canAdvanceToStep('case1', 1, 'sop-v1');
    expect(result.canAdvance).toBe(false);
    expect(result.blockedReason).toBe('PREVIOUS_STEP_NOT_COMPLETED');
  });

  it('最初の Step（index 0）は前 Step チェックをスキップして証拠・サインチェックへ進む', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1 })];
    const engine = makeEngine(steps, []);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    // requiresEvidence/requiresSign が false なので canAdvance: true
    expect(result.canAdvance).toBe(true);
  });
});

describe('StepEngine.canAdvanceToStep — BR-BUS-002 証拠必須ゲート', () => {
  it('requiresEvidence=true で photo_capture イベントがあれば通過する', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1, requiresEvidence: true })];
    const events = [
      makeEvent({
        caseId: 'case1',
        stepId: 's1',
        activity: 'evidence_captured',
        payload: '{"inputType":"photo_capture","stepId":"s1","stepNumber":1,"evidenceId":"ev1","fileHash":"h1"}',
      }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(true);
  });

  it('requiresEvidence=true でイベントがなければ EVIDENCE_REQUIRED を返す', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1, requiresEvidence: true })];
    const engine = makeEngine(steps, []);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(false);
    expect(result.blockedReason).toBe('EVIDENCE_REQUIRED');
  });

  it('requiresEvidence=true で qr_scan イベントでも通過する', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1, requiresEvidence: true })];
    const events = [
      makeEvent({
        caseId: 'case1',
        stepId: 's1',
        activity: 'qr_scanned',
        payload: '{"inputType":"qr_scan","stepId":"s1","stepNumber":1,"qrValue":"QR123","scanVerifications":[]}',
      }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(true);
  });

  it('requiresEvidence=true で boolean_check イベントのみでは EVIDENCE_REQUIRED を返す', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1, requiresEvidence: true })];
    const events = [
      makeEvent({ caseId: 'case1', stepId: 's1', activity: 'step_done',
        payload: '{"inputType":"boolean_check","stepId":"s1","stepNumber":1,"value":true}' }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(false);
    expect(result.blockedReason).toBe('EVIDENCE_REQUIRED');
  });
});

describe('StepEngine.canAdvanceToStep — FR-AU-001 電子サイン必須ゲート', () => {
  it('requiresSign=true で signature_pad イベントがあれば通過する', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1, requiresSign: true })];
    const events = [
      makeEvent({
        caseId: 'case1',
        stepId: 's1',
        activity: 'signed',
        payload: '{"inputType":"signature_pad","stepId":"s1","stepNumber":1,"evidenceId":"ev1","signedAt":"2026-01-01T00:00:00.000Z","pinHash":"hash"}',
      }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(true);
  });

  it('requiresSign=true でイベントがなければ SIGN_REQUIRED を返す', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1, requiresSign: true })];
    const engine = makeEngine(steps, []);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(false);
    expect(result.blockedReason).toBe('SIGN_REQUIRED');
  });

  it('requiresSign=true で photo_capture イベントのみでは SIGN_REQUIRED を返す', async () => {
    const steps = [makeStep({ id: 's1', stepNumber: 1, requiresSign: true })];
    const events = [
      makeEvent({
        caseId: 'case1',
        stepId: 's1',
        activity: 'evidence_captured',
        payload: '{"inputType":"photo_capture","stepId":"s1","stepNumber":1,"evidenceId":"ev1","fileHash":"h1"}',
      }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(false);
    expect(result.blockedReason).toBe('SIGN_REQUIRED');
  });
});

describe('StepEngine.canAdvanceToStep — FR-EV-013 ポカヨケ照合ゲート', () => {
  const makeStepWithRequiredScan = (verified: boolean) =>
    makeStep({
      id: 's1',
      stepNumber: 1,
      payload: JSON.stringify({
        requiredScans: [
          { target: 'tool', refId: 'tool-001', required: true },
        ],
      }),
    });

  it('required スキャンが全て verified: true なら通過する', async () => {
    const steps = [makeStepWithRequiredScan(true)];
    const events = [
      makeEvent({
        caseId: 'case1',
        stepId: 's1',
        activity: 'step_completed',
        payload: JSON.stringify({
          inputType: 'qr_scan',
          stepId: 's1',
          stepNumber: 1,
          qrValue: 'tool-001',
          scanVerifications: [
            { target: 'tool', expectedRefId: 'tool-001', actualScanValue: 'tool-001', verified: true, verifiedAt: '2026-01-01T00:00:00.000Z', isManualInput: false },
          ],
        }),
      }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(true);
  });

  it('required スキャンが verified: false なら WRONG_TOOL_SCAN を返す', async () => {
    const steps = [makeStepWithRequiredScan(false)];
    const events = [
      makeEvent({
        caseId: 'case1',
        stepId: 's1',
        activity: 'step_completed',
        payload: JSON.stringify({
          inputType: 'qr_scan',
          stepId: 's1',
          stepNumber: 1,
          qrValue: 'wrong-tool',
          scanVerifications: [
            { target: 'tool', expectedRefId: 'tool-001', actualScanValue: 'wrong-tool', verified: false, verifiedAt: '2026-01-01T00:00:00.000Z', isManualInput: false },
          ],
        }),
      }),
    ];
    const engine = makeEngine(steps, events);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(false);
    expect(result.blockedReason).toBe('WRONG_TOOL_SCAN');
  });

  it('step_completed イベントが存在しなければ WRONG_TOOL_SCAN を返す', async () => {
    const steps = [makeStepWithRequiredScan(true)];
    const engine = makeEngine(steps, []);
    const result = await engine.canAdvanceToStep('case1', 0, 'sop-v1');
    expect(result.canAdvance).toBe(false);
    expect(result.blockedReason).toBe('WRONG_TOOL_SCAN');
  });
});

// =================== resolveBranch Tests ===================

describe('StepEngine.resolveBranch', () => {
  it('returns passStepId when rule evaluates true', () => {
    const engine = makeEngine([]);
    const payload: BranchingStepPayload = {
      inputType: 'condition_branch',
      stepId: 'step-1',
      stepNumber: 1,
      branchResult: true,
      judgmentCondition: {
        rule: { '>': [{ var: 'measuredValue' }, 10] },
        passStepId: 'pass-id',
        failStepId: 'fail-id',
      },
    };
    const result = engine.resolveBranch(payload, { measuredValue: 15 });
    expect(result.passed).toBe(true);
    expect(result.nextStepId).toBe('pass-id');
  });

  it('returns failStepId when rule evaluates false', () => {
    const engine = makeEngine([]);
    const payload: BranchingStepPayload = {
      inputType: 'condition_branch',
      stepId: 'step-1',
      stepNumber: 1,
      branchResult: false,
      judgmentCondition: {
        rule: { '>': [{ var: 'measuredValue' }, 10] },
        passStepId: 'pass-id',
        failStepId: 'fail-id',
      },
    };
    const result = engine.resolveBranch(payload, { measuredValue: 5 });
    expect(result.passed).toBe(false);
    expect(result.nextStepId).toBe('fail-id');
  });
});

// =================== fallback Tests ===================

describe('StepEngine.fallback', () => {
  it('halt fallback should block progression', () => {
    const engine = makeEngine([]);
    const result = engine.fallback('halt');
    expect(result.blocked).toBe(true);
  });

  it('skip fallback should not block', () => {
    const engine = makeEngine([]);
    const result = engine.fallback('skip');
    expect(result.blocked).toBe(false);
  });
});
