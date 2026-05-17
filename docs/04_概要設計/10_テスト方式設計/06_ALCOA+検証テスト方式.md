# 06 ALCOA+ 検証テスト方式

本章の責務は、ALCOA+ 9 原則（Attributable / Legible / Contemporaneous / Original / Accurate / Complete / Consistent / Enduring / Available）に対応した検証テストケースの設計命題を確定することである。ALCOA+ は電子記録の品質保証原則として製造業の規制当局が要求する。本システムの NFR-DQ-001〜009 はこれを NFR として定義しており、本章の自動化検証テストがその受入基準の実装的担保となる。

---

## 1. ALCOA+ テスト対応マトリクス

| ALCOA+ 原則 | NFR-ID | 主検証テストレベル | 担当 TST-ID（概要） | 自動化可否 |
|---|---|---|---|---|
| A：Attributable（帰属可能） | NFR-DQ-001 | L2 統合テスト | TST-060 | 自動化 |
| L：Legible（読取可能） | NFR-DQ-002 | L3 E2E + UAT | TST-061 | 自動化（PDF）+ UAT |
| C：Contemporaneous（同時記録） | NFR-DQ-003 | L2 統合テスト | TST-062 | 自動化 |
| O：Original（元データ） | NFR-DQ-004 | L2 統合テスト | TST-063 | 自動化 |
| A：Accurate（正確） | NFR-DQ-005 | L1 ユニット + L2 統合 | TST-064 | 自動化 |
| C：Complete（完全） | NFR-DQ-006 | L2 統合テスト | TST-065 | 自動化 |
| C：Consistent（一貫） | NFR-DQ-007 | L2 統合テスト | TST-066 | 自動化 |
| E：Enduring（耐久） | NFR-DQ-008 | L3 E2E（PDF/A-3） | TST-067 | 自動化（veraPDF） |
| A：Available（利用可能） | NFR-DQ-009 | L3 E2E | TST-068 | 自動化 |

---

## 2. TST-060：A（Attributable）帰属可能性検証

### 2-1. 検証観点

全 WorkEvent レコードに `worker_id`（NULL 禁止）・`terminal_id`（NULL 禁止）が存在し、共有アカウントを使用した操作が存在しないことを検証する。

| テスト項目 | 検証内容 | 期待結果 |
|---|---|---|
| worker_id NOT NULL 制約 | WorkEvent INSERT 時に worker_id を NULL で送信 | DB 制約違反・HTTP 422 |
| terminal_id NOT NULL 制約 | WorkEvent INSERT 時に terminal_id を NULL で送信 | DB 制約違反・HTTP 422 |
| JWT 認証なし操作拒否 | Authorization ヘッダなしで作業イベント POST | HTTP 401 |
| 無効 JWT での操作拒否 | 改ざんした JWT トークンで作業イベント POST | HTTP 401 |
| created_by 自動付与 | 認証済みリクエストでの WorkEvent 作成後、DB の created_by が JWT の sub と一致 | DB 照合で一致 |

### 2-2. 対応する要件

- NFR-DQ-001（帰属明確性）
- FR-AU-001（電子サイン取得）
- BR-BUS-010（共有アカウント禁止）

---

## 3. TST-061：L（Legible）読取可能性検証

### 3-1. 検証観点

全テキストデータが UTF-8 で保存され、PDF 出力が人間が読める形式であることを検証する。PDF/A-3 適合性は veraPDF ツールで自動検証し、視覚的確認は UAT で実施する。

| テスト項目 | 検証内容 | ツール | 期待結果 |
|---|---|---|---|
| PostgreSQL エンコーディング確認 | `SHOW server_encoding` の結果 | sqlx 統合テスト | UTF8 |
| 多言語テキスト格納・復元 | JSONB 型の instruction_text に日本語・英語テキストを保存 → 取得 | L2 統合テスト | 文字化けなし |
| PDF/A-3 適合性 | 生成された PDF を veraPDF で検証 | veraPDF CLI | 適合エラー 0 件 |
| Noto フォント埋め込み | PDF バイナリにフォントが埋め込まれていること | PDFBox / pdfinfo | フォント埋め込み確認 |

### 3-2. 対応する要件

- NFR-DQ-002（データの可読性）
- RP-001（作業実績帳票）

---

## 4. TST-062：C（Contemporaneous）同時記録検証

### 4-1. 検証観点

WorkEvent に `timestamp_device` と `timestamp_server` の両方が存在し、`sync_lag_ms` が記録され、クロックスキューが許容範囲外の場合に ERR-VAL-010 で拒否されることを検証する。

