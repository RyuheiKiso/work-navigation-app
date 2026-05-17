# 15 IqcDashboard 詳細設計

本章は MOD-FE-MC-007 IqcDashboard の責務・型定義・コンポーネント・バリデーション仕様を確定する。FR-IQ-014（仕入先品質実績管理）・FR-IQ-015（集計・レポート）を充足する。BR-BUS-038・NFR-ETH-002（個人別集計禁止）を技術的に遵守する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-007 |
| 物理名 | IqcDashboard |
| ファイルパス | `src/features/iqc-dashboard/` |
| 関連 FR | FR-IQ-014, FR-IQ-015 |
| 関連 SCR | SCR-MC-011, SCR-MC-012 |
| アクセスロール | quality_admin（フル閲覧）/ executive（集計のみ）|

**責務境界:**
- 本モジュール: 仕入先×材料×月次の品質実績集計・表示・エクスポート
- ConcessionApprovalConsole（MOD-FE-MC-006）: 特採承認操作（本モジュールは集計表示のみ）
- SupplierMasterEditor（MOD-FE-MA-008）: 仕入先マスタ管理（本モジュールは仕入先 ID を参照するのみ）

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export interface SupplierQualitySummary {
  supplierId: string;
  supplierName: string;
  materialId: string;
  materialName: string;
  reportMonth: string;              // YYYY-MM
  totalLots: number;
  passedLots: number;
  rejectedLots: number;
  concessionLots: number;           // 特採承認ロット数
  defectRatePct: number;            // 不良率（%）
  currentSeverity: 'NORMAL' | 'TIGHTENED' | 'REDUCED';
  // ※ inspector_id 単位の集計は含まない（NFR-ETH-002 遵守）
}

export interface IqcDashboardFilter {
  supplierId?: string;
  materialId?: string;
  fromMonth: string;                // YYYY-MM
  toMonth: string;                  // YYYY-MM
}

// 検査の厳しさ切替履歴
export interface SeverityHistoryEntry {
  changedAt: string;                // ISO 8601
  fromSeverity: 'NORMAL' | 'TIGHTENED' | 'REDUCED';
  toSeverity: 'NORMAL' | 'TIGHTENED' | 'REDUCED';
  triggerReason: string;
}
```

---

## 3. 個人別集計禁止の実装（NFR-ETH-002）

IqcDashboard は `inspector_id` 列を一切表示しない。

- **API 設計レベル**: `GET /api/v1/iqc/supplier-quality-summary` は仕入先×材料×月次の集計のみを返し、個人別クエリパラメータ（inspector_id）を受け付けない
- **フロントエンドレベル**: SupplierQualitySummary 型に `inspector_id` フィールドを含まず、型安全にデータアクセスを制限
- **エクスポートレベル**: CSV/PDF エクスポートにも個人識別情報を含めない

---

## 4. 主要コンポーネント Props

### 4-1. SupplierQualityTable

仕入先×材料×月次の品質実績テーブル。不良率を赤/黄/緑でカラーコード。

```typescript
interface SupplierQualityTableProps {
  summaries: SupplierQualitySummary[];
  filter: IqcDashboardFilter;
  onFilterChange: (filter: IqcDashboardFilter) => void;
  onExportCsv: () => void;
}
```

**カラーコード基準:**
- 緑: defectRatePct ≤ 1.0%
- 黄: 1.0% < defectRatePct ≤ 3.0%
- 赤: defectRatePct > 3.0%

### 4-2. SeverityStateChip

現在の検査の厳しさ状態（なみ/きつい/ゆるい）をチップ表示。

```typescript
interface SeverityStateChipProps {
  severity: 'NORMAL' | 'TIGHTENED' | 'REDUCED';
  size?: 'sm' | 'md';
}
```

### 4-3. DefectRateTrendChart

仕入先×材料の月次不良率推移を折れ線グラフで表示する。

```typescript
interface DefectRateTrendChartProps {
  supplierId: string;
  materialId: string;
  fromMonth: string;
  toMonth: string;
}
```

### 4-4. SeverityHistoryTimeline

検査の厳しさ状態切替履歴をタイムライン形式で表示する。

```typescript
interface SeverityHistoryTimelineProps {
  planId: string;
  entries: SeverityHistoryEntry[];
}
```

---

## 5. 主要アクション / フック

```typescript
// 仕入先品質集計取得
export function useSupplierQualitySummary(filter: IqcDashboardFilter) {
  return useQuery({
    queryKey: ['iqc-dashboard', filter],
    queryFn: () =>
      apiClient.get('/api/v1/iqc/supplier-quality-summary', { params: filter }),
    staleTime: 300_000,  // 5 分キャッシュ
  });
}

// 月次不良率推移取得
export function useDefectRateTrend(supplierId: string, materialId: string, fromMonth: string, toMonth: string) {
  return useQuery({
    queryKey: ['defect-rate-trend', supplierId, materialId, fromMonth, toMonth],
    queryFn: () =>
      apiClient.get('/api/v1/iqc/defect-rate-trend', {
        params: { supplierId, materialId, fromMonth, toMonth },
      }),
  });
}

// CSV エクスポート
export function useExportIqcSummary() {
  return useMutation({
    mutationFn: (filter: IqcDashboardFilter) =>
      apiClient.get('/api/v1/iqc/supplier-quality-summary/export', {
        params: { ...filter, format: 'csv' },
        responseType: 'blob',
      }),
    onSuccess: (blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `iqc-summary-${new Date().toISOString().slice(0, 10)}.csv`;
      a.click();
    },
  });
}
```

---

## 6. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| fromMonth | 必須・toMonth 以前 | ERR-VAL-055 |
| toMonth | 必須・fromMonth 以降・本日以前 | ERR-VAL-056 |
| 期間上限 | fromMonth〜toMonth は 24 ヶ月以内（大量データ防止） | ERR-VAL-057 |

---

## 7. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| ERR-VAL-057（期間超過） | フィルタフォームに「期間は 24 ヶ月以内で指定してください」を表示 |
| 集計データなし（204） | 「指定期間の検査データがありません」を空状態コンポーネントで表示 |
| エクスポート失敗 | トースト通知「エクスポートに失敗しました。再試行してください」 |
| ネットワークエラー | テーブルにスケルトンローダーを表示し、リトライボタンを提供 |

---

## 8. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| 品質集計取得（12 ヶ月・50 仕入先） | P95 ≤ 1s |
| 月次不良率推移取得 | P95 ≤ 800ms |
| CSV エクスポート | P95 ≤ 3s（バックグラウンドで生成） |

DefectRateTrendChart は Recharts を使用し、1000 点以上のデータポイントがある場合はデータポイントを間引き（sampling）する。

---

## 9. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| SupplierQualityTable | カラーコードを色のみに依存せず、アイコン（▲/●/▼）とツールチップで補足 |
| SeverityStateChip | `accessibilityLabel`「検査の厳しさ: なみ/きつい/ゆるい」を付与 |
| DefectRateTrendChart | グラフに `role="img"` と `aria-label` でサマリテキストを付与 |

---

## 参照業界分析

### 必須

[`90_業界分析/24_作業者プライバシー・データ倫理と労務監視.md`](../../../../90_業界分析/24_作業者プライバシー・データ倫理と労務監視.md)

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)

### 参考

[`90_業界分析/11_計測・工程能力と統計的品質工学.md`](../../../../90_業界分析/11_計測・工程能力と統計的品質工学.md)
