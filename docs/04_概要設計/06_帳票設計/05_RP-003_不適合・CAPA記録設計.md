# 05 RP-003 不適合・CAPA記録設計

本章は帳票 RP-003「不適合・CAPA 記録」のレイアウト・データフィールド・生成規則を確定する。RP-003 は ISO 9001:2015 要求事項 10.2（不適合及び是正処置）に基づく規制要求事項への対応を主目的とし、4M 分析（Man / Machine / Material / Method）と CAPA（是正処置・予防処置）プロセスを一帳票に統合した設計とする。

---

## 1. 帳票概要

RP-003 は作業中に発見された不適合事象を品質管理プロセスとして記録・追跡するための電子帳票である。ISO 9001:2015 要求事項 10.2 が求める「不適合の処置・根本原因分析・是正処置の有効性確認」を 1 帳票で完結させ、定期品質監査のエビデンスとして機能させる。

| 項目 | 内容 |
|---|---|
| 帳票 ID | RP-003 |
| 帳票名 | 不適合・CAPA 記録 |
| 準拠規格 | ISO 9001:2015 要求事項 10.2 / ISO 13485:2016 §8.5.2（医療機器向け拡張）|
| 主目的 | 品質監査エビデンス（不適合管理・是正処置・予防処置の記録）|
| 副目的 | 4M 分析による根本原因の組織的把握・Kaizen 活動との連携 |
| 出力形式 | PDF/A-3 |
| テンプレート ID | TPL-003 |
| 生成単位 | nonconformities 1 レコード（不適合 1 件）ごとに 1 帳票 |
| 保存先 | NAS（`/reports/rp003/{year}/{month}/`）+ PostgreSQL ハッシュ参照テーブル |
| 保存期間 | 7 年以上（NFR-OPS-035 準拠）|
| 担当要件 | FR-QM-001〜004, NFR-ALC-001〜009, BR-BUS-015 |

### 1-1. 4M 分析との関連

不適合の原因を以下の 4 つのカテゴリ（4M）に分類して記録する。これにより統計的な傾向分析と体系的な根本原因特定を可能にする。

| カテゴリ | 英語表記 | 対象範囲 |
|---|---|---|
| Man（人）| Man | 作業者のスキル不足・手順逸脱・疲労等 |
| Machine（設備）| Machine | 機械の誤作動・校正不良・設備劣化等 |
| Material（材料）| Material | 原材料の品質不良・規格外・誤使用等 |
| Method（方法）| Method | 作業手順の不備・SOP の記載誤り・条件設定ミス等 |

---

## 2. レイアウト構造

RP-003 は Header・Body（3 セクション）・Footer の 5 部構成とする。各領域の詳細は下図を参照。

> 図 2-1 RP-003 レイアウト構造図（fig_des_report_rp003（img/ 配下）を参照）

### 2-1. Header（不適合識別・発生状況領域）

Header は帳票の上部固定領域に配置し、不適合の一意識別と発生状況のトレーサビリティを確保する。

| フィールド名 | 表示ラベル（日本語）| データ型 | 文字数上限 | ALCOA+ 該当原則 |
|---|---|---|---|---|
| nonconformity_id | 不適合 ID | VARCHAR | 50 | O（原本性）/ A（帰属可能性）|
| occurred_at | 発生日時 | TIMESTAMPTZ | — | C（同時性）|
| detected_by_hash | 発見者 ID（ハッシュ）| CHAR(64) | 64 | A（帰属可能性）/ 個人情報保護 |
| process_name | 工程名 | VARCHAR | 100 | A（帰属可能性）|
| sop_name | SOP 名 | VARCHAR | 200 | A（帰属可能性）|
| sop_version | SOP バージョン | VARCHAR | 20 | O（原本性）|
| step_no | Step 番号 | INTEGER | — | A（帰属可能性）|
| work_execution_id | 関連作業実行 ID | UUID | — | A（帰属可能性）/ RP-001 リンク |

#### 2-1-1. detected_by_hash の算出方法

```
detected_by_hash = SHA-256(detected_by_worker_id || salt_daily)
```

`salt_daily` は不適合発生日（`occurred_at::DATE` を `YYYY-MM-DD` 形式に変換した値）を使用する。

