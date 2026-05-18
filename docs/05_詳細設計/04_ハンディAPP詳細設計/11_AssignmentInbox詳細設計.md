# 11 AssignmentInbox 詳細設計

本章は MOD-FE-HA-003 AssignmentInbox の責務・EventSource クライアント設計・SQLite キャッシュ設計・コンポーネント仕様・関数要件を確定する。本章の実装によって FR-SY-013（外部システムからの作業割当受信）・FR-NV-014（割当リアルタイム配信による作業開始）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-HA-003 |
| 物理名 | AssignmentInbox |
| ファイルパス | `src/features/assignment-inbox/`（予定） |
| 関連 FR | FR-SY-013, FR-NV-014 |
| 関連 SCR | SCR-HA-002（作業一覧画面）|
| アクセスロール | operator, supervisor |
| 連携モジュール | MOD-FE-HA-005 LocalDbService（SQLite 永続化）、MOD-FE-HA-008 LocalDbService（Outbox バッファ） |

**責務境界:**
- 本モジュール: SSE 接続管理・割当イベント受信・SQLite キャッシュ更新・AssignmentBanner/List/Detail の UI 表示・割当からの作業開始フロー
- LocalDbService: SQLite の CRUD 実装詳細（呼び出し側のみ）
- StepEngine: 作業開始後のナビゲーション実行（呼び出し側のみ）

**関連コンポーネント:**
- CMP-HA-021 AssignmentBanner
- CMP-HA-022 AssignmentListPanel
- CMP-HA-023 AssignmentDetailDialog

---

## 2. EventSource クライアント設計

### 2-1. 接続仕様

| 項目 | 内容 |
|---|---|
| エンドポイント | `GET /api/v1/sse/assignments`（API-sync-004） |
| ポリフィル | `react-native-sse`（または同等の EventSource ポリフィル） |
| 認証 | Authorization ヘッダ（JWT Bearer トークン） |
| `Last-Event-ID` | `AsyncStorage` から読み出して接続時にヘッダにセット（再起動時の漏れ受信防止） |

### 2-2. 接続フロー（FNC-FE-017）

```typescript
// src/features/assignment-inbox/sseClient.ts

import EventSource from 'react-native-sse';
import AsyncStorage from '@react-native-async-storage/async-storage';

const SSE_ENDPOINT = '/api/v1/sse/assignments';
const LAST_EVENT_ID_KEY = 'assignment_inbox_last_event_id';
const MAX_SSE_RETRY = 3;
const POLL_INTERVAL_MS = 30_000;

let retryCount = 0;
let sseInstance: EventSource | null = null;
let pollTimerId: ReturnType<typeof setInterval> | null = null;
let mode: 'sse' | 'poll' = 'sse';

/**
 * (FNC-FE-017) connectSseOrFallback
 *
 * SSE 接続を試みる。
 * 接続失敗が MAX_SSE_RETRY 回を超えた場合は Pull モード（API-sync-005 ポーリング）に切替。
 * Last-Event-ID を AsyncStorage から読み出し、再接続時の漏れ受信を防ぐ。
 */
export async function connectSseOrFallback(): Promise<void> {
  const lastEventId = await AsyncStorage.getItem(LAST_EVENT_ID_KEY);
  const token = await getAuthToken();   // 認証トークン取得（実装は auth モジュールに委譲）

  const headers: Record<string, string> = {
    Authorization: `Bearer ${token}`,
  };
  if (lastEventId) {
    headers['Last-Event-ID'] = lastEventId;
  }

  sseInstance = new EventSource(`${SSE_ENDPOINT}?terminal_id=${getTerminalId()}`, { headers });

  sseInstance.addEventListener('open', () => {
    retryCount = 0;
    mode = 'sse';
    if (pollTimerId !== null) {
      clearInterval(pollTimerId);
      pollTimerId = null;
    }
  });

  sseInstance.addEventListener('assignment.created', (event) => {
    handleAssignmentEvent({ type: 'assignment.created', data: event.data ?? '', lastEventId: event.lastEventId ?? null });
  });

  sseInstance.addEventListener('assignment.cancelled', (event) => {
    handleAssignmentEvent({ type: 'assignment.cancelled', data: event.data ?? '', lastEventId: event.lastEventId ?? null });
  });

  sseInstance.addEventListener('error', () => {
    retryCount += 1;
    if (retryCount > MAX_SSE_RETRY) {
      sseInstance?.close();
      sseInstance = null;
      switchToPollMode();
    }
  });
}

/**
 * MAX_SSE_RETRY 超過後に Pull モードへ切替。
 * API-sync-005 を POLL_INTERVAL_MS 間隔でポーリングする。
 */
function switchToPollMode(): void {
  mode = 'poll';
  pollTimerId = setInterval(async () => {
    await pollAssignments();
  }, POLL_INTERVAL_MS);
}

/**
 * Pull モード用ポーリング（API-sync-005）。
 * SSE が再接続できた場合は Pull モードを停止して SSE モードに戻る。
 */
async function pollAssignments(): Promise<void> {
  const token = await getAuthToken();
  const lastEventId = await AsyncStorage.getItem(LAST_EVENT_ID_KEY);
  const url = lastEventId
    ? `/api/v1/work-assignments?since=${lastEventId}`
    : '/api/v1/work-assignments';

  const response = await fetch(url, {
    headers: { Authorization: `Bearer ${token}` },
  });

  if (!response.ok) return;

  const assignments: AssignmentPayload[] = await response.json();
  for (const a of assignments) {
    handleAssignmentEvent({
      type: 'assignment.created',
      data: JSON.stringify(a),
      lastEventId: a.assignment_id,
    });
  }

  // Pull 中でも SSE への再接続を試みる
  if (sseInstance === null) {
    await connectSseOrFallback();
  }
}
```

