# 04 MasterVersionDiff 詳細設計

本章は MOD-FE-MA-004（MasterVersionDiff）の TypeScript インターフェース・差分計算アルゴリズム仕様・react-query フック定義・VersionDiffViewer コンポーネント Props を確定する。MasterVersionDiff は FR-MA-013 で要求されるバージョン差分表示を担い、SCR-MA-010（バージョン差分）のコアロジックを提供する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-004 |
| 物理名 | MasterVersionDiff |
| ファイルパス | `src/features/sop-editor/diff/` |
| 関連 FR | FR-MA-013（バージョン差分表示）|
| 関連 SCR | SCR-MA-010（バージョン差分）|
| アクセスロール | master_admin・quality_admin |

---

## 2. 型定義

```typescript
import type { StepDraft } from '../types';

// バージョン差分閲覧コンポーネント Props
export interface VersionDiffViewerProps {
  /** 比較基準バージョン（旧版）UUID */
  baseVersionId: string;
  /** 比較対象バージョン（新版）UUID */
  headVersionId: string;
  sopId: string;
}

// Step 差分エントリ
export interface StepDiff {
  stepId: string;
  changeType: 'added' | 'modified' | 'deleted' | 'unchanged';
  /** 旧版の Step（added の場合は null）*/
  before: StepEntity | null;
  /** 新版の Step（deleted の場合は null）*/
  after: StepEntity | null;
  /** modified の場合に変更されたフィールド名の配列 */
  changedFields: string[];
}

// バージョン差分全体
export interface VersionDiff {
  baseVersionId: string;
  headVersionId: string;
  sopId: string;
  stepDiffs: StepDiff[];
  summary: DiffSummary;
}

export interface DiffSummary {
  addedCount: number;
  modifiedCount: number;
  deletedCount: number;
  unchangedCount: number;
  totalStepCount: number;
}

// Step の永続エンティティ（バックエンド API レスポンス由来）
export interface StepEntity {
  stepId: string;
  stepNumber: number;
  inputType: import('../types').StepInputType;
  instructionText: import('../types').MultilingualText;
  evidenceRequired: boolean;
  judgmentCondition: import('../types').JudgmentCondition | null;
  createdAt: Date;
  updatedAt: Date;
}
```

---

## 3. 差分計算アルゴリズム（FNC-FE-008）

```typescript
/**
 * FNC-FE-008: 2 つのバージョンの Step 配列を比較し StepDiff[] を返す
 *
 * アルゴリズム:
 * 1. baseSteps と headSteps を stepId でインデックス化する
 * 2. headSteps を走査し、baseSteps に存在しない stepId は 'added' とする
 * 3. baseSteps を走査し、headSteps に存在しない stepId は 'deleted' とする
 * 4. 双方に存在する stepId はフィールド単位で比較し、差異がある場合は 'modified' とする
 *    - changedFields に変更されたフィールド名を列挙する
 * 5. すべてのフィールドが一致する場合は 'unchanged' とする
 * 6. 結果を headSteps の stepNumber 順（deleted は baseSteps の stepNumber 順）でソートする
 *
 * @param baseSteps - 旧版の Step エンティティ配列
 * @param headSteps - 新版の Step エンティティ配列
 * @returns StepDiff の配列（表示順ソート済み）
 */
export declare function computeStepDiff(
  baseSteps: readonly StepEntity[],
  headSteps: readonly StepEntity[],
): StepDiff[];

// フィールド単位比較対象
const COMPARED_FIELDS: ReadonlyArray<keyof StepEntity> = [
  'stepNumber',
  'inputType',
  'instructionText',
  'evidenceRequired',
  'judgmentCondition',
] as const;
```

---

## 4. react-query フック定義

```typescript
import { useQuery, UseQueryResult } from '@tanstack/react-query';

/**
 * バージョン差分を取得する react-query フック
 * バックエンドが差分計算済みのレスポンスを返す場合はそれを使用し、
 * クライアント側では computeStepDiff による補完計算を行わない
 *
 * staleTime: 5 分（差分は変更されない）
 * gcTime: 30 分
 */
export declare function useVersionDiff(
  sopId: string,
  baseVersionId: string,
  headVersionId: string,
): UseQueryResult<VersionDiff, Error>;
```

---

## 5. コンポーネントツリー

```
VersionDiffViewer (MOD-FE-MA-004)
  DiffSummaryBar（added/modified/deleted/unchanged の件数バッジ）
  VersionSelector（baseVersionId / headVersionId のプルダウン）
  StepDiffList
    StepDiffRow (×N, changeType に応じて背景色変更)
      [unchanged] StepReadOnlyCard（グレー背景）
      [added]     StepReadOnlyCard（緑背景・追加アイコン）
      [deleted]   StepReadOnlyCard（赤背景・削除アイコン・before 表示）
      [modified]  StepDiffCard（左右分割: before / after, changedFields をハイライト）
  DiffLegend（色凡例）
```

---

## 6. フィールドレベルハイライト仕様

modified の StepDiffCard では `changedFields` を使用して変更箇所のみをハイライトする。

```typescript
// フィールドハイライト判定
export function isFieldChanged(
  diff: StepDiff,
  fieldName: keyof StepEntity,
): boolean {
  return diff.changeType === 'modified' && diff.changedFields.includes(fieldName);
}

// ハイライト CSS クラス（Tailwind CSS 想定）
export const DIFF_HIGHLIGHT_CLASS = 'bg-yellow-100 ring-1 ring-yellow-400' as const;
```

---

## 7. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-030 | baseVersionId === headVersionId | 「同一バージョンは比較不可」バナー表示 |
| ERR-BIZ-010 | sopId と versionId の対応不一致 | エラーページへリダイレクト |
| ERR-SYS-001 | API タイムアウト | スケルトンローダー → エラーバナー・再試行ボタン |

---

**本節で確定した方針**
- **差分計算は stepId をキーとした O(n) 比較アルゴリズムで実装し、フィールド単位の変更を changedFields 配列で明示することを確定した。**
- **バックエンドが差分計算済みレスポンスを返す場合はクライアント側の再計算を省略し、computeStepDiff はオフライン時・テスト時の補完用途に限定することを確定した。**
- **baseVersionId === headVersionId の同一バージョン比較は UI レベルでブロックし、API 呼び出しを行わないことを確定した（ERR-VAL-030）。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
