# 05 マスタ初期投入手順（CSV/Excel）

本章の責務は、INST-A4-e（マスタ初期投入実行）および INST-A4-f（LEGACY_IMPORT イベント生成）に基づき、工程・作業・製品・ユーザー・SOP の各マスタを CSV/Excel テンプレートを用いて本番 DB に投入し、LEGACY_IMPORT イベントをハッシュチェーンに連結して移行データの改竄検知可能性を確立することである。DES-MIG-076〜085 で確定した移行ツール構成・バリデーション規則・エラーレポート仕様に従い、全マスタを品質ゲート通過後に確定させる。

---

## 1 本章の責務と設計文書との対応

### 1-1. INST-A4-e+f との対応関係

本章は IPA 共通フレーム 2013「3.2 導入プロセス」タスク INST-A4-e（マスタ初期投入実行）および INST-A4-f（LEGACY_IMPORT）に対応する実施手順書である。DES-MIG-076〜085（移行ツール構成）が設計した内容の実施版として位置づける。

**MIG-X-136**: 本章の全手順を実施し、MIG-CK-076〜082 の全チェックを通過した時点をもって INST-A4-e+f の完了と判定する。（DES-MIG-076 対応）

| 設計 ID | 設計内容 | 本章での実施箇所 |
|---|---|---|
| DES-MIG-076 | 移行ツール 4 コンポーネント構成 | §3・§4 |
| DES-MIG-078 | インポートフロー 10 段階 | §4 |
| DES-MIG-079 | 大量データのチャンク処理 | §4 |
| DES-MIG-080 | 対応ファイル形式 | §3 |
| DES-MIG-081 | エクスポート方式 3 種 | §6 |
| DES-MIG-082 | エクスポートファイル仕様 | §6 |
| DES-MIG-083 | テンプレートファイル仕様 | §3 |
| DES-MIG-084 | テンプレートファイル一覧 | §3 |
| DES-MIG-085 | エラーレポート仕様 | §4 |

---

**本節で確定した方針**
- 本章の手順は DES-MIG-076〜085 が設計した移行ツールの実施手順として位置づけることを確定する。
- MIG-CK-076〜082 の全チェック通過をもって INST-A4-e+f の完了と判定することを確定する。
- インポート実施前に 04 章のウィザード完了（wizard_completed = TRUE）を前提条件とすることを確定する。

---

## 2 移行計画/02 との関係

### 2-1. 移行計画との位置づけ

本章は `docs/08_移行/移行計画/02_マスタ初期投入実施計画.md` が定義した手順の実施版である。両文書の関係を明示する。

| 文書 | 役割 | 記載内容 |
|---|---|---|
| `移行計画/02_マスタ初期投入実施計画.md` | 計画文書（WHY・WHAT） | 投入対象の定義・スケジュール・担当者・品質基準 |
| 本章（`導入手順/05`） | 実施手順文書（HOW） | 実際の操作手順・コマンド・確認方法・記録テンプレート |

**MIG-X-137**: 本章の手順を実施する前に `移行計画/02` で定義された以下の事項を確認する。（DES-MIG-076 対応）

| 確認事項 | 参照先 |
|---|---|
| 投入対象マスタの最終リスト | `移行計画/02` §2 |
| 投入順序の依存関係 | `移行計画/02` §3 |
| 各マスタの品質基準（件数・欠損率） | `移行計画/02` §5 |
| 担当者アサイン | `移行計画/02` §4 |

---

**本節で確定した方針**
- 移行計画/02 で定義した投入対象・順序・品質基準を本章の手順実施の前提とすることを確定する。
- 本章は実施手順（HOW）のみを記載し、計画の変更は移行計画/02 を更新することに準拠する。
- 本章の手順実施前に 04 章の GUI ウィザード完了を確認することを確定する。

---

## 3 CSV/Excel テンプレートの確認

### 3-1. 各マスタのテンプレート確認

**MIG-X-136（続）**: インポート実施前に、使用するテンプレートファイルが正規のものであることを確認する。（DES-MIG-083〜084 対応）

テンプレートファイルは `docs/08_移行/付録/templates/` に格納する。以下のファイルが存在することを確認する。