#### 2-1-2. nonconformity_id の採番規則

```
nonconformity_id = NC-{YYYY}-{MM}-{NNN}
  YYYY : 発生年（4 桁）
  MM   : 発生月（2 桁・ゼロ埋め）
  NNN  : 当月連番（3 桁・ゼロ埋め）
  例   : NC-2026-05-001
```

### 2-2. Body セクション 1（不適合内容・4M 分析）

不適合事象の内容とその 4M カテゴリ分類を記録する領域。

| フィールド名 | 表示ラベル | データ型 | 文字数上限 | ALCOA+ 該当原則 |
|---|---|---|---|---|
| category | 不適合カテゴリ | VARCHAR | 50 | A（正確性）|
| description | 不適合内容（詳細説明）| TEXT | 1000 | L（読み取り可能）|
| severity | 重大度 | VARCHAR | 20 | A（正確性）|
| affected_product_code | 影響製品番号 | VARCHAR | 50 | A（帰属可能性）|
| affected_lot_number | 影響ロット番号 | VARCHAR | 50 | A（帰属可能性）|
| cause_man | 4M: Man 分析内容 | TEXT | 500 | A（正確性）|
| cause_machine | 4M: Machine 分析内容 | TEXT | 500 | A（正確性）|
| cause_material | 4M: Material 分析内容 | TEXT | 500 | A（正確性）|
| cause_method | 4M: Method 分析内容 | TEXT | 500 | A（正確性）|
| primary_cause_category | 主要因カテゴリ | VARCHAR | 20 | A（正確性）|

#### 2-2-1. category 列挙値

| 値 | 意味 |
|---|---|
| `PROCESS_DEVIATION` | 工程逸脱 |
| `MEASUREMENT_OOT` | 測定値異常（Out of Trend）|
| `MATERIAL_DEFECT` | 材料不良 |
| `EQUIPMENT_FAILURE` | 設備異常 |
| `PROCEDURE_ERROR` | 手順書誤り |
| `OTHER` | その他 |

#### 2-2-2. severity 列挙値

| 値 | 意味 | CAPA 期限目安 |
|---|---|---|
| `CRITICAL` | 重大（安全・法規制違反）| 24 時間以内 |
| `MAJOR` | 重要（品質への直接影響）| 5 営業日以内 |
| `MINOR` | 軽微（品質への軽微な影響）| 20 営業日以内 |

### 2-3. Body セクション 2（CAPA 詳細）

是正処置（Corrective Action）および予防処置（Preventive Action）の詳細を記録する領域。

| フィールド名 | 表示ラベル | データ型 | 文字数上限 | ALCOA+ 該当原則 |
|---|---|---|---|---|
| capa_id | CAPA ID | VARCHAR | 50 | O（原本性）|
| root_cause_analysis | 根本原因分析 | TEXT | 2000 | A（正確性）/ L（読み取り可能）|
| corrective_action | 是正措置（発生した不適合への対処）| TEXT | 2000 | A（正確性）|
| preventive_action | 予防措置（再発防止策）| TEXT | 2000 | A（正確性）|
| responsible_person_hash | 担当者 ID（ハッシュ）| CHAR(64) | 64 | A（帰属可能性）|
| due_date | 完了期限 | DATE | — | C（同時性）|
| status | CAPA ステータス | VARCHAR | 20 | A（正確性）|
| sop_revision_required | SOP 改訂要否 | BOOLEAN | — | A（正確性）|
| sop_revision_id | 改訂 SOP バージョン ID（改訂時）| UUID | — | A（帰属可能性）|

#### 2-3-1. capa_id の採番規則

```
capa_id = CAPA-{YYYY}-{MM}-{NNN}
  YYYY : 登録年（4 桁）
  MM   : 登録月（2 桁・ゼロ埋め）
  NNN  : 当月連番（3 桁・ゼロ埋め）
  例   : CAPA-2026-05-001
```

#### 2-3-2. status 列挙値

| 値 | 意味 |
|---|---|
| `OPEN` | 未着手 |
| `IN_PROGRESS` | 対応中 |
| `PENDING_VERIFICATION` | 有効性確認待ち |
| `CLOSED` | 完了（有効性確認済み）|
| `OVERDUE` | 期限超過 |

