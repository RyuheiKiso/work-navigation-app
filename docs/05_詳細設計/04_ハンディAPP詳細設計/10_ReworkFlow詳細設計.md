# 10 ReworkFlow 詳細設計

本章は MOD-FE-HA-010 ReworkFlow の責務・型定義・ストア設計・コンポーネント・バリデーション仕様を確定する。本章の実装によって FR-ST-014（リワーク作業実施）・FR-EV-015（再検査）・FR-MA-017（廃却処理）を充足する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-HA-010 |
| 物理名 | ReworkFlow |
| ファイルパス | `src/features/rework-flow/` |
| 関連 FR | FR-ST-014, FR-EV-015, FR-MA-017, FR-MA-018 |
| 関連 SCR | SCR-HA-019（リワーク実施）/ SCR-HA-020（再検査）/ SCR-HA-021（廃却）/ SCR-HA-022（返却）|
| アクセスロール | operator, supervisor, quality_admin |
| 連携モジュール | MOD-FE-HA-004 EvidenceCapture（前後写真）、MOD-FE-HA-007 ElectronicSignPad（電子サイン） |

**責務境界:**
- 本モジュール: リワーク作業 QR スキャン→実施→再検査→廃却/返却のフロー管理
- EvidenceCapture: 前後写真撮影の実装詳細（呼び出し側のみ）
- ElectronicSignPad: 廃却立会者サイン UI の実装詳細（呼び出し側のみ）

---

## 2. 状態定義（Zustand ストア）

### 2-1. 型定義

```typescript
export type ReworkType = 'TOUCH_UP' | 'REWORK_FULL' | 'SORTING' | 'SCRAP' | 'RETURN';

export type ReworkStatus =
  | 'PENDING_DISPOSITION'
  | 'REWORK_IN_PROGRESS'
  | 'REWORK_COMPLETED'
  | 'VERIFICATION_IN_PROGRESS'
  | 'CLOSED_OK_RELEASE'
  | 'CLOSED_SCRAP'
  | 'CLOSED_RETURN';

export interface ReworkFlowState {
  reworkId: string | null;
  reworkType: ReworkType | null;
  status: ReworkStatus | null;
  parentCaseId: string | null;    // 不変参照: ALCOA+ Original（NFR-DQ-010）
  reworkCaseId: string | null;    // 新規 WorkExecution ID
  reworkQrGiai: string | null;    // 修正品 QR ラベル GIAI
  beforeEvidenceFileId: string | null;
  afterEvidenceFileId: string | null;
  currentVerifierId: string | null;
  reworkCount: number;            // CFG-026 超過チェック用
}
```

### 2-2. Zustand ストアアクション

```typescript
export interface ReworkFlowActions {
  initRework(reworkId: string, parentCaseId: string, reworkType: ReworkType): void;
  setBeforeEvidence(fileId: string): void;
  setAfterEvidence(fileId: string): void;
  setReworkCaseId(caseId: string): void;
  setVerifier(verifierId: string): void;
  advanceStatus(status: ReworkStatus): void;
  reset(): void;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. ReworkQrLabelCard

修正品 QR ラベル（GS1 AI 8003 + AI 91）を表示・印刷する。

```typescript
interface ReworkQrLabelCardProps {
  giai: string;
  reworkId: string;
  parentLotId: string;
  reworkSopVersionId: string;
  onPrint: () => void;
}
```

**表示要件:** QR コードには GS1 AI 8003（グローバル個別資産識別子）+ AI 91（リワーク管理番号）をエンコードする。ラベルには元ロット ID と承認済みリワーク SOP バージョンを人間可読形式でも印字。

### 3-2. TwoPersonIntegrityCheck

再検査・廃却時に「実施者と異なる worker_id」であることを UI レベルで警告するバナー。

```typescript
interface TwoPersonIntegrityCheckProps {
  originalWorkerId: string;  // リワーク実施者
  currentUserId: string;     // 現在のログインユーザー
  operationType: 'verification' | 'scrap_witness';
  onViolationDetected: () => void;
}
```

### 3-3. ReworkProgressStepper

現在のリワークフロー段階を視覚的に表すステッパー。

```typescript
interface ReworkProgressStepperProps {
  currentStatus: ReworkStatus;
  reworkType: ReworkType;
}
```

**表示ステップ:** QR スキャン → 前写真 → リワーク実施 → 後写真 → 再検査 → 廃却/返却/リリース

### 3-4. DispositionSummaryCard

ディスポジション判定（REWORK/SCRAP/RETURN/USE_AS_IS）の内容と承認者を表示するカード。

```typescript
interface DispositionSummaryCardProps {
  decision: 'REWORK' | 'SCRAP' | 'RETURN' | 'USE_AS_IS';
  decisionReason: string;
  qualityAdminName: string;
  supervisorName: string;
}
```

---

## 4. 主要アクション / フック

```typescript
// リワーク情報取得（QR スキャン後）
export function useGetRework(reworkId: string | null) {
  return useQuery({
    queryKey: ['rework', reworkId],
    queryFn: () =>
      apiClient.get(`/api/v1/rework/${reworkId}`),
    enabled: reworkId !== null,
  });
}

