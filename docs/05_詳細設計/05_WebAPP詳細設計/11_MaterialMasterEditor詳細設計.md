# 11 MaterialMasterEditor 詳細設計

本章は MOD-FE-MA-007 MaterialMasterEditor の責務・型定義・コンポーネント・バリデーション仕様を確定する。FR-MA-017（材料マスタ管理）・FR-IQ-001（材料コード管理）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-007 |
| 物理名 | MaterialMasterEditor |
| ファイルパス | `src/features/material-master/` |
| 関連 FR | FR-MA-017, FR-IQ-001 |
| 関連 SCR | SCR-MA-012 |
| アクセスロール | master_admin（CRUD）/ quality_admin（閲覧）|

**責務境界:**
- 本モジュール: 材料マスタの CRUD・バージョン管理・材料種別コード体系の管理
- SamplingPlanEditor（MOD-FE-MA-009）: 材料×仕入先のサンプリング計画管理（本モジュールは材料 ID を提供するのみ）

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export type MaterialType = 'RAW_MATERIAL' | 'COMPONENT' | 'TOOL' | 'PACKAGING';

export interface Material {
  materialId: string;           // UUID v7
  materialCode: string;         // ユニーク・英数字とハイフン
  name: string;                 // 256 文字以内
  materialType: MaterialType;
  description: string;
  version: number;              // 楽観的ロック用
  isActive: boolean;
  createdAt: string;            // ISO 8601
  updatedAt: string;
}

// フォーム編集用（materialId / version / isActive は送信不要）
export interface MaterialFormValues {
  materialCode: string;
  name: string;
  materialType: MaterialType;
  description: string;
}

// 一覧フィルタ
export interface MaterialListFilter {
  materialType?: MaterialType;
  isActive?: boolean;
  searchText?: string;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. MaterialMasterTable

材料マスタ一覧を表示するテーブル。フィルタ・ソート・ページネーション付き。

```typescript
interface MaterialMasterTableProps {
  filter: MaterialListFilter;
  onFilterChange: (filter: MaterialListFilter) => void;
  onEdit: (material: Material) => void;
  onDeactivate: (materialId: string) => void;
}
```

### 3-2. MaterialMasterForm

材料マスタの新規作成・編集フォーム。

```typescript
interface MaterialMasterFormProps {
  material?: Material;   // 省略時は新規作成モード
  onSave: (values: MaterialFormValues) => void;
  onCancel: () => void;
}
```

### 3-3. MaterialTypeChip

材料種別（RAW_MATERIAL 等）をカラーチップで表示するプレゼンテーションコンポーネント。

```typescript
interface MaterialTypeChipProps {
  materialType: MaterialType;
  size?: 'sm' | 'md';
}
```

---

## 4. 主要アクション / フック

```typescript
// 材料マスタ一覧取得
export function useMaterialList(filter: MaterialListFilter) {
  return useQuery({
    queryKey: ['materials', filter],
    queryFn: () =>
      apiClient.get('/api/v1/materials', { params: filter }),
  });
}

// 材料マスタ作成
export function useCreateMaterial() {
  return useMutation({
    mutationFn: (values: MaterialFormValues) =>
      apiClient.post('/api/v1/materials', values),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['materials'] }),
  });
}

// 材料マスタ更新
export function useUpdateMaterial(materialId: string) {
  return useMutation({
    mutationFn: (payload: MaterialFormValues & { version: number }) =>
      apiClient.put(`/api/v1/materials/${materialId}`, payload),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['materials'] }),
  });
}

// 材料マスタ無効化（物理削除不可）
export function useDeactivateMaterial(materialId: string) {
  return useMutation({
    mutationFn: () =>
      apiClient.patch(`/api/v1/materials/${materialId}/deactivate`),
  });
}
```

---

## 5. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| materialCode | 必須・ユニーク・英数字とハイフンのみ（正規表現: `^[A-Za-z0-9-]+$`）・64 文字以内 | ERR-VAL-040 |
| name | 必須・256 文字以内 | ERR-VAL-041 |
| materialType | RAW_MATERIAL / COMPONENT / TOOL / PACKAGING の 4 種 ENUM から選択必須 | ERR-VAL-042 |
| description | 任意・1000 文字以内 | ERR-VAL-043 |
| バージョン衝突 | 更新時 version が DB 値と一致しない場合（楽観的ロック） | ERR-BIZ-030 |

---

## 6. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| materialCode 重複（409） | フォームの materialCode フィールドにインラインエラー「このコードはすでに使用されています」を表示 |
| 楽観的ロック衝突（409）| 「他のユーザーが更新しました。最新データを再取得してください」ダイアログ |
| 無効化不可（参照中 409） | 「この材料はサンプリング計画で使用中のため無効化できません」ダイアログ |
| ネットワークエラー | トースト通知＋リトライボタン |

---

## 7. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| 材料一覧取得（100 件） | P95 ≤ 500ms |
| 材料作成 / 更新 | P95 ≤ 500ms |
| フィルタ変更 → 再表示 | P95 ≤ 300ms（React Query キャッシュ活用） |

一覧は react-query の `staleTime: 60_000` でキャッシュし、頻繁な再フェッチを抑制する。

---

## 8. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| MaterialMasterTable | テーブルに `role="grid"`、各行に `aria-rowindex` を付与 |
| MaterialMasterForm | 全入力フィールドに `htmlFor` 対応の `<label>` を関連付け |
| MaterialTypeChip | 色のみに依存せず、テキストラベルを常時表示 |

---

## 9. 参照業界分析

### 必須

[`90_業界分析/17_サプライチェーンと作業依存性.md`](../../../../90_業界分析/17_サプライチェーンと作業依存性.md)

### 参考

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)