### 2-3. イベント処理（FNC-FE-018）

```typescript
// src/features/assignment-inbox/sseClient.ts（続き）

import { upsertLocalAssignment, updateLocalAssignmentStatus } from './localAssignmentRepository';

export interface SSEEvent {
  type: 'assignment.created' | 'assignment.cancelled';
  data: string;
  lastEventId: string | null;
}

export interface AssignmentPayload {
  assignment_id: string;
  work_pattern_id: string;
  lot_id: string | null;
  due_at: string | null;
  priority: number;
  status: string;
  suggested_worker_key: string | null;
}

/**
 * (FNC-FE-018) handleAssignmentEvent
 *
 * SSE イベントタイプに応じて SQLite の local_assignments を更新する。
 * - 'assignment.created': INSERT OR REPLACE
 * - 'assignment.cancelled': status を 'cancelled' に UPDATE
 */
export function handleAssignmentEvent(event: SSEEvent): void {
  try {
    const payload: AssignmentPayload = JSON.parse(event.data);

    if (event.type === 'assignment.created') {
      const now = new Date().toISOString();
      upsertLocalAssignment({
        assignment_id: payload.assignment_id,
        work_pattern_id: payload.work_pattern_id,
        lot_id: payload.lot_id,
        due_at: payload.due_at,
        priority: payload.priority,
        status: payload.status,
        suggested_worker_id: payload.suggested_worker_key ?? null,
        raw_payload: event.data,
        synced_at: now,
      });
    } else if (event.type === 'assignment.cancelled') {
      updateLocalAssignmentStatus(payload.assignment_id, 'cancelled');
    }

    // Last-Event-ID を AsyncStorage に永続化（再起動時の漏れ受信防止）
    if (event.lastEventId) {
      AsyncStorage.setItem(LAST_EVENT_ID_KEY, event.lastEventId).catch(() => {});
    }
  } catch (e) {
    // パース失敗はサイレントフォール（ログのみ）
    console.warn('[AssignmentInbox] handleAssignmentEvent parse error', e);
  }
}
```

---

## 3. SQLite キャッシュ設計

### 3-1. テーブル定義

テーブル名: `local_assignments`（端末側 SQLite のローカルテーブル）

