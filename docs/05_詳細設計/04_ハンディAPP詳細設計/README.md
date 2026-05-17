# 04 ハンディ APP 詳細設計

本サブは IPA 共通フレーム 2013「2.5.1 ソフトウェアコンポーネント詳細設計」に準拠し、MOD-FE-HA-001〜008 の各モジュールを TypeScript インターフェース・ステートマシン・SQLite スキーマ・状態管理まで精緻化する。コーディング直前仕様としての完結性を担保する。

---

## IPA 2.5.1 カバレッジ

| IPA 2.5.1 要求タスク | 担当章 | 備考 |
|---|---|---|
| コンポーネントの責務・依存関係の確定 | `00_本書の位置づけと識別子規約.md` | MOD-FE-HA 依存方向・FNC-FE-NNN 採番規約 |
| 関数・メソッドシグネチャの定義 | `01_`〜`06_` | FNC-FE-NNN でトレース |
| データ構造の完全定義 | `07_SQLite_TypeORMスキーマ設計.md` | 全エンティティの列・型・制約 |
| ステートマシン・アルゴリズムの仕様 | `01_StepEngine詳細設計.md` | StepExecutionState × イベント遷移表 |
| エラー処理の詳細設計 | 各章の ERR-NNN 節 | ERR-BIZ/ERR-VAL/ERR-SYS 対応 |
| 設定パラメータの定義 | `05_OutboxWorker詳細設計.md`・`06_LocalDbService詳細設計.md` | CFG-NNN 対応 |
| 状態管理設計 | `08_状態管理（Context_Store）設計.md` | React Context + useReducer |

---

## モジュール → 章 カバレッジ表

| MOD-ID | 物理名 | 担当章 | 関連 FR |
|---|---|---|---|
| MOD-FE-HA-001 | NetworkProvider | `08_状態管理（Context_Store）設計.md` §1 | FR-SY-008/009 |
| MOD-FE-HA-002 | OutboxWorker | `05_OutboxWorker詳細設計.md` | FR-SY-005/006 |
| MOD-FE-HA-003 | StepEngine | `01_StepEngine詳細設計.md` | FR-NV-001〜013 |
| MOD-FE-HA-004 | EvidenceCapture | `02_EvidenceCapture詳細設計.md` | FR-EV-001〜012 |
| MOD-FE-HA-005 | SuspensionFlow | `03_SuspensionFlow詳細設計.md` | FR-ST-001〜012 |
| MOD-FE-HA-006 | AndonKaizenFlow | `04_AndonKaizenFlow詳細設計.md` | FR-KZ-001〜007 |
| MOD-FE-HA-007 | ElectronicSignPad | `01_StepEngine詳細設計.md` §4（サインゲート） | FR-AU-001 |
| MOD-FE-HA-008 | LocalDbService | `06_LocalDbService詳細設計.md` | FR-SY-002〜004 |
| — | SQLite/TypeORM スキーマ全体 | `07_SQLite_TypeORMスキーマ設計.md` | FR-SY-002〜004 |

---

## 読む順序

```
README → 00_本書の位置づけと識別子規約
  → 01_StepEngine詳細設計（コア、最重要）
  → 02_EvidenceCapture詳細設計
  → 03_SuspensionFlow詳細設計
  → 04_AndonKaizenFlow詳細設計
  → 05_OutboxWorker詳細設計
  → 06_LocalDbService詳細設計
  → 07_SQLite_TypeORMスキーマ設計
  → 08_状態管理（Context_Store）設計
  → 99_前提制約と本書が約束しないこと
```

---

## バージョン履歴

| 版 | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-17 | RyuheiKiso | 初版（全 10 章フル執筆） |
