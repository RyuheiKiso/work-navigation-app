# 05_外部インターフェース設計

IPA 共通フレーム 2013「2.3/2.4 インタフェース方式設計タスク」に対応する。

要件定義フェーズで確定した 7 外部インタフェース（IF-001〜007）を概要設計レベルで具体化し、REST API 方針・認証認可方式・各 IF の仕様・シーケンス図・TLS 設計・テスト方式を確定する。

---

## 担当する上流要件

| 種別 | 範囲 |
|---|---|
| 外部 IF | IF-001〜007 全 7 件（全件をカバーする）|
| 機能要件 FR | FR-SY 全 9 件（同期・認証）・FR-AU 全 6 件（認証認可）|
| 非機能要件 NFR | NFR-SEC（TLS・JWT・mTLS）・NFR-PRF（API 応答時間）|
| シーケンス図 SEQ | SEQ-001〜008 全 8 件のシーケンス図を本サブで描画 |

---

## 章構成

| ファイル | 目的 |
|---|---|
| `README.md` | 本書 |
| `00_本書の位置づけと識別子規約.md` | IPA 対応・IF/API/WH/SEQ 識別子の確定 |
| `01_外部IF総覧（IF-001〜007継承）.md` | 7 IF の概要・方式・認証・担当章を一覧化 |
| `02_REST_API設計方針（OpenAPI3.1）.md` | 命名・バージョニング・エラーモデル・冪等性・レート制御 |
| `03_認証認可方式（JWT_RS256・OAuth2.1）.md` | JWT 構造・RBAC 判定フロー・OAuth 2.1（mTLS 対応）|
| `04_親機マスタ同期API（IF-001）.md` | Pull 型差分同期エンドポイント・スキーマ・認証 |
| `05_Outbox実績送信API（IF-002）.md` | Exactly-once 意味論・Idempotency Key・受信側契約 |
| `06_認証連携IF（IF-003）.md` | LDAP/AD 連携・ローカル認証フォールバック |
| `07_印刷・ラベル出力IF（IF-004）.md` | IPP/ZPL・PDF 生成パイプライン |
| `08_スキャナ・計測器IF（IF-005_006）.md` | USB HID/BLE GATT/CDC-ACM・GS1 解釈 |
| `09_カメラ・ファイル取込IF（IF-007）.md` | OS Camera API・SHA-256 計算タイミング |
| `10_シーケンス図集（主要フロー）.md` | SEQ-001〜008 の全シーケンス記述 |
| `11_通信暗号化・mTLS・証明書運用.md` | TLS 1.3・社内 CA・証明書ローテーション |
| `12_外部IFテスト方式と契約テスト.md` | スキーマ駆動・Contract Test・モック親機 |
| `99_前提制約と本書が約束しないこと.md` | OT 直結・クラウド SaaS 連携・GraphQL を対象外と明示 |
| `img/` | 図ファイル格納 |

---

## 図一覧

| 図ファイル名（img/ 配下）| 内容 |
|---|---|
| `fig_des_api_topology.{drawio,svg}` | 全 API / IF トポロジ |
| `fig_des_seq_step_execution.{drawio,svg}` | SEQ-001: Step 実行 |
| `fig_des_seq_evidence_sign.{drawio,svg}` | SEQ-002: 証拠記録と電子サイン |
| `fig_des_seq_outbox_sync.{drawio,svg}` | SEQ-003: Outbox 同期 |
| `fig_des_seq_suspend_resume.{drawio,svg}` | SEQ-004: 中断・再開 |
| `fig_des_seq_master_publish.{drawio,svg}` | SEQ-005: マスタ Publish |
| `fig_des_seq_webhook_dlq.{drawio,svg}` | SEQ-006: Webhook DLQ |
| `fig_des_seq_hash_chain_verify.{drawio,svg}` | SEQ-007: ハッシュチェーン検証 |
| `fig_des_seq_master_pull.{drawio,svg}` | SEQ-008: 親機マスタ同期 |