| 列名 | 型 | 制約 | 説明 |
|---|---|---|---|
| assignment_id | TEXT | PRIMARY KEY | サーバ側 UUID（文字列として格納） |
| work_pattern_id | TEXT | NOT NULL | 作業パターン UUID |
| lot_id | TEXT | NULL 許容 | ロット UUID |
| due_at | TEXT | NULL 許容 | 期限（ISO 8601） |
| priority | INTEGER | NOT NULL | 優先度（1 = 最高） |
| status | TEXT | NOT NULL | pending / dispatched / in_progress / completed / cancelled |
| suggested_worker_id | TEXT | NULL 許容 | 推奨作業員（参考情報・拘束しない） |
| raw_payload | TEXT | NOT NULL | SSE から受信した生 JSON（デバッグ・再処理用） |
| synced_at | TEXT | NOT NULL | 端末側での受信・更新日時（ISO 8601） |

### 3-2. TypeORM エンティティ

```typescript
// src/features/assignment-inbox/LocalAssignment.entity.ts

import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('local_assignments')
export class LocalAssignment {
  @PrimaryColumn('text')
  assignment_id!: string;

  @Column('text')
  work_pattern_id!: string;

  @Column({ type: 'text', nullable: true })
  lot_id!: string | null;

  @Column({ type: 'text', nullable: true })
  due_at!: string | null;

  @Column('integer')
  priority!: number;

  @Column('text')
  status!: string;

  @Column({ type: 'text', nullable: true })
  suggested_worker_id!: string | null;

  @Column('text')
  raw_payload!: string;

  @Column('text')
  synced_at!: string;
}
```

### 3-3. リポジトリ

```typescript
// src/features/assignment-inbox/localAssignmentRepository.ts

import { getDataSource } from '../../infrastructure/db/dataSource';
import { LocalAssignment } from './LocalAssignment.entity';

export interface LocalAssignmentInput {
  assignment_id: string;
  work_pattern_id: string;
  lot_id: string | null;
  due_at: string | null;
  priority: number;
  status: string;
  suggested_worker_id: string | null;
  raw_payload: string;
  synced_at: string;
}

/**
 * assignment.created 受信時: INSERT OR REPLACE（upsert）
 */
export async function upsertLocalAssignment(input: LocalAssignmentInput): Promise<void> {
  const ds = getDataSource();
  const repo = ds.getRepository(LocalAssignment);
  await repo.save(input);
}

/**
 * assignment.cancelled 受信時: status を 'cancelled' に UPDATE
 */
export async function updateLocalAssignmentStatus(
  assignmentId: string,
  status: string,
): Promise<void> {
  const ds = getDataSource();
  const repo = ds.getRepository(LocalAssignment);
  await repo.update({ assignment_id: assignmentId }, { status });
}

/**
 * priority=1 かつ status IN ('pending','dispatched') の行が存在するか確認（AssignmentBanner 用）
 */
export async function hasUrgentPendingAssignment(): Promise<boolean> {
  const ds = getDataSource();
  const repo = ds.getRepository(LocalAssignment);
  const count = await repo.count({
    where: [
      { priority: 1, status: 'pending' },
      { priority: 1, status: 'dispatched' },
    ],
  });
  return count > 0;
}

/**
 * status IN ('pending','dispatched') の行を priority 昇順 → due_at 昇順で取得（AssignmentListPanel 用）
 */
export async function getPendingAssignments(): Promise<LocalAssignment[]> {
  const ds = getDataSource();
  const repo = ds.getRepository(LocalAssignment);
  return repo
    .createQueryBuilder('a')
    .where("a.status IN ('pending', 'dispatched')")
    .orderBy('a.priority', 'ASC')
    .addOrderBy('a.due_at', 'ASC', 'NULLS LAST')
    .getMany();
}

/**
 * 単一割当の詳細取得（AssignmentDetailDialog 用）
 */
export async function getAssignmentById(assignmentId: string): Promise<LocalAssignment | null> {
  const ds = getDataSource();
  const repo = ds.getRepository(LocalAssignment);
  return repo.findOneBy({ assignment_id: assignmentId });
}
```

### 3-4. 期限超過表示ロジック

- `due_at < now()` の行は端末側でも **expired** として表示する。
- ただし actual な status 変更（`completed`/`cancelled` 等）はバックエンド管理とし、端末側は status を変更しない。
- `due_at` が NULL の行は期限なしとして扱い、期限超過判定から除外する。

---

## 4. コンポーネント仕様

### 4-1. CMP-HA-021 AssignmentBanner