| テンプレートファイル名 | 対象エンティティ | ファイルサイズ目安 |
|---|---|---|
| `template_processes.xlsx` | 工程マスタ（processes） | 20 KB 以上 |
| `template_operations.xlsx` | 作業マスタ（operations） | 20 KB 以上 |
| `template_products.xlsx` | 製品マスタ（products） | 20 KB 以上 |
| `template_users.xlsx` | ユーザーマスタ（users） | 20 KB 以上 |
| `template_user_skills.xlsx` | ロール・スキルマスタ（user_skills） | 20 KB 以上 |
| `template_sops.xlsx` | SOP マスタ（sops + sop_steps 複数シート） | 30 KB 以上 |
| `template_lots_legacy.xlsx` | ロットマスタ（過去 1 年分・is_legacy=true 用） | 20 KB 以上 |

```bash
# テンプレートファイルの存在確認
ls -la /opt/wnav/templates/template_*.xlsx
```

テンプレートのシート構成（「入力シート」「記入例」「注意事項」の 3 シート）が DES-MIG-083 の仕様通りであることを Excel で開いて確認する。

### 3-2. 必須列・型・文字数制限の最終確認

**MIG-X-137（続）**: 各マスタの必須列・型・文字数制限が DES-MIG-023 の仕様通りであることを確認する。旧システムからのデータ移行準備として、以下の点を事前にチェックする。

| マスタ | 最重要必須列 | 型 | 文字数上限 | 特記事項 |
|---|---|---|---|---|
| 工程マスタ | `process_no`, `process_name` | TEXT | 100 文字 | `process_no` は英数字のみ（半角） |
| 作業マスタ | `process_no`, `operation_no`, `operation_name` | TEXT | 100 文字 | `process_no` は工程マスタに存在する値のみ |
| 製品マスタ | `product_no`, `product_name` | TEXT | 100 文字 | 製品番号は重複不可 |
| ユーザーマスタ | `username`, `display_name`, `role`, `hire_date` | 各種 | 100 文字 | `role` は 6 種の固定値のみ |
| SOP マスタ | `sop_code`, `sop_title`, `operation_no`, `version` | TEXT | 200 文字 | `operation_no` は作業マスタに存在する値のみ |

**MIG-CK-076**: テンプレートの形式検証として、以下を確認し記録する。

| 検証項目 | 確認方法 | 合格基準 |
|---|---|---|
| ファイル形式 | ファイル拡張子を確認 | `.xlsx` であること（`.xls` は非対応） |
| シート構成 | Excel で開いてシート名を確認 | 「入力シート」「記入例」「注意事項」の 3 シートが存在 |
| 列ヘッダー行 | 1 行目に日本語ラベル、2 行目にシステムフィールド名が存在する | DES-MIG-083 の仕様通り |
| データバリデーション | 入力規則（ドロップダウン等）が設定されている | ロール列でドロップダウンが動作する |

---

**本節で確定した方針**
- テンプレートファイルは `docs/08_移行/付録/templates/` に格納し、7 種全てが存在することを確認することを確定する。
- MIG-CK-076 の 4 検証項目全通過をインポート実施の前提条件とすることを確定する。
- ファイル形式は `.xlsx` のみ対応し、`.xls` は対象外と判断する。

---

## 4 マスタ投入の実行（工程→作業→製品の順）

### 4-1. インポートの操作手順

**MIG-X-138**: マスタ投入は外部キー依存関係の順序（工程→作業→製品→ユーザー→ロール/スキル）に従い実施する。この順序を逆にした場合、外部キー制約違反が発生してインポートが失敗する。（DES-MIG-078 対応）

インポートの操作手順（DES-MIG-078 の 10 段階フローに対応）を以下に示す。

マスタメンテ画面（`https://<サーバーIP>/masters/import`）にアクセスし、以下の操作を実施する。