### 2-4. Body セクション 3（有効性確認）

CAPA 完了後の有効性確認結果を記録する領域。有効性確認は是正措置・予防措置の実施が不適合の再発防止に実際に効果があったかを検証する。

| フィールド名 | 表示ラベル | データ型 | 文字数上限 | ALCOA+ 該当原則 |
|---|---|---|---|---|
| verification_date | 確認日 | DATE | — | C（同時性）|
| verification_result | 有効性確認結果 | VARCHAR | 20 | A（正確性）|
| verification_comments | 確認コメント | TEXT | 1000 | L（読み取り可能）|
| verifier_sign_data | 確認者電子サイン | BYTEA | — | A（帰属可能性）|
| follow_up_required | 追加対応要否 | BOOLEAN | — | A（正確性）|
| follow_up_capa_id | 追加 CAPA ID（要対応時）| VARCHAR | 50 | A（帰属可能性）|

#### 2-4-1. verification_result 列挙値

| 値 | 意味 |
|---|---|
| `EFFECTIVE` | 有効（再発なし・処置完了）|
| `PARTIALLY_EFFECTIVE` | 部分的に有効（追加対応要）|
| `NOT_EFFECTIVE` | 無効（再発あり・再 CAPA 要）|

### 2-5. Footer（最終承認・改ざん防止領域）

| フィールド名 | 表示ラベル | データ型 | ALCOA+ 該当原則 |
|---|---|---|---|
| final_sign_data | 品質管理者電子サイン（quality_admin）| BYTEA | A（帰属可能性）|
| document_hash | SHA-256 ハッシュ値 | CHAR(64) | O（原本性）・C（一貫性）|
| exported_at | 出力日時 | TIMESTAMPTZ | C（同時性）|

#### 2-5-1. document_hash の算出対象

```
document_hash = SHA-256(
    nonconformity_id
    || capa_id
    || occurred_at (ISO 8601)
    || detected_by_hash
    || root_cause_analysis
    || corrective_action
    || preventive_action
    || verification_result
    || final_sign_data
)
```

---

## 3. データソースマッピング

RP-003 の各フィールドは以下のテーブルから取得する。

| RP-003 領域 | フィールド | 参照テーブル | 参照カラム | 結合条件 |
|---|---|---|---|---|
| Header | nonconformity_id | nonconformities | id | — |
| Header | occurred_at | nonconformities | occurred_at | — |
| Header | detected_by_hash | workers | id（ハッシュ変換）| nonconformities.detected_by = workers.id |
| Header | process_name | processes | process_name | nonconformities.process_id = processes.id |
| Header | sop_name | sop_versions | name | nonconformities.sop_version_id = sop_versions.id |
| Header | step_no | nonconformities | step_no | — |
| Header | work_execution_id | nonconformities | work_execution_id | — |
| Body S1 | category | nonconformities | category | — |
| Body S1 | description | nonconformities | description | — |
| Body S1 | severity | nonconformities | severity | — |
| Body S1 | cause_man | nonconformities | cause_man | — |
| Body S1 | cause_machine | nonconformities | cause_machine | — |
| Body S1 | cause_material | nonconformities | cause_material | — |
| Body S1 | cause_method | nonconformities | cause_method | — |
| Body S2 | capa_id | capas | id | capas.nonconformity_id = nonconformities.id |
| Body S2 | root_cause_analysis | capas | root_cause_analysis | 同上 |
| Body S2 | corrective_action | capas | corrective_action | 同上 |
| Body S2 | preventive_action | capas | preventive_action | 同上 |
| Body S2 | responsible_person_hash | workers | id（ハッシュ変換）| capas.responsible_person_id = workers.id |
| Body S2 | due_date | capas | due_date | 同上 |
| Body S2 | status | capas | status | 同上 |
| Body S3 | verification_date | capas | verification_date | 同上 |
| Body S3 | verification_result | capas | verification_result | 同上 |
| Body S3 | verifier_sign_data | electronic_signs | sign_data | electronic_signs.capa_id = capas.id AND role = 'verifier' |
| Footer | final_sign_data | electronic_signs | sign_data | electronic_signs.nonconformity_id = nonconformities.id AND role = 'quality_admin' |

