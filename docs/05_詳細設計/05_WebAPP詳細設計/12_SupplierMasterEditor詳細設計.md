# 12 SupplierMasterEditor 詳細設計

本章は MOD-FE-MA-008 SupplierMasterEditor の責務・型定義・コンポーネント・バリデーション仕様を確定する。FR-MA-018（仕入先マスタ管理）・FR-IQ-014（仕入先品質実績リンク）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-008 |
| 物理名 | SupplierMasterEditor |
| ファイルパス | `src/features/supplier-master/` |
| 関連 FR | FR-MA-018, FR-IQ-014 |
| 関連 SCR | SCR-MA-013 |
| アクセスロール | master_admin（CRUD）/ quality_admin（閲覧）|

**責務境界:**
- 本モジュール: 仕入先マスタの CRUD・バージョン管理・仕入先コード体系の管理
- IqcDashboard（MOD-FE-MC-007）: 仕入先品質実績の集計・表示（本モジュールは仕入先 ID を提供するのみ）
- SamplingPlanEditor（MOD-FE-MA-009）: 仕入先×材料のサンプリング計画管理（本モジュールは仕入先 ID を提供するのみ）

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export interface Supplier {
  supplierId: string;      // UUID v7
  supplierCode: string;    // ユニーク・64 文字以内
  name: string;            // 256 文字以内
  address: string;
  contact: string;         // 担当者名または電話番号
  version: number;         // 楽観的ロック用
  isActive: boolean;
  createdAt: string;       // ISO 8601
  updatedAt: string;
}

// フォーム編集用
export interface SupplierFormValues {
  supplierCode: string;
  name: string;
  address: string;
  contact: string;
}

// 一覧フィルタ
export interface SupplierListFilter {
  isActive?: boolean;
  searchText?: string;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. SupplierMasterTable

仕入先マスタ一覧を表示するテーブル。フィルタ・ソート・ページネーション付き。

```typescript
interface SupplierMasterTableProps {
  filter: SupplierListFilter;
  onFilterChange: (filter: SupplierListFilter) => void;
  onEdit: (supplier: Supplier) => void;
  onDeactivate: (supplierId: string) => void;
  onViewQualityHistory: (supplierId: string) => void;
}
```

### 3-2. SupplierMasterForm

仕入先マスタの新規作成・編集フォーム。

```typescript
interface SupplierMasterFormProps {
  supplier?: Supplier;   // 省略時は新規作成モード
  onSave: (values: SupplierFormValues) => void;
  onCancel: () => void;
}
```

### 3-3. SupplierQualityHistoryDrawer

仕入先の品質実績サマリ（月次不良率推移）を右ドロワーで表示する。IqcDashboard の SupplierQualitySummary を参照（閲覧専用）。

```typescript
interface SupplierQualityHistoryDrawerProps {
  supplierId: string;
  supplierName: string;
  isOpen: boolean;
  onClose: () => void;
}
```

---

## 4. 主要アクション / フック

```typescript
// 仕入先マスタ一覧取得
export function useSupplierList(filter: SupplierListFilter) {
  return useQuery({
    queryKey: ['suppliers', filter],
    queryFn: () =>
      apiClient.get('/api/v1/suppliers', { params: filter }),
    staleTime: 60_000,
  });
}

// 仕入先マスタ作成
export function useCreateSupplier() {
  return useMutation({
    mutationFn: (values: SupplierFormValues) =>
      apiClient.post('/api/v1/suppliers', values),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['suppliers'] }),
  });
}

// 仕入先マスタ更新
export function useUpdateSupplier(supplierId: string) {
  return useMutation({
    mutationFn: (payload: SupplierFormValues & { version: number }) =>
      apiClient.put(`/api/v1/suppliers/${supplierId}`, payload),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['suppliers'] }),
  });
}

// 仕入先マスタ無効化（物理削除不可）
export function useDeactivateSupplier(supplierId: string) {
  return useMutation({
    mutationFn: () =>
      apiClient.patch(`/api/v1/suppliers/${supplierId}/deactivate`),
  });
}
```

---

## 5. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| supplierCode | 必須・ユニーク・64 文字以内 | ERR-VAL-044 |
| name | 必須・256 文字以内 | ERR-VAL-045 |
| address | 任意・500 文字以内 | ERR-VAL-046 |
| contact | 任意・256 文字以内 | ERR-VAL-047 |
| バージョン衝突 | 更新時 version が DB 値と一致しない場合（楽観的ロック） | ERR-BIZ-031 |

---

## 6. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| supplierCode 重複（409） | フォームの supplierCode フィールドにインラインエラー「このコードはすでに使用されています」を表示 |
| 楽観的ロック衝突（409） | 「他のユーザーが更新しました。最新データを再取得してください」ダイアログ |
| 無効化不可（サンプリング計画参照中 409） | 「この仕入先はサンプリング計画で使用中のため無効化できません」ダイアログ |
| ネットワークエラー | トースト通知＋リトライボタン |

---

## 7. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| 仕入先一覧取得（100 件） | P95 ≤ 500ms |
| 仕入先作成 / 更新 | P95 ≤ 500ms |
| 品質履歴ドロワー表示 | P95 ≤ 800ms |

---

## 8. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| SupplierMasterTable | テーブルに `role="grid"`、各行に `aria-rowindex` を付与 |
| SupplierMasterForm | 全入力フィールドに `htmlFor` 対応の `<label>` を関連付け |
| SupplierQualityHistoryDrawer | `role="dialog"` と `aria-labelledby` を設定、Escape キーで閉じる |

---

## 9. 参照業界分析

### 必須

[`90_業界分析/17_サプライチェーンと作業依存性.md`](../../../../90_業界分析/17_サプライチェーンと作業依存性.md)

### 参考

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)
