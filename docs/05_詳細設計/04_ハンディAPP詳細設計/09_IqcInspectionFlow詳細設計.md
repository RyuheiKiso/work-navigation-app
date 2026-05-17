# 09 IqcInspectionFlow 詳細設計

本章は MOD-FE-HA-009 IqcInspectionFlow の責務・型定義・ストア設計・コンポーネント・バリデーション仕様を確定する。本章の実装によって FR-IQ-001〜008（入荷受入登録・サンプリング計画解決・測定値入力・AQL 合否判定）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-HA-009 |
| 物理名 | IqcInspectionFlow |
| ファイルパス | `src/features/iqc-inspection/` |
| 関連 FR | FR-IQ-001〜008（主）|
| 関連 SCR | SCR-HA-016（受入登録）/ SCR-HA-017（測定入力）/ SCR-HA-018（合否確認）|
| アクセスロール | operator（IQC）/ quality_admin |
| 連携モジュール | MOD-FE-HA-004 EvidenceCapture（写真撮影）、MOD-FE-HA-008 LocalDbService（Offline Outbox）|

**責務境界:**
- 本モジュール: 入荷ロット QR スキャン・サンプリング計画取得・測定値逐次入力・AQL 判定表示のフロー管理
- EvidenceCapture: サンプル写真撮影の実装詳細（呼び出し側のみ）
- LocalDbService: オフライン時の測定値 Outbox バッファリング

---

## 2. 状態定義（Zustand ストア）

### 2-1. 型定義

```typescript
// 入荷ロット
export interface IncomingLot {
  lotId: string;
  materialCode: string;
  materialName: string;
  supplierCode: string;
  lotQuantity: number;
}

// サンプリング計画（解決済み）
export interface ResolvedSamplingPlan {
  planId: string;
  aql: number;
  inspectionLevel: string;
  sampleSizeN: number;
  acceptNumberAc: number;
  rejectNumberRe: number;
  severityState: 'NORMAL' | 'TIGHTENED' | 'REDUCED';
}

// 測定値明細
export interface InspectionMeasurement {
  sampleNo: number;
  measuredValue: number | null;
  defectFlag: boolean;
  evidenceFileId: string | null;
}

// IQC フロー状態
export interface IqcInspectionState {
  inspectionId: string | null;
  lot: IncomingLot | null;
  samplingPlan: ResolvedSamplingPlan | null;
  measurements: InspectionMeasurement[];
  qcStatus: 'PENDING' | 'INSPECTING' | 'PASSED' | 'CONDITIONAL_PASS' | 'SCREENING_REQUIRED' | 'REJECTED' | null;
  currentStep: 'lot_scan' | 'plan_review' | 'measurement' | 'judgment';
  isOffline: boolean;
}
```

### 2-2. Zustand ストアアクション

```typescript
export interface IqcInspectionActions {
  setLot(lot: IncomingLot): void;
  setSamplingPlan(plan: ResolvedSamplingPlan): void;
  upsertMeasurement(sampleNo: number, value: number | null, defect: boolean): void;
  setEvidenceFileId(sampleNo: number, fileId: string): void;
  advanceStep(step: IqcInspectionState['currentStep']): void;
  setQcStatus(status: IqcInspectionState['qcStatus']): void;
  reset(): void;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. LotScanCard

QR スキャンで入荷ロットを特定する。

```typescript
interface LotScanCardProps {
  onLotResolved: (lot: IncomingLot) => void;
  onError: (code: string) => void;
}
```

**内部動作:** QR スキャン結果の GIAI を `GET /api/v1/iqc/lots?giai={giai}` に送信し、IncomingLot を解決する。

### 3-2. SamplingPlanCard

解決されたサンプリング計画（n/Ac/Re/厳しさ）を表示する。

```typescript
interface SamplingPlanCardProps {
  plan: ResolvedSamplingPlan;
  onConfirm: () => void;
}
```

**表示要件:** n/Ac/Re の値と現在の厳しさ状態（なみ/きつい/ゆるい）を表示。検査水準と AQL 値も明示。

### 3-3. MeasurementInputGrid

サンプル No. ごとに測定値・不良フラグを入力するグリッド。

```typescript
interface MeasurementInputGridProps {
  sampleSizeN: number;
  measurements: InspectionMeasurement[];
  onMeasurementChange: (sampleNo: number, value: number | null, defect: boolean) => void;
  onPhotoCapture: (sampleNo: number) => void;
}
```

**UX 要件:** 不良フラグ ON 時にセルを赤色ハイライト。累積不良数を常時表示し、Ac/Re との比較状況をリアルタイムで確認できる。

### 3-4. AqlResultBanner

AQL 判定結果（PASSED / REJECTED）を全幅バナーで表示する。REJECTED 時は赤、PASSED 時は緑。

```typescript
interface AqlResultBannerProps {
  defectCount: number;
  acceptNumberAc: number;
  rejectNumberRe: number;
  qcStatus: 'PASSED' | 'REJECTED' | 'INSPECTING';
}
```

**アクセシビリティ:** `role="alert"` で PASSED/REJECTED を音声読み上げ対応（後述 8 章）。

---

## 4. 主要アクション / フック

```typescript
// 入荷ロット登録
export function useCreateIncomingInspection() {
  return useMutation({
    mutationFn: (payload: CreateInspectionPayload) =>
      apiClient.post('/api/v1/iqc/incoming-inspections', payload),
    // Idempotency-Key ヘッダ自動付与（UUID v7 を Request Interceptor で生成）
  });
}

