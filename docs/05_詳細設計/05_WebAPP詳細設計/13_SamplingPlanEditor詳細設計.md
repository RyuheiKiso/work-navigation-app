# 13 SamplingPlanEditor 詳細設計

本章は MOD-FE-MA-009 SamplingPlanEditor の責務・型定義・コンポーネント・バリデーション仕様を確定する。FR-IQ-003（サンプリング計画設定）・FR-IQ-004（AQL 値・検査水準設定）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-009 |
| 物理名 | SamplingPlanEditor |
| ファイルパス | `src/features/sampling-plan/` |
| 関連 FR | FR-IQ-003, FR-IQ-004 |
| 関連 SCR | SCR-MA-014 |
| アクセスロール | quality_admin（作成・承認）/ master_admin（閲覧）|

**責務境界:**
- 本モジュール: 材料×仕入先のサンプリング計画（AQL/検査水準/n/Ac/Re）の設定・バージョン管理
- IqcInspectionFlow（MOD-FE-HA-009）: サンプリング計画の参照・適用（本モジュールは計画 ID を提供するのみ）
- AQL 計算: サーバーサイド（`POST /api/v1/iqc/sampling-plans/preview`）に集中、フロントは結果表示のみ

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export type InspectionLevel = 'S-1' | 'S-2' | 'S-3' | 'S-4' | 'I' | 'II' | 'III';

// JIS Z 9015-1 準拠の AQL 規格値リスト
export const AQL_VALUES = [0.065, 0.1, 0.15, 0.25, 0.4, 0.65, 1.0, 1.5, 2.5, 4.0, 6.5] as const;
export type AqlValue = typeof AQL_VALUES[number];

// サンプルサイズ文字（JIS Z 9015-1 表 1 に対応）
export type SampleSizeLetter = 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'J' | 'K' | 'L' | 'M' | 'N' | 'P' | 'Q' | 'R';

export interface SampleSizeLetterRow {
  lotSizeMin: number;
  lotSizeMax: number | null;  // null = 上限なし
  letter: SampleSizeLetter;
}

export interface AqlMasterRow {
  letter: SampleSizeLetter;
  sampleSize: number;
  aqlValue: AqlValue;
  acceptNumberAc: number;
  rejectNumberRe: number;
}

// JIS Z 9015-1 JSONB スナップショット
export interface AqlTableSnapshot {
  jisVersion: string;                    // 例: "JIS Z 9015-1:2006"
  snapshotDate: string;                  // ISO 8601 日付
  sampleSizeTable: SampleSizeLetterRow[];
  aqlMasterTable: AqlMasterRow[];
}

export interface SamplingPlan {
  planId: string;
  materialId: string;
  supplierId: string;
  aql: AqlValue;
  inspectionLevel: InspectionLevel;
  aqlTableSnapshot: AqlTableSnapshot;
  version: number;
  isActive: boolean;
  approvedBy: string | null;
  approvedAt: string | null;
}

// フォーム用（サーバー計算前の入力値）
export interface SamplingPlanFormValues {
  materialId: string;
  supplierId: string;
  aql: AqlValue;
  inspectionLevel: InspectionLevel;
}

