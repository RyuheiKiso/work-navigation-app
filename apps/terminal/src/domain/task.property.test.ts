// 対応 §: ロードマップ §13.1 §3.4.1 ／ ルート CLAUDE.md（不変条件は型または property test で守る）
// Task Aggregate の HSM 不変条件を fast-check で網羅する。
// - Idle から start を直接呼ぶと必ず例外
// - Lamport は遷移ごとに単調増加 (INV-08)
// - 完了条件 Photo に対し photoAttached=false の Evidence は必ず例外

import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import { Task, type CompletionCriteria, type Evidence } from './task';
import { TaskId, DeviceId } from './value-object';

const idArb = fc.stringMatching(/^[a-z][a-z0-9-]{0,16}$/);
const criteriaArb = fc.constantFrom<CompletionCriteria>('Manual', 'Photo');

function freshTask(id = 't1', criteria: CompletionCriteria = 'Manual'): Task {
  return Task.create(TaskId.of(id), criteria, DeviceId.of('dev-1'));
}

describe('Task HSM invariants (property-based)', () => {
  it('Idle から start を直接呼ぶと常に例外', () => {
    fc.assert(
      fc.property(idArb, criteriaArb, (id, c) => {
        const t = freshTask(id, c);
        expect(() => t.start()).toThrow(/開始条件|不正な状態遷移/);
      })
    );
  });

  it('Lamport は markPreconditionSatisfied → start ごとに単調増加', () => {
    fc.assert(
      fc.property(idArb, criteriaArb, (id, c) => {
        const t = freshTask(id, c);
        const l0 = t.lamport.toBigInt();
        t.markPreconditionSatisfied();
        const l1 = t.lamport.toBigInt();
        t.start();
        const l2 = t.lamport.toBigInt();
        expect(l1 > l0).toBe(true);
        expect(l2 > l1).toBe(true);
      })
    );
  });

  it('Photo 完了条件に対し photoAttached=false の Evidence は常に例外', () => {
    fc.assert(
      fc.property(fc.boolean(), (manuallyMarked) => {
        const t = freshTask('p1', 'Photo');
        t.markPreconditionSatisfied();
        t.start();
        const ev: Evidence = { manuallyMarked, photoAttached: false };
        expect(() => t.complete(ev)).toThrow(/完了条件/);
      })
    );
  });

  it('Manual 完了条件に対し manuallyMarked=true なら必ず Completed に遷移', () => {
    fc.assert(
      fc.property(fc.boolean(), (photoAttached) => {
        const t = freshTask('m1', 'Manual');
        t.markPreconditionSatisfied();
        t.start();
        const ev: Evidence = { manuallyMarked: true, photoAttached };
        t.complete(ev);
        expect(t.state).toBe('Completed');
      })
    );
  });

  it('Running 以外から suspend を呼ぶと常に例外', () => {
    fc.assert(
      fc.property(criteriaArb, (c) => {
        const t = freshTask('s1', c);
        // 初期 Idle 状態から
        expect(() => t.suspend()).toThrow(/不正な状態遷移/);
      })
    );
  });
});
