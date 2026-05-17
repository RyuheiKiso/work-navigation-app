# 04 AndonKaizenFlow 詳細設計

本章は MOD-FE-HA-006（AndonKaizenFlow）の詳細設計を確定する。アンドン発報（FR-KZ-001〜003）・不適合登録（FR-KZ-004〜006）・改善提案（FR-KZ-007）のオフライン挙動・WebSocket プッシュ通知・Outbox 蓄積の全仕様を定める。

---

## 1. モジュールインターフェース

```typescript
// src/features/kaizen/AndonKaizenFlowModule.ts

export type AndonAlertType =
  | 'EQUIPMENT_FAILURE'   // 設備異常
  | 'QUALITY_DEFECT'      // 品質不良
  | 'MATERIAL_SHORTAGE'   // 資材不足
  | 'SAFETY_INCIDENT'     // 安全事案
  | 'LINE_STOP';          // ライン停止

export type AndonAlertStatus =
  | 'ALERTING'    // 発報中（未対応）
  | 'RESPONDING'  // 対応中
  | 'RESOLVED'    // 解決済み
  | 'CANCELLED';  // キャンセル

export interface AndonAlertEntity {
  alertId: string;             // UUID v7
  execId: string | null;       // 作業実行 ID（作業中以外の発報時は null）
  alertType: AndonAlertType;
  description: string;         // 発報説明（最大 500 文字）
  status: AndonAlertStatus;
  raisedBy: string;            // userId
  raisedAt: string;            // ISO 8601 UTC
  resolvedAt: string | null;
  resolvedBy: string | null;
  terminalId: string;
}

/** 不適合登録コマンド（4M 分類） */
export interface RegisterNonconformityCmd {
  execId: string;
  stepId: string;
  category: NonconformityCategory;
  cause4M: Cause4M;
  description: string;          // 最大 1000 文字
  immediateAction: string;      // 応急処置（最大 500 文字）
  detectedBy: string;           // userId
  evidenceIds: string[];        // 関連する証拠ファイル ID
}

export type NonconformityCategory =
  | 'DIMENSION_NG'       // 寸法不良
  | 'APPEARANCE_NG'      // 外観不良
  | 'FUNCTION_NG'        // 機能不良
  | 'PROCESS_DEVIATION'  // 工程逸脱
  | 'OTHER';

export type Cause4M =
  | 'MAN'       // 人
  | 'MACHINE'   // 機械
  | 'MATERIAL'  // 材料
  | 'METHOD';   // 方法

export interface NonconformityEntity {
  nonconformityId: string;   // UUID v7
  execId: string;
  stepId: string;
  category: NonconformityCategory;
  cause4M: Cause4M;
  description: string;
  immediateAction: string;
  status: 'OPEN' | 'UNDER_REVIEW' | 'CLOSED';
  detectedBy: string;
  detectedAt: string;
  evidenceIds: string[];
}

/** 改善提案 */
export interface KaizenSuggestionEntity {
  suggestionId: string;      // UUID v7
  execId: string | null;
  stepId: string | null;
  title: string;             // 最大 200 文字
  description: string;       // 最大 2000 文字
  suggestedBy: string;
  suggestedAt: string;
  status: 'DRAFT' | 'SUBMITTED';
}

export interface AndonKaizenFlowModule {
  /**
   * FNC-FE-009: アンドン発報
   * オフライン時は Outbox に蓄積し、接続回復後に送信
   */
  raiseAndon(
    execId: string | null,
    alertType: AndonAlertType,
    description: string,
  ): Promise<AndonAlertEntity>;

  /**
   * FNC-FE-010: 不適合登録
   * 4M 分類・証拠ファイル添付・WorkEvent 連携
   */
  registerNonconformity(
    cmd: RegisterNonconformityCmd,
  ): Promise<NonconformityEntity>;

  /** 改善提案の登録 */
  submitKaizenSuggestion(
    cmd: Omit<KaizenSuggestionEntity, 'suggestionId' | 'suggestedAt' | 'status'>,
  ): Promise<KaizenSuggestionEntity>;

  /** 未解決アンドン一覧（端末ローカル）*/
  getActiveAlerts(): Promise<AndonAlertEntity[]>;
}
```

---

## 2. アンドン発報フロー（FNC-FE-009）

### 2-1. オンライン時フロー

```
1. AndonAlertEntity を UUID v7 で生成（status: 'ALERTING'）
2. SQLite local_andon_alerts に INSERT
3. WorkEvent（activity: 'andon_raised'）を生成し SQLite に INSERT
4. OutboxWorker.enqueue(andonEvent) → POST /api/v1/andon/alerts
5. バックエンドが WebSocket で supervisor/system_admin に PUSH（§4 参照）
6. AndonAlertEntity を返す
```