```typescript
// src/features/assignment-inbox/AssignmentBanner.tsx

import React from 'react';
import { View, Text, Pressable, StyleSheet } from 'react-native';
import { LocalAssignment } from './LocalAssignment.entity';

interface AssignmentBannerProps {
  /**
   * priority=1 かつ status IN ('pending','dispatched') の最上位割当。
   * 存在しない場合は null を渡し、本コンポーネントは非表示。
   */
  urgentAssignment: LocalAssignment | null;
  onPress: (assignmentId: string) => void;
}

/**
 * CMP-HA-021 AssignmentBanner
 *
 * 表示条件:
 *   local_assignments に priority=1 かつ status IN ('pending','dispatched') の行が存在する場合のみ表示。
 *
 * 表示位置:
 *   SCR-HA-002（作業一覧画面）の最上部（既存の作業一覧の上）。
 *
 * 表示内容:
 *   「緊急割当あり」ラベル + SOP 名 + 期限（残り時間）。
 *
 * タップ:
 *   AssignmentDetailDialog を開く。
 */
export const AssignmentBanner: React.FC<AssignmentBannerProps> = ({
  urgentAssignment,
  onPress,
}) => {
  if (!urgentAssignment) return null;

  const remainingLabel = computeRemainingLabel(urgentAssignment.due_at);

  return (
    <Pressable
      style={styles.banner}
      onPress={() => onPress(urgentAssignment.assignment_id)}
      accessibilityRole="button"
      accessibilityLabel={`緊急割当あり。作業パターン: ${urgentAssignment.work_pattern_id}。期限: ${remainingLabel}`}
    >
      <View style={styles.row}>
        <Text style={styles.urgentLabel}>緊急割当あり</Text>
        <Text style={styles.sopName}>{urgentAssignment.work_pattern_id}</Text>
        {remainingLabel !== null && (
          <Text style={styles.deadline}>{remainingLabel}</Text>
        )}
      </View>
    </Pressable>
  );
};

/**
 * due_at から残り時間ラベルを生成する。
 * due_at が null の場合は null を返す。
 * 期限超過の場合は「期限超過」を返す。
 */
function computeRemainingLabel(dueAt: string | null): string | null {
  if (!dueAt) return null;
  const diffMs = new Date(dueAt).getTime() - Date.now();
  if (diffMs < 0) return '期限超過';
  const diffMin = Math.floor(diffMs / 60_000);
  if (diffMin < 60) return `残り ${diffMin} 分`;
  const diffHour = Math.floor(diffMin / 60);
  return `残り ${diffHour} 時間`;
}

const styles = StyleSheet.create({
  banner: {
    backgroundColor: '#D32F2F',
    paddingHorizontal: 16,
    paddingVertical: 12,
    minHeight: 56,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  urgentLabel: {
    color: '#FFFFFF',
    fontWeight: 'bold',
    fontSize: 14,
  },
  sopName: {
    color: '#FFFFFF',
    flex: 1,
    fontSize: 14,
  },
  deadline: {
    color: '#FFEB3B',
    fontSize: 12,
  },
});
```

### 4-2. CMP-HA-022 AssignmentListPanel

