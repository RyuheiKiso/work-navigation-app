// 対応 §: ロードマップ §10.1 §3.1.1
// 端末側 Start Task ユースケース。Tauri command を介してバックエンドへ伝播する経路を想定。

// ドメイン依存
import type { TaskRepository } from '../domain/repository';
import type { Task } from '../domain/task';
import type { TaskId } from '../domain/value-object';

/** Start Task コマンド */
export interface StartTaskCommand {
  // 開始対象 ID
  readonly taskId: TaskId;
}

/** Start Task ユースケース */
export class StartTaskUseCase {
  // Repository を保持する
  private readonly repository: TaskRepository;

  /** コンストラクタ */
  constructor(repository: TaskRepository) {
    // 依存を保持する
    this.repository = repository;
  }

  /** 実行 */
  async execute(cmd: StartTaskCommand): Promise<Task> {
    // 対象 Task を取得
    const task = await this.repository.findById(cmd.taskId);
    // 未存在
    if (task === null) {
      // ドメインエラー
      throw new Error('対象の作業が存在しません');
    }
    // 開始
    task.start();
    // 永続化
    await this.repository.save(task);
    // 更新後を返す
    return task;
  }
}