// 測定値逐次登録
export function useAddMeasurement(inspectionId: string) {
  return useMutation({
    mutationFn: (m: MeasurementPayload) =>
      apiClient.post(`/api/v1/iqc/incoming-inspections/${inspectionId}/measurements`, m),
  });
}

// AQL 判定実行
export function useJudgeInspection(inspectionId: string) {
  return useMutation({
    mutationFn: () =>
      apiClient.post(`/api/v1/iqc/incoming-inspections/${inspectionId}/judge`, {}),
    onSuccess: (data) => {
      useIqcStore.getState().setQcStatus(data.qcStatus);
    },
  });
}

// サンプリング計画取得（ロット解決後に自動実行）
export function useResolveSamplingPlan(lotId: string | null) {
  return useQuery({
    queryKey: ['sampling-plan', lotId],
    queryFn: () =>
      apiClient.get(`/api/v1/iqc/lots/${lotId}/sampling-plan`),
    enabled: lotId !== null,
  });
}
```

---

## 5. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| 測定値数 | measurements.length === samplingPlan.sampleSizeN（全サンプル入力必須） | ERR-VAL-030 |
| 重複判定 | 判定済み（PASSED/REJECTED/CONDITIONAL_PASS 等）の inspection への POST は拒否 | ERR-BIZ-017 |
| 測定値範囲 | measuredValue が検査規格の上下限範囲外の場合 defectFlag を自動 ON（任意上書き可） | ERR-VAL-031 |
| Offline 保護 | isOffline 時は LocalDbService に測定値をバッファし、復帰後に自動同期（Outbox パターン） | — |
| ロット未検査 | lot_qc_states が存在しない場合の POST を許可（新規検査として受け付け） | — |

---

## 6. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| QR スキャン失敗 | トースト通知（ERR-QR-001）、再スキャンを促す |
| ロット未登録（404） | ダイアログで「受入登録が必要です」を表示 |
| サンプリング計画未設定（404） | quality_admin に計画設定を依頼するガイダンスを表示 |
| 測定値 POST 失敗（オフライン） | LocalDbService Outbox に積み、「オフライン保存済み」バッジを表示 |
| AQL 判定 API タイムアウト | リトライボタンを提示、判定結果が不確定の間は qcStatus を INSPECTING のままに保つ |
| ERR-BIZ-017（重複判定） | 「すでに判定済みです」ダイアログ、既存判定結果を表示して処理を終了 |

---

## 7. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| 測定値入力 → UI 反映 | 16ms 以内（React 同期レンダリング） |
| AQL 判定 API 呼び出し | P95 ≤ 500ms |
| QR スキャン → ロット解決 API | P95 ≤ 1s |
| サンプリング計画取得 | P95 ≤ 800ms |
| オフライン Outbox 同期（復帰後） | 全測定値を 10s 以内にバックグラウンドで送信 |

MeasurementInputGrid は `React.memo` と `useCallback` で不要な再レンダリングを防止する。サンプル数が 125 件（JIS Z 9015-1 最大サンプルサイズ）を超える場合は仮想スクロール（FlashList）を使用。

---

## 8. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| AqlResultBanner | `role="alert"` で PASSED/REJECTED を VoiceOver/TalkBack 音声読み上げ対応 |
| MeasurementInputGrid | グローブ対応 min 72dp タップターゲット（CFG-013 準拠） |
| LotScanCard | カメラアクセス権限拒否時に代替テキスト入力フォームを提供 |
| SamplingPlanCard | n/Ac/Re の数値に `accessibilityLabel` でフルテキスト説明を付与 |

色覚多様性対応: PASSED/REJECTED の区別を色のみに依存せず、アイコン（✓/✗）と文字でも表示。

---

## 9. 参照業界分析

### 必須

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)

[`90_業界分析/11_計測・工程能力と統計的品質工学.md`](../../../../90_業界分析/11_計測・工程能力と統計的品質工学.md)

### 参考

[`90_業界分析/17_サプライチェーンと作業依存性.md`](../../../../90_業界分析/17_サプライチェーンと作業依存性.md)
