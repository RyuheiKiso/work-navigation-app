# 08 データ品質要件（ALCOA+ 整合）

本章は ALCOA+ 9 原則（Attributable / Legible / Contemporaneous / Original / Accurate / Complete / Consistent / Enduring / Available）を NFR-DQ 識別子付きの計測可能要件として確定する。計画 06 章（データモデル中核設計）6 節が権威章であり、本章は計画 06 章の「原則→要件化」版として機能する。

---

## 1. ALCOA+ 9 原則の要件 ID 化

### 1-1. Attributable（帰属可能）

| 要件 ID | NFR-DQ-001 |
|---|---|
| 要件名 | データの帰属明確性 |
| 検証可能命題 | 全 WorkEvent レコードに worker_id（NULL 禁止）・terminal_id（NULL 禁止）が存在すること |
| 受入基準 | WorkEvent テーブルの全レコードで worker_id IS NOT NULL かつ terminal_id IS NOT NULL であること。DB 制約として NULL 禁止が設定されていること |
| 関連システム機能 | ログイン認証（IF-003）・JWT 検証・RBAC 6 ロール（NFR-SEC-010）|

共有アカウントの禁止は RBAC ポリシーとして運用規程に明記する。匿名操作を許容しないことを UI 設計の前提とする。

### 1-2. Legible（読取可能）

| 要件 ID | NFR-DQ-002 |
|---|---|
| 要件名 | データの可読性 |
| 検証可能命題 | 全テキストデータが UTF-8 エンコーディングで保存されており、50 年後に一般的な UTF-8 対応ビューワで読取可能であること |
| 受入基準 | PostgreSQL のエンコーディングが UTF8 に設定されていること。PDF 出力が ISO 32000 準拠・Noto フォント埋め込みであること（機能要件 RP-001 参照）|
| 関連システム機能 | PDF 生成（RP-001）・多言語テキスト JSONB（instruction_text）|

### 1-3. Contemporaneous（同時記録）

| 要件 ID | NFR-DQ-003 |
|---|---|
| 要件名 | 同時記録の保証 |
| 検証可能命題 | 全 WorkEvent に timestamp_device（端末時刻）と timestamp_server（サーバー受信時刻）の両方が存在し、両タイムスタンプの差（sync_lag_ms）が記録されていること |
| 受入基準 | WorkEvent テーブルの timestamp_device・timestamp_server が NULL 禁止で設定されていること。is_offline フラグで未同期を区別できること |
| 関連システム機能 | Offline-First 設計（FR-OFF）・Outbox Pattern（IF-002）|

後入力（is_retroactive: true）については、後入力理由・後入力日時・承認者 ID を必須とし（BR-BUS-036）、「後から記録した」事実を透明に記録することで Contemporaneous の精神を保つ。

### 1-4. Original（元データ）

| 要件 ID | NFR-DQ-004 |
|---|---|
| 要件名 | データの原本性 |
| 検証可能命題 | WorkEvent テーブルへの UPDATE / DELETE が物理的に禁止されており、修正は訂正イベント追記のみで行われること |
| 受入基準 | PostgreSQL のアプリケーション用ロールに UPDATE / DELETE 権限が付与されていないこと。訂正イベントが定義された形式（event_type: correction・訂正理由・訂正者 ID・承認者 ID）で追記されていること |
| 関連システム機能 | Append-only 設計（計画 06 章）・ハッシュチェーン（NFR-SEC-040）|

### 1-5. Accurate（正確）

| 要件 ID | NFR-DQ-005 |
|---|---|
| 要件名 | データの正確性 |
| 検証可能命題 | 数値フィールドが NUMERIC(18,6) 型で保存され、UCUM 単位が付与され、USL/LSL での即時合否判定が実施されていること |
| 受入基準 | Step の numeric_input で入力された値が measured_value（SI 基本単位）+ unit_code（UCUM）+ uncertainty_u + calibration_ref のセットで記録されること。False Precision 防止として計測器分解能で自動丸めが実施されること（BR-BUS-034）|
| 関連システム機能 | 計測器 IF（IF-006）・校正期限プレフライト（BR-BUS-007）|

