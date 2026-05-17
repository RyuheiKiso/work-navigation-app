# 07 SQLite / TypeORM スキーマ設計

本章は端末 SQLite の全 TypeORM エンティティを確定する。PostgreSQL TBL カタログ（TBL-001〜035）と 1:1 対応させ、型変換（UUID→TEXT・JSONB→TEXT・TIMESTAMPTZ→TEXT・BOOLEAN→INTEGER）をエンティティ層で吸収する。全テーブルは SQLCipher で透過的に暗号化される。

---

## 1. 型変換規約

| PostgreSQL 型 | SQLite 型 | TypeORM デコレータ |
|---|---|---|
| `UUID` | `TEXT` | `@PrimaryColumn('text')` / `@Column('text')` |
| `JSONB` | `TEXT` | `@Column('text')`（JSON 文字列として格納・取得時に JSON.parse）|
| `TIMESTAMPTZ` | `TEXT` | `@Column('text')`（ISO 8601 UTC 文字列）|
| `BOOLEAN` | `INTEGER` | `@Column('integer')` / `@Column({ type: 'integer', default: 0 })`（0=false, 1=true）|
| `INTEGER` | `INTEGER` | `@Column('integer')` |
| `TEXT` | `TEXT` | `@Column('text')` |
| `NUMERIC` | `REAL` | `@Column('real')` |

---

## 2. LocalWorkEvent（TBL-001 work_events の端末サブセット）

```typescript
// src/shared/db/entities/LocalWorkEvent.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('work_events')
@Index(['caseId'])                      // IDX-HA-001 相当
@Index(['caseId', 'stepId'])            // IDX-HA-002 ロックステップ確認
export class LocalWorkEvent {
  @PrimaryColumn('text')
  eventId!: string;                     // UUID v7（TEXT）

  @Column('text')
  @Index()
  caseId!: string;                      // 作業セッション ID（UUID v7）

  @Column('text')
  activity!: string;
  // 許容値: step_completed / work_started / work_completed
  //         suspended / resumed / andon_raised / andon_resolved
  //         nonconformity_registered / evidence_uploaded / worker_delegated

  @Column('text')
  timestampClient!: string;             // ISO 8601 UTC

  @Column('text')
  resource!: string;                    // worker userId

  @Column('text')
  sopVersionId!: string;               // UUID v7

  @Column('text')
  stepId!: string;                     // UUID v7（activity が step 無関係の場合は空文字）

  @Column('text')
  payload!: string;                    // JSON 文字列（StepPayload のシリアライズ）

  @Column('text')
  prevHash!: string;                   // 直前イベントの contentHash（64 文字 hex）

  @Column('text')
  contentHash!: string;               // SHA-256(canonical JSON)（64 文字 hex）

  @Column('text')
  terminalId!: string;                 // 端末 ID

  @Column('integer', { default: 0 })
  synced!: boolean;                    // 0=未同期, 1=同期済み（OutboxWorker が SENT 後に更新）
}
```

---

## 3. LocalOutboxEvent（TBL-003 outbox_events の端末版）

```typescript
// src/shared/db/entities/LocalOutboxEvent.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('outbox_events')
@Index(['status', 'createdAt'])         // IDX-HA-003 OutboxWorker 取得用
export class LocalOutboxEvent {
  @PrimaryColumn('text')
  outboxEventId!: string;              // UUID v7

  @Column('text')
  eventType!: string;
  // 許容値: STEP_COMPLETED / WORK_STARTED / WORK_COMPLETED
  //         SUSPENDED / RESUMED / ANDON_RAISED / ANDON_RESOLVED
  //         NONCONFORMITY_REGISTERED / EVIDENCE_UPLOADED

  @Column('text')
  payload!: string;                    // JSON 文字列（WorkEventEntity のシリアライズ）

  @Column('text')
  createdAt!: string;                  // ISO 8601 UTC

  @Column('text')
  status!: string;
  // 許容値: PENDING / SENDING / SENT / DEAD_LETTERED

  @Column('integer', { default: 0 })
  retryCount!: number;

  @Column('text', { nullable: true })
  lastAttemptAt!: string | null;       // ISO 8601 UTC

  @Column('text', { nullable: true })
  errorMessage!: string | null;
}
```