// サーバーサイドプレビュー結果
export interface SamplingPlanPreview {
  letter: SampleSizeLetter;
  sampleSizeN: number;
  acceptNumberAc: number;
  rejectNumberRe: number;
  aqlTableSnapshot: AqlTableSnapshot;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. SamplingPlanTable

既存計画一覧表示。材料×仕入先×AQL 値×検査水準でフィルタ可能。

```typescript
interface SamplingPlanTableProps {
  plans: SamplingPlan[];
  onEdit: (plan: SamplingPlan) => void;
  onDeactivate: (planId: string) => void;
}
```

### 3-2. SamplingPlanForm

AQL 値・検査水準を入力し、サーバーから n/Ac/Re を取得して確認後に保存する。

```typescript
interface SamplingPlanFormProps {
  materialId: string;
  supplierId: string;
  existingPlan?: SamplingPlan;   // 省略時は新規作成モード
  onSave: (plan: SamplingPlan) => void;
  onCancel: () => void;
}
```

### 3-3. AqlPreviewCard

サーバー計算結果（n/Ac/Re/サンプルサイズ文字）をプレビュー表示する。保存前確認に使用。

```typescript
interface AqlPreviewCardProps {
  preview: SamplingPlanPreview | null;
  isLoading: boolean;
}
```

### 3-4. SeverityStateIndicator

現在の検査の厳しさ状態（なみ/きつい/ゆるい）と切替履歴を表示する。

```typescript
interface SeverityStateIndicatorProps {
  planId: string;
  currentSeverity: 'NORMAL' | 'TIGHTENED' | 'REDUCED';
}
```

---

## 4. 主要アクション / フック

```typescript
// サンプリング計画一覧取得
export function useSamplingPlanList(materialId?: string, supplierId?: string) {
  return useQuery({
    queryKey: ['sampling-plans', materialId, supplierId],
    queryFn: () =>
      apiClient.get('/api/v1/iqc/sampling-plans', { params: { materialId, supplierId } }),
    staleTime: 300_000,  // 5 分キャッシュ（計画は頻繁に変更しない）
  });
}

// AQL 自動計算プレビュー（入力変更のたびにデバウンス実行）
export function useSamplingPlanPreview() {
  return useMutation({
    mutationFn: (values: SamplingPlanFormValues) =>
      apiClient.post('/api/v1/iqc/sampling-plans/preview', values),
  });
}

// サンプリング計画保存
export function useCreateSamplingPlan() {
  return useMutation({
    mutationFn: (values: SamplingPlanFormValues) =>
      apiClient.post('/api/v1/iqc/sampling-plans', values),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['sampling-plans'] }),
  });
}

// サンプリング計画無効化
export function useDeactivateSamplingPlan(planId: string) {
  return useMutation({
    mutationFn: () =>
      apiClient.patch(`/api/v1/iqc/sampling-plans/${planId}/deactivate`),
  });
}
```

---

## 5. AQL 自動計算ロジック

サーバーサイドで `POST /api/v1/iqc/sampling-plans/preview` を実行時に `aqlTableSnapshot` を参照して n/Ac/Re を決定する。フロントエンドはスナップショットの表示・確認のみを行い、計算ロジックはサーバーに集中する。

**デバウンス:** SamplingPlanForm の AQL 値または検査水準変更時、300ms のデバウンス後に自動プレビューを実行。

---

## 6. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| materialId | 必須・存在する材料 ID | ERR-VAL-048 |
| supplierId | 必須・存在する仕入先 ID | ERR-VAL-049 |
| aql | 0.065〜6.5 の JIS 規格値リストから選択必須 | ERR-VAL-050 |
| inspectionLevel | S-1/S-2/S-3/S-4/I/II/III の 7 種から選択必須 | ERR-VAL-051 |
| 重複計画 | 同一 materialId × supplierId の有効計画が既存の場合は警告ダイアログを表示（旧計画を自動無効化して置き換え） | ERR-BIZ-032 |

---

## 7. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| プレビュー計算失敗 | AqlPreviewCard に「計算できませんでした。入力値を確認してください」を表示 |
| 重複計画（ERR-BIZ-032） | 「既存の計画を無効化して新しい計画を作成しますか?」確認ダイアログ |
| 計画無効化不可（検査実施中 409） | 「この計画は現在実施中の検査で使用中です」ダイアログ |
| ネットワークエラー | トースト通知＋リトライボタン |

---

## 8. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| サンプリング計画一覧取得 | P95 ≤ 500ms |
| AQL プレビュー計算（デバウンス後） | P95 ≤ 300ms |
| 計画保存 | P95 ≤ 500ms |

---

## 9. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| AqlPreviewCard | ローディング中は `aria-busy="true"` を設定 |
| SamplingPlanForm | AQL 値のドロップダウンに JIS 規格値の説明を `aria-describedby` で補足 |
| SeverityStateIndicator | 厳しさ状態に `accessibilityLabel`「現在の検査の厳しさ: なみ/きつい/ゆるい」を付与 |

---

## 参照業界分析

### 必須

[`90_業界分析/11_計測・工程能力と統計的品質工学.md`](../../../../90_業界分析/11_計測・工程能力と統計的品質工学.md)

### 参考

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)