### 3-1. 主要テーブル定義（概略）

| テーブル名 | 主要カラム | 説明 |
|---|---|---|
| nonconformities | id, occurred_at, detected_by, process_id, sop_version_id, step_no, category, description, severity, cause_man, cause_machine, cause_material, cause_method | 不適合マスタ |
| capas | id, nonconformity_id, root_cause_analysis, corrective_action, preventive_action, responsible_person_id, due_date, status, verification_date, verification_result | CAPA 詳細 |
| electronic_signs | id, nonconformity_id, capa_id, role, sign_data, signed_at | 電子サイン |

---

## 4. 生成トリガー

RP-003 の生成はバッチ処理 BAT-007 が担当する。

| 項目 | 内容 |
|---|---|
| バッチ ID | BAT-007 |
| 処理名 | 不適合・CAPA 記録 PDF 生成バッチ |
| トリガー条件 | `nonconformities` テーブルへの INSERT（不適合登録時）を PostgreSQL LISTEN/NOTIFY で検知 |
| 処理フロー | (1) nonconformity_id を受取 → (2) nonconformities・capas テーブルからデータ取得 → (3) quality_admin 電子サイン確認（サイン未完了時は下書き PDF を生成し、サイン完了後に正式版を再生成）→ (4) テンプレート TPL-003 へのデータバインド → (5) Rust PDF 生成エンジンで PDF/A-3 出力 → (6) document_hash 算出 → (7) NAS へ格納 → (8) document_hashes テーブルへ登録 |
| 失敗時挙動 | リトライ 3 回。失敗時は `report_generation_failures` テーブルに記録し、ops_admin へ通知 |
| 実行保証 | 冪等設計（同一 nonconformity_id に対して重複生成しない。更新時は新バージョンを採番）|
| CAPA 更新時の再生成 | `capas.status` が `CLOSED` に更新された場合も BAT-007 を起動し、有効性確認結果を追記した最終版を生成する |

### 4-1. 生成フロー概略

> 図 4-1 BAT-007 生成フロー（fig_des_report_rp003（img/ 配下）を参照）

### 4-2. 帳票バージョン管理

RP-003 は不適合登録時（初版）と CAPA クローズ時（最終版）の 2 時点で生成される。帳票バージョンは以下の規則で管理する。

| バージョン | 生成タイミング | document_hash の状態 |
|---|---|---|
| v1（下書き）| 不適合登録直後（quality_admin 未署名）| 未確定（draft フラグ = true）|
| v2（暫定版）| quality_admin が不適合内容に署名完了 | 確定（CAPA セクションは空白）|
| v3（最終版）| CAPA クローズ・有効性確認完了・quality_admin 再署名 | 確定（全セクション充足）|

---

**本節で確定した方針**
- **RP-003 は ISO 9001:2015 要求事項 10.2 準拠の不適合・CAPA 記録帳票として PDF/A-3 で生成し、Header / Body（4M 分析・CAPA 詳細・有効性確認の 3 セクション）/ Footer の 5 部構成・TPL-003 テンプレートを使用する。発見者・担当者は SHA-256 ハッシュのみ帳票に出力する。**
- **BAT-007 は nonconformities テーブルへの INSERT を LISTEN/NOTIFY で検知して起動し、不適合登録時（v2 暫定版）と CAPA クローズ時（v3 最終版）の 2 時点で帳票を生成する。各バージョンは document_hash で改ざん検知を保証する。**
- **4M 分析（Man / Machine / Material / Method）フィールドを構造化して収録することで、複数不適合件数の統計的傾向分析（主要因カテゴリ別集計）および Kaizen 活動（RP-004）との連携を可能にする。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../90_業界分析/21_電子記録の法規制とALCOA+.md)
- [`90_業界分析/23_不適合管理と是正処置（CAPA）.md`](../../90_業界分析/23_不適合管理と是正処置（CAPA）.md)

### 関連
- [`90_業界分析/07_プロセスマイニングとXES標準.md`](../../90_業界分析/07_プロセスマイニングとXES標準.md)