---

## 4. LocalSop（TBL-007 sops の端末キャッシュ）

```typescript
// src/shared/db/entities/LocalSop.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('sops')
@Index(['operationId', 'isActive'])     // IDX-HA-004 オペレーション別 SOP
export class LocalSop {
  @PrimaryColumn('text')
  sopId!: string;                       // UUID v7

  @Column('text')
  operationId!: string;                 // UUID v7

  @Column('text')
  name!: string;                        // JSON 文字列（MultilingualText）

  @Column('text')
  version!: string;                     // セマンティックバージョン例: "1.2.0"

  @Column('integer', { default: 0 })
  isActive!: boolean;                   // 0=inactive, 1=active

  @Column('text')
  publishedAt!: string;                 // ISO 8601 UTC

  @Column('text', { nullable: true })
  cachedAt!: string | null;             // 端末キャッシュ日時（TTL 管理用）
}
```

---

## 5. LocalStep（TBL-008 steps の端末キャッシュ）

```typescript
// src/shared/db/entities/LocalStep.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('steps')
@Index(['sopVersionId', 'stepNumber'])  // IDX-HA-005 SOP 内ステップ順序
export class LocalStep {
  @PrimaryColumn('text')
  stepId!: string;                      // UUID v7

  @Column('text')
  @Index()
  sopVersionId!: string;               // UUID v7

  @Column('integer')
  stepNumber!: number;                  // 1-indexed

  @Column('text')
  inputType!: string;
  // 許容値: boolean_check / numeric_input / photo_capture / text_input
  //         slider_range / qr_scan / signature_pad / condition_branch / custom

  @Column('text')
  instructionText!: string;            // JSON 文字列（MultilingualText）

  @Column('integer', { default: 0 })
  evidenceRequired!: boolean;          // 0=不要, 1=必須

  @Column('text', { nullable: true })
  judgmentCondition!: string | null;  // JSON 文字列（JudgmentCondition）

  @Column('integer', { default: 0 })
  skillLevelRequired!: number;

  @Column('text', { nullable: true })
  expectedUnit!: string | null;

  @Column('real', { nullable: true })
  usl!: number | null;                  // Upper Spec Limit

  @Column('real', { nullable: true })
  lsl!: number | null;                  // Lower Spec Limit

  @Column('integer', { default: 0 })
  signRequired!: boolean;              // 0=不要, 1=必須

  @Column('integer', { default: 0 })
  estimatedSeconds!: number;
}
```

---

## 6. LocalWorkExecution（TBL-005 work_executions の端末版）

```typescript
// src/shared/db/entities/LocalWorkExecution.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('work_executions')
@Index(['status'])                      // IDX-HA-006 進行中・中断中の絞り込み
@Index(['primaryWorkerId'])             // IDX-HA-007 作業員別
export class LocalWorkExecution {
  @PrimaryColumn('text')
  execId!: string;                      // UUID v7

  @Column('text')
  sopVersionId!: string;

  @Column('text')
  primaryWorkerId!: string;            // userId

  @Column('text')
  status!: string;
  // 許容値: IN_PROGRESS / SUSPENDED / COMPLETED / ABANDONED

  @Column('integer', { default: 0 })
  currentStepIndex!: number;           // 現在の Step インデックス（0-indexed）

  @Column('text')
  startedAt!: string;                  // ISO 8601 UTC

  @Column('text', { nullable: true })
  completedAt!: string | null;

  @Column('text')
  terminalId!: string;

  @Column('text', { nullable: true })
  lotNumber!: string | null;           // 製造ロット番号

  @Column('text', { nullable: true })
  productId!: string | null;           // 製品 ID（UUID v7）
}
```

---

## 7. LocalAppSettings（端末設定）