```typescript
// src/features/assignment-inbox/AssignmentListPanel.tsx

import React from 'react';
import { View, Text, Pressable, FlatList, StyleSheet } from 'react-native';
import { LocalAssignment } from './LocalAssignment.entity';

interface AssignmentListPanelProps {
  /**
   * status IN ('pending','dispatched') の割当一覧（priority 昇順 → due_at 昇順）。
   * 空配列の場合は本コンポーネントを非表示にすることを推奨。
   */
  assignments: LocalAssignment[];
  onItemPress: (assignmentId: string) => void;
}

/**
 * CMP-HA-022 AssignmentListPanel
 *
 * 表示条件:
 *   local_assignments に status IN ('pending','dispatched') の行が存在する場合のみ表示。
 *
 * ソート順:
 *   priority 昇順 → due_at 昇順（NULL 最後）。
 *   ソートは localAssignmentRepository.getPendingAssignments() に委譲。
 *
 * 各行の表示内容:
 *   優先度バッジ + SOP 名（work_pattern_id）+ ロット番号（lot_id）+ 期限（due_at）。
 *
 * タップ:
 *   AssignmentDetailDialog を開く。
 */
export const AssignmentListPanel: React.FC<AssignmentListPanelProps> = ({
  assignments,
  onItemPress,
}) => {
  if (assignments.length === 0) return null;

  return (
    <FlatList
      data={assignments}
      keyExtractor={(item) => item.assignment_id}
      renderItem={({ item }) => (
        <AssignmentListItem assignment={item} onPress={onItemPress} />
      )}
    />
  );
};

interface AssignmentListItemProps {
  assignment: LocalAssignment;
  onPress: (assignmentId: string) => void;
}

const AssignmentListItem: React.FC<AssignmentListItemProps> = ({ assignment, onPress }) => {
  const isExpired = assignment.due_at
    ? new Date(assignment.due_at).getTime() < Date.now()
    : false;

  return (
    <Pressable
      style={styles.item}
      onPress={() => onPress(assignment.assignment_id)}
      accessibilityRole="button"
    >
      <PriorityBadge priority={assignment.priority} />
      <View style={styles.itemContent}>
        <Text style={styles.sopName}>{assignment.work_pattern_id}</Text>
        {assignment.lot_id && (
          <Text style={styles.lotId}>ロット: {assignment.lot_id}</Text>
        )}
        {assignment.due_at && (
          <Text style={[styles.dueAt, isExpired && styles.expired]}>
            {isExpired ? '期限超過' : `期限: ${assignment.due_at}`}
          </Text>
        )}
      </View>
    </Pressable>
  );
};

const PriorityBadge: React.FC<{ priority: number }> = ({ priority }) => (
  <View style={[styles.badge, priority === 1 && styles.urgentBadge]}>
    <Text style={styles.badgeText}>{priority === 1 ? '緊急' : `P${priority}`}</Text>
  </View>
);

const styles = StyleSheet.create({
  item: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
    minHeight: 72,
  },
  itemContent: { flex: 1, marginLeft: 8 },
  sopName: { fontSize: 14, fontWeight: '600' },
  lotId: { fontSize: 12, color: '#757575', marginTop: 2 },
  dueAt: { fontSize: 12, color: '#757575', marginTop: 2 },
  expired: { color: '#D32F2F' },
  badge: {
    backgroundColor: '#1565C0',
    borderRadius: 4,
    paddingHorizontal: 6,
    paddingVertical: 2,
    minWidth: 40,
    alignItems: 'center',
  },
  urgentBadge: { backgroundColor: '#D32F2F' },
  badgeText: { color: '#FFFFFF', fontSize: 10, fontWeight: 'bold' },
});
```

### 4-3. CMP-HA-023 AssignmentDetailDialog