### 1-6. Complete（完全）

| 要件 ID | NFR-DQ-006 |
|---|---|
| 要件名 | データの完全性 |
| 検証可能命題 | SOP の全 Step について step_completed または step_skipped イベントが存在し、「記録なし」という欠損状態が存在しないこと |
| 受入基準 | 作業完了前の「全 Step 完了チェック」（BR-BUS-005）が通過していること。スキップ Step には step_skipped イベントと理由テキストが存在すること（BR-BUS-035）|
| 関連システム機能 | ロックステップ進行（BR-BUS-001〜005）|

### 1-7. Consistent（一貫）

| 要件 ID | NFR-DQ-007 |
|---|---|
| 要件名 | データの一貫性 |
| 検証可能命題 | 同一 WorkExecution 内の全 WorkEvent が同一の sop_version_id を参照しており、作業途中での版変更が発生していないこと |
| 受入基準 | WorkExecution.sop_version_id が発行後に変更されないこと（BR-BUS-009）。全 WorkEvent が WorkExecution の sop_version_id を継承していること |
| 関連システム機能 | SOP 版固定（BR-BUS-008）・作業指示発行フロー |

### 1-8. Enduring（耐久）

| 要件 ID | NFR-DQ-008 |
|---|---|
| 要件名 | データの耐久性 |
| 検証可能命題 | 作業実績データが 7 年以上にわたりオンラインで照会可能な状態を維持できること |
| 受入基準 | NFR-AVL（可用性要件）・NFR-OPS-030〜033（バックアップ要件）が達成されていること。年次復旧テスト（NFR-AVL-020）が通過していること |
| 関連システム機能 | PostgreSQL WAL PITR・日次 pg_dump・週次オフサイトバックアップ |

### 1-9. Available（利用可能）

| 要件 ID | NFR-DQ-009 |
|---|---|
| 要件名 | データの利用可能性 |
| 検証可能命題 | 品質担当または監査者が任意のロット・作業指示・作業者を起点に順方向・逆方向のトレサビクエリを実行でき、5 秒以内に結果を取得できること |
| 受入基準 | トレサビ照会画面（SCR-MC-004）でのクエリ応答時間 P95 = 5 秒以内。CSV / XES エクスポートが正常に生成されること |
| 関連システム機能 | トレサビ照会機能（SCR-MC-004）・監査ログ出力（RP-002）|

図: fig_alcoa_requirement_trace（img/ 配下）を参照

**本節で確定した方針**
- ALCOA+ 9 原則を NFR-DQ-001〜009 として要件 ID 化し、各原則に検証可能命題・受入基準・関連システム機能を確定する。
- 全 9 要件は DB 制約・アプリケーション実装・E2E テストの 3 層で検証可能な形式で記述することを確定する。
- NFR-DQ-004（Original）と NFR-SEC-040（ハッシュチェーン）が整合することを本節で確認する。

---

## 2. 属性別 DoD（Definition of Done）

### 2-1. WorkEvent の DoD

以下の全条件を満たした WorkEvent が「完成」とみなされる。

| 条件 | 検証方法 |
|---|---|
| event_id が UUID 形式で NULL でない | DB NOT NULL 制約 |
| timestamp_device と timestamp_server の両方が UTC タイムスタンプである | DB NOT NULL 制約 + アプリケーション検証 |
| worker_id・terminal_id・sop_version_id が NULL でない | DB NOT NULL 制約 |
| event_type が有効な列挙値である | DB ENUM 制約 |
| prev_hash が前レコードの SHA-256 と一致する | ハッシュチェーン検証バッチ |
| payload が event_type に対応するスキーマを満たす | アプリケーション層バリデーション |

### 2-2. ElectronicSign の DoD

| 条件 | 検証方法 |
|---|---|
| signer_id・timestamp_device・timestamp_server・server_signature が全て非 NULL | DB NOT NULL 制約 |
| server_signature が HMAC-SHA256 で正しく計算されている | サーバー側の署名検証処理 |
| 同一 WorkExecution・同一 Step での重複署名が存在しない（BR-BUS-012 除く）| アプリケーション層バリデーション |

