# 06 RP-004 Kaizen 記録設計

本章は IPA 共通フレーム 2013「2.4 ソフトウェア方式設計プロセス」出力設計タスクに準拠し、帳票 RP-004（Kaizen 記録）のレイアウト・データソース・生成トリガ・個人別集計禁止の技術的担保を確定する。改善提案の可視化と追跡を目的とし、集計は工程・オペレーション・SOP レベルに限定する。個人別集計は BR-BUS-029 により一切禁止であり、本章でその実施手段を設計レベルで確定する。

---

## 1. 概要

Kaizen 記録（RP-004）は、現場作業者から提出された改善提案を電子記録として確定し、承認プロセス・実施結果・効果測定を一元管理するための帳票である。製造現場における継続的改善（Continuous Improvement）活動を可視化し、管理者層が改善活動の状況を工程・オペレーション単位で把握することを目的とする。

### 1-1. 目的と適用範囲

| 項目 | 内容 |
|---|---|
| 帳票 ID | RP-004 |
| 帳票名 | Kaizen 記録 |
| 目的 | 改善提案の可視化・追跡・効果測定 |
| 適用範囲 | 全工程・全オペレーションの Kaizen 提案 |
| 個人別集計 | **禁止（BR-BUS-029）** |
| 集計粒度 | プロセス別・オペレーション別・SOP 別のみ |
| 主利用者 | 現場管理者・品質管理担当・改善推進担当 |
| テンプレート ID | TPL-004 |

### 1-2. 個人別集計禁止の根拠

ビジネスルール BR-BUS-029「個人別生産性ランキングの禁止」に基づき、本帳票は提案者個人を識別・集計する機能を持たない。提案者 ID は SHA-256 ハッシュ値として格納され、帳票上では非表示とする。集計セクションはプロセス別・オペレーション別・SOP 別の改善実施率のみを示す。

---

## 2. レイアウト設計

帳票のビジュアルレイアウトは `img/` 配下の drawio/svg ファイルを参照すること。

> **図参照**: fig_des_report_rp004（img/ 配下）を参照

帳票は論理的に 4 つのセクションで構成される。

### 2-1. ヘッダーセクション

| フィールド名 | データ型 | 必須 | 説明 |
|---|---|---|---|
| Kaizen 提案 ID | VARCHAR(20) | 必須 | KZ-YYYYMMDD-NNNN 形式 |
| 提案日 | DATE | 必須 | 提案が登録された日付 |
| SOP 名 | VARCHAR(200) | 必須 | 対象 SOP の正式名称 |
| SOP バージョン | VARCHAR(20) | 必須 | 例: v3.2.1 |
| 工程名 | VARCHAR(100) | 必須 | 対象工程の名称 |
| オペレーション名 | VARCHAR(100) | 必須 | 対象オペレーションの名称 |

### 2-2. ボディセクション

| フィールド名 | データ型 | 必須 | 説明 |
|---|---|---|---|
| 問題点 | TEXT | 必須 | 現状の課題・問題の具体的記述 |
| 改善案 | TEXT | 必須 | 提案する改善内容の詳細 |
| 期待効果 | TEXT | 必須 | 改善により期待される定性・定量的効果 |
| 承認状態 | ENUM | 必須 | 未審査 / 承認済 / 否決 / 実施中 / 完了 |
| 承認者署名 | BYTEA | 承認時必須 | 電子サインデータ（承認者のみ付与） |
| 実施結果 | TEXT | 実施後必須 | 改善実施後の実績・効果測定結果 |
| 実施日 | DATE | 実施後必須 | 改善が実施された日付 |

### 2-3. 集計セクション（個人別集計は含まない）

集計粒度はプロセス別・オペレーション別に限定する。提案者 ID を GROUP BY 軸に使用することは技術的に禁止される（第 3 節参照）。

| 集計軸 | 集計項目 | 算出方法 |
|---|---|---|
| プロセス別 | 改善提案件数 | COUNT(kaizen_proposals) WHERE process_id = ? |
| プロセス別 | 改善実施率 | COUNT(status = '完了') / COUNT(*) × 100 |
| オペレーション別 | 改善提案件数 | COUNT(kaizen_proposals) WHERE operation_id = ? |
| オペレーション別 | 承認率 | COUNT(status IN ('承認済','実施中','完了')) / COUNT(*) × 100 |
| SOP 別 | 関連 Kaizen 件数 | COUNT(kaizen_proposals) WHERE sop_id = ? |
| SOP 別 | 平均完了日数 | AVG(completed_at - approved_at) |

### 2-4. フッターセクション

| フィールド名 | データ型 | 必須 | 説明 |
|---|---|---|---|
| ドキュメントハッシュ | VARCHAR(64) | 必須 | SHA-256（ヘッダー + ボディ + 集計セクション全体） |
| 出力日時 | TIMESTAMPTZ | 必須 | ISO 8601 形式（UTC+9 表示） |
| 出力者ロール | VARCHAR(50) | 必須 | 出力を実行したユーザーのロール名（個人名は非表示） |