### 2-2. オフライン時フロー（Outbox 蓄積）

```
1. AndonAlertEntity を生成（isOffline: true）
2. SQLite local_andon_alerts に INSERT
3. WorkEvent を SQLite に INSERT（isOffline: true）
4. OutboxEvent（type: 'ANDON_RAISED'）を SQLite outbox_events に INSERT
5. NetworkState が CONNECTED / DEGRADED に回復した時点で OutboxWorker が送信
6. Emergency Mode（切断 5 分超）では UI に「アンドン発報は接続回復後に送信されます」バナーを表示
```

```typescript
// src/features/kaizen/AndonService.ts
import { v7 as uuidv7 } from 'uuid';
import type { LocalDbService } from '../../shared/db/LocalDbService';
import type { OutboxWorker } from '../network/outbox/OutboxWorker';
import type { ClockService } from '../../shared/clock/ClockService';
import type { AndonAlertEntity, AndonAlertType, AndonKaizenFlowModule } from './types';
import { DomainError } from '../../shared/errors/DomainError';

export class AndonService implements Pick<AndonKaizenFlowModule, 'raiseAndon' | 'getActiveAlerts'> {
  constructor(
    private readonly localDb: LocalDbService,
    private readonly outboxWorker: OutboxWorker,
    private readonly clock: ClockService,
  ) {}

  async raiseAndon(
    execId: string | null,
    alertType: AndonAlertType,
    description: string,
  ): Promise<AndonAlertEntity> {
    if (description.trim().length === 0) {
      throw new DomainError('ERR-VAL-007', 'アンドン発報には説明が必須です');
    }
    if (description.length > 500) {
      throw new DomainError('ERR-VAL-008', 'アンドン発報の説明は 500 文字以内にしてください');
    }

    const alertId = uuidv7();
    const now = this.clock.nowIso();

    const alert: AndonAlertEntity = {
      alertId,
      execId,
      alertType,
      description,
      status: 'ALERTING',
      raisedBy: await this.localDb.getCurrentUserId(),
      raisedAt: now,
      resolvedAt: null,
      resolvedBy: null,
      terminalId: await this.localDb.getTerminalId(),
    };

    // ローカル保存
    await this.localDb.insertAndonAlert(alert);

    // WorkEvent 生成・Outbox エンキュー
    const event = await this.buildAndonEvent(alert);
    await this.localDb.insertWorkEvent(event);
    await this.outboxWorker.enqueue(event);

    // EQUIPMENT_FAILURE は中断処理とセットで自動発報（FR-KZ-001 連携）
    if (alertType === 'EQUIPMENT_FAILURE' && execId != null) {
      await this.localDb.setEquipmentFailureFlag(execId);
    }

    return alert;
  }

  async getActiveAlerts(): Promise<AndonAlertEntity[]> {
    return this.localDb.getAndonAlerts({ status: 'ALERTING' });
  }

  private async buildAndonEvent(alert: AndonAlertEntity): Promise<WorkEventEntity> {
    const prevEvent = alert.execId != null
      ? await this.localDb.getLastWorkEvent(alert.execId)
      : null;
    const prevHash = prevEvent?.contentHash ?? '0'.repeat(64);
    const eventId = uuidv7();
    const payloadJson = JSON.stringify({
      alertId: alert.alertId,
      alertType: alert.alertType,
      description: alert.description,
    });
    const canonicalObj = {
      eventId,
      caseId: alert.execId ?? alert.alertId,
      activity: 'andon_raised',
      timestampClient: alert.raisedAt,
      resource: alert.raisedBy,
      payload: payloadJson,
      prevHash,
    };
    const contentHash = computeSha256(JSON.stringify(canonicalObj));

    return {
      eventId,
      caseId: alert.execId ?? alert.alertId,
      activity: 'andon_raised',
      timestampClient: alert.raisedAt,
      resource: alert.raisedBy,
      sopVersionId: '',
      stepId: '',
      payload: payloadJson,
      prevHash,
      contentHash,
      terminalId: alert.terminalId,
      isOffline: false,
    };
  }
}
```

---

## 3. 不適合登録フロー（FNC-FE-010）

