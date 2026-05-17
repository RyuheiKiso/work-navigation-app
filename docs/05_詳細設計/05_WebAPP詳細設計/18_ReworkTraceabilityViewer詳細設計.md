# 18 ReworkTraceabilityViewer 詳細設計

本章は MOD-FE-MC-009 ReworkTraceabilityViewer の責務・型定義・コンポーネント・バリデーション仕様を確定する。FR-KZ-009（リワーク進捗管理）・FR-KZ-010（リワーク一覧）を充足する。ALCOA+ Original 原則（NFR-DQ-010）に基づく parent_case_id ↔ rework_case_id の双方向トレサビを提供する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-009 |
| 物理名 | ReworkTraceabilityViewer |
| ファイルパス | `src/features/rework-trace/` |
| 関連 FR | FR-KZ-009, FR-KZ-010 |
| 関連 SCR | SCR-MC-014, SCR-MC-015 |
| アクセスロール | quality_admin（フル閲覧）|

**責務境界:**
- 本モジュール: リワーク進捗の一覧・詳細・ALCOA+ Original 確認・フィルタ・エクスポート（閲覧専用）
- ReworkFlow（MOD-FE-HA-010）: リワーク作業の実施（本モジュールは閲覧のみ）
- DispositionApprovalConsole（MOD-FE-MC-008）: ディスポジション判定操作（本モジュールは閲覧のみ）

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export interface ReworkTraceabilityRecord {
  reworkId: string;
  status: 'PENDING_DISPOSITION' | 'REWORK_IN_PROGRESS' | 'REWORK_COMPLETED' | 'VERIFICATION_IN_PROGRESS' | 'CLOSED_OK_RELEASE' | 'CLOSED_SCRAP' | 'CLOSED_RETURN';
  reworkType: 'TOUCH_UP' | 'REWORK_FULL' | 'SORTING' | 'SCRAP' | 'RETURN';
  parentCaseId: string;             // 元 WorkExecution（ALCOA+ Original: 不変参照）
  reworkCaseId: string | null;      // リワーク WorkExecution
  parentCaseEventCount: number;     // 元 WorkExecution のイベント数（ALCOA+ Original 確認用）
  reworkCaseEventCount: number;     // リワーク WorkExecution のイベント数
  parentCaseEventCountAtReworkStart: number;  // リワーク開始時点の親イベント数（変更検知用）
  dispositionDecision: 'REWORK' | 'SCRAP' | 'RETURN' | 'USE_AS_IS' | null;
  nonconformityCategory: string;
  materialName: string;
  supplierName: string;
  reworkWorkerName: string | null;
  createdAt: string;                // ISO 8601
  dueDate: string | null;           // ISO 8601 日付
  closedAt: string | null;
}

// ALCOA+ Original 確認結果
export interface AlcoaOriginalVerification {
  parentCaseId: string;
  isIntact: boolean;                // イベント数が変化していない場合 true
  eventCountAtReworkStart: number;
  eventCountCurrent: number;
  deltaCount: number;               // = eventCountCurrent - eventCountAtReworkStart（0 が正常）
}