```typescript
// src/features/assignment-inbox/AssignmentDetailDialog.tsx

import React, { useState } from 'react';
import {
  Modal,
  View,
  Text,
  Pressable,
  TextInput,
  ScrollView,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import { LocalAssignment } from './LocalAssignment.entity';
import { startFromAssignment } from './assignmentActions';

interface AssignmentDetailDialogProps {
  assignment: LocalAssignment | null;
  visible: boolean;
  onClose: () => void;
}

/**
 * CMP-HA-023 AssignmentDetailDialog
 *
 * 表示情報:
 *   ロット番号・SOP 名（work_pattern_id）・期限・優先度・推奨作業員（参考情報・拘束しない）。
 *
 * ボタン:
 *   [開始]（FR-NV-014）: startFromAssignment を呼び出して case_id を生成しナビゲーション画面へ遷移。
 *   [拒否]: 拒否理由入力テキストボックスを表示し、入力後に
 *           POST /api/v1/work-assignments/{id}/reject を送信。
 */
export const AssignmentDetailDialog: React.FC<AssignmentDetailDialogProps> = ({
  assignment,
  visible,
  onClose,
}) => {
  const [rejectMode, setRejectMode] = useState(false);
  const [rejectReason, setRejectReason] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  if (!assignment) return null;

  const handleStart = async () => {
    setIsLoading(true);
    try {
      await startFromAssignment(assignment.assignment_id);
      onClose();
    } catch (e) {
      console.error('[AssignmentDetailDialog] start error', e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleReject = async () => {
    if (!rejectReason.trim()) return;
    setIsLoading(true);
    try {
      await fetch(`/api/v1/work-assignments/${assignment.assignment_id}/reject`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reason: rejectReason }),
      });
      setRejectMode(false);
      setRejectReason('');
      onClose();
    } catch (e) {
      console.error('[AssignmentDetailDialog] reject error', e);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Modal visible={visible} transparent animationType="slide" onRequestClose={onClose}>
      <View style={styles.overlay}>
        <View style={styles.dialog}>
          <ScrollView>
            <Text style={styles.title}>作業割当詳細</Text>

            <DetailRow label="SOP" value={assignment.work_pattern_id} />
            <DetailRow label="ロット番号" value={assignment.lot_id ?? '—'} />
            <DetailRow
              label="期限"
              value={assignment.due_at ?? '期限なし'}
              highlight={
                assignment.due_at !== null &&
                new Date(assignment.due_at).getTime() < Date.now()
              }
            />
            <DetailRow label="優先度" value={String(assignment.priority)} />
            <DetailRow
              label="推奨作業員（参考）"
              value={assignment.suggested_worker_id ?? '—'}
            />

            {rejectMode ? (
              <View style={styles.rejectSection}>
                <Text style={styles.rejectLabel}>拒否理由を入力してください</Text>
                <TextInput
                  style={styles.rejectInput}
                  value={rejectReason}
                  onChangeText={setRejectReason}
                  multiline
                  numberOfLines={3}
                  placeholder="拒否理由"
                  accessibilityLabel="拒否理由入力"
                />
                <View style={styles.buttonRow}>
                  <Pressable style={styles.rejectConfirmBtn} onPress={handleReject} disabled={isLoading}>
                    <Text style={styles.btnText}>拒否確定</Text>
                  </Pressable>
                  <Pressable style={styles.cancelBtn} onPress={() => setRejectMode(false)}>
                    <Text style={styles.cancelBtnText}>キャンセル</Text>
                  </Pressable>
                </View>
              </View>
            ) : (
              <View style={styles.buttonRow}>
                <Pressable
                  style={styles.startBtn}
                  onPress={handleStart}
                  disabled={isLoading}
                  accessibilityRole="button"
                  accessibilityLabel="作業を開始"
                >
                  {isLoading ? (
                    <ActivityIndicator color="#FFFFFF" />
                  ) : (
                    <Text style={styles.btnText}>開始</Text>
                  )}
                </Pressable>
                <Pressable
                  style={styles.rejectBtn}
                  onPress={() => setRejectMode(true)}
                  accessibilityRole="button"
                  accessibilityLabel="作業を拒否"
                >
                  <Text style={styles.rejectBtnText}>拒否</Text>
                </Pressable>
              </View>
            )}
          </ScrollView>
        </View>
      </View>
    </Modal>
  );
};

const DetailRow: React.FC<{ label: string; value: string; highlight?: boolean }> = ({
  label,
  value,
  highlight = false,
}) => (
  <View style={styles.detailRow}>
    <Text style={styles.detailLabel}>{label}</Text>
    <Text style={[styles.detailValue, highlight && styles.highlightValue]}>{value}</Text>
  </View>
);

const styles = StyleSheet.create({
  overlay: { flex: 1, backgroundColor: 'rgba(0,0,0,0.5)', justifyContent: 'flex-end' },
  dialog: {
    backgroundColor: '#FFFFFF',
    borderTopLeftRadius: 16,
    borderTopRightRadius: 16,
    padding: 24,
    maxHeight: '80%',
  },
  title: { fontSize: 18, fontWeight: 'bold', marginBottom: 16 },
  detailRow: { flexDirection: 'row', paddingVertical: 8, borderBottomWidth: 1, borderBottomColor: '#F5F5F5' },
  detailLabel: { width: 140, fontSize: 14, color: '#757575' },
  detailValue: { flex: 1, fontSize: 14 },
  highlightValue: { color: '#D32F2F', fontWeight: 'bold' },
  buttonRow: { flexDirection: 'row', gap: 12, marginTop: 24 },
  startBtn: { flex: 1, backgroundColor: '#1565C0', borderRadius: 8, padding: 16, alignItems: 'center', minHeight: 56 },
  rejectBtn: { flex: 1, borderWidth: 1, borderColor: '#D32F2F', borderRadius: 8, padding: 16, alignItems: 'center', minHeight: 56 },
  rejectConfirmBtn: { flex: 1, backgroundColor: '#D32F2F', borderRadius: 8, padding: 16, alignItems: 'center' },
  cancelBtn: { flex: 1, borderWidth: 1, borderColor: '#757575', borderRadius: 8, padding: 16, alignItems: 'center' },
  btnText: { color: '#FFFFFF', fontWeight: 'bold', fontSize: 16 },
  rejectBtnText: { color: '#D32F2F', fontWeight: 'bold', fontSize: 16 },
  cancelBtnText: { color: '#757575', fontSize: 16 },
  rejectSection: { marginTop: 16 },
  rejectLabel: { fontSize: 14, marginBottom: 8 },
  rejectInput: { borderWidth: 1, borderColor: '#BDBDBD', borderRadius: 8, padding: 12, fontSize: 14, textAlignVertical: 'top' },
});
```