| フェーズ | 操作手順 | 備考 |
|---|---|---|
| ファイル選択 | 「ファイルを選択」から対象の `.xlsx` または `.csv` を選択 | 最大 10 MB。10,000 行以内 |
| エンコーディング確認 | UTF-8（BOM 付き）での保存を確認 | Excel の「名前を付けて保存」→「CSV UTF-8 (BOM 付き)」で出力 |
| エンティティ種別選択 | インポート対象のマスタ種別を選択 | 例：「工程マスタ（processes）」 |
| プレビュー表示 | 「プレビューを表示」ボタンを押す | バリデーション結果がテーブル形式で表示される |
| エラー確認 | クリティカルエラー件数・警告件数・正常件数を確認 | クリティカルエラーが 1 件でもある場合はインポートボタンが無効 |
| エラー修正 | エラーレポートを確認して Excel を修正し再アップロード | エラーレポートの `row_number` と `suggestion` を参照 |
| インポート実行 | クリティカルエラー 0 件を確認後「インポートを実行」を押す | 警告のみの場合は自動修正内容を確認して実行 |
| 完了確認 | 成功件数・エラー件数が表示される | 全件成功を確認 |

### 4-2. 工程マスタの投入

**MIG-X-139**: 工程マスタを最初に投入する。工程マスタは全マスタの根幹であり、他のマスタが工程マスタの `process_no` を参照する。（DES-MIG-078 対応）

```bash
# 投入後の件数照合（API で確認）
curl -s -H "Authorization: Bearer <JWTトークン>" \
    http://localhost:8080/api/masters/processes | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(f'工程マスタ件数: {len(data[\"data\"])}')
"
```

**MIG-CK-077**: 工程マスタの投入後件数が移行計画/02 で定義した期待件数と一致することを確認して記録する。

| 確認項目 | 確認方法 | 合格基準 |
|---|---|---|
| 投入件数照合 | API または `SELECT COUNT(*)` | 移行計画/02 の期待件数と一致 |
| process_no の重複なし | `SELECT process_no, COUNT(*) FROM processes GROUP BY process_no HAVING COUNT(*) > 1` | 0 件（重複なし） |
| NULL チェック | `SELECT COUNT(*) FROM processes WHERE process_name IS NULL` | 0 件 |

### 4-3. 作業マスタの投入

**MIG-X-140**: 工程マスタ投入完了後に作業マスタを投入する。`process_no` の外部キー参照が解決できることを事前に確認する。（DES-MIG-078 対応）

**MIG-CK-078**: 作業マスタの投入後件数照合と整合性確認を実施する。

```sql
-- 作業マスタの件数照合
SELECT COUNT(*) FROM operations;

-- 存在しない process_no を参照している作業がないことを確認する
SELECT o.operation_no, o.process_no
FROM operations o
LEFT JOIN processes p ON o.process_no = p.process_no
WHERE p.process_no IS NULL;
-- 0 件であることを確認する
```

### 4-4. 製品マスタの投入と全件照合

**MIG-X-141**: 製品マスタは工程マスタと並列で投入可能だが、作業マスタ投入完了後に実施する。（DES-MIG-078 対応）

**MIG-CK-079**: 工程・作業・製品の各マスタについて投入件数が移行計画/02 の期待件数と一致することを確認して記録する。

```sql
-- 全マスタの件数照合
SELECT 'processes' AS master, COUNT(*) AS count FROM processes
UNION ALL
SELECT 'operations', COUNT(*) FROM operations
UNION ALL
SELECT 'products', COUNT(*) FROM products;
```

### 4-5. エラー発生時の即時確認と再投入手順

インポート時にエラーが発生した場合は以下の手順で対処する。

エラーレポート CSV（DES-MIG-085 の仕様）のダウンロードリンクがインポート画面に表示される。エラーレポートの `error_code` と `suggestion` を参照して Excel を修正し、修正後のファイルを再アップロードする。

インポートはトランザクションでロールバック保証されているため（DES-MIG-026）、エラー発生時でも部分投入は発生しない。DB は投入前の状態に戻っているため、修正後のファイルを再アップロードして再実行する。

```bash
# エラーレポートのダウンロード（API 経由）
curl -s -H "Authorization: Bearer <JWTトークン>" \
    "http://localhost:8080/api/masters/import/errors/<インポートジョブID>/download" \
    -o error_report.csv
```

---

**本節で確定した方針**
- マスタ投入は工程→作業→製品→ユーザー→ロール/スキルの順序で実施することを確定する。
- MIG-CK-077〜079 の件数照合が全て合格であることを次手順進行の前提条件とすることを確定する。
- エラー発生時はトランザクションロールバックにより DB が投入前の状態に戻ることを確認してから再投入することを確定する。

