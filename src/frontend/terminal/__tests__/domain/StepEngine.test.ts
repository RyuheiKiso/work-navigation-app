// StepEngine の純粋関数（resolveBranch・validate・fallback）を端末コンテキストで検証する
import { StepEngine } from '../../domain/step-engine/StepEngine';
import type { BranchingStepPayload } from '@wnav/shared/domain/step-engine';

class FakeWorkEventRepo {
  events: unknown[] = [];
  async findLatestByCaseId(): Promise<null> {
    return null;
  }
  async append(event: unknown): Promise<unknown> {
    this.events.push(event);
    return event;
  }
  async findByCaseId(): Promise<unknown[]> {
    return this.events;
  }
  async findUnsynced(): Promise<unknown[]> {
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

describe('StepEngine.resolveBranch', () => {
  it('returns passStepId when rule evaluates true', () => {
    const engine = new StepEngine(new FakeWorkEventRepo() as never, new FakeOutboxRepo() as never);
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
    const engine = new StepEngine(new FakeWorkEventRepo() as never, new FakeOutboxRepo() as never);
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

describe('StepEngine.fallback', () => {
  it('halt fallback should block progression', () => {
    const engine = new StepEngine(new FakeWorkEventRepo() as never, new FakeOutboxRepo() as never);
    const result = engine.fallback('halt');
    expect(result.blocked).toBe(true);
  });

  it('skip fallback should not block', () => {
    const engine = new StepEngine(new FakeWorkEventRepo() as never, new FakeOutboxRepo() as never);
    const result = engine.fallback('skip');
    expect(result.blocked).toBe(false);
  });
});