---

## 5. 関数要件（FNC）

### 5-1. FNC-FE-017: connectSseOrFallback

| 項目 | 内容 |
|---|---|
| FNC-ID | FNC-FE-017 |
| シグネチャ | `connectSseOrFallback(): Promise<void>` |
| 配置 | `src/features/assignment-inbox/sseClient.ts` |
| 責務 | SSE 接続を試みる。接続失敗が MAX_SSE_RETRY（3 回）を超えた場合に Pull モード（API-sync-005 を 30 秒間隔でポーリング）に切替。`Last-Event-ID` を `AsyncStorage` から読み出して再接続時の漏れ受信を防ぐ。 |
| 入力 | なし（内部状態として MAX_SSE_RETRY・POLL_INTERVAL_MS・LAST_EVENT_ID_KEY を使用） |
| 出力 | `Promise<void>` |
| 副作用 | `sseInstance` および `pollTimerId` モジュール変数を更新。AsyncStorage から `Last-Event-ID` を読み出す。 |
| エラー処理 | SSE `error` イベント発火時に retryCount を加算。MAX_SSE_RETRY 超過後は `switchToPollMode()` を呼び出す。 |

### 5-2. FNC-FE-018: handleAssignmentEvent

| 項目 | 内容 |
|---|---|
| FNC-ID | FNC-FE-018 |
| シグネチャ | `handleAssignmentEvent(event: SSEEvent): void` |
| 配置 | `src/features/assignment-inbox/sseClient.ts` |
| 責務 | SSE イベントタイプに応じて SQLite の local_assignments を更新する。`assignment.created` → INSERT OR REPLACE、`assignment.cancelled` → status を 'cancelled' に UPDATE。受信後に `Last-Event-ID` を AsyncStorage に永続化する。 |
| 入力 | `event: SSEEvent`（type / data / lastEventId を含む） |
| 出力 | `void` |
| 副作用 | `upsertLocalAssignment` または `updateLocalAssignmentStatus` を呼び出して SQLite を更新。`AsyncStorage.setItem` で `Last-Event-ID` を永続化。 |
| エラー処理 | JSON パース失敗はサイレントフォール（console.warn のみ）。DB 書き込みエラーは呼び出し元に伝播しない（ログのみ）。 |

### 5-3. FNC-FE-019: startFromAssignment

| 項目 | 内容 |
|---|---|
| FNC-ID | FNC-FE-019 |
| シグネチャ | `startFromAssignment(assignmentId: string): Promise<void>` |
| 配置 | `src/features/assignment-inbox/assignmentActions.ts` |
| 責務 | 割当から case_id を生成し、ナビゲーション画面を起動する。1. `POST /api/v1/work-executions`（assignment_id を指定）で case_id を生成。2. `local_assignments` の status を `in_progress` に UPDATE。3. ナビゲーション画面（StepEngine）へ遷移する。 |
| 入力 | `assignmentId: string` |
| 出力 | `Promise<void>` |
| 副作用 | バックエンド API `POST /api/v1/work-executions` を呼び出す。SQLite `local_assignments.status` を更新。React Navigation のナビゲーションスタックを変更する。 |
| エラー処理 | API 呼び出し失敗時は例外を呼び出し元（CMP-HA-023）にスローし、ダイアログ内でエラー表示を行う。 |