// 一覧フィルタ
export interface ReworkTraceabilityFilter {
  status?: ReworkTraceabilityRecord['status'];
  reworkType?: ReworkTraceabilityRecord['reworkType'];
  fromDate?: string;
  toDate?: string;
  materialId?: string;
  supplierId?: string;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. ReworkProgressTable

リワーク進捗一覧（status フィルタ・担当者・期限でソート可）を表示。

```typescript
interface ReworkProgressTableProps {
  records: ReworkTraceabilityRecord[];
  filter: ReworkTraceabilityFilter;
  onFilterChange: (filter: ReworkTraceabilityFilter) => void;
  onRowClick: (reworkId: string) => void;
  onExportCsv: () => void;
}
```

**カラーコード（期限）:**
- 緑: dueDate が 3 日以上先、または null
- 黄: dueDate が 1〜2 日以内
- 赤: dueDate が過去（期限超過）

### 3-2. ReworkAlcoaOriginalPanel

parent_case_id と rework_case_id の双方向リンクと各々のイベント数を表示。

```typescript
interface ReworkAlcoaOriginalPanelProps {
  verification: AlcoaOriginalVerification;
  parentCaseId: string;
  reworkCaseId: string | null;
  parentCaseEventCount: number;
  reworkCaseEventCount: number;
}
```

**表示要件:**
- 「元作業記録: {parentCaseId の先頭 8 桁}（{parentCaseEventCount} イベント）」
- 「リワーク記録: {reworkCaseId の先頭 8 桁}（{reworkCaseEventCount} イベント）」を並列表示
- `isIntact === true` の場合:「✓ ALCOA+ Original 確認済み」バッジを緑色で表示
- `isIntact === false` の場合:「! 元作業記録に変更が検出されました（+{deltaCount} イベント）」を赤バナーで表示

### 3-3. ReworkDetailDrawer

リワーク詳細（ディスポジション判定・前後写真・電子サイン履歴）を右ドロワーで表示。

```typescript
interface ReworkDetailDrawerProps {
  reworkId: string;
  isOpen: boolean;
  onClose: () => void;
}
```

### 3-4. ReworkStatusStepper

リワークフローの現在ステータスをステッパーで表示する。

```typescript
interface ReworkStatusStepperProps {
  currentStatus: ReworkTraceabilityRecord['status'];
  reworkType: ReworkTraceabilityRecord['reworkType'];
}
```

---

## 4. 主要アクション / フック

```typescript
// リワーク一覧取得
export function useReworkTraceabilityList(filter: ReworkTraceabilityFilter) {
  return useQuery({
    queryKey: ['rework-trace', filter],
    queryFn: () =>
      apiClient.get('/api/v1/rework', { params: filter }),
    staleTime: 60_000,
    refetchInterval: 60_000,  // 1 分ごとに自動更新
  });
}

// リワーク詳細取得
export function useReworkDetail(reworkId: string | null) {
  return useQuery({
    queryKey: ['rework-trace', reworkId],
    queryFn: () =>
      apiClient.get(`/api/v1/rework/${reworkId}`),
    enabled: reworkId !== null,
  });
}

// ALCOA+ Original 確認
export function useAlcoaOriginalVerification(reworkId: string | null) {
  return useQuery({
    queryKey: ['alcoa-original', reworkId],
    queryFn: () =>
      apiClient.get(`/api/v1/rework/${reworkId}/alcoa-original-verification`),
    enabled: reworkId !== null,
    staleTime: 300_000,  // 5 分キャッシュ（確認結果は頻繁に変わらない）
  });
}

// CSV エクスポート
export function useExportReworkTrace() {
  return useMutation({
    mutationFn: (filter: ReworkTraceabilityFilter) =>
      apiClient.get('/api/v1/rework/export', {
        params: { ...filter, format: 'csv' },
        responseType: 'blob',
      }),
    onSuccess: (blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `rework-trace-${new Date().toISOString().slice(0, 10)}.csv`;
      a.click();
    },
  });
}
```

---

## 5. ALCOA+ Original 表示仕様

ReworkTraceabilityViewer が提供する ALCOA+ Original 確認は以下の 3 層で構成される:

| 層 | 内容 |
|---|---|
| 一覧レイヤ | ReworkProgressTable の各行に「✓ / !」アイコンで ALCOA+ Original 状態を表示 |
| 詳細レイヤ | ReworkDetailDrawer 内の ReworkAlcoaOriginalPanel で詳細情報（イベント数・差分）を表示 |
| API レイヤ | `GET /api/v1/rework/{reworkId}/alcoa-original-verification` がリワーク開始時のスナップショットと現在値を比較して `isIntact` を返す |

---

## 6. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| fromDate | 任意・toDate 以前 | ERR-VAL-065 |
| toDate | 任意・fromDate 以降・本日以前 | ERR-VAL-066 |
| 期間上限 | fromDate〜toDate は 365 日以内（大量データ防止） | ERR-VAL-067 |

---

## 7. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| ERR-VAL-067（期間超過） | フィルタフォームに「期間は 365 日以内で指定してください」を表示 |
| ALCOA+ Original 異常（isIntact=false） | ReworkAlcoaOriginalPanel に赤バナーで「元作業記録に変更が検出されました」を表示、品質管理者への確認を促す |
| reworkId 未存在（404） | 「指定されたリワーク記録が見つかりません」をドロワーに表示 |
| エクスポート失敗 | トースト通知「エクスポートに失敗しました。再試行してください」 |
| ネットワークエラー | テーブルにスケルトンローダーを表示し、リトライボタンを提供 |

---

## 8. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| リワーク一覧取得（100 件） | P95 ≤ 800ms |
| リワーク詳細取得 | P95 ≤ 500ms |
| ALCOA+ Original 確認 | P95 ≤ 500ms |
| CSV エクスポート（1000 件） | P95 ≤ 3s（バックグラウンドで生成） |

ReworkProgressTable は 100 件以上の場合に仮想スクロール（@tanstack/react-virtual）を使用する。

---

## 9. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| ReworkAlcoaOriginalPanel | ALCOA+ 異常時は `role="alert"` で即時音声通知 |
| ReworkProgressTable | 期限カラーを色のみに依存せず、アイコン（●/▲/✗）とツールチップで補足 |
| ReworkDetailDrawer | `role="dialog"` と `aria-labelledby` を設定、Escape キーで閉じる |
| ReworkStatusStepper | 現在ステップに `aria-current="step"` を付与 |

---

## 参照業界分析

### 必須

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)

[`90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)

### 参考

[`90_業界分析/13_安全文化と安全管理システム.md`](../../../../90_業界分析/13_安全文化と安全管理システム.md)
