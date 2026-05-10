// 対応 §: ロードマップ §3.1.1 §3.4.1 §10.1 §10.6.1
// 端末側「作業（Task）」Aggregate。バックエンドと同じ HSM（[`hsm-task.puml`]）と整合する。

// 値オブジェクト
import { TaskId, DeviceId, LamportTimestamp } from './value-object';

/** 作業状態（HSM の状態名と一致） */
export type TaskState =
  | 'Idle'
  | 'Ready'
  | 'Running'
  | 'Suspended'
  | 'Exception'
  | 'Completed'
  | 'Failed'
  | 'Aborted';

/** 完了条件 */
export type CompletionCriteria = 'Manual' | 'Photo';

/** 完了条件判定のための証跡 */
export interface Evidence {
  // 人手で完了マークされたか
  manuallyMarked: boolean;
  // 写真が添付されたか
  photoAttached: boolean;
}

/**
 * Task Aggregate（端末側）
 *
 * 不変条件:
 * - state は HSM 遷移グラフに従う
 * - lamport は単調増加（INV-08）
 */
export class Task {
  // 識別
  readonly id: TaskId;
  // 状態（読み取り専用、変更は内部メソッドのみ）
  private _state: TaskState;
  // 完了条件
  readonly completionCriteria: CompletionCriteria;
  // 主体端末
  readonly deviceId: DeviceId;
  // 直近の Lamport
  private _lamport: LamportTimestamp;
  // 前提条件充足フラグ
  private preconditionSatisfied: boolean;

  // private コンストラクタ
  private constructor(
    id: TaskId,
    completionCriteria: CompletionCriteria,
    deviceId: DeviceId
  ) {
    // 識別を保持
    this.id = id;
    // 完了条件を保持
    this.completionCriteria = completionCriteria;
    // 主体端末を保持
    this.deviceId = deviceId;
    // 初期状態は Idle
    this._state = 'Idle';
    // Lamport は 0 から開始
    this._lamport = LamportTimestamp.zero();
    // 前提条件は未充足
    this.preconditionSatisfied = false;
  }

  /** 新規 Task を Idle 状態で作る */
  static create(
    id: TaskId,
    completionCriteria: CompletionCriteria,
    deviceId: DeviceId
  ): Task {
    // 直接コンストラクタを呼ぶ
    return new Task(id, completionCriteria, deviceId);
  }

  /** 状態の getter */
  get state(): TaskState {
    // フィールド値を返す
    return this._state;
  }

  /** Lamport の getter */
  get lamport(): LamportTimestamp {
    // フィールド値を返す
    return this._lamport;
  }

  /** 前提条件を満たした旨をマークする（Idle → Ready） */
  markPreconditionSatisfied(): void {
    // Lamport を進める
    this._lamport = this._lamport.next();
    // フラグを立てる
    this.preconditionSatisfied = true;
    // Idle → Ready
    if (this._state === 'Idle') {
      this._state = 'Ready';
    }
  }

  /** 開始する（Ready → Running） */
  start(): void {
    // 前提条件チェック
    if (!this.preconditionSatisfied) {
      // ドメインエラー
      throw new Error('開始条件が満たされていません');
    }
    // 状態が Ready 以外なら遷移不正
    if (this._state !== 'Ready') {
      throw new Error(`不正な状態遷移: ${this._state} -> Running`);
    }
    // Lamport を進める
    this._lamport = this._lamport.next();
    // Running に遷移
    this._state = 'Running';
  }

  /** 中断する（Running → Suspended） */
  suspend(): void {
    // 状態チェック
    if (this._state !== 'Running') {
      throw new Error(`不正な状態遷移: ${this._state} -> Suspended`);
    }
    // Lamport を進める
    this._lamport = this._lamport.next();
    // Suspended に遷移
    this._state = 'Suspended';
  }

  /** 再開する（Suspended → Running） */
  resume(): void {
    // 状態チェック
    if (this._state !== 'Suspended') {
      throw new Error(`不正な状態遷移: ${this._state} -> Running`);
    }
    // Lamport を進める
    this._lamport = this._lamport.next();
    // Running に戻る
    this._state = 'Running';
  }

  /** 完了する（Running → Completed） */
  complete(evidence: Evidence): void {
    // 状態チェック
    if (this._state !== 'Running') {
      throw new Error(`不正な状態遷移: ${this._state} -> Completed`);
    }
    // 完了条件判定
    if (!Task.evidenceMeets(this.completionCriteria, evidence)) {
      throw new Error('完了条件が満たされていません');
    }
    // Lamport を進める
    this._lamport = this._lamport.next();
    // Completed に遷移
    this._state = 'Completed';
  }

  /** 完了条件と証跡の整合性を判定する */
  private static evidenceMeets(
    criteria: CompletionCriteria,
    ev: Evidence
  ): boolean {
    // バリアントごとに判定
    switch (criteria) {
      case 'Manual':
        // 人手判定はフラグのみ
        return ev.manuallyMarked;
      case 'Photo':
        // 写真証跡を要求する
        return ev.photoAttached;
      default:
        // 想定外
        return false;
    }
  }
}