```typescript
// src/shared/db/entities/LocalAppSettings.ts
import { Entity, PrimaryColumn, Column } from 'typeorm';

/**
 * 端末ごとに 1 レコードのみ存在（PK = 'singleton'）
 * INSERT OR REPLACE で常に上書き
 */
@Entity('app_settings')
export class LocalAppSettings {
  @PrimaryColumn('text')
  settingsId!: string;                 // 常に 'singleton'

  @Column('text', { default: 'ja' })
  locale!: string;                     // 'ja' | 'en' | 'ja-simple'

  @Column('integer', { default: 0 })
  darkMode!: boolean;                  // 0=light, 1=dark

  @Column('text')
  deviceId!: string;                   // UUID v7（初回起動時に生成・以後固定）

  @Column('text', { nullable: true })
  jwtCache!: string | null;           // JWT トークン（暗号化済み: SQLCipher で保護）

  @Column('text', { nullable: true })
  jwtExpiresAt!: string | null;       // ISO 8601 UTC

  @Column('text', { nullable: true })
  lastMasterSyncAt!: string | null;   // マスタ最終同期日時

  @Column('text', { nullable: true })
  currentUserId!: string | null;      // ログイン中の userId

  @Column('integer', { default: 30000 })
  outboxIntervalMs!: number;           // CFG-OBX-001

  @Column('integer', { default: 300000 })
  emergencyThresholdMs!: number;      // CFG-OBX-005

  @Column('integer', { default: 60 })
  masterSyncIntervalMinutes!: number;  // CFG-007
}
```

---

## 8. LocalEvidenceFile（TBL-009 evidence_files の端末版）

```typescript
// src/shared/db/entities/LocalEvidenceFile.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('evidence_files')
@Index(['stepId'])                      // IDX-HA-008 Step 別証拠
export class LocalEvidenceFile {
  @PrimaryColumn('text')
  evidenceId!: string;                  // UUID v7

  @Column('text')
  stepId!: string;                      // UUID v7

  @Column('text')
  fileHash!: string;                    // SHA-256 hex（Exif 除去後）

  @Column('text')
  filePath!: string;                    // 端末ローカルパス

  @Column('text')
  mimeType!: string;                    // 'image/jpeg' 等

  @Column('text')
  capturedAt!: string;                  // ISO 8601 UTC

  @Column('integer', { default: 0 })
  synced!: boolean;                     // 0=未同期, 1=同期済み
}
```

---

## 9. LocalSuspension（TBL-011 suspensions の端末版）

```typescript
// src/shared/db/entities/LocalSuspension.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('suspensions')
@Index(['execId'])
export class LocalSuspension {
  @PrimaryColumn('text')
  suspensionId!: string;               // UUID v7

  @Column('text')
  @Index()
  execId!: string;

  @Column('text')
  reason!: string;
  // 許容値: MATERIAL_WAIT / EQUIPMENT_FAILURE / QUALITY_HOLD / BREAK / SHIFT_END / OTHER

  @Column('text', { nullable: true })
  comment!: string | null;

  @Column('text')
  suspendedAt!: string;

  @Column('text')
  suspendedBy!: string;

  @Column('text', { nullable: true })
  resumedAt!: string | null;

  @Column('text', { nullable: true })
  resumedBy!: string | null;

  @Column('integer', { default: 0 })
  currentStepIndex!: number;
}
```

---

## 10. LocalAndonAlert（TBL-012 andon_alerts の端末版）

```typescript
// src/shared/db/entities/LocalAndonAlert.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('andon_alerts')
@Index(['status'])
export class LocalAndonAlert {
  @PrimaryColumn('text')
  alertId!: string;                    // UUID v7

  @Column('text', { nullable: true })
  execId!: string | null;

  @Column('text')
  alertType!: string;
  // 許容値: EQUIPMENT_FAILURE / QUALITY_DEFECT / MATERIAL_SHORTAGE / SAFETY_INCIDENT / LINE_STOP

  @Column('text')
  description!: string;

  @Column('text', { default: 'ALERTING' })
  status!: string;
  // 許容値: ALERTING / RESPONDING / RESOLVED / CANCELLED

  @Column('text')
  raisedBy!: string;

  @Column('text')
  raisedAt!: string;

  @Column('text', { nullable: true })
  resolvedAt!: string | null;

  @Column('text', { nullable: true })
  resolvedBy!: string | null;

  @Column('text')
  terminalId!: string;
}
```

