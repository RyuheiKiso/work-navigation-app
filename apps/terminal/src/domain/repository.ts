// 対応 §: ロードマップ §9.1 §10.1 §10.6
// 端末側 Repository インタフェース。Tauri command / バックエンド REST のいずれの実装も差替可能。

// ドメイン依存
import type { Task } from './task';
import type { TaskId } from './value-object';

/**
 * Task Repository インタフェース（端末側）
 *
 * 実装は adapter 層（Tauri command 経由・REST 経由）。
 */
export interface TaskRepository {
  /** Task を ID で取得する。未存在は null。 */
  findById(id: TaskId): Promise<Task | null>;

  /** Task を保存する（新規・更新の両用） */
  save(task: Task): Promise<void>;
}