---

## 5 SOP・Step の投入

### 5-1. Excel 形式 SOP テンプレートのインポート

**MIG-X-142**: SOP マスタは `template_sops.xlsx`（複数シート構成）を使用して投入する。SOP 投入は工程・作業マスタが全件投入済みであることが前提である。（DES-MIG-078 対応）

`template_sops.xlsx` は以下のシート構成で提供する。

| シート名 | 内容 | 必須列 |
|---|---|---|
| sops | SOP ヘッダー情報 | `sop_code`, `sop_title`, `operation_no`, `version`, `author_username` |
| sop_steps | SOP ステップ詳細 | `sop_code`, `step_no`, `step_title`, `step_description` |
| step_params | ステップパラメータ | `sop_code`, `step_no`, `param_key`, `param_value`, `param_type` |

SOP テンプレートのインポートはマスタメンテ画面の「SOP インポート」タブから実施する。

```
操作手順:
1. マスタメンテ画面 → 「SOP 管理」→「CSV/Excel インポート」を選択する
2. `template_sops.xlsx` をアップロードする
3. プレビュー画面で SOP 件数・ステップ件数・エラー件数を確認する
4. クリティカルエラー 0 件を確認後「インポートを実行」を押す
5. 成功件数（SOP 件数・ステップ件数）を確認する
```

### 5-2. Step の紐付け確認

**MIG-X-143**: SOP 投入後に、工程→作業→Step の連鎖が正しく確立されていることを確認する。（DES-MIG-023 対応）

```sql
-- 工程→作業→SOP の連鎖確認
SELECT
    p.process_no,
    p.process_name,
    o.operation_no,
    o.operation_name,
    s.sop_code,
    s.sop_title,
    COUNT(ss.id) AS step_count
FROM processes p
INNER JOIN operations o ON o.process_no = p.process_no
LEFT JOIN sops s ON s.operation_no = o.operation_no
LEFT JOIN sop_steps ss ON ss.sop_id = s.id
GROUP BY p.process_no, p.process_name, o.operation_no, o.operation_name, s.sop_code, s.sop_title
ORDER BY p.process_no, o.operation_no;
```

SOP が紐付いていない作業（`sop_code IS NULL`）が移行計画/02 の定義と一致していることを確認する。

### 5-3. quality_admin による全 SOP の電子承認付与

**MIG-X-143（続）**: 投入された全 SOP に quality_admin が電子承認を付与する。承認なしの SOP はハンディ APP での表示対象外となる。（MIG-T-007 対応）

電子承認の操作手順は以下の通りとする。

```
操作手順:
1. マスタメンテ画面 → 「SOP 管理」→ 「未承認 SOP 一覧」を表示する
2. 全未承認 SOP を選択する（一括選択チェックボックス）
3. 「一括承認」ボタンを押す
4. quality_admin のパスワードを入力して承認を確定する
5. 全 SOP の承認ステータスが「承認済み」になることを確認する
```

```sql
-- 承認済み SOP の件数確認
SELECT
    status,
    COUNT(*) AS count
FROM sops
GROUP BY status;
-- 全件が 'approved' であることを確認する
```

---

**本節で確定した方針**
- SOP 投入は工程・作業マスタ全件投入完了後に実施することを確定する。
- 投入後の工程→作業→SOP の連鎖確認は SQL で実施し、結果を記録することを確定する。
- 全 SOP への quality_admin 電子承認付与を SOP 投入完了の必要条件とすることを確定する。

---

## 6 LEGACY_IMPORT の実行

### 6-1. ケース A（電子化済み記録）の LEGACY_IMPORT イベント生成

**MIG-X-144**: 旧システムに電子化済みの作業記録（ロットレコード等）が存在する場合、これを LEGACY_IMPORT イベントとして work_events テーブルに登録し、ハッシュチェーンに連結する。（DES-MIG-085 対応）

LEGACY_IMPORT の対象となる記録は以下の条件を満たすものとする。

