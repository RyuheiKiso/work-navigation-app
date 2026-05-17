# 05 WebAPP 詳細設計

本サブは IPA 共通フレーム 2013「2.5.1 ソフトウェアコンポーネント詳細設計」に準拠し、MOD-FE-MA-001〜006（マスタメンテナンス APP）および MOD-FE-MC-001〜005（管理コンソール）の各モジュールを TypeScript インターフェース・Props 型・状態定義・カスタムフックシグネチャまで精緻化する。コーディング直前仕様としての完結性を担保する。

---

## IPA 2.5.1 カバレッジ

| IPA 2.5.1 要求タスク | 担当章 | 備考 |
|---|---|---|
| コンポーネントの責務・依存関係の確定 | `00_本書の位置づけと識別子規約.md` | MOD-FE-MA/MC 依存方向・FNC-FE-NNN 採番規約 |
| Props・State 型の完全定義 | `01_`〜`10_` | FNC-FE-NNN でトレース |
| ステートマシン・状態管理の仕様 | `04_ApprovalWorkflow詳細設計.md` | ApprovalState × イベント遷移表 |
| エラー処理の詳細設計 | 各章の ERR-NNN 節 | ERR-BIZ/ERR-VAL/ERR-SYS 対応 |
| 非同期コールバック型定義 | `01_`〜`10_` | react-query v5 useMutation/useQuery シグネチャ |
| DSL・アルゴリズムの仕様 | `02_DslConditionBuilder詳細設計.md` / `03_SopFlowEditor詳細設計.md` | JSON Logic ホワイトリスト・DAG 検証 |
| 設定パラメータの定義 | `01_`・`06_` | Auto-Save 間隔・Dashboard 更新間隔 |

---

## モジュール → 章 カバレッジ表

| MOD-ID | 物理名 | 担当章 | 関連 FR |
|---|---|---|---|
| MOD-FE-MA-001 | SopEditor | `01_SopEditor詳細設計.md` | FR-MA-001〜007 |
| MOD-FE-MA-002 | DslConditionBuilder | `02_DslConditionBuilder詳細設計.md` | FR-MA-004/007 |
| MOD-FE-MA-003 | SopFlowEditor | `03_SopFlowEditor詳細設計.md` | FR-MA-016 |
| MOD-FE-MA-004 | ApprovalWorkflow | `04_ApprovalWorkflow詳細設計.md` | FR-MA-008〜010 |
| MOD-FE-MA-005 | MasterVersionDiff | `05_MasterVersionDiff詳細設計.md` | FR-MA-013 |
| MOD-FE-MA-006 | UserRoleAdmin | `00_本書の位置づけと識別子規約.md` §5 | FR-MA-014/015 |
| MOD-FE-MC-001 | OperationDashboard | `06_OperationDashboard詳細設計.md` | OPS-036〜053 |
| MOD-FE-MC-002 | AuditLogViewer | `07_AuditLogViewer詳細設計.md` | FR-AU-004/005 |
| MOD-FE-MC-003 | HashChainVerifier | `08_HashChainVerifier詳細設計.md` | FR-AU-006 |
| MOD-FE-MC-004 | OutboxMonitor | `09_OutboxMonitor詳細設計.md` | FR-SY-007/008 |
| MOD-FE-MC-005 | ReportGenerator | `10_ReportGenerator詳細設計.md` | RP-001〜006 |

---

## 読む順序

```
README → 00_本書の位置づけと識別子規約
  → 01_SopEditor詳細設計（マスタメンテコア、最重要）
  → 02_DslConditionBuilder詳細設計
  → 03_SopFlowEditor詳細設計（Step-DAG ビジュアルフロー編集）
  → 04_ApprovalWorkflow詳細設計
  → 05_MasterVersionDiff詳細設計
  → 06_OperationDashboard詳細設計
  → 07_AuditLogViewer詳細設計
  → 08_HashChainVerifier詳細設計
  → 09_OutboxMonitor詳細設計
  → 10_ReportGenerator詳細設計
  → 99_前提制約と本書が約束しないこと
```

---

## バージョン履歴

| 版 | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-17 | RyuheiKiso | 初版（全 11 章フル執筆） |
| 0.2.0 | 2026-05-17 | RyuheiKiso | MOD-FE-MA-003 SopFlowEditor を新規追加（FR-MA-016）、既存 03〜09 を 04〜10 にシフト |