---

## 11. LocalNonconformity（TBL-013 nonconformities の端末版）

```typescript
// src/shared/db/entities/LocalNonconformity.ts
import { Entity, PrimaryColumn, Column, Index } from 'typeorm';

@Entity('nonconformities')
@Index(['execId'])
export class LocalNonconformity {
  @PrimaryColumn('text')
  nonconformityId!: string;            // UUID v7

  @Column('text')
  execId!: string;

  @Column('text')
  stepId!: string;

  @Column('text')
  category!: string;
  // 許容値: DIMENSION_NG / APPEARANCE_NG / FUNCTION_NG / PROCESS_DEVIATION / OTHER

  @Column('text')
  cause4M!: string;
  // 許容値: MAN / MACHINE / MATERIAL / METHOD

  @Column('text')
  description!: string;

  @Column('text')
  immediateAction!: string;

  @Column('text', { default: 'OPEN' })
  status!: string;
  // 許容値: OPEN / UNDER_REVIEW / CLOSED

  @Column('text')
  detectedBy!: string;

  @Column('text')
  detectedAt!: string;

  @Column('text')
  evidenceIds!: string;                // JSON 文字列（string[] のシリアライズ）
}
```

---

## 12. インデックス一覧

| IDX-ID | テーブル | 対象列 | 種別 | 用途 |
|---|---|---|---|---|
| IDX-HA-001 | work_events | caseId | B-tree | 作業単位イベント取得 |
| IDX-HA-002 | work_events | (caseId, stepId) | B-tree 複合 | ロックステップ確認 |
| IDX-HA-003 | outbox_events | (status, createdAt) | B-tree 複合（Partial: status='PENDING'）| OutboxWorker バッチ取得 |
| IDX-HA-004 | sops | (operationId, isActive) | B-tree 複合 | オペレーション別公開 SOP |
| IDX-HA-005 | steps | (sopVersionId, stepNumber) | B-tree 複合 | SOP 内ステップ順序 |
| IDX-HA-006 | work_executions | status | B-tree（Partial: status != 'COMPLETED'）| 進行中・中断中の取得 |
| IDX-HA-007 | work_executions | primaryWorkerId | B-tree | 作業員別作業一覧 |
| IDX-HA-008 | evidence_files | stepId | B-tree | Step 別証拠ファイル |

---

## 13. SQLCipher 暗号化範囲

SQLCipher は DataSource を通じて接続されたすべてのテーブルに対してページ単位（デフォルト 4 KB）で AES-256-CBC 暗号化を適用する。特定テーブルの除外は行わない。

```
暗号化範囲: 全テーブル（work_events / outbox_events / sops / steps /
  work_executions / app_settings / evidence_files / suspensions /
  andon_alerts / nonconformities）
暗号化アルゴリズム: AES-256-CBC（SQLCipher デフォルト）
HMAC アルゴリズム: HMAC-SHA512（SQLCipher v4 デフォルト）
KDF: PBKDF2-SHA512, 256,000 イテレーション（SQLCipher v4 デフォルト）
```

---

**本節で確定した方針**
- **全エンティティの UUID 列を TEXT 型で格納し、JSONB / TIMESTAMPTZ / BOOLEAN の PostgreSQL 型を SQLite の TEXT / INTEGER / REAL に変換するエンティティ層規約を確定した。これにより TBL カタログ（TBL-001〜035）との 1:1 対応を保証する。**
- **端末側インデックス（IDX-HA-001〜008）を全エンティティに定義し、OutboxWorker の PENDING 取得（IDX-HA-003）・ロックステップ確認（IDX-HA-002）・進行中作業絞り込み（IDX-HA-006）のクエリ性能を設計レベルで担保した。**
- **SQLCipher v4 の AES-256-CBC + HMAC-SHA512 + PBKDF2-SHA512 256,000 イテレーションで全テーブルを暗号化し、TypeORM の synchronize: false と明示的マイグレーション管理により本番環境での自動スキーマ変更を禁止した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)

### 関連
- [`90_業界分析/12_認知工学と状況認識.md`](../../90_業界分析/12_認知工学と状況認識.md)
