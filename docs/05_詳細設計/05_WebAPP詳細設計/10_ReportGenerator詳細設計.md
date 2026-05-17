# 09 ReportGenerator 詳細設計

本章は MOD-FE-MC-005（ReportGenerator）の TypeScript インターフェース・帳票種別定義・パラメータ型・react-query Mutation フック・ダウンロードフロー仕様を確定する。ReportGenerator は RP-001〜006 で要求される全帳票の出力フォームを担い、SCR-MC-009（帳票出力）の UI ロジックを提供する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-005 |
| 物理名 | ReportGenerator |
| ファイルパス | `src/features/reports/` |
| 関連 ID | RP-001〜006（帳票 6 種）|
| 関連 SCR | SCR-MC-009（帳票出力）|
| アクセスロール | quality_admin・system_admin |

---

## 2. 型定義

```typescript
// 帳票種別（RP-001〜006 全 6 種）
export type ReportType =
  | 'RP-001'   // 作業完了記録（ケース単位）
  | 'RP-002'   // SOP 変更履歴レポート
  | 'RP-003'   // 不適合・アンドン集計レポート
  | 'RP-004'   // スキルトレーサビリティレポート
  | 'RP-005'   // 電子サイン監査レポート
  | 'RP-006';  // 日次作業サマリレポート

// 帳票パラメータ（全帳票共通 + 帳票固有）
export interface ReportParams {
  startDate: Date;
  endDate: Date;
  /** プロセス絞り込み（null = 全プロセス）*/
  processId?: string;
  /** SOP 絞り込み（null = 全 SOP、RP-001/002 で有効）*/
  sopId?: string;
  /** 出力フォーマット */
  format: ReportFormat;
}

// 出力フォーマット（帳票種別により利用可能フォーマットが異なる）
export type ReportFormat = 'pdf' | 'xlsx' | 'xes' | 'csv';

// コンポーネント Props
export interface ReportGeneratorProps {
  /** 帳票生成・Blob 取得コールバック */
  onGenerate: (reportType: ReportType, params: ReportParams) => Promise<Blob>;
  /** Blob のブラウザダウンロード実行 */
  onDownload: (blob: Blob, filename: string) => void;
}
```

### 2-1. 帳票種別 × 利用可能フォーマット

| ReportType | pdf | xlsx | xes | csv | 備考 |
|---|---|---|---|---|---|
| RP-001 | ○ | ○ | ○ | ○ | XES = プロセスマイニング用 |
| RP-002 | ○ | ○ | — | ○ | SOP 変更履歴 |
| RP-003 | ○ | ○ | — | ○ | 不適合・アンドン集計 |
| RP-004 | ○ | ○ | — | ○ | スキルトレーサビリティ |
| RP-005 | ○ | ○ | — | ○ | 電子サイン監査 |
| RP-006 | ○ | ○ | — | ○ | mv_daily_work_summary 基準 |

```typescript
// 帳票種別ごとの利用可能フォーマット定義
export const AVAILABLE_FORMATS: Record<ReportType, ReadonlyArray<ReportFormat>> = {
  'RP-001': ['pdf', 'xlsx', 'xes', 'csv'],
  'RP-002': ['pdf', 'xlsx', 'csv'],
  'RP-003': ['pdf', 'xlsx', 'csv'],
  'RP-004': ['pdf', 'xlsx', 'csv'],
  'RP-005': ['pdf', 'xlsx', 'csv'],
  'RP-006': ['pdf', 'xlsx', 'csv'],
} as const;
```

---

## 3. react-query フック定義（FNC-FE-016）

```typescript
import { useMutation, UseMutationResult } from '@tanstack/react-query';

/**
 * FNC-FE-016: 帳票生成・ダウンロード Mutation フック
 *
 * @returns Mutation フックと、ダウンロードハンドラ
 *
 * 生成中はプログレスバーを表示する
 * Blob 取得後に onDownload を呼び出してブラウザにダウンロードさせる
 * ファイル名: {reportType}-{YYYYMMDD-HHmmss}.{ext}
 */
export declare function useReportDownload(props: {
  onGenerate: (reportType: ReportType, params: ReportParams) => Promise<Blob>;
  onDownload: (blob: Blob, filename: string) => void;
}): {
  generate: UseMutationResult<
    void,
    Error,
    { reportType: ReportType; params: ReportParams }
  >;
  /** 生成中の進捗（0.0〜1.0、バックエンドが進捗を返さない場合は undefined）*/
  progress: number | undefined;
};
```

