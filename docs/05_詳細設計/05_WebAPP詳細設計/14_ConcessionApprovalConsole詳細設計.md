# 14 ConcessionApprovalConsole 詳細設計

本章は MOD-FE-MC-006 ConcessionApprovalConsole の責務・型定義・コンポーネント・バリデーション仕様を確定する。FR-IQ-010（特採承認フロー）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-006 |
| 物理名 | ConcessionApprovalConsole |
| ファイルパス | `src/features/concession-approval/` |
| 関連 FR | FR-IQ-010 |
| 関連 SCR | SCR-MC-010 |
| アクセスロール | quality_admin（承認操作）|

**責務境界:**
- 本モジュール: AQL 不合格ロットへの特採申請確認・承認・有効期限管理・電子サイン連携
- ElectronicSignPad（MOD-FE-MA-006）: 電子サインの実装詳細（本モジュールは signId を受け取るのみ）
- IqcDashboard（MOD-FE-MC-007）: 特採後のロット品質集計（本モジュールは承認操作のみ）

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export interface ConcessionRequest {
  inspectionId: string;
  lotId: string;
  materialName: string;
  supplierName: string;
  defectCount: number;
  rejectNumberRe: number;
  rejectedAt: string;        // ISO 8601
  requestedBy: string;       // 申請者 worker_id
}

export interface ConcessionApproval {
  reason: string;
  validityScope: {
    processes?: string[];    // 対象工程 ID リスト（空の場合は全工程）
    maxQuantity?: number;    // 使用可能数量上限（null の場合は上限なし）
  };
  validUntil: string | null; // ISO 8601 日付（null = 永続特採）
  electronicSignId: string;  // quality_admin の電子サイン ID
}

// フォーム状態
export interface ConcessionApprovalFormState {
  selectedInspectionId: string | null;
  formValues: Partial<ConcessionApproval>;
  isSubmitting: boolean;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. ConcessionRequestList

特採申請待ち（REJECTED 状態）のロット一覧を表示する。

```typescript
interface ConcessionRequestListProps {
  requests: ConcessionRequest[];
  selectedInspectionId: string | null;
  onSelect: (inspectionId: string) => void;
  isLoading: boolean;
}
```

**表示要件:** 不良数 / Re 数・申請日時・材料名・仕入先名を表示。不良率が高い順にデフォルトソート。

### 3-2. ConcessionApprovalForm

特採理由・有効範囲・有効期限・電子サインを入力する。`validUntil` は CFG-023 で設定された最大日数（デフォルト 90 日）を超えられない。

```typescript
interface ConcessionApprovalFormProps {
  request: ConcessionRequest;
  onApprove: (approval: ConcessionApproval) => void;
  onReject: () => void;
  maxValidityDays: number;   // CFG-023 の値をプロップとして受け取る
}
```

### 3-3. ConcessionValidityScopeSelector

特採の有効範囲（対象工程・使用可能数量）を設定するセレクタ。

```typescript
interface ConcessionValidityScopeSelectorProps {
  availableProcesses: Array<{ processId: string; processName: string }>;
  value: ConcessionApproval['validityScope'];
  onChange: (scope: ConcessionApproval['validityScope']) => void;
}
```

### 3-4. ConcessionHistoryDrawer

過去の特採承認履歴（同一材料×仕入先）を右ドロワーで表示する。

```typescript
interface ConcessionHistoryDrawerProps {
  materialId: string;
  supplierId: string;
  isOpen: boolean;
  onClose: () => void;
}
```

---

## 4. 主要アクション / フック

```typescript
// 特採申請待ちロット一覧取得
export function usePendingConcessionList() {
  return useQuery({
    queryKey: ['concession-requests'],
    queryFn: () =>
      apiClient.get('/api/v1/iqc/incoming-inspections?qcStatus=REJECTED&hasNoConcession=true'),
    refetchInterval: 30_000,  // 30 秒ごとに自動更新
  });
}

// 特採承認
export function useApproveConcession(inspectionId: string) {
  return useMutation({
    mutationFn: (approval: ConcessionApproval) =>
      apiClient.post(`/api/v1/iqc/incoming-inspections/${inspectionId}/concession`, approval),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['concession-requests'] }),
  });
}

// CFG-023（最大特採有効日数）取得
export function useConcessionConfig() {
  return useQuery({
    queryKey: ['config', 'concession'],
    queryFn: () =>
      apiClient.get('/api/v1/config/concession'),
    staleTime: Infinity,  // 設定変更時のみ再取得
  });
}
```

---

## 5. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| reason | 必須・20 文字以上 | ERR-VAL-052 |
| validUntil | 今日から CFG-023 日以内（null は永続特採として許可） | ERR-VAL-053 |
| electronicSignId | quality_admin ロールの電子サイン必須 | ERR-VAL-054 |
| 重複特採 | 同一 inspectionId への 2 回目の特採 POST は拒否 | ERR-BIZ-033 |

---

## 6. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| ERR-VAL-053（有効期限超過） | validUntil フィールドに「CFG-023 日以内の日付を入力してください（最大 {N} 日）」を表示 |
| ERR-BIZ-033（重複特採） | 「この検査ロットはすでに特採承認済みです」ダイアログ、既存の特採情報を表示 |
| 電子サイン失敗 | ElectronicSignPad のエラーをトースト通知で表示 |
| ネットワークエラー | 承認操作は冪等性保証のため Idempotency-Key を付与してリトライ可能 |

---

## 7. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| 申請待ちロット一覧取得 | P95 ≤ 500ms |
| 特採承認 POST | P95 ≤ 800ms |
| 特採履歴ドロワー表示 | P95 ≤ 600ms |

---

## 8. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| ConcessionRequestList | 選択中の行に `aria-selected="true"` を付与 |
| ConcessionApprovalForm | 承認ボタンに `aria-describedby` で特採理由フィールドを関連付け |
| ConcessionHistoryDrawer | `role="dialog"` と `aria-labelledby` を設定、Escape キーで閉じる |

---

## 9. 参照業界分析

### 必須

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)

### 参考

[`90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)