// リワーク作業開始
export function useStartRework() {
  return useMutation({
    mutationFn: (payload: StartReworkPayload) =>
      apiClient.post('/api/v1/rework/start', payload),
  });
}

// リワーク作業完了
export function useCompleteRework(reworkId: string) {
  return useMutation({
    mutationFn: (payload: CompleteReworkPayload) =>
      apiClient.post(`/api/v1/rework/${reworkId}/complete`, payload),
  });
}

// 再検査（verification）
export function useVerifyRework(reworkId: string) {
  return useMutation({
    mutationFn: (payload: VerifyReworkPayload) =>
      apiClient.post(`/api/v1/rework/${reworkId}/verify`, payload),
  });
}

// 廃却処理
export function useScrapRework(reworkId: string) {
  return useMutation({
    mutationFn: (payload: ScrapPayload) =>
      apiClient.post(`/api/v1/rework/${reworkId}/scrap`, payload),
  });
}
```

---

## 5. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| 前後写真 | rework_type ≠ SCRAP/RETURN の場合は before/after 両方必須 | ERR-VAL-002 |
| 再検査者 | verifier_id ≠ リワーク実施者（CFG-025 依存: false の場合は警告のみ） | ERR-BIZ-023 |
| 廃却立会者 | witness_id ≠ 廃却実施者 | ERR-BIZ-024 |
| 追跡番号 | RETURN 時は tracking_no が必須 | ERR-BIZ-025 |
| リワーク上限 | CFG-026 超過時はブロック（ERR-BIZ-022）、SCRAP を提案 | ERR-BIZ-022 |
| ALCOA+ Original | rework_case_id は parent_case_id と異なる WorkExecution ID を生成（元記録の変更禁止） | NFR-DQ-010 |

---

## 6. Two-Person Integrity ハンドリング

再検査・廃却立会者の worker_id が リワーク実施者と一致する場合:

1. **UI レイヤ**: TwoPersonIntegrityCheck が赤バナーで警告表示し onViolationDetected を呼び出す
2. **API レイヤ**: ERR-BIZ-023（再検査）/ ERR-BIZ-024（廃却）を返す（最終防衛線）
3. **設定依存**: CFG-025（rework.verifier_must_differ）が false の場合のみ警告表示のみでブロックしない

---

## 7. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| ERR-BIZ-022（リワーク上限超過） | ダイアログで超過回数と CFG-026 上限を表示、廃却フローへの遷移ボタンを提供 |
| ERR-BIZ-023（再検査者同一） | 「リワーク実施者と再検査者は別の作業者が必要です」ダイアログ |
| ERR-BIZ-024（廃却立会者同一） | 「廃却処理は立会者が必要です」ダイアログ |
| ERR-BIZ-025（追跡番号未入力） | RETURN フォームの tracking_no フィールドにバリデーションエラーを表示 |
| 写真アップロード失敗 | LocalDbService Outbox に積み、復帰後に自動同期 |
| QR スキャン失敗 | トースト通知、再スキャンを促す |

---

## 8. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| QR スキャン → リワーク情報取得 | P95 ≤ 1s |
| 前後写真アップロード（EvidenceCapture 経由） | バックグラウンドで非同期処理 |
| リワーク作業完了 API 呼び出し | P95 ≤ 800ms |
| 再検査結果 POST | P95 ≤ 500ms |

---

## 9. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| TwoPersonIntegrityCheck | `role="alert"` で警告を即時音声通知 |
| ReworkProgressStepper | 現在ステップに `aria-current="step"` を付与 |
| DispositionSummaryCard | ディスポジション判定種別を `accessibilityLabel` でフルテキスト説明 |
| ReworkQrLabelCard | QR コード画像に alt テキスト（GIAI + リワーク番号）を付与 |

---

## 参照業界分析

### 必須

[`90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)

[`90_業界分析/13_安全文化と安全管理システム.md`](../../../../90_業界分析/13_安全文化と安全管理システム.md)

### 参考

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)
