# 06 AuditLogViewer 詳細設計

本章は MOD-FE-MC-002（AuditLogViewer）の TypeScript インターフェース・フィルタ型定義・ページネーション仕様・XES/CSV エクスポートフック・コンポーネントツリーを確定する。AuditLogViewer は FR-AU-004/005 で要求される監査ログ閲覧・XES エクスポートを担い、SCR-MC-004/005（監査ログ閲覧・XES エクスポート）の UI ロジックを提供する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-002 |
| 物理名 | AuditLogViewer |
| ファイルパス | `src/features/audit/` |
| 関連 FR | FR-AU-004（監査ログ閲覧）・FR-AU-005（XES エクスポート）|
| 関連 SCR | SCR-MC-004（監査ログ閲覧）・SCR-MC-005（XES エクスポート）|
| アクセスロール | quality_admin・system_admin |

---

## 2. 型定義

```typescript
// 監査ログフィルタ条件
export interface AuditLogFilter {
  /** WorkEvent イベント種別（null = 全種別）*/
  eventType?: string;
  /** 操作ユーザー UUID（null = 全ユーザー）*/
  userId?: string;
  /** 期間開始（null = 制限なし）*/
  startDate?: Date;
  /** 期間終了（null = 制限なし）*/
  endDate?: Date;
  /** ケース ID（null = 全ケース）*/
  caseId?: string;
}

// 監査ログエントリ（TBL-001 work_events から変換）
export interface AuditLogEntry {
  eventId: string;
  caseId: string;
  eventType: string;
  /** 操作ユーザー（resource カラム）*/
  userId: string;
  userDisplayName: string;
  /** サーバー記録時刻（ALCOA+ Contemporaneous 基準）*/
  timestampServer: Date;
  payload: Record<string, unknown>;
  /** SHA-256 コンテンツハッシュ（FR-EV-001）*/
  contentHash: string;
}

// ページネーション結果
export interface AuditLogPage {
  entries: AuditLogEntry[];
  totalCount: number;
  page: number;
  perPage: number;
  totalPages: number;
}

// コンポーネント Props
export interface AuditLogViewerProps {
  /** XES XML ダウンロード（FR-AU-005）*/
  onExportXes: (filter: AuditLogFilter) => Promise<void>;
  /** CSV ダウンロード */
  onExportCsv: (filter: AuditLogFilter) => Promise<void>;
}

// ページネーション設定定数
export const DEFAULT_PER_PAGE = 50 as const;
export const PER_PAGE_OPTIONS = [20, 50, 100, 200] as const;
```

---

## 3. react-query フック定義

### 3-1. 監査ログ取得フック（FNC-FE-012）

```typescript
import { useQuery, UseQueryResult } from '@tanstack/react-query';

/**
 * FNC-FE-012: 監査ログを取得する react-query フック
 *
 * @param filter - フィルタ条件
 * @param page   - ページ番号（1 始まり）
 * @param perPage - 1 ページあたりの件数（デフォルト 50）
 *
 * staleTime: 30 s（監査ログは高頻度更新のため短め）
 * keepPreviousData: true（ページ遷移時のちらつき防止）
 */
export declare function useAuditLogQuery(
  filter: AuditLogFilter,
  page: number,
  perPage?: number,
): UseQueryResult<AuditLogPage, Error>;
```

### 3-2. XES エクスポートフック（FNC-FE-013）

```typescript
import { useMutation, UseMutationResult } from '@tanstack/react-query';

/**
 * FNC-FE-013: XES XML をサーバーサイドで生成しダウンロードする Mutation フック
 * 内部で onExportXes コールバックを呼び出す
 * ダウンロードファイル名: audit-{caseId or 'all'}-{YYYYMMDD}.xes
 */
export declare function useXesExport(): UseMutationResult<
  void,
  Error,
  { filter: AuditLogFilter; onExport: (filter: AuditLogFilter) => Promise<void> }
>;

/**
 * CSV エクスポート Mutation フック
 * ダウンロードファイル名: audit-{YYYYMMDD}.csv
 */
export declare function useCsvExport(): UseMutationResult<
  void,
  Error,
  { filter: AuditLogFilter; onExport: (filter: AuditLogFilter) => Promise<void> }
>;
```

---

## 4. コンポーネントツリー

```
AuditLogViewer (MOD-FE-MC-002)
  AuditLogFilterPanel
    EventTypeSelect（WorkEvent 種別プルダウン）
    UserSearchInput（ユーザー名前方一致検索）
    DateRangePicker（startDate / endDate）
    CaseIdInput（ケース ID 入力）
    FilterApplyButton
    FilterResetButton
  ExportButtonGroup
    XesExportButton（useXesExport Mutation）
    CsvExportButton（useCsvExport Mutation）
  AuditLogTable（useAuditLogQuery の結果を表示）
    AuditLogTableHeader（列: 時刻 / ケースID / イベント種別 / ユーザー / ハッシュ）
    AuditLogTableRow (×N)
      TimestampCell（ISO 8601 表示）
      CaseIdCell（クリックで caseId フィルタ適用）
      EventTypeBadge
      UserCell
      ContentHashCell（先頭 8 文字表示・クリックでフルハッシュコピー）
  PaginationControls（page / totalPages / perPage セレクタ）
```

---

## 5. Zustand フィルタストア

AuditLogFilter の状態は Zustand ローカルストアで管理し、URL クエリパラメータと同期する。

```typescript
import { create } from 'zustand';

export interface AuditLogFilterStore {
  filter: AuditLogFilter;
  page: number;
  perPage: number;
  setFilter: (patch: Partial<AuditLogFilter>) => void;
  resetFilter: () => void;
  setPage: (page: number) => void;
  setPerPage: (perPage: number) => void;
}

const DEFAULT_FILTER: AuditLogFilter = {};

export const useAuditLogFilterStore = create<AuditLogFilterStore>((set) => ({
  filter: DEFAULT_FILTER,
  page: 1,
  perPage: DEFAULT_PER_PAGE,
  setFilter: (patch) =>
    set((state) => ({ filter: { ...state.filter, ...patch }, page: 1 })),
  resetFilter: () => set({ filter: DEFAULT_FILTER, page: 1 }),
  setPage: (page) => set({ page }),
  setPerPage: (perPage) => set({ perPage, page: 1 }),
}));
```

---

## 6. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-VAL-040 | startDate > endDate | DateRangePicker バリデーション・フィルタ適用ブロック |
| ERR-VAL-041 | エクスポート期間が 1 年超過 | 「期間を 1 年以内に絞ってください」バリデーション |
| ERR-SYS-001 | API タイムアウト | テーブルにスケルトンローダー・エラーバナー |
| ERR-AUTH-003 | RBAC 不足 | XES エクスポートボタン非表示 |

---

**本節で確定した方針**
- **ページネーションは page + perPage（デフォルト 50、上限 200）方式とし、keepPreviousData: true でページ遷移時のちらつきを防止することを確定した。**
- **XES エクスポートはサーバーサイド生成方式とし、フロントエンドは onExportXes コールバック経由で Blob ダウンロードを受け取る設計を確定した（FR-AU-005 準拠）。**
- **AuditLogFilter の状態は Zustand ストアで管理し、フィルタ変更時に page を 1 にリセットすることで不整合なページ表示を防止することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
