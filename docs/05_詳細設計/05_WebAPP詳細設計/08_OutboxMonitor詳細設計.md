# 08 OutboxMonitor 詳細設計

本章は MOD-FE-MC-004（OutboxMonitor）の TypeScript インターフェース・DLQ エントリ型定義・react-query フック・CMP-MC-002 コンポーネント Props・手動再投入フロー仕様を確定する。OutboxMonitor は FR-SY-007/008 で要求される DLQ 監視と手動再投入 UI を担い、SCR-MC-007（Outbox/DLQ 監視）の UI ロジックを提供する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-004 |
| 物理名 | OutboxMonitor |
| ファイルパス | `src/features/outbox-mon/` |
| 関連 FR | FR-SY-007（DLQ 監視）・FR-SY-008（手動再投入）|
| 関連 SCR | SCR-MC-007（Outbox/DLQ 監視）|
| アクセスロール | system_admin |

---

## 2. 型定義

```typescript
// DLQ エントリ（TBL-003 outbox_events status='DEAD' のエントリ）
export interface DlqEntry {
  /** outbox_events.id（UUID v7）*/
  eventId: string;
  /** イベント種別（例: 'WorkStepCompleted'）*/
  eventType: string;
  /** 最初に作成されたタイムスタンプ */
  createdAt: Date;
  /** 再試行済み回数 */
  retryCount: number;
  /** 最後の失敗エラーメッセージ（先頭 500 文字）*/
  lastError: string;
  /** イベントペイロード（JSON）*/
  payload: Record<string, unknown>;
}

// Outbox 全体ステータス
export interface OutboxStatus {
  pendingCount: number;
  processingCount: number;
  deadCount: number;
  /** 最古の PENDING エントリの createdAt（null = PENDING なし）*/
  oldestPendingAt: Date | null;
  measuredAt: Date;
}

// コンポーネント Props
export interface OutboxMonitorProps {
  dlqEntries: DlqEntry[];
  /** 単一 DLQ エントリの手動再投入 */
  onRetry: (eventId: string) => Promise<void>;
  /** DLQ 全エントリの一括再投入 */
  onRetryAll: () => Promise<void>;
  /** DLQ エントリの破棄（再試行しない）*/
  onDismiss: (eventId: string) => Promise<void>;
}
```

---

## 3. react-query フック定義（FNC-FE-015）

```typescript
import { useQuery, useMutation, UseQueryResult, UseMutationResult } from '@tanstack/react-query';

/**
 * FNC-FE-015: DLQ エントリと Outbox ステータスを定期ポーリングするフック
 *
 * staleTime: 10 s（DLQ は変化が低頻度）
 * refetchInterval: 30 s
 */
export declare function useDlqEntries(): UseQueryResult<
  { entries: DlqEntry[]; status: OutboxStatus },
  Error
>;

/**
 * 単一 DLQ エントリ再投入 Mutation
 * 成功後に useDlqEntries のクエリキャッシュを無効化する
 */
export declare function useOutboxRetry(): {
  retry: UseMutationResult<void, Error, { eventId: string }>;
  retryAll: UseMutationResult<void, Error, void>;
  dismiss: UseMutationResult<void, Error, { eventId: string }>;
};
```

---

## 4. CMP-MC-002 DlqMonitorTable コンポーネント仕様

```typescript
// CMP-MC-002: DLQ エントリ一覧テーブル
export interface DlqMonitorTableProps {
  entries: DlqEntry[];
  /** 再投入ハンドラ（useOutboxRetry.retry を渡す）*/
  onRetry: (eventId: string) => Promise<void>;
  /** 破棄ハンドラ（useOutboxRetry.dismiss を渡す）*/
  onDismiss: (eventId: string) => Promise<void>;
  /** 一括再投入中は true（RetryAll ボタン用）*/
  isRetryAllPending: boolean;
}
```

---

## 5. コンポーネントツリー

```
OutboxMonitor (MOD-FE-MC-004)
  OutboxStatusPanel（全体ステータス: PENDING/PROCESSING/DEAD 件数・最古 PENDING 時刻）
  RetryAllButton（onRetryAll・dlqEntries.length > 0 時のみ有効・確認ダイアログ付き）
  DlqMonitorTable (CMP-MC-002)
    DlqTableHeader（列: イベント ID / 種別 / 作成日時 / 再試行回数 / 最終エラー / 操作）
    DlqTableRow (×N)
      EventIdCell（先頭 8 文字 + クリップボードコピー）
      EventTypeBadge
      CreatedAtCell（ISO 8601）
      RetryCountCell（3 回以上は赤色）
      LastErrorCell（先頭 80 文字・クリックで全文モーダル）
      ActionCell
        RetryButton（個別再投入・確認なし即時実行）
        DismissButton（個別破棄・確認ダイアログ付き）
  EmptyState（dlqEntries.length === 0 時: 「DLQ エントリはありません」）
```

---

## 6. 手動再投入フロー仕様

### 6-1. 単一再投入（onRetry）

```
1. system_admin が RetryButton をクリックする
2. useOutboxRetry.retry.mutate({ eventId }) を呼び出す
3. バックエンドが status を 'DEAD' → 'PENDING' に変更する（TBL-003 更新）
4. Mutation 成功後に useDlqEntries のキャッシュを無効化する
5. テーブルから当該エントリが消える（または status 変化で非表示）
```

### 6-2. 一括再投入（onRetryAll）

```
1. system_admin が RetryAllButton をクリックする
2. 「全 N 件を再投入しますか？」確認ダイアログを表示する
3. 確認後に useOutboxRetry.retryAll.mutate() を呼び出す
4. バックエンドが全 DLQ エントリの status を 'PENDING' に変更する
5. Mutation 成功後に useDlqEntries のキャッシュを無効化する
```

### 6-3. 破棄（onDismiss）

```
1. system_admin が DismissButton をクリックする
2. 「このイベントを破棄しますか？この操作は元に戻せません。」確認ダイアログを表示する
3. 確認後に useOutboxRetry.dismiss.mutate({ eventId }) を呼び出す
4. バックエンドが status を 'DISMISSED' に変更する
5. Mutation 成功後に useDlqEntries のキャッシュを無効化する
```

---

## 7. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-BIZ-020 | 再投入対象イベントが DLQ に存在しない（並行削除）| 「イベントが見つかりません」トースト・テーブル再取得 |
| ERR-SYS-001 | API タイムアウト | エラートースト・ボタンを再活性化（状態ロールバック）|
| ERR-AUTH-003 | RBAC 不足（system_admin 以外）| RetryButton・DismissButton・RetryAllButton 非表示 |

---

**本節で確定した方針**
- **DlqMonitorTable（CMP-MC-002）を Props 駆動の純粋コンポーネントとして定義し、Mutation ロジックはすべて親の OutboxMonitor が保有する useOutboxRetry フックに委譲することを確定した。**
- **破棄操作（onDismiss）には「この操作は元に戻せません」という不可逆性を明示する確認ダイアログを必置とし、誤操作によるイベント消失を防止することを確定した。**
- **DLQ ポーリング間隔は 30 s とし、OutboxStatusPanel の pendingCount・deadCount が系統的に 0 に収束していることを system_admin が視認できる更新頻度を確保することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
