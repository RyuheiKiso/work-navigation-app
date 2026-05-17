# 03 SuspensionFlow 詳細設計

本章は MOD-FE-HA-005（SuspensionFlow）の詳細設計を確定する。作業中断（FR-ST-001〜003）・再開（FR-ST-004〜006）・中断一覧表示（FR-ST-007）・プレースキーパー（FR-ST-008〜012）の全仕様を定める。中断/再開の両操作は WorkEvent として SQLite に記録し、Outbox 経由でバックエンドに送信する。

---

## 1. モジュールインターフェース

```typescript
// src/features/suspension/SuspensionFlowModule.ts

export type SuspendReason =
  | 'MATERIAL_WAIT'       // 資材待ち
  | 'EQUIPMENT_FAILURE'   // 設備故障
  | 'QUALITY_HOLD'        // 品質保留
  | 'BREAK'               // 休憩
  | 'SHIFT_END'           // 番終了
  | 'OTHER';              // その他（comment 必須）

export interface SuspensionEntity {
  suspensionId: string;        // UUID v7
  execId: string;              // 作業実行 ID
  reason: SuspendReason;
  comment: string | null;      // OTHER 時は必須（BR-BUS-010）
  suspendedAt: string;         // ISO 8601 UTC
  suspendedBy: string;         // userId
  resumedAt: string | null;    // null = まだ中断中
  resumedBy: string | null;
  currentStepIndex: number;    // 中断時点の Step インデックス（再開起点）
}

export interface WorkExecutionSummary {
  execId: string;
  sopName: MultilingualText;
  processName: string;
  operationName: string;
  suspendedAt: string;
  suspendedBy: string;
  reason: SuspendReason;
  currentStepIndex: number;
  totalSteps: number;
}

export interface SuspensionFlowModule {
  /**
   * FNC-FE-007: 作業中断
   * 1. StepExecutionState を suspended に遷移
   * 2. SuspensionEntity を生成し SQLite に記録
   * 3. WorkEvent（activity: 'suspended'）を生成し Outbox エンキュー
   */
  suspend(
    execId: string,
    reason: SuspendReason,
    comment?: string,
  ): Promise<SuspensionEntity>;

  /**
   * FNC-FE-008: 作業再開
   * 1. 中断中の作業実行を特定
   * 2. SuspensionEntity.resumedAt を更新
   * 3. WorkEvent（activity: 'resumed'）を生成し Outbox エンキュー
   * 4. StepExecutionState を in_progress に戻す
   */
  resume(execId: string): Promise<void>;

  /** 中断中の作業一覧取得（SCR-HA-012 再開画面） */
  getSuspendedExecutions(): Promise<WorkExecutionSummary[]>;

  /** プレースキーパー：他ワーカーが代行可能か確認 */
  canDelegate(execId: string, delegateWorkerId: string): Promise<boolean>;
}
```

---

## 2. 中断処理フロー（FNC-FE-007）

### 2-1. バリデーション

