// 対応 §: ロードマップ §13.1 §10.1 §3.1.1 ／ ルート CLAUDE.md
// StartTaskUseCase の入出力性質を fast-check で検証する。
// - 未存在 ID で実行すると常に例外
// - 存在する Task に対して必ず start() が呼ばれ Running になる
// - save が exactly once 呼ばれる

import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import { StartTaskUseCase } from './start-task';
import type { TaskRepository } from '../domain/repository';
import { Task } from '../domain/task';
import { TaskId, DeviceId } from '../domain/value-object';

const idArb = fc.stringMatching(/^[a-z][a-z0-9-]{0,16}$/);

class InMemoryTaskRepository implements TaskRepository {
  saveCount = 0;
  constructor(private readonly tasks: Map<string, Task>) {}
  async findById(id: TaskId): Promise<Task | null> {
    return this.tasks.get(id.toString()) ?? null;
  }
  async save(task: Task): Promise<void> {
    this.saveCount += 1;
    this.tasks.set(task.id.toString(), task);
  }
}

function readyTask(id: string): Task {
  const t = Task.create(TaskId.of(id), 'Manual', DeviceId.of('dev'));
  t.markPreconditionSatisfied();
  return t;
}

describe('StartTaskUseCase (property-based)', () => {
  it('未存在 ID で実行すると例外を投げる', async () => {
    await fc.assert(
      fc.asyncProperty(idArb, async (id) => {
        const repo = new InMemoryTaskRepository(new Map());
        const uc = new StartTaskUseCase(repo);
        await expect(uc.execute({ taskId: TaskId.of(id) })).rejects.toThrow(/存在しません/);
        // 失敗時は save が呼ばれない
        expect(repo.saveCount).toBe(0);
      })
    );
  });

  it('存在する Ready Task は Running に遷移し save が一度だけ呼ばれる', async () => {
    await fc.assert(
      fc.asyncProperty(idArb, async (id) => {
        const repo = new InMemoryTaskRepository(new Map([[id, readyTask(id)]]));
        const uc = new StartTaskUseCase(repo);
        const result = await uc.execute({ taskId: TaskId.of(id) });
        expect(result.state).toBe('Running');
        expect(repo.saveCount).toBe(1);
      })
    );
  });

  it('Idle のままの Task に対しては start() が失敗する', async () => {
    await fc.assert(
      fc.asyncProperty(idArb, async (id) => {
        const idleTask = Task.create(TaskId.of(id), 'Manual', DeviceId.of('dev'));
        const repo = new InMemoryTaskRepository(new Map([[id, idleTask]]));
        const uc = new StartTaskUseCase(repo);
        await expect(uc.execute({ taskId: TaskId.of(id) })).rejects.toThrow(/開始条件|不正な状態遷移/);
        expect(repo.saveCount).toBe(0);
      })
    );
  });
});
