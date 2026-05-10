// 対応 §: ロードマップ §7.1 §10.1 §10.6
// Tauri command を経由してバックエンドへ Task を読み書きする Repository 実装。

// ドメイン依存
import type { TaskRepository } from '../domain/repository';
import { Task, type CompletionCriteria } from '../domain/task';
import { TaskId, DeviceId } from '../domain/value-object';

/** Tauri command の戻り値 DTO */
interface TaskDto {
  id: string;
  state: string;
  device_id: string;
  lamport: number;
}

/**
 * Tauri command 経由の TaskRepository 実装
 *
 * 実コマンド呼び出し（@tauri-apps/api invoke）は Tauri が提供する。
 * テスト容易性のため、invoke 関数を依存注入できる形にする。
 */
export class TauriTaskRepository implements TaskRepository {
  // invoke 関数（差替可能）
  private readonly invokeFn: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

  /** コンストラクタ */
  constructor(invokeFn: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>) {
    // 依存を保持
    this.invokeFn = invokeFn;
  }

  /** ID で Task を取得する */
  async findById(id: TaskId): Promise<Task | null> {
    // Tauri コマンド呼び出し（バックエンドへ中継、未実装時は null を期待）
    const dto = await this.invokeFn<TaskDto | null>('get_task', { id: id.toString() });
    // 未存在
    if (dto === null) {
      return null;
    }
    // 復元（最小: Task.create で初期 Idle として再構築する。後続の §15 マイグレーションで状態復元へ拡充。）
    const taskId = TaskId.of(dto.id);
    const deviceId = DeviceId.of(dto.device_id);
    // 完了条件は manual 既定（§15 拡張で DB から取得するよう拡充）
    const cri: CompletionCriteria = 'Manual';
    // 構築
    return Task.create(taskId, cri, deviceId);
  }

  /** Task を保存する */
  async save(task: Task): Promise<void> {
    // Tauri コマンド呼び出し
    await this.invokeFn<void>('save_task', {
      id: task.id.toString(),
      state: task.state,
      device_id: task.deviceId.toString(),
      lamport: Number(task.lamport.toBigInt())
    });
  }
}