| 条件 | 内容 |
|---|---|
| 対象期間 | 移行計画/02 で定義した過去データの対象期間（例: 過去 1 年分） |
| 対象フォーマット | 旧システムからの CSV エクスポート（`template_lots_legacy.xlsx` 準拠） |
| is_legacy フラグ | `is_legacy = TRUE` として投入（新規作業記録との区別） |
| ハッシュチェーン連結 | LEGACY_IMPORT イベント生成時にハッシュチェーンへの連結を実施 |

LEGACY_IMPORT の実行は以下の手順で行う。

```
操作手順:
1. マスタメンテ画面 → 「移行管理」→「LEGACY_IMPORT」タブを選択する
2. `template_lots_legacy.xlsx`（旧システムデータ入力済み）をアップロードする
3. プレビュー画面で件数・バリデーション結果を確認する
4. 「LEGACY_IMPORT 実行」ボタンを押す（この操作はロールバック不可・慎重に実施する）
5. 進捗バーで処理状況を確認する（1000 行/チャンクで処理）
```

### 6-2. ハッシュチェーンへの連結確認

**MIG-X-145**: LEGACY_IMPORT 完了後に、各 LEGACY_IMPORT イベントがハッシュチェーンに正しく連結されていることを確認する。（DES-MIG-085 対応）

**MIG-CK-080**: ハッシュチェーンの連続性を確認する。

```sql
-- ハッシュチェーンの連続性確認（前イベントの hash が次イベントの prev_hash に一致する）
WITH chain_check AS (
    SELECT
        id,
        event_type,
        hash,
        prev_hash,
        LAG(hash) OVER (ORDER BY id) AS expected_prev_hash
    FROM work_events
    WHERE event_type = 'LEGACY_IMPORT'
    ORDER BY id
)
SELECT COUNT(*) AS broken_chain_count
FROM chain_check
WHERE prev_hash != expected_prev_hash
   OR (expected_prev_hash IS NOT NULL AND prev_hash IS NULL);
-- 0 件であることを確認する（ハッシュチェーンが切断されていない）
```

**MIG-CK-081**: LEGACY_IMPORT の件数が `template_lots_legacy.xlsx` の行数と一致することを確認する。

```sql
-- LEGACY_IMPORT イベントの件数確認
SELECT COUNT(*) AS legacy_import_count
FROM work_events
WHERE event_type = 'LEGACY_IMPORT' AND is_legacy = TRUE;
```

**MIG-CK-082**: 移行データ品質チェック（移行計画/08 の品質ゲート）を実施する。

移行計画/08 で定義した品質ゲートの各チェック項目を実施し、全項目が合格であることを確認してから LEGACY_IMPORT 完了と判定する。

| 品質ゲート項目 | 確認 SQL | 合格基準 |
|---|---|---|
| 必須フィールドの非 NULL 率 | `SELECT COUNT(*) FROM work_events WHERE event_type='LEGACY_IMPORT' AND lot_no IS NULL` | 0 件（欠損なし） |
| 日付範囲の正当性 | `SELECT COUNT(*) FROM work_events WHERE event_type='LEGACY_IMPORT' AND occurred_at > NOW()` | 0 件（未来日付なし） |
| ハッシュ値の完整性 | `SELECT COUNT(*) FROM work_events WHERE event_type='LEGACY_IMPORT' AND hash IS NULL` | 0 件（ハッシュ欠損なし） |
| 外部キー整合性 | `SELECT COUNT(*) FROM work_events we LEFT JOIN lots l ON we.lot_id = l.id WHERE we.lot_id IS NOT NULL AND l.id IS NULL` | 0 件（孤立レコードなし） |

---

**本節で確定した方針**
- LEGACY_IMPORT はロールバック不可の操作であるため、実施前に全マスタ投入・SOP 承認の完了を確認することを確定する。
- MIG-CK-080〜082 の全チェック通過を LEGACY_IMPORT 完了の必要条件とすることを確定する。
- ハッシュチェーンの切断が 0 件であることを改竄検知機能の確立判定基準とすることを確定する。

---

## 参照業界分析

### 必須
- [`../../90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)

### 関連
- [`../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md`](../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md)
- [`../../90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)
- [`../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

---

## 更新履歴

| バージョン | 日付 | 変更内容 | 担当者 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | 初版 | RyuheiKiso |
