# 01 SopEditor 詳細設計

本章は MOD-FE-MA-001（SopEditor）の TypeScript インターフェース・Zustand ストア設計・react-query v5 フック定義・Auto-Save 仕様・Undo/Redo 仕様を確定する。SOP の Step 編集は FR-MA-001〜007 の中核機能であり、本章がマスタメンテナンス APP の最重要コンポーネント設計を担う。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MA-001 |
| 物理名 | SopEditor |
| ファイルパス | `src/features/sop-editor/` |
| 関連 FR | FR-MA-001〜007 |
| 関連 SCR | SCR-MA-004（SOP 編集）・SCR-MA-005（インポート）・SCR-MA-006（プレビュー）|
| アクセスロール | master_admin |

---

## 2. 状態定義

SopEditor の全 UI 状態は Zustand ストア（FNC-FE-001）で管理する。

```typescript
import type { StepInputType, MultilingualText, JudgmentCondition } from '@/shared/types';

// Step 草稿（編集中の未保存状態）
export interface StepDraft {
  /** 新規 Step は 'new-{uuid}' 形式の一時 ID、既存は永続 UUID */
  stepId: string;
  stepNumber: number;
  inputType: StepInputType;
  instructionText: MultilingualText;
  evidenceRequired: boolean;
  judgmentCondition: JudgmentCondition | null;
}

// SopEditor Zustand ストア状態
export interface SopEditorState {
  sopId: string;
  versionId: string;
  steps: StepDraft[];
  isDirty: boolean;
  lastAutoSaveAt: Date | null;
  /** 最大 50 ステップ履歴（FR-MA-003 implied）*/
  undoStack: StepDraft[][];
  /** 最大 50 ステップ履歴 */
  redoStack: StepDraft[][];
}

// Zustand ストアアクション（FNC-FE-001）
export interface SopEditorActions {
  /** Step 追加（stepNumber は既存最大値 + 1）*/
  addStep: (draft: Omit<StepDraft, 'stepId' | 'stepNumber'>) => void;
  /** Step 更新（undoStack に現状態を push する）*/
  updateStep: (stepId: string, patch: Partial<StepDraft>) => void;
  /** Step 削除（後続の stepNumber を詰める）*/
  deleteStep: (stepId: string) => void;
  /** Step 並べ替え（DnD 完了時）*/
  reorderSteps: (orderedStepIds: string[]) => void;
  /** アンドゥ（undoStack.pop → steps へ復元、現状態を redoStack に push）*/
  undo: () => void;
  /** リドゥ（redoStack.pop → steps へ復元、現状態を undoStack に push）*/
  redo: () => void;
  /** Auto-Save 完了時刻を記録 */
  markAutoSaved: (at: Date) => void;
  /** isDirty をリセット（保存完了後）*/
  markClean: () => void;
}

// 型制約
export type SopEditorStore = SopEditorState & SopEditorActions;
```

### 2-1. undoStack 上限制御

```typescript
/** undoStack は UNDO_LIMIT 件を超えたとき先頭エントリを破棄する */
const UNDO_LIMIT = 50 as const;

function pushUndo(state: SopEditorState): StepDraft[][] {
  const next = [...state.undoStack, state.steps];
  return next.length > UNDO_LIMIT ? next.slice(1) : next;
}
```

---

## 3. コンポーネント Props 定義

```typescript
// SopEditor コンポーネント（SCR-MA-004 に対応）
export interface SopEditorProps {
  sopId: string;
  versionId: string;
  /** quality_admin によるレビュー時は true（編集不可）*/
  readOnly?: boolean;
  /** 保存完了コールバック（react-query mutation を想定）*/
  onSave: (steps: StepDraft[]) => Promise<void>;
}
```

---

## 4. react-query フック定義

### 4-1. SOP 草稿取得（FNC-FE-003）

```typescript
import { useQuery, UseQueryResult } from '@tanstack/react-query';
import type { SopDraftResponse } from '@/shared/api/generated';

/**
 * FNC-FE-003: SOP バージョン草稿を取得する react-query フック
 *
 * @param sopId  - SOP UUID
 * @param versionId - バージョン UUID
 * @returns SopDraftResponse を含む UseQueryResult
 *
 * staleTime: 0（常に最新草稿を取得）
 * gcTime: 5 分
 */
export declare function useSopDraft(
  sopId: string,
  versionId: string,
): UseQueryResult<SopDraftResponse, Error>;
```

