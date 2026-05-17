# 04 RP-002 監査ログ出力（XES/CSV）設計

本章は帳票 RP-002「監査ログ出力」の XES XML 形式および CSV 形式の仕様を確定する。RP-002 は IEEE 1849-2016（XES 標準規格）に準拠した構造を持ち、ProM・Celonis・Disco 等の主要プロセスマイニングツールとの相互運用性を保証する。エクスポートされたデータはプロセスマイニングによる作業フローの可視化・ボトルネック分析・コンプライアンス検証に使用する。

---

## 1. 帳票概要

RP-002 は工場内で発生した作業イベントを IEEE 1849-2016 XES 形式および CSV 形式で出力する監査ログ帳票である。プロセスマイニングを通じた QMS 継続的改善（ISO 9001:2015 要求事項 10.3）と規制当局への監査証拠提供を主目的とする。

| 項目 | 内容 |
|---|---|
| 帳票 ID | RP-002 |
| 帳票名 | 監査ログ出力（XES/CSV）|
| 準拠規格 | IEEE 1849-2016（XES: eXtensible Event Stream）|
| 対応ツール | ProM 6.x / Celonis EMS / Disco 3.x 以降 |
| 出力形式 | XES XML（.xes）+ CSV（.csv）を ZIP で一括提供 |
| テンプレート ID | TPL-002 |
| 生成単位 | 期間指定（開始日〜終了日）による一括エクスポート |
| 保存先 | 一時生成（ダウンロード後自動削除）+ 監査ログ出力履歴を PostgreSQL に記録 |
| 担当要件 | FR-AU-005, FR-AU-006, NFR-ALC-001〜009 |

### 1-1. 主要な設計制約

| 制約 | 内容 |
|---|---|
| 個人情報匿名化必須 | worker_id は SHA-256 ハッシュのみ出力。氏名・平文 ID は含めない |
| XES 準拠 | IEEE 1849-2016 の必須属性を全件付与。XES Validator による自動検証を CI に組み込む |
| 出力操作ログ | エクスポート実行者・期間・件数を audit_export_logs テーブルに記録（FR-AU-006）|
| 大容量対応 | 1 回の出力上限は 500,000 イベント行。超過時はページング分割 ZIP を提供 |

---

## 2. XES 構造

XES（eXtensible Event Stream）は IEEE 1849-2016 で標準化されたプロセスマイニング用のイベントログ形式である。RP-002 における XES 構造の全体を下図に示す。

> 図 2-1 RP-002 XES 構造図（fig_des_report_rp002（img/ 配下）を参照）

### 2-1. XES 必須属性マッピング

IEEE 1849-2016 が定める 4 つの必須属性を以下のとおりマッピングする。

| XES 標準属性 | 意味 | 参照カラム | テーブル | 変換規則 |
|---|---|---|---|---|
| `concept:name` | イベント名（アクティビティ名）| event_type | work_events | そのまま使用（例: `STEP_COMPLETE`）|
| `time:timestamp` | イベント発生時刻 | occurred_at | work_events | ISO 8601 + タイムゾーン付き（例: `2026-05-17T09:23:41+09:00`）|
| `org:resource` | 実行リソース（作業者）| worker_id | workers | SHA-256(worker_id \|\| current_date) に置換 |
| `lifecycle:transition` | ライフサイクル遷移 | status | work_events | `STEP_START` → `start`, `STEP_COMPLETE` → `complete`, `STEP_SKIP` → `complete`（skip）|

### 2-2. XES 拡張属性

標準拡張機能（org / concept）および本システム固有の名前空間 `wnav:` を定義する。

#### 2-2-1. org 拡張（org Extension）

| XES 属性 | 意味 | 参照カラム | テーブル |
|---|---|---|---|
| `org:group` | プロセス名（工程グループ）| process_name | processes |

#### 2-2-2. concept 拡張（concept Extension）

| XES 属性 | 意味 | 参照カラム | テーブル |
|---|---|---|---|
| `concept:instance` | ケースインスタンス ID | work_execution_id | work_events |

#### 2-2-3. wnav: カスタム属性

本システム固有の属性は名前空間 `wnav:` で定義し、XES のグローバル拡張として宣言する。

| XES 属性 | 意味 | データ型 | 参照カラム | テーブル |
|---|---|---|---|---|
| `wnav:step_id` | Step 識別子 | string | sop_step_id | work_events |
| `wnav:sop_version_id` | SOP バージョン識別子 | string | sop_version_id | work_executions |
| `wnav:terminal_id` | 実行端末識別子 | string | terminal_id | work_executions |
| `wnav:is_offline` | オフライン実行フラグ | boolean | is_offline | work_events |
| `wnav:sync_lag_ms` | オフライン同期遅延（ミリ秒）| int | sync_lag_ms | work_events |