| テスト項目 | 検証内容 | 期待結果 |
|---|---|---|
| timestamp_device NOT NULL | WorkEvent INSERT 時に timestamp_device を省略 | HTTP 422・ERR-VAL-001 |
| timestamp_server 自動付与 | WorkEvent INSERT 後の DB レコードで timestamp_server が NULL でない | DB 照合 |
| sync_lag_ms 記録 | timestamp_server - timestamp_device の差分が sync_lag_ms として保存 | 計算値と一致 |
| クロックスキュー 5 秒超拒否 | timestamp_device がサーバー時刻より 5 秒超過した WorkEvent を送信 | HTTP 422・ERR-VAL-010 |
| is_offline フラグ | オフライン中の WorkEvent が is_offline = true で保存 | DB 照合 |

### 4-2. 対応する要件

- NFR-DQ-003（同時記録の保証）
- ERR-VAL-010（クロックスキュー拒否）
- CFG-010（max_clock_skew = 5 秒）

---

## 5. TST-063：O（Original）元データ検証

### 5-1. 検証観点

WorkEvent テーブルへの UPDATE / DELETE がアプリケーション用ロールによって物理的に拒否されることを検証する。修正は訂正イベント追記のみ許容される。

| テスト項目 | 検証内容 | 期待結果 |
|---|---|---|
| UPDATE 拒否 | アプリロールで WorkEvent の UPDATE を試行 | PostgreSQL PermissionError・HTTP 403 |
| DELETE 拒否 | アプリロールで WorkEvent の DELETE を試行 | PostgreSQL PermissionError・HTTP 403 |
| EvidenceFile UPDATE 拒否 | アプリロールで EvidenceFile の UPDATE を試行 | PostgreSQL PermissionError・HTTP 403 |
| EvidenceFile DELETE 拒否 | アプリロールで EvidenceFile の DELETE を試行 | PostgreSQL PermissionError・HTTP 403 |
| 訂正イベント追記 | WorkEvent の修正を correction イベント追記で実施 | DB に correction イベントが追記（既存行は変更なし） |

### 5-2. 対応する要件

- NFR-DQ-004（データの原本性）
- BR-BUS-015（Append-only 保護）

---

## 6. TST-064：A（Accurate）正確性検証

### 6-1. 検証観点

数値フィールドの精度・UCUM 単位の付与・ERR-BIZ-003（必須フィールド NULL 拒否）が正しく動作することを検証する。

| テスト項目 | 検証内容 | ツール | 期待結果 |
|---|---|---|---|
| 数値範囲バリデーション | USL/LSL を超えた measured_value の WorkEvent を送信 | L2 統合テスト | HTTP 422・ERR-VAL-002（numeric_out_of_range） |
| UCUM 単位コード必須 | unit_code が空の numeric_input WorkEvent を送信 | L2 統合テスト | HTTP 422・ERR-VAL-001 |
| SHA-256 計算精度 | ゴールデンデータとの完全一致テスト | L1 ユニットテスト | バイト列完全一致 |
| JSON Logic 評価精度 | JSON Logic 公式テストスイート全件 | L1 ユニットテスト | 全件 PASS |
| Cp/Cpk 計算精度 | 既知データに対する計算結果（小数点 4 桁以内誤差） | L1 ユニットテスト | 許容誤差内 |

### 6-2. 対応する要件

- NFR-DQ-005（データの正確性）
- NFR-QUA-002（機能正確性）
- ERR-BIZ-003（sign_required）
- ERR-VAL-002（numeric_out_of_range）

---

## 7. TST-065：C（Complete）完全性検証

### 7-1. 検証観点

全 Step 種別（数値入力・選択肢・テキスト・チェックリスト・写真・音声）の完了時に、XES 必須属性（concept:name / time:timestamp / org:resource / lifecycle:transition）が揃った WorkEvent が生成されることを検証する。