### 3-1. ファイル名生成ロジック

```typescript
// 出力フォーマット → ファイル拡張子マッピング
const FORMAT_EXTENSION: Record<ReportFormat, string> = {
  pdf:  'pdf',
  xlsx: 'xlsx',
  xes:  'xes',
  csv:  'csv',
} as const;

/**
 * ダウンロードファイル名を生成する
 * 例: RP-001-20260517-143022.pdf
 */
export function buildReportFilename(
  reportType: ReportType,
  format: ReportFormat,
  generatedAt: Date,
): string {
  const timestamp = generatedAt
    .toISOString()
    .replace(/[-:]/g, '')
    .replace('T', '-')
    .slice(0, 15);
  return `${reportType}-${timestamp}.${FORMAT_EXTENSION[format]}`;
}
```

---

## 4. コンポーネントツリー

```
ReportGenerator (MOD-FE-MC-005)
  ReportTypeSelector（RP-001〜006 ラジオボタン、各種別に説明ラベル付き）
  ReportParamsForm（パラメータ入力フォーム）
    DateRangePicker（startDate / endDate）
    ProcessSelector（processId、null = 全プロセス）
    SopSelector（sopId、RP-001/002 で表示）
    FormatSelector（AVAILABLE_FORMATS[reportType] の中からラジオ選択）
  GenerateButton（useReportDownload.generate を呼び出す）
  ProgressBar（generate.isPending 中に表示）
  DownloadCompleteToast（成功後 3 s 表示）
```

---

## 5. バリデーション仕様

```typescript
import { z } from 'zod';

// Zod スキーマ（react-hook-form と組み合わせて使用）
export const reportParamsSchema = z
  .object({
    startDate: z.date(),
    endDate: z.date(),
    processId: z.string().uuid().optional(),
    sopId: z.string().uuid().optional(),
    format: z.enum(['pdf', 'xlsx', 'xes', 'csv']),
  })
  .refine((data) => data.startDate <= data.endDate, {
    message: '終了日は開始日以降を指定してください',
    path: ['endDate'],
  })
  .refine(
    (data) => {
      const diffMs = data.endDate.getTime() - data.startDate.getTime();
      const diffDays = diffMs / (1000 * 60 * 60 * 24);
      return diffDays <= 366;
    },
    { message: '期間は 1 年以内に指定してください', path: ['endDate'] },
  );
```

---

## 6. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-050 | startDate > endDate | Zod バリデーション・送信ブロック |
| ERR-VAL-051 | 期間が 1 年超過 | Zod バリデーション・送信ブロック |
| ERR-VAL-052 | ReportType に対して無効なフォーマット選択 | FormatSelector で該当フォーマットを非活性表示 |
| ERR-BIZ-025 | 指定期間に対象データが存在しない | 「対象データがありません」ダイアログ（Blob サイズ 0 チェック）|
| ERR-SYS-001 | 帳票生成タイムアウト（PDF/XES は生成時間が長い場合あり）| プログレスバー継続・タイムアウトバナー（60 s 後）|
| ERR-AUTH-003 | RBAC 不足 | ReportGenerator ページへのルートアクセス拒否 |

---

**本節で確定した方針**
- **利用可能フォーマットを AVAILABLE_FORMATS として帳票種別ごとに静的定義し、FormatSelector は選択中の reportType に応じて有効なフォーマットのみを表示・選択可能とすることを確定した。**
- **帳票生成は Mutation フックで実装し、生成完了後に onDownload コールバック経由でブラウザダウンロードを実行する。ファイル名は buildReportFilename で一意に生成することを確定した。**
- **ReportParams の Zod スキーマで startDate ≤ endDate かつ期間 ≤ 366 日のバリデーションを行い、不正なパラメータが API に送信されることを防止することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