**本節で確定した方針**
- WorkEvent の DoD を 6 条件で確定し、DB 制約・バリデーション・ハッシュチェーンの 3 層で検証することを確定する。
- ElectronicSign の DoD を 3 条件で確定し、server_signature の HMAC 検証を必須とすることを確定する。

---

## 3. 監査証跡要件

### 3-1. 監査証跡の必須 4 フィールド

| 要件 ID | NFR-DQ-030 |
|---|---|
| 要件名 | 監査証跡の必須属性 |
| 要件内容 | すべての監査証跡レコードに以下の 4 フィールドを必須とする。1. タイムスタンプ（timestamp_server: UTC ミリ秒）2. エンティティ ID（対象レコードの UUID）3. ユーザー ID（操作者の worker_id）4. 変更種別（event_type: ALCOA+ に対応したイベント分類）|

**本節で確定した方針**
- タイムスタンプ・エンティティ ID・ユーザー ID・変更種別の 4 フィールドを監査証跡の必須属性として確定する。
- 4 フィールドの全てに NULL 禁止制約を設けることを確定する。

---

## 4. 時点参照の整合性

### 4-1. 過去記録の正確な再現

| 要件 ID | NFR-DQ-040 |
|---|---|
| 要件名 | 時点参照の整合性 |
| 要件内容 | 過去の WorkExecution を照会した際に、当時適用された SOP 版の内容（Step 定義・判定条件）が現在のマスタ変更に関わらず正確に再現できること |
| 検証可能命題 | WorkExecution.sop_version_id が参照する MasterVersion が廃止後も存在し、照会クエリで取得可能であること |
| 受入基準 | MasterVersion の物理削除が禁止されており（NFR-SEC-047 参照）、廃止後も ARCHIVED 状態で存在すること |

**本節で確定した方針**
- 過去記録の時点参照整合性を NFR-DQ-040 として確定し、MasterVersion の永続存在を受入基準とする。
- 時点参照はデータモデル（EN-006 MasterVersion）・状態遷移（ARCHIVED 状態）・業務ルール（BR-BUS-009）の 3 層で保証することを確定する。

---

## 5. 改ざん検知連動

### 5-1. NFR-DQ と NFR-SEC の整合確認

| NFR-DQ 要件 | 対応する NFR-SEC 要件 | 整合内容 |
|---|---|---|
| NFR-DQ-004（Original）| NFR-SEC-040（ハッシュチェーン）| ハッシュチェーンが Original 原則（更新不可性）を補強する |
| NFR-DQ-004（Original）| NFR-SEC-047（削除禁止）| ElectronicSign の削除禁止が Original 原則を保証する |
| NFR-DQ-001（Attributable）| NFR-SEC-045（電子サイン構成要素）| signer_id 必須が Attributable 原則を電子サインに実装する |
| NFR-DQ-003（Contemporaneous）| NFR-SEC-045（timestamp 双方保持）| timestamp_device + timestamp_server が Contemporaneous を保証する |
| NFR-DQ-008（Enduring）| NFR-OPS-030〜033（バックアップ）| バックアップ 3 層が Enduring を実装する |

**本節で確定した方針**
- NFR-DQ（データ品質）と NFR-SEC（セキュリティ）の整合を 5 点で確認し、矛盾が存在しないことを確定する。
- NFR-DQ-004（Original）は NFR-SEC-040（ハッシュチェーン）と NFR-SEC-047（削除禁止）の両方で実装されることを確定する。

---

## 参照業界分析

### 必須

[`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)

### 関連

[`90_業界分析/33_計量法・JCSS校正トレーサビリティとSI単位.md`](../../../90_業界分析/33_計量法・JCSS校正トレーサビリティとSI単位.md)

[`90_業界分析/21_作業ログ分析とプロセスマイニング.md`](../../../90_業界分析/21_作業ログ分析とプロセスマイニング.md)

[`90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)