```typescript
// src/features/kaizen/NonconformityService.ts

export class NonconformityService implements Pick<AndonKaizenFlowModule, 'registerNonconformity'> {
  constructor(
    private readonly localDb: LocalDbService,
    private readonly outboxWorker: OutboxWorker,
    private readonly clock: ClockService,
  ) {}

  async registerNonconformity(
    cmd: RegisterNonconformityCmd,
  ): Promise<NonconformityEntity> {
    // バリデーション
    if (cmd.description.trim().length === 0) {
      throw new DomainError('ERR-VAL-009', '不適合説明が必須です');
    }
    if (cmd.description.length > 1000) {
      throw new DomainError('ERR-VAL-010', '不適合説明は 1000 文字以内にしてください');
    }

    const nonconformityId = uuidv7();
    const now = this.clock.nowIso();

    const entity: NonconformityEntity = {
      nonconformityId,
      execId: cmd.execId,
      stepId: cmd.stepId,
      category: cmd.category,
      cause4M: cmd.cause4M,
      description: cmd.description,
      immediateAction: cmd.immediateAction,
      status: 'OPEN',
      detectedBy: cmd.detectedBy,
      detectedAt: now,
      evidenceIds: cmd.evidenceIds,
    };

    // SQLite に INSERT
    await this.localDb.insertNonconformity(entity);

    // WorkEvent（activity: 'nonconformity_registered'）を生成
    const event = await this.buildNonconformityEvent(entity);
    await this.localDb.insertWorkEvent(event);
    await this.outboxWorker.enqueue(event);

    return entity;
  }

  private async buildNonconformityEvent(entity: NonconformityEntity): Promise<WorkEventEntity> {
    const prevEvent = await this.localDb.getLastWorkEvent(entity.execId);
    const prevHash = prevEvent?.contentHash ?? '0'.repeat(64);
    const eventId = uuidv7();
    const payloadJson = JSON.stringify({
      nonconformityId: entity.nonconformityId,
      category: entity.category,
      cause4M: entity.cause4M,
      evidenceIds: entity.evidenceIds,
    });
    const canonicalObj = {
      eventId,
      caseId: entity.execId,
      activity: 'nonconformity_registered',
      timestampClient: entity.detectedAt,
      resource: entity.detectedBy,
      payload: payloadJson,
      prevHash,
    };
    const contentHash = computeSha256(JSON.stringify(canonicalObj));

    return {
      eventId,
      caseId: entity.execId,
      activity: 'nonconformity_registered',
      timestampClient: entity.detectedAt,
      resource: entity.detectedBy,
      sopVersionId: '',
      stepId: entity.stepId,
      payload: payloadJson,
      prevHash,
      contentHash,
      terminalId: await this.localDb.getTerminalId(),
      isOffline: false,
    };
  }
}
```

---

## 4. WebSocket プッシュ通知（オンライン時）

アンドン発報が POST /api/v1/andon/alerts でバックエンドに届くと、MOD-BE-006（wnav_outbox）が WebSocket チャンネルに PUSH する。

| イベント | WebSocket チャンネル | 受信者ロール | ペイロード |
|---|---|---|---|
| andon_raised | `/ws/alerts` | supervisor・system_admin | `{ alertId, alertType, description, raisedBy, raisedAt }` |
| nonconformity_registered | `/ws/quality` | quality_admin・supervisor | `{ nonconformityId, category, cause4M, detectedAt }` |
| andon_resolved | `/ws/alerts` | supervisor・system_admin | `{ alertId, resolvedBy, resolvedAt }` |

オフライン時は Outbox に蓄積されるため、接続回復後に順次送信される。WebSocket 接続が切れている受信者には HTTP ポーリング（GET /api/v1/andon/alerts?status=ALERTING）でフォールバックする。

---

## 5. エラーコード対応表

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-007 | アンドン説明が空 | インライン警告、発報ボタン無効化 |
| ERR-VAL-008 | アンドン説明が 500 文字超 | 文字数カウンター赤表示 |
| ERR-VAL-009 | 不適合説明が空 | インライン警告 |
| ERR-VAL-010 | 不適合説明が 1000 文字超 | 文字数カウンター赤表示 |
| ERR-SYS-003 | SQLite 書き込み失敗 | リトライ 3 回後エラー画面 |

---

**本節で確定した方針**
- **アンドン発報・不適合登録の両操作でオフライン時は Outbox に蓄積し、接続回復後に自動送信する Offline-First 設計を採用した（P1 原則の完全遵守）。**
- **EQUIPMENT_FAILURE アンドンは中断処理（SuspensionFlow）と連動して自動発報し、設備故障時の即時エスカレーションを保証した。**
- **アンドン発報・不適合登録は WorkEvent（ハッシュチェーン連結）として記録し、ALCOA+ Traceable・Contemporaneous の要件を満たした。**

---

## 参照業界分析

### 必須
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)

### 関連
- [`90_業界分析/12_認知工学と状況認識.md`](../../90_業界分析/12_認知工学と状況認識.md)