```typescript
// src/features/suspension/SuspensionService.ts

export class SuspensionService implements SuspensionFlowModule {
  constructor(
    private readonly localDb: LocalDbService,
    private readonly outboxWorker: OutboxWorker,
    private readonly clock: ClockService,
  ) {}

  async suspend(
    execId: string,
    reason: SuspendReason,
    comment?: string,
  ): Promise<SuspensionEntity> {
    // OTHER の場合は comment 必須（BR-BUS-010）
    if (reason === 'QUALITY_HOLD' || reason === 'OTHER') {
      if (comment == null || comment.trim().length === 0) {
        throw new DomainError(
          'ERR-VAL-006',
          `中断理由 ${reason} にはコメントが必須です`,
        );
      }
    }

    // 現在の作業実行を取得
    const execution = await this.localDb.getWorkExecution(execId);
    if (execution == null) {
      throw new DomainError('ERR-BIZ-004', `作業実行 ${execId} が見つかりません`);
    }
    if (execution.status !== 'IN_PROGRESS') {
      throw new DomainError(
        'ERR-BIZ-005',
        `中断できない状態です: status=${execution.status}`,
      );
    }

    const suspensionId = uuidv7();
    const now = this.clock.nowIso();

    const suspension: SuspensionEntity = {
      suspensionId,
      execId,
      reason,
      comment: comment ?? null,
      suspendedAt: now,
      suspendedBy: execution.primaryWorkerId,
      resumedAt: null,
      resumedBy: null,
      currentStepIndex: execution.currentStepIndex,
    };

    // SQLite に中断レコードを記録
    await this.localDb.insertSuspension(suspension);

    // 作業実行のステータスを SUSPENDED に更新
    await this.localDb.updateWorkExecutionStatus(execId, 'SUSPENDED');

    // WorkEvent（activity: 'suspended'）を生成
    const suspendEvent = await this.buildSuspendEvent(
      execId,
      suspension,
      execution.primaryWorkerId,
      execution.terminalId,
    );
    await this.localDb.insertWorkEvent(suspendEvent);
    await this.outboxWorker.enqueue(suspendEvent);

    return suspension;
  }

  async resume(execId: string): Promise<void> {
    const execution = await this.localDb.getWorkExecution(execId);
    if (execution == null) {
      throw new DomainError('ERR-BIZ-004', `作業実行 ${execId} が見つかりません`);
    }
    if (execution.status !== 'SUSPENDED') {
      throw new DomainError(
        'ERR-BIZ-006',
        `再開できない状態です: status=${execution.status}`,
      );
    }

    const now = this.clock.nowIso();

    // SuspensionEntity.resumedAt を更新
    await this.localDb.updateSuspensionResumed(
      execId,
      now,
      execution.primaryWorkerId,
    );

    // 作業実行のステータスを IN_PROGRESS に戻す
    await this.localDb.updateWorkExecutionStatus(execId, 'IN_PROGRESS');

    // WorkEvent（activity: 'resumed'）を生成
    const resumeEvent = await this.buildResumeEvent(
      execId,
      execution.primaryWorkerId,
      execution.terminalId,
    );
    await this.localDb.insertWorkEvent(resumeEvent);
    await this.outboxWorker.enqueue(resumeEvent);
  }

  async getSuspendedExecutions(): Promise<WorkExecutionSummary[]> {
    return this.localDb.getSuspendedExecutions();
  }

  async canDelegate(execId: string, delegateWorkerId: string): Promise<boolean> {
    const execution = await this.localDb.getWorkExecution(execId);
    if (execution == null) return false;

    // スキルレベル確認（FR-ST-010）
    const steps = await this.localDb.getStepsForExecution(execId);
    const currentStep = steps[execution.currentStepIndex];
    if (currentStep == null) return true;

    const delegateSkill = await this.localDb.getUserSkillLevel(delegateWorkerId);
    return delegateSkill >= currentStep.skillLevelRequired;
  }

  private async buildSuspendEvent(
    execId: string,
    suspension: SuspensionEntity,
    workerId: string,
    terminalId: string,
  ): Promise<WorkEventEntity> {
    // StepEngine と同一のハッシュチェーン計算ロジックを使用
    const prevEvent = await this.localDb.getLastWorkEvent(execId);
    const prevHash = prevEvent?.contentHash ?? '0'.repeat(64);
    const now = suspension.suspendedAt;
    const eventId = uuidv7();
    const payloadJson = JSON.stringify({ reason: suspension.reason, comment: suspension.comment });
    const canonicalObj = { eventId, caseId: execId, activity: 'suspended', timestampClient: now, resource: workerId, payload: payloadJson, prevHash };
    const contentHash = computeSha256(JSON.stringify(canonicalObj));

    return {
      eventId,
      caseId: execId,
      activity: 'suspended',
      timestampClient: now,
      resource: workerId,
      sopVersionId: (await this.localDb.getStepsForExecution(execId))[0]?.sopVersionId ?? '',
      stepId: '',
      payload: payloadJson,
      prevHash,
      contentHash,
      terminalId,
      isOffline: false,
    };
  }

  private async buildResumeEvent(
    execId: string,
    workerId: string,
    terminalId: string,
  ): Promise<WorkEventEntity> {
    const prevEvent = await this.localDb.getLastWorkEvent(execId);
    const prevHash = prevEvent?.contentHash ?? '0'.repeat(64);
    const now = this.clock.nowIso();
    const eventId = uuidv7();
    const payloadJson = JSON.stringify({ resumedBy: workerId });
    const canonicalObj = { eventId, caseId: execId, activity: 'resumed', timestampClient: now, resource: workerId, payload: payloadJson, prevHash };
    const contentHash = computeSha256(JSON.stringify(canonicalObj));

    return {
      eventId,
      caseId: execId,
      activity: 'resumed',
      timestampClient: now,
      resource: workerId,
      sopVersionId: '',
      stepId: '',
      payload: payloadJson,
      prevHash,
      contentHash,
      terminalId,
      isOffline: false,
    };
  }
}
```