### 2-3. XES ファイル構造例

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="nested-attributes"
     xmlns="http://www.xes-standard.org/"
     xmlns:concept="http://www.xes-standard.org/concept.xesext"
     xmlns:time="http://www.xes-standard.org/time.xesext"
     xmlns:org="http://www.xes-standard.org/org.xesext"
     xmlns:lifecycle="http://www.xes-standard.org/lifecycle.xesext"
     xmlns:wnav="http://wnav.example.com/xes-ext/1.0">

  <!-- グローバル属性定義 -->
  <global scope="event">
    <string key="concept:name"       value="__UNDEFINED__"/>
    <date   key="time:timestamp"     value="1970-01-01T00:00:00.000+00:00"/>
    <string key="org:resource"       value="__UNDEFINED__"/>
    <string key="lifecycle:transition" value="complete"/>
    <string key="org:group"          value="__UNDEFINED__"/>
    <string key="concept:instance"   value="__UNDEFINED__"/>
    <string key="wnav:step_id"       value="__UNDEFINED__"/>
    <string key="wnav:sop_version_id" value="__UNDEFINED__"/>
    <string key="wnav:terminal_id"   value="__UNDEFINED__"/>
    <boolean key="wnav:is_offline"   value="false"/>
    <int    key="wnav:sync_lag_ms"   value="0"/>
  </global>

  <!-- ケース（= 1 作業実行）-->
  <trace>
    <string key="concept:name" value="{work_execution_id}"/>
    <!-- イベント（= 1 Step 実行）-->
    <event>
      <string  key="concept:name"         value="STEP_COMPLETE"/>
      <date    key="time:timestamp"       value="2026-05-17T09:23:41+09:00"/>
      <string  key="org:resource"         value="{SHA-256ハッシュ}"/>
      <string  key="lifecycle:transition" value="complete"/>
      <string  key="org:group"            value="組立工程A"/>
      <string  key="concept:instance"     value="{work_execution_id}"/>
      <string  key="wnav:step_id"         value="{sop_step_id}"/>
      <string  key="wnav:sop_version_id"  value="{sop_version_id}"/>
      <string  key="wnav:terminal_id"     value="{terminal_id}"/>
      <boolean key="wnav:is_offline"      value="false"/>
      <int     key="wnav:sync_lag_ms"     value="0"/>
    </event>
  </trace>
</log>
```

---

## 3. データ抽出クエリ概略

RP-002 の XES/CSV 生成に使用するデータ抽出クエリの概略を示す。

### 3-1. 抽出対象テーブルとマッピング

| XES 要素 | 参照テーブル | 主要カラム | 結合条件 |
|---|---|---|---|
| trace（ケース）| work_executions | id, sop_version_id, lot_id, started_at | — |
| event（イベント）| work_events | id, work_execution_id, event_type, occurred_at, is_offline, sync_lag_ms | work_events.work_execution_id = work_executions.id |
| org:resource（匿名化作業者）| workers | id（ハッシュ変換）| work_executions.worker_id = workers.id |
| org:group（工程名）| processes | process_name | work_executions.process_id = processes.id |
| wnav:terminal_id（端末）| terminals | device_id | work_executions.terminal_id = terminals.id |
| wnav:step_id（Step ID）| sop_steps | id | work_events.sop_step_id = sop_steps.id |

### 3-2. 抽出クエリ骨格

```sql
SELECT
    we.id                                                    AS work_execution_id,
    wev.id                                                   AS event_id,
    wev.event_type                                           AS concept_name,
    wev.occurred_at                                          AS time_timestamp,
    encode(digest(w.id::text || $export_date::text, 'sha256'), 'hex')
                                                             AS org_resource,
    CASE wev.event_type
        WHEN 'STEP_START'    THEN 'start'
        WHEN 'STEP_COMPLETE' THEN 'complete'
        WHEN 'STEP_SKIP'     THEN 'complete'
        ELSE 'complete'
    END                                                      AS lifecycle_transition,
    p.process_name                                           AS org_group,
    we.id                                                    AS concept_instance,
    wev.sop_step_id                                          AS wnav_step_id,
    we.sop_version_id                                        AS wnav_sop_version_id,
    t.device_id                                              AS wnav_terminal_id,
    wev.is_offline                                           AS wnav_is_offline,
    COALESCE(wev.sync_lag_ms, 0)                             AS wnav_sync_lag_ms
FROM work_events       wev
JOIN work_executions   we  ON we.id  = wev.work_execution_id
JOIN workers           w   ON w.id   = we.worker_id
JOIN processes         p   ON p.id   = we.process_id
JOIN terminals         t   ON t.id   = we.terminal_id
WHERE wev.occurred_at >= $start_date
  AND wev.occurred_at <  $end_date