```typescript
// src/features/assignment-inbox/assignmentActions.ts

import { updateLocalAssignmentStatus } from './localAssignmentRepository';

export interface StartFromAssignmentResponse {
  case_id: string;
  work_pattern_id: string;
}

/**
 * (FNC-FE-019) startFromAssignment
 *
 * 1. POST /api/v1/work-executions で case_id を生成（FR-NV-014）
 * 2. local_assignments を in_progress に更新
 * 3. ナビゲーション画面（StepEngine）へ遷移
 */
export async function startFromAssignment(assignmentId: string): Promise<void> {
  const token = await getAuthToken();

  const response = await fetch('/api/v1/work-executions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ assignment_id: assignmentId }),
  });

  if (!response.ok) {
    throw new Error(`work-executions POST failed: ${response.status}`);
  }

  const data: StartFromAssignmentResponse = await response.json();

  // local_assignments を in_progress に更新
  await updateLocalAssignmentStatus(assignmentId, 'in_progress');

  // StepEngine ナビゲーション画面へ遷移
  navigateToStepEngine(data.case_id, data.work_pattern_id);
}

/**
 * StepEngine 画面へのナビゲーション。
 * React Navigation の navigate を呼び出す実装は navigation モジュールに委譲。
 */
function navigateToStepEngine(caseId: string, workPatternId: string): void {
  // 実装: useNavigation().navigate('StepEngine', { caseId, workPatternId })
  // 本関数は navigation インスタンスが必要なためコンポーネント外から呼ぶ際は
  // navigationRef.navigate('StepEngine', { caseId, workPatternId }) を使用する
}
```

---

## 6. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| assignment_id | UUID 形式であること | ERR-VAL-032 |
| priority | 1 以上の整数であること | ERR-VAL-032 |
| status | pending / dispatched / in_progress / completed / cancelled のいずれかであること | ERR-VAL-032 |
| 拒否理由 | [拒否] 押下時は 1 文字以上の入力が必須 | ERR-VAL-002 |
| 期限超過表示 | `due_at < now()` の場合に「期限超過」表示（actual status は変更しない） | — |

---

## 7. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| SSE 接続失敗（3 回リトライ後） | Pull モード（API-sync-005 を 30 秒間隔でポーリング）に切替。バナー「オフラインモード」を表示。 |
| JSON パース失敗（handleAssignmentEvent） | サイレントフォール（console.warn のみ）。該当イベントはスキップして処理を継続。 |
| startFromAssignment API 失敗 | CMP-HA-023 がエラーダイアログを表示。local_assignments の status は変更しない。 |
| 拒否 POST 失敗 | CMP-HA-023 がエラートースト通知を表示。再試行ボタンを提供。 |
| SQLite 書き込みエラー | LocalDbService のエラーハンドリングに委譲。最悪ケースは画面リロード後に SSE 再接続で復旧。 |

---

## 8. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| SSE イベント受信 → SQLite 書き込み | P95 ≤ 100ms |
| AssignmentListPanel 表示（100 件） | P95 ≤ 200ms（FlatList 仮想スクロール使用） |
| startFromAssignment API 呼び出し | P95 ≤ 1s |
| AssignmentDetailDialog 開閉 | P95 ≤ 100ms |

---

## 9. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| AssignmentBanner | `accessibilityRole="button"` + `accessibilityLabel` でフルテキスト説明（「緊急割当あり。SOP名。期限」） |
| AssignmentListPanel（各行） | タップターゲット minHeight 72dp（CFG-013 準拠）|
| AssignmentDetailDialog | `[開始]`/`[拒否]` ボタンに `accessibilityRole="button"` + `accessibilityLabel` |
| AssignmentDetailDialog（DetailRow） | 各行の label/value を `accessibilityLabel` でペア化 |

---

## 参照業界分析

### 必須

[`90_業界分析/17_サプライチェーンと作業依存性.md`](../../../../90_業界分析/17_サプライチェーンと作業依存性.md)

[`90_業界分析/09_セキュリティとアクセス制御.md`](../../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 参考

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)
