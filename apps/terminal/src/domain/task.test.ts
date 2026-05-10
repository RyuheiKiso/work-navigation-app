// 対応 §: ロードマップ §13.1 §13.2 §3.1.1
// 端末側「作業（Task）」Aggregate の単体テスト。

// vitest API
import { describe, it, expect } from 'vitest';
// テスト対象
import { Task, type Evidence } from './task';
import { TaskId, DeviceId } from './value-object';

describe('Task aggregate', () => {
  // 補助: 新鮮な Task を作る
  function fresh(): Task {
    // テスト用 ID
    const id = TaskId.of('task-1');
    // テスト用 Device
    const dev = DeviceId.of('device-1');
    // Manual 完了条件で構築
    return Task.create(id, 'Manual', dev);
  }

  // 初期状態は Idle
  it('starts in Idle state', () => {
    // 新鮮な Task
    const t = fresh();
    // 初期状態を確認
    expect(t.state).toBe('Idle');
  });

  // 前提充足で Ready
  it('transitions to Ready when precondition satisfied', () => {
    // 新鮮な Task
    const t = fresh();
    // 前提を満たす
    t.markPreconditionSatisfied();
    // Ready に遷移していること
    expect(t.state).toBe('Ready');
  });

  // 前提なしの start は拒否
  it('rejects start without precondition', () => {
    // 新鮮な Task
    const t = fresh();
    // 即時 start はエラー
    expect(() => t.start()).toThrow(/開始条件/);
  });

  // start → suspend → resume の経路
  it('supports start -> suspend -> resume cycle', () => {
    // 新鮮な Task
    const t = fresh();
    // 前提充足
    t.markPreconditionSatisfied();
    // 開始
    t.start();
    // 中断
    t.suspend();
    // Suspended に
    expect(t.state).toBe('Suspended');
    // 再開
    t.resume();
    // Running に戻る
    expect(t.state).toBe('Running');
  });

  // complete: 証跡なしは拒否
  it('rejects complete without evidence', () => {
    // 新鮮な Task
    const t = fresh();
    // 前提充足
    t.markPreconditionSatisfied();
    // 開始
    t.start();
    // 空の証跡
    const ev: Evidence = { manuallyMarked: false, photoAttached: false };
    // エラーを期待
    expect(() => t.complete(ev)).toThrow(/完了条件/);
  });

  // complete: 証跡があれば完了
  it('completes when evidence is sufficient', () => {
    // 新鮮な Task
    const t = fresh();
    // 前提充足
    t.markPreconditionSatisfied();
    // 開始
    t.start();
    // 証跡を満たす
    const ev: Evidence = { manuallyMarked: true, photoAttached: false };
    // 完了
    t.complete(ev);
    // Completed に遷移
    expect(t.state).toBe('Completed');
  });

  // Lamport は遷移ごとに単調増加（INV-08）
  it('increments lamport monotonically across transitions', () => {
    // 新鮮な Task
    const t = fresh();
    // 初期は 0
    expect(t.lamport.toBigInt()).toBe(0n);
    // 前提充足 → 1
    t.markPreconditionSatisfied();
    // 開始 → 2
    t.start();
    // 中断 → 3
    t.suspend();
    // 再開 → 4
    t.resume();
    // 4 に達している
    expect(t.lamport.toBigInt()).toBe(4n);
  });
});