| テスト項目 | 検証内容 | 期待結果 |
|---|---|---|
| 数値 Step 完了 → WorkEvent | XES 必須属性全件の存在確認 | 全属性 NOT NULL |
| 写真 Step 完了 → EvidenceFile + WorkEvent | EvidenceFile と WorkEvent の両方が生成 | 双方のレコードが存在 |
| チェックリスト Step 完了 → WorkEvent | checked_items が全チェック項目を含む | JSON 配列の長さが一致 |
| 音声メモ Step 完了 → EvidenceFile | MIME タイプ audio/* の EvidenceFile が生成 | MIME タイプ確認 |
| XES エクスポートの完全性 | エクスポートされた XES ファイルに全 WorkEvent が含まれる | レコード件数一致 |

### 7-2. 対応する要件

- NFR-DQ-006（データの完全性）
- FR-AU-005（監査ログ XES エクスポート）

---

## 8. TST-066：C（Consistent）一貫性検証

### 8-1. 検証観点

ハッシュチェーンの prev_hash 連結が正しく、BAT-001（週次ハッシュチェーン検証）が改ざん検知シミュレーションで正しく機能することを検証する。

| テスト項目 | 検証内容 | 期待結果 |
|---|---|---|
| prev_hash 連結確認 | WorkEvent n の hash_current が WorkEvent n+1 の prev_hash と一致 | 全レコードで一致 |
| BAT-001 正常完了 | ハッシュチェーンが改ざんされていない状態での BAT-001 実行 | PASS・ログ LOG-007 に SUCCESS 記録 |
| BAT-001 改ざん検知 | WorkEvent の hash_current を直接書き換えた状態での BAT-001 実行（テスト DB のみ） | ERR-DB-003 + LOG-007 に INTEGRITY_VIOLATION 記録 |
| FR-AU-006 ハッシュチェーン検証 API | `/api/v1/system/health` でハッシュチェーン状態が正常表示 | HTTP 200・status: ok |

### 8-2. 対応する要件

- NFR-DQ-007（データの一貫性）
- FR-AU-006（ハッシュチェーン検証・週次）
- BAT-001（hash_chain_verifier）
- NFR-SEC-040（SHA-256 ハッシュチェーン）

---

## 9. TST-067：E（Enduring）耐久性検証

### 9-1. 検証観点

生成された PDF が PDF/A-3 規格に適合し、50 年後もフォント埋め込み状態で読取可能であることを veraPDF ツールで自動検証する。

| テスト項目 | 検証内容 | ツール | 期待結果 |
|---|---|---|---|
| PDF/A-3 適合性（作業実績帳票） | RP-001 生成 PDF の veraPDF 検証 | veraPDF CLI | 適合プロファイル: PDF/A-3b |
| PDF/A-3 適合性（電子サイン証跡） | RP-005 生成 PDF の veraPDF 検証 | veraPDF CLI | 適合プロファイル: PDF/A-3b |
| フォント埋め込み確認 | 全 PDF の埋め込みフォントリスト確認 | pdffonts（poppler-utils）| Noto フォント埋め込み確認 |
| SHA-256 ハッシュ値の耐久性 | ハッシュ値が再計算後も一致 | L1 ユニットテスト | 完全一致 |

### 9-2. 対応する要件

- NFR-DQ-008（データの耐久性）
- RP-001〜006（帳票・出力）

---

## 10. TST-068：A（Available）利用可能性検証

### 10-1. 検証観点

監査ログの 3 つのエクスポート方法（Web ダウンロード・ファイルシステム出力・API）が全て正常に動作することを検証する。

| テスト項目 | 検証内容 | ツール | 期待結果 |
|---|---|---|---|
| Web ダウンロード | UC-020 ゴールデンパス（XES ファイルのダウンロード） | Playwright | Content-Disposition: attachment ヘッダ・ファイル生成 |
| API エクスポート | `GET /api/v1/reports/audit-log?format=xes` | L2 統合テスト | HTTP 200・XES XML バリデーション PASS |
| ファイルシステム出力 | BAT-007（document_hash_recorder）の定期出力 | L2 統合テスト | 指定パスにファイルが生成 |
| エクスポート権限確認 | quality_admin 以外のロールでの監査ログエクスポート試行 | L2 統合テスト | HTTP 403 |

### 10-2. 対応する要件

- NFR-DQ-009（データの利用可能性）
- FR-AU-004（監査ログ閲覧）
- FR-AU-005（監査ログ XES エクスポート）

---

**本節で確定した方針**
- ALCOA+ 9 原則の各原則（A/L/C/O/A/C/C/E/A）に対して個別の検証テストケース群（TST-060〜068）を設計命題として確定し、自動化可能な全項目を L1/L2/L3 テストに割り当てた。
- TST-063（Original 検証）において WorkEvent・EvidenceFile への UPDATE/DELETE 拒否をアプリロールレベルで検証することを確定し、DB 制約とアプリ層の両方を検証対象とした。
- TST-066（Consistent 検証）において BAT-001 改ざん検知シミュレーションをテスト DB 環境で実施することを確定した。

---

## 参照業界分析

### 必須

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)

### 関連

[`90_業界分析/21_電子チェックリストと手順遵守の科学.md`](../../../90_業界分析/21_電子チェックリストと手順遵守の科学.md)