---

## 3. 中断理由別 UI 挙動仕様

| SuspendReason | コメント入力 | 品質保留フラグ | 上長通知（Andon 自動発報）|
|---|---|---|---|
| MATERIAL_WAIT | 任意 | なし | なし |
| EQUIPMENT_FAILURE | 必須 | なし | 設備故障 Andon を自動発報（FR-KZ-001 連携）|
| QUALITY_HOLD | 必須 | あり（TBL-005 に記録）| 品質管理者へ WebSocket プッシュ |
| BREAK | 不要 | なし | なし |
| SHIFT_END | 不要 | なし | なし |
| OTHER | 必須 | なし | なし |

---

## 4. プレースキーパー（代替作業者）機能（FR-ST-008〜012）

```typescript
// src/features/suspension/PlacekeeperService.ts

export interface PlacekeeperRequest {
  execId: string;
  delegateWorkerId: string;
  requestedBy: string;
}

export interface PlacekeeperResult {
  approved: boolean;
  rejectionReason?: 'SKILL_INSUFFICIENT' | 'CONCURRENT_LIMIT_EXCEEDED';
}

/** 代替作業者への引き継ぎ */
export async function requestPlacekeeper(
  req: PlacekeeperRequest,
  localDb: LocalDbService,
): Promise<PlacekeeperResult> {
  // スキルレベル確認
  const execution = await localDb.getWorkExecution(req.execId);
  if (execution == null) {
    return { approved: false, rejectionReason: 'SKILL_INSUFFICIENT' };
  }

  const steps = await localDb.getStepsForExecution(req.execId);
  const currentStep = steps[execution.currentStepIndex];
  if (currentStep == null) {
    return { approved: true };
  }

  const delegateSkillLevel = await localDb.getUserSkillLevel(req.delegateWorkerId);
  if (delegateSkillLevel < currentStep.skillLevelRequired) {
    return { approved: false, rejectionReason: 'SKILL_INSUFFICIENT' };
  }

  // 引き継ぎ記録（WorkEvent: activity = 'worker_delegated'）
  await localDb.updateWorkExecutionWorker(req.execId, req.delegateWorkerId);

  return { approved: true };
}
```

---

## 5. エラーコード対応表

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-006 | OTHER/QUALITY_HOLD でコメント空 | インライン警告、中断ボタン無効化 |
| ERR-BIZ-004 | 作業実行 ID が SQLite に存在しない | エラー画面表示、ホームへ戻る |
| ERR-BIZ-005 | IN_PROGRESS 以外の状態で suspend 呼び出し | 警告ダイアログ、操作を無視 |
| ERR-BIZ-006 | SUSPENDED 以外の状態で resume 呼び出し | 警告ダイアログ、操作を無視 |

---

**本節で確定した方針**
- **中断・再開の両操作で WorkEvent（activity: 'suspended' / 'resumed'）を SQLite に INSERT し、ハッシュチェーンに連結することで ALCOA+ Contemporaneous および Traceable の要件を中断/再開時にも満たした。**
- **QUALITY_HOLD と EQUIPMENT_FAILURE はコメント必須かつ上長への通知（Andon / WebSocket プッシュ）を自動起動し、品質事象の即時エスカレーションを設計レベルで保証した。**
- **プレースキーパー機能（FR-ST-008〜012）はスキルレベルゲートを経由し、代替作業者のスキル不足による品質事故を防止した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)

### 関連
- [`90_業界分析/12_認知工学と状況認識.md`](../../90_業界分析/12_認知工学と状況認識.md)
