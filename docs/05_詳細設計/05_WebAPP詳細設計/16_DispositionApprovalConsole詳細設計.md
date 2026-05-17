# 16 DispositionApprovalConsole 詳細設計

本章は MOD-FE-MC-008 DispositionApprovalConsole の責務・型定義・コンポーネント・バリデーション仕様を確定する。FR-ST-013（ディスポジション判定）・FR-EV-014（二者電子サイン）を充足する。NFR-SEC-048（Two-Person Integrity）を UI レイヤで実装する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-008 |
| 物理名 | DispositionApprovalConsole |
| ファイルパス | `src/features/disposition-approval/` |
| 関連 FR | FR-ST-013, FR-EV-014 |
| 関連 SCR | SCR-MC-013 |
| アクセスロール | quality_admin（品質担当署名）+ supervisor（現場監督署名）|

**責務境界:**
- 本モジュール: 不適合品のディスポジション判定（REWORK/SCRAP/RETURN/USE_AS_IS）・二者電子サイン・Two-Person Integrity の UI レイヤ実装
- ElectronicSignPad（MOD-FE-MA-006）: 電子サインの実装詳細（本モジュールは signId を受け取るのみ）
- ReworkFlow（MOD-FE-HA-010）: ディスポジション判定後のリワーク実施（本モジュールは判定操作のみ）

---

## 2. 状態定義（TypeScript 型定義）

```typescript
export type DispositionDecision = 'REWORK' | 'SCRAP' | 'RETURN' | 'USE_AS_IS';

export interface NonconformityCase {
  nonconformityId: string;
  lotId: string;
  materialName: string;
  supplierName: string;
  nonconformityCategory: string;
  defectCount: number;
  detectedAt: string;           // ISO 8601
  status: 'PENDING_DISPOSITION' | 'DISPOSITION_APPROVED';
}

export interface DispositionForm {
  nonconformityId: string;
  decision: DispositionDecision;
  decisionReason: string;
  qualityAdminSignId: string | null;   // quality_admin の電子サイン ID
  supervisorSignId: string | null;     // supervisor の電子サイン ID
}

// ディスポジション判定フォーム状態（ローカル）
export interface DispositionFormState {
  selectedNonconformityId: string | null;
  decision: DispositionDecision | null;
  decisionReason: string;
  qualityAdminSignId: string | null;
  supervisorSignId: string | null;
}
```

---

## 3. 主要コンポーネント Props

### 3-1. NonconformityList

ディスポジション待ち不適合品の一覧を表示する。

```typescript
interface NonconformityListProps {
  cases: NonconformityCase[];
  selectedId: string | null;
  onSelect: (nonconformityId: string) => void;
  isLoading: boolean;
}
```

### 3-2. DispositionDecisionSelector

ディスポジション判定種別（REWORK/SCRAP/RETURN/USE_AS_IS）を選択するラジオグループ。

```typescript
interface DispositionDecisionSelectorProps {
  value: DispositionDecision | null;
  onChange: (decision: DispositionDecision) => void;
  disabled: boolean;
}
```

**表示ラベル:**
- REWORK → 「リワーク（手直し）」
- SCRAP → 「廃却」
- RETURN → 「返却」
- USE_AS_IS → 「特採使用」

### 3-3. DispositionSignaturePanel

品質担当または現場監督の電子サインエリア。同一ロールのみが署名可能。

```typescript
interface DispositionSignaturePanelProps {
  role: 'quality_admin' | 'supervisor';
  signedBy: string | null;           // 署名済み worker_id（null = 未署名）
  onSign: (signId: string) => void;
  disabled: boolean;
}
```

### 3-4. TwoPersonIntegrityWarning

品質担当と現場監督に同一 worker_id が署名しようとした場合に表示する警告バナー。

```typescript
interface TwoPersonIntegrityWarningProps {
  qualityAdminSignId: string | null;
  supervisorSignId: string | null;
  currentUserId: string;
  onViolationDetected: () => void;
}
```

---

## 4. Two-Person Integrity UI 設計

DispositionApprovalConsole は独立した 2 つのサインエリアを表示する:

1. **品質担当サインエリア**: quality_admin ロールの worker のみが署名可能
2. **現場監督サインエリア**: supervisor ロールの worker のみが署名可能

**ロック機構:**
- 両エリアが未署名の間は「承認」ボタンを `disabled` 状態に保つ
- 品質担当と現場監督に同一 worker_id が署名しようとした場合、TwoPersonIntegrityWarning を表示
- 最終防衛線は DB トリガ（ERR-BIZ-021）

---

## 5. 主要アクション / フック

```typescript
// ディスポジション待ち一覧取得
export function usePendingDispositionList() {
  return useQuery({
    queryKey: ['disposition-pending'],
    queryFn: () =>
      apiClient.get('/api/v1/nonconformities?status=PENDING_DISPOSITION'),
    refetchInterval: 30_000,
  });
}

// ディスポジション判定 POST
export function useApproveDisposition() {
  return useMutation({
    mutationFn: (form: DispositionForm) =>
      apiClient.post(`/api/v1/nonconformities/${form.nonconformityId}/disposition`, {
        decision: form.decision,
        decisionReason: form.decisionReason,
        qualityAdminSignId: form.qualityAdminSignId,
        supervisorSignId: form.supervisorSignId,
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['disposition-pending'] }),
  });
}
```

---

## 6. バリデーション仕様

| フィールド | ルール | エラーコード |
|---|---|---|
| decision | 必須・4 種の DispositionDecision から選択 | ERR-VAL-058 |
| decisionReason | 必須・50 文字以上 | ERR-VAL-059 |
| qualityAdminSignId | quality_admin ロールの電子サイン必須 | ERR-VAL-060 |
| supervisorSignId | supervisor ロールの電子サイン必須 | ERR-VAL-061 |
| 異一性（UI レイヤ） | qualityAdminSignId ≠ supervisorSignId（signer_id 比較）| ERR-BIZ-021 |
| 重複判定 | DISPOSITION_APPROVED 状態の nonconformity への POST は拒否 | ERR-BIZ-034 |

---

## 7. エラーハンドリング

| エラー種別 | 対応方針 |
|---|---|
| ERR-BIZ-021（Two-Person Integrity 違反） | TwoPersonIntegrityWarning 表示、承認ボタンを無効化 |
| ERR-BIZ-034（重複判定） | 「このケースはすでに判定済みです」ダイアログ、既存判定内容を表示 |
| 電子サイン失敗 | ElectronicSignPad のエラーをトースト通知で表示 |
| ネットワークエラー | Idempotency-Key を付与して冪等リトライ可能 |

---

## 8. パフォーマンス要件

| 操作 | 目標値 |
|---|---|
| ディスポジション待ち一覧取得 | P95 ≤ 500ms |
| ディスポジション判定 POST | P95 ≤ 800ms |

---

## 9. アクセシビリティ

| コンポーネント | 要件 |
|---|---|
| TwoPersonIntegrityWarning | `role="alert"` で Two-Person Integrity 違反を即時音声通知 |
| DispositionDecisionSelector | ラジオグループに `role="radiogroup"` と `aria-labelledby` を設定 |
| DispositionSignaturePanel | 署名済み状態に `aria-checked="true"` を付与 |

---

## 参照業界分析

### 必須

[`90_業界分析/13_安全文化と安全管理システム.md`](../../../../90_業界分析/13_安全文化と安全管理システム.md)

[`90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)

### 参考

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)