### 4-2. Step 一括保存 Mutation

```typescript
import { useMutation, UseMutationResult } from '@tanstack/react-query';
import type { SaveStepsRequest } from '@/shared/api/generated';

/**
 * Step 草稿を API に一括保存する Mutation フック
 * 成功時に isDirty をリセットし lastAutoSaveAt を更新する
 */
export declare function useSaveStepsMutation(
  sopId: string,
  versionId: string,
): UseMutationResult<void, Error, SaveStepsRequest>;
```

---

## 5. Auto-Save 仕様（FR-MA-005）

Auto-Save は `useEffect` + debounce 30 s で実装する（FNC-FE-002）。

```typescript
import { useEffect, useRef } from 'react';
import type { StepDraft } from './types';

/** Auto-Save デバウンス間隔（ms） */
const AUTO_SAVE_DEBOUNCE_MS = 30_000 as const;

/**
 * FNC-FE-002: isDirty が true の場合、最終変更から 30 s 後に onSave を呼び出す
 * onSave が Promise を返し、解決したら markAutoSaved を呼ぶ
 * アンマウント時にはタイマーをキャンセルする
 */
export declare function useAutoSave(params: {
  steps: StepDraft[];
  isDirty: boolean;
  onSave: (steps: StepDraft[]) => Promise<void>;
  onSaved: (at: Date) => void;
}): {
  /** Auto-Save 中は true */
  isSaving: boolean;
  /** 最後の Auto-Save エラー（null = 成功）*/
  saveError: Error | null;
};
```

Auto-Save の動作仕様:

| 条件 | 動作 |
|---|---|
| isDirty === false | Auto-Save タイマーを起動しない |
| isDirty === true | 最終変更から 30 s 後に onSave を呼び出す |
| readOnly === true | Auto-Save を無効にする |
| onSave が失敗 | saveError にセットし isDirty を維持する（データ損失防止）|
| アンマウント | clearTimeout でタイマーをキャンセルする |

---

## 6. StepInputType 定義

```typescript
export type StepInputType =
  | 'OK_NG'        // OK/NG 判定入力
  | 'NUMERIC'      // 数値測定入力
  | 'TEXT'         // テキスト入力
  | 'PHOTO'        // 写真撮影
  | 'SIGN'         // 電子サイン
  | 'QR_SCAN'      // QR スキャン
  | 'CUSTOM';      // カスタム入力（FR-NV-006）

export interface MultilingualText {
  ja: string;
  en: string;
  jaSimple?: string;  // やさしい日本語（FR-UI-003）
}

export interface JudgmentCondition {
  /** JSON Logic ルール（DslConditionBuilder が生成）*/
  rule: import('./dsl/types').JsonLogicRule;
  /** 合格ラベル（多言語）*/
  passLabel: MultilingualText;
  /** 不合格ラベル（多言語）*/
  failLabel: MultilingualText;
}
```

---

## 7. コンポーネントツリー

SopEditor の内部コンポーネント構成を以下に示す。

```
SopEditor (MOD-FE-MA-001)
  StepList
    StepCard (×N)
      StepInputTypeSelector
      InstructionTextEditor (多言語タブ付き)
      EvidenceToggle
      JudgmentConditionField → DslConditionBuilder (MOD-FE-MA-002)
  StepAddButton
  UndoRedoToolbar (undo/redo ボタン・isDirty インジケータ)
  AutoSaveIndicator (lastAutoSaveAt 表示)
```

---

## 8. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-010 | stepNumber 重複（並べ替え後の整合性エラー）| トースト通知・編集継続可能 |
| ERR-VAL-011 | instructionText.ja が空文字 | フィールドレベルバリデーション |
| ERR-BIZ-005 | published バージョンへの直接編集 | 保存ボタン非活性・readOnly 強制 |
| ERR-SYS-001 | Auto-Save API タイムアウト | saveError 表示・手動保存を促すバナー |

---

**本節で確定した方針**
- **SopEditorState を Zustand ストアで一元管理し、undoStack/redoStack は 50 ステップ上限・先頭破棄方式で実装することを確定した。**
- **Auto-Save は useEffect + debounce 30 s（FR-MA-005）で実装し、readOnly モード時は無効化する。onSave 失敗時は isDirty を維持してデータ損失を防止する。**
- **StepDraft の stepId は新規作成時に 'new-{uuid}' 形式の一時 ID を使用し、保存完了後にバックエンドが払い出す永続 UUID に置換する方式を確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