ORDER BY we.id, wev.occurred_at;
```

---

## 4. CSV 形式

XES と同じデータを平坦化した CSV 形式でも提供する。プロセスマイニングツールが XES を直接インポートできない場合や、Excel での簡易確認に使用する。

### 4-1. CSV カラム定義

| 列番号 | カラム名 | データ型 | 説明 |
|---|---|---|---|
| 1 | case_id | UUID | ケース ID（= work_execution_id）|
| 2 | event_id | UUID | イベント ID（= work_events.id）|
| 3 | concept_name | VARCHAR | イベント名（XES concept:name と同値）|
| 4 | time_timestamp | TIMESTAMPTZ | イベント発生時刻（ISO 8601）|
| 5 | org_resource | CHAR(64) | 作業者 SHA-256 ハッシュ |
| 6 | lifecycle_transition | VARCHAR | ライフサイクル遷移（start / complete）|
| 7 | org_group | VARCHAR | 工程名 |
| 8 | wnav_step_id | UUID | Step ID |
| 9 | wnav_sop_version_id | UUID | SOP バージョン ID |
| 10 | wnav_terminal_id | VARCHAR | 端末識別子 |
| 11 | wnav_is_offline | BOOLEAN | オフライン実行フラグ |
| 12 | wnav_sync_lag_ms | INTEGER | 同期遅延（ミリ秒）|

### 4-2. CSV フォーマット規約

| 項目 | 規約 |
|---|---|
| 文字コード | UTF-8（BOM なし）|
| 区切り文字 | カンマ（,）|
| 改行コード | LF（UNIX 形式）|
| ヘッダー行 | 1 行目にカラム名を出力 |
| 日時形式 | ISO 8601 拡張形式（`2026-05-17T09:23:41+09:00`）|
| NULL 値 | 空文字（`""`）で表現 |
| 文字列クォート | ダブルクォート（必要な場合のみ）|

---

## 5. エクスポートインターフェース

RP-002 のエクスポートは管理コンソール画面 SCR-MC-005「監査ログエクスポート」から実行する。

| 項目 | 内容 |
|---|---|
| 操作画面 | SCR-MC-005（管理コンソール > 監査 > ログエクスポート）|
| アクセス権限 | quality_admin ロール以上 |
| 期間選択 | 開始日・終了日をカレンダーピッカーで指定（最大 93 日間）|
| 出力形式選択 | XES のみ / CSV のみ / XES + CSV（ZIP）の 3 択 |
| ダウンロード形式 | ZIP アーカイブ（ファイル名: `rp002_{start}_{end}_{uuid}.zip`）|
| 担当機能要件 | FR-AU-005（XES エクスポート）|
| 出力操作ログ要件 | FR-AU-005 に基づき audit_export_logs テーブルへ記録 |

### 5-1. エクスポートフロー

> 図 5-1 RP-002 エクスポートフロー（fig_des_report_rp002（img/ 配下）を参照）

### 5-2. audit_export_logs テーブル仕様

| カラム名 | データ型 | 説明 |
|---|---|---|
| id | UUID | 操作ログ ID |
| exported_by | UUID | エクスポート実行者 ID（workers.id）|
| exported_at | TIMESTAMPTZ | エクスポート実行日時 |
| period_start | DATE | 出力対象期間開始日 |
| period_end | DATE | 出力対象期間終了日 |
| event_count | INTEGER | エクスポートしたイベント件数 |
| format | VARCHAR | 出力形式（`XES` / `CSV` / `ZIP`）|
| file_hash | CHAR(64) | 出力ファイルの SHA-256 ハッシュ |

---

## 6. プライバシー保護

RP-002 は個人情報保護の観点から、作業者を特定できる情報を出力ファイルに含めない。

| 保護対象情報 | 処理方法 | 実装箇所 |
|---|---|---|
| 作業者 ID（UUID）| SHA-256(worker_id \|\| export_date) に置換 | データ抽出クエリ（Section 3-2）|
| 作業者氏名 | CSV・XES いずれにも出力しない | クエリに workers.name を含めない |
| 作業者の所属部署 | 出力しない（org:group は工程名のみ）| processes.process_name のみ使用 |
| 生体情報（電子サイン）| 出力しない | electronic_signs テーブルは参照しない |

### 6-1. ハッシュ化の詳細

```
org_resource = SHA-256(worker_id || export_date)
  worker_id   : workers.id（UUID 文字列）
  export_date : エクスポート実行日（YYYY-MM-DD）
               ※ 抽出期間の開始日ではなくエクスポート実行日を使用する
```

エクスポート日を salt に使用することで、同一期間のデータを異なる日に再エクスポートすると org_resource の値が変化する。これにより複数回のエクスポート結果を突き合わせた名寄せを防止する。

---

**本節で確定した方針**
- **RP-002 は IEEE 1849-2016 XES 準拠のイベントログとして work_events テーブルから直接生成し、標準属性 4 件（concept:name / time:timestamp / org:resource / lifecycle:transition）と wnav: カスタム属性 5 件を付与する。XES Validator による自動検証を CI に組み込む。**
- **org:resource（作業者）は SHA-256(worker_id || エクスポート実行日) でハッシュ化し、平文 ID・氏名・生体情報は出力ファイルに一切含めない。同一期間の再エクスポートで値が変化する設計により名寄せを防止する。**
- **エクスポートは SCR-MC-005 から quality_admin ロールのみ実行可能とし、実行者・期間・件数・出力ファイルハッシュを audit_export_logs テーブルに記録して FR-AU-005 のトレーサビリティ要件を充足する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../90_業界分析/21_電子記録の法規制とALCOA+.md)
- [`90_業界分析/07_プロセスマイニングとXES標準.md`](../../90_業界分析/07_プロセスマイニングとXES標準.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
