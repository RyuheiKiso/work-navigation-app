# 07 HashChainVerifier 詳細設計

本章は MOD-FE-MC-003（HashChainVerifier）の TypeScript インターフェース・検証結果型定義・react-query フック・CMP-MC-003 コンポーネント Props を確定する。HashChainVerifier は FR-AU-006 で要求されるハッシュチェーン週次検証の結果表示と手動実行トリガーを担い、SCR-MC-008（ハッシュチェーン検証）の UI を提供する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-003 |
| 物理名 | HashChainVerifier |
| ファイルパス | `src/features/hash-verify/` |
| 関連 FR | FR-AU-006（ハッシュチェーン週次検証）|
| 関連 SCR | SCR-MC-008（ハッシュチェーン検証）|
| アクセスロール | quality_admin・system_admin |

---

## 2. 型定義

```typescript
// ハッシュチェーン検証結果（CMP-MC-003 HashChainVerifyResult）
export interface HashChainVerificationResult {
  /** チェーン全体の整合性（true = 破断なし）*/
  isValid: boolean;
  /** 検証済みブロック数 */
  checkedBlockCount: number;
  /** 最初に破断を検出したブロック ID（null = 破断なし）*/
  brokenAtBlockId: string | null;
  /** 最終検証完了日時 */
  lastVerifiedAt: Date;
  /** 次回スケジュール検証日時（週次: 毎週月曜 02:00）*/
  nextScheduledAt: Date;
}

// コンポーネント Props
export interface HashChainVerifierProps {
  result: HashChainVerificationResult;
  /** 手動検証実行（quality_admin のみ）*/
  onRunVerification: () => Promise<void>;
}

// 検証ステータス（UI 表示用）
export type VerificationStatus = 'valid' | 'broken' | 'running' | 'unknown';
```

---

## 3. react-query フック定義（FNC-FE-014）

```typescript
import { useQuery, useMutation, UseQueryResult, UseMutationResult } from '@tanstack/react-query';

/**
 * FNC-FE-014: ハッシュチェーン検証結果を取得するフック
 *
 * staleTime: 5 分（週次検証のため高頻度更新は不要）
 * gcTime: 30 分
 */
export declare function useHashChainVerification(): UseQueryResult<
  HashChainVerificationResult,
  Error
>;

/**
 * 手動検証実行 Mutation フック
 * 検証開始後は UI をポーリングモード（refetchInterval: 5000）に切替える
 * 検証完了（isValid が確定）で通常モードに戻す
 */
export declare function useRunVerificationMutation(): UseMutationResult<void, Error, void>;
```

---

## 4. CMP-MC-003 HashChainVerifyResult コンポーネント仕様

```typescript
// CMP-MC-003: 検証結果の表示コンポーネント
export interface HashChainVerifyResultProps {
  result: HashChainVerificationResult;
  /** 検証実行中は true（ポーリング中）*/
  isRunning: boolean;
}

// 検証ステータス判定
export function resolveVerificationStatus(
  result: HashChainVerificationResult,
  isRunning: boolean,
): VerificationStatus {
  if (isRunning) return 'running';
  if (!result.isValid) return 'broken';
  if (result.checkedBlockCount === 0) return 'unknown';
  return 'valid';
}

// ステータス別表示設定
export const STATUS_DISPLAY = {
  valid:   { label: 'チェーン整合性 OK', colorClass: 'text-green-700', iconType: 'check-circle' },
  broken:  { label: 'チェーン破断検出', colorClass: 'text-red-700',   iconType: 'exclamation-circle' },
  running: { label: '検証実行中',       colorClass: 'text-blue-700',  iconType: 'spinner' },
  unknown: { label: '未検証',           colorClass: 'text-gray-500',  iconType: 'question-circle' },
} as const satisfies Record<VerificationStatus, { label: string; colorClass: string; iconType: string }>;
```

---

## 5. コンポーネントツリー

```
HashChainVerifier (MOD-FE-MC-003)
  VerificationStatusBadge（VerificationStatus に応じた色付きバッジ）
  HashChainVerifyResult (CMP-MC-003)
    StatusIconLabel（statusDisplay から icon + label 表示）
    CheckedBlockCount（検証済みブロック数）
    BrokenAtBlockIdPanel（isValid === false 時のみ表示: 破断ブロック ID）
    LastVerifiedAt（最終検証日時・ISO 8601）
    NextScheduledAt（次回スケジュール日時）
  ManualRunPanel（quality_admin のみ表示）
    RunVerificationButton（useRunVerificationMutation を呼び出す）
    RunningSpinner（isRunning 中に表示）
    ConfirmationDialog（「手動検証を開始しますか？」確認ダイアログ）
  VerificationHistoryList（過去 10 件の検証結果履歴）
    HistoryRow (×N)
      VerifiedAt
      CheckedBlockCount
      IsValidBadge
```

---

## 6. ポーリングモード切替ロジック

```typescript
// 手動検証実行中のポーリング制御
const POLLING_INTERVAL_MS = 5_000 as const;
const NORMAL_INTERVAL_MS  = 300_000 as const; // 5 分

export function useVerificationPollingMode(isRunning: boolean): {
  refetchInterval: number;
} {
  return { refetchInterval: isRunning ? POLLING_INTERVAL_MS : NORMAL_INTERVAL_MS };
}
```

---

## 7. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-BIZ-015 | 検証がすでに実行中の重複実行試行 | RunVerificationButton 非活性・「検証実行中です」バナー |
| ERR-AUTH-003 | RBAC 不足（quality_admin 以外の手動実行試行）| ManualRunPanel 非表示 |
| ERR-SYS-001 | 検証 API タイムアウト | エラーバナー・5 分後に再試行可能 |

---

**本節で確定した方針**
- **HashChainVerificationResult の isValid・brokenAtBlockId を discriminated union として扱い、isValid === false の場合のみ brokenAtBlockId が非 null であることを型レベルで保証する設計を確定した。**
- **手動検証実行中は refetchInterval を 5 s に切替えるポーリングモードに移行し、検証完了後（isValid 確定）に 5 分間隔の通常モードに戻すことを確定した。**
- **CMP-MC-003 HashChainVerifyResult を Props 駆動の純粋コンポーネントとして定義し、ネットワーク通信はすべて親の HashChainVerifier が保有するフックに委譲することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
