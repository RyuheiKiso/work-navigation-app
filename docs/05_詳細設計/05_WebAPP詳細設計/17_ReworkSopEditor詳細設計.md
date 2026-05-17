# 17 ReworkSopEditor 詳細設計

本章は MOD-FE-MA-010 ReworkSopEditor の責務・型定義・コンポーネント・バリデーション仕様を確定する。sop_type=REWORK の SOP 管理と不適合カテゴリ×リワーク種別×SOP のマッピング管理を担当する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-010 |
| 物理名 | ReworkSopEditor |
| ファイルパス | `src/features/rework-sop-editor/` |
| 関連 FR | FR-MA-017, FR-ST-014 |
| 関連 SCR | SCR-MA-015, SCR-MA-016 |
| アクセスロール | master_admin（CRUD）/ quality_admin（閲覧・マッピング承認）|

**責務境界:**
- 本モジュール: sop_type=REWORK の SOP 新規作成・編集・バージョン管理、不適合カテゴリ×リワーク種別×SOP のマッピング管理
- SopEditor（MOD-FE-MA-001）: SOP のステップ単体編集（本モジュールはリワーク SOP の CRUD のみ、ステップ編集は SopEditor に委譲）
- ReworkFlow（MOD-FE-HA-010）: リワーク SOP の参照・実行（本モジュールは管理のみ）

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export type ReworkType = 'TOUCH_UP' | 'REWORK_FULL' | 'SORTING' | 'SCRAP' | 'RETURN';

// リワーク SOP（sop_type=REWORK の SOP）
export interface ReworkSop {
  sopId: string;
  sopTitle: string;
  sopType: 'REWORK';
  version: number;
  isActive: boolean;
  stepCount: number;
  createdAt: string;
  updatedAt: string;
}

// 不適合カテゴリ×リワーク種別×SOP マッピング
export interface ReworkSopMapping {
  mappingId: string;
  nonconformityCategory: string;
  sourceSopId: string | null;      // 起点 SOP（null = 全 SOP に適用）
  sourceStepId: string | null;     // 起点ステップ（null = SOP 全体から）
  targetReworkSopId: string;
  reworkType: ReworkType;
  version: number;
  isActive: boolean;
  approvedBy: string | null;
  approvedAt: string | null;
}

// マッピングフォーム入力値
export interface ReworkSopMappingFormValues {
  nonconformityCategory: string;
  sourceSopId: string | null;
  sourceStepId: string | null;
  targetReworkSopId: string;
  reworkType: ReworkType;
}

// フィルタ
export interface ReworkSopMappingFilter {
  nonconformityCategory?: string;
  reworkType?: ReworkType;
  isActive?: boolean;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. ReworkSopList

sop_type=REWORK の SOP 一覧を表示。バージョン・有効/無効でフィルタ可能。

```typescript
interface ReworkSopListProps {
  sops: ReworkSop[];
  onEdit: (sopId: string) => void;
  onDeactivate: (sopId: string) => void;
}
```

### 3-2. ReworkSopMappingTable

不適合カテゴリ × リワーク種別 × 対象 SOP のマッピング一覧を表示・編集する。

```typescript
interface ReworkSopMappingTableProps {
  mappings: ReworkSopMapping[];
  filter: ReworkSopMappingFilter;
  onFilterChange: (filter: ReworkSopMappingFilter) => void;
  onEdit: (mapping: ReworkSopMapping) => void;
  onDeactivate: (mappingId: string) => void;
}
```

### 3-3. ReworkSopMappingForm

新規マッピングの作成・編集フォーム。

```typescript
interface ReworkSopMappingFormProps {
  mapping?: ReworkSopMapping;               // 省略時は新規作成モード
  availableCategories: string[];            // 不適合カテゴリリスト
  availableReworkSops: ReworkSop[];         // sop_type=REWORK の SOP リスト
  onSave: (values: ReworkSopMappingFormValues) => void;
  onCancel: () => void;
}
```

### 3-4. NonconformityCategorySelector

不適合カテゴリを選択するセレクタ。カテゴリ階層（大分類/小分類）をサポート。

```typescript
interface NonconformityCategorySelectorProps {
  value: string;
  onChange: (category: string) => void;
  placeholder?: string;
}
```

---

## 4. 主要アクション / フック

```typescript
// リワーク SOP 一覧取得
export function useReworkSopList() {
  return useQuery({
    queryKey: ['rework-sops'],
    queryFn: () =>
      apiClient.get('/api/v1/sops?sopType=REWORK'),
    staleTime: 300_000,
  });
}

// リワーク SOP マッピング一覧取得
export function useReworkSopMappingList(filter: ReworkSopMappingFilter) {
  return useQuery({
    queryKey: ['rework-sop-mappings', filter],
    queryFn: () =>
      apiClient.get('/api/v1/rework-sop-mappings', { params: filter }),
  });
}

// マッピング作成
export function useCreateReworkSopMapping() {
  return useMutation({
    mutationFn: (values: ReworkSopMappingFormValues) =>
      apiClient.post('/api/v1/rework-sop-mappings', values),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['rework-sop-mappings'] }),
  });
}

// マッピング更新
export function useUpdateReworkSopMapping(mappingId: string) {
  return useMutation({
    mutationFn: (payload: ReworkSopMappingFormValues & { version: number }) =>
      apiClient.put(`/api/v1/rework-sop-mappings/${mappingId}`, payload),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['rework-sop-mappings'] }),
  });
}

// マッピング無効化
export function useDeactivateReworkSopMapping(mappingId: string) {
  return useMutation({
    mutationFn: () =>
      apiClient.patch(`/api/v1/rework-sop-mappings/${mappingId}/deactivate`),
  });
}
```

---

## 5. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| targetReworkSopId | sop_type=REWORK の SOP のみ選択可（通常 SOP は選択不可） | ERR-VAL-062 |
| nonconformityCategory | 必須・既存カテゴリリストから選択 | ERR-VAL-063 |
| reworkType | 5 種の ReworkType から選択必須 | ERR-VAL-064 |
| 重複マッピング | 同一 nonconformityCategory × reworkType の有効マッピングが既存の場合は警告（旧マッピングを自動無効化して置き換え） | ERR-BIZ-035 |
| バージョン衝突 | 更新時 version が DB 値と一致しない場合（楽観的ロック） | ERR-BIZ-036 |

---

## 6. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| ERR-VAL-062（非リワーク SOP 選択） | セレクタのフィルタで sop_type=REWORK 以外の SOP を非表示にすることで UI レベルで防止 |
| ERR-BIZ-035（重複マッピング） | 「既存のマッピングを無効化して新しいマッピングを作成しますか?」確認ダイアログ |
| ERR-BIZ-036（楽観的ロック衝突） | 「他のユーザーが更新しました。最新データを再取得してください」ダイアログ |
| マッピング無効化不可（リワーク実施中 409） | 「このマッピングは現在実施中のリワークで使用中です」ダイアログ |

---

## 7. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| リワーク SOP 一覧取得 | P95 ≤ 500ms |
| マッピング一覧取得（50 件） | P95 ≤ 500ms |
| マッピング保存 | P95 ≤ 500ms |

---

## 8. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| ReworkSopMappingTable | テーブルに `role="grid"`、各行に `aria-rowindex` を付与 |
| ReworkSopMappingForm | 全選択フィールドに `htmlFor` 対応の `<label>` を関連付け |
| NonconformityCategorySelector | セレクタに `aria-label`「不適合カテゴリを選択」を付与 |

---

## 9. 参照業界分析

### 必須

[`90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)

### 参考

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)