---

## 3. 個人別集計禁止の技術的担保

BR-BUS-029 の遵守を設計レベルで強制するため、以下の 2 つの技術的制御を実装する。

### 3-1. DB ロールによるクエリ制限

| 制御 ID | 内容 |
|---|---|
| DB-ROLE-02 | `noedit_role` は `kaizen_proposals` テーブルに対して `worker_id`・`proposer_id` を GROUP BY 軸に含むクエリの実行権限を持たない |

具体的には、PostgreSQL のロールレベルで以下の制御を適用する。

```sql
-- noedit_role への制限（参考: 実装はDB-ROLE定義書を参照）
-- worker_id / proposer_id を GROUP BY に含む SELECT は DENY
-- 帳票生成ロール report_gen_role も同制限に準拠
REVOKE ALL ON kaizen_proposals FROM noedit_role;
GRANT SELECT (id, sop_id, process_id, operation_id, status,
              proposed_at, approved_at, completed_at, description,
              improvement_plan, expected_effect, result)
  ON kaizen_proposals TO report_gen_role;
-- proposer_id_hash 列は SELECT 権限から除外
```

`proposer_id_hash` 列はロールレベルで SELECT 権限を付与しないことで、アプリケーション層の実装ミスによる誤集計を防止する。

### 3-2. kaizen_proposals テーブルの提案者 ID ハッシュ化

| フィールド名 | 実装 | 説明 |
|---|---|---|
| `proposer_id_hash` | SHA-256(worker_id \|\| salt) | 提案者の worker_id を一方向ハッシュ化して格納 |
| `worker_id`（元値）| 別テーブル管理 | `kaizen_proposals` テーブルには平文で格納しない |
| 逆引き手段 | なし | salt は環境変数管理。ハッシュから worker_id への逆引きは不可能 |

この設計により、万一 `proposer_id_hash` が取得できたとしても個人の同定は不可能であり、個人別集計・ランキングを実施する手段が存在しない。

---

## 4. データソース

RP-004 のデータソースは以下のテーブルを結合して生成する。

| テーブル名 | 用途 | 結合条件 |
|---|---|---|
| `kaizen_proposals` | 改善提案の主データ | 主テーブル |
| `sops` | SOP 名・バージョンの解決 | `kaizen_proposals.sop_id = sops.id` |
| `processes` | 工程名の解決 | `kaizen_proposals.process_id = processes.id` |
| `operations` | オペレーション名の解決 | `kaizen_proposals.operation_id = operations.id` |
| `user_signatures` | 承認者電子サインの取得 | `kaizen_proposals.approver_id = user_signatures.user_id` |

集計クエリは `process_id`・`operation_id`・`sop_id` を GROUP BY 軸とする。`proposer_id_hash` は SELECT 対象から除外し、GROUP BY 軸には使用しない。

---

## 5. 生成トリガ

| 項目 | 内容 |
|---|---|
| バッチ ID | BAT-008 |
| トリガ条件 | Kaizen 提案の承認（`kaizen_proposals.status` が `承認済` に遷移した時点） |
| 生成方式 | イベント駆動（DB トリガ → Rust バックエンドの非同期ジョブキュー） |
| 出力形式 | PDF/A-3（埋め込みメタデータ付き）+ JSON サイドカー |
| 格納先 | NAS: `/reports/RP-004/{YYYY}/{MM}/{kaizen_id}.pdf` |
| エラー時 | リトライ 3 回後にアラートキューへ投入（OPS-ALERT-002） |

---

**本節で確定した方針**

- **RP-004 の集計セクションはプロセス別・オペレーション別・SOP 別に限定し、提案者個人を識別・集計する手段を DB ロール（DB-ROLE-02）とカラム権限制御によって設計レベルで封じ、BR-BUS-029 を技術的に担保する。**
- **`kaizen_proposals.proposer_id_hash` は SHA-256 一方向ハッシュで格納し、逆引き不可能な設計とすることで個人別ランキング生成を構造的に不可能にする。**
- **RP-004 の生成は BAT-008（Kaizen 承認時イベント駆動）により自動起動し、PDF/A-3 形式で NAS に格納、SHA-256 フッターで改ざん防止を担保する。**

---

## 参照業界分析

### 必須

- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)

### 関連

- [`90_業界分析/39_QCサークル・Kaizen Teianとボトムアップ品質活動.md`](../../90_業界分析/39_QCサークル・Kaizen%20Teianとボトムアップ品質活動.md)
- [`90_業界分析/24_作業者プライバシー・データ倫理と労務監視.md`](../../90_業界分析/24_作業者プライバシー・データ倫理と労務監視.md)
