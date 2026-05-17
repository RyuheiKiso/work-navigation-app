# 02_ソフトウェア方式設計

IPA 共通フレーム 2013「2.4 ソフトウェア方式設計プロセス」全タスクに対応する。

3 つのフロントエンド（ハンディAPP・マスタメンテAPP・管理コンソール）と 1 つのバックエンド（Rust axum）のソフトウェアアーキテクチャ・コンポーネント分割・モジュール責務・エラーハンドリング・バッチ設計を確定する。

---

## 担当する上流要件

| 種別 | 範囲 |
|---|---|
| 機能要件 FR | 全 86 件（NV/EV/ST/MA/SY/KZ/AU/UI）の実装責任をモジュールに配分 |
| NFR-QUA（品質特性）| レイヤードアーキテクチャ・OpenAPI 3.1・E2E テスト |
| NFR-MNT（保守性）| ADR・ログ設計・カバレッジ目標 |
| アーキテクチャ 7 原則 | P1〜P7 全原則のソフトウェア設計への反映 |

---

## 章構成

| ファイル | 目的 |
|---|---|
| `README.md` | 本書 |
| `00_本書の位置づけと識別子規約.md` | IPA 対応・MOD/CMP/TRN/BAT/MSG/ERR/CFG 識別子の確定 |
| `01_ソフトウェア全体アーキテクチャと7原則継承.md` | 7 原則のソフトウェア設計への降ろし方・3 アプリの技術構成 |
| `02_レイヤー構成とパッケージ分割.md` | 4 層（P/A/D/I）・Rust crate / React feature 構成 |
| `03_ドメインモデル骨格と境界づけられたコンテキスト.md` | 5 コンテキスト・27 EN の配置・状態機械 5 種 |
| `04_拡張Stepエンジン設計（プラグイン機構）.md` | 標準 4 タイプ・拡張 Step DSL・DAG 検証・JSON Logic |
| `05_イベント駆動と内部メッセージング.md` | Append-only WorkEvent・Outbox Pattern・ドメインイベント分離 |
| `06_共通基盤コンポーネント設計.md` | i18n・ロギング・ID生成（UUID v7）・時刻・ハッシュチェーン |
| `07_例外・エラーハンドリング統一方式.md` | ERR-NNN カタログ・3 層エラー分類・RFC 9457 |
| `08_並行制御・トランザクション境界.md` | 楽観ロック・Outbox トランザクション境界・SQLite 直列化 |
| `09_キャッシュとオフラインストレージ方式.md` | SQLCipher + TypeORM・マスタキャッシュ TTL・WAL・破損自己修復 |
| `10_バッチ・常駐ジョブ設計.md` | BAT カタログ・Outbox Consumer・バックグラウンドジョブ |
| `11_モジュール一覧（MODカタログ）.md` | 全 MOD-NNN の責務・依存・FR-ID マトリクス |
| `12_デプロイ単位とビルド成果物.md` | Docker Compose 構成・IIS リバプロ・ビルド成果物 |
| `13_実装方針カード集（FR×86件分の方針カード）.md` | 全 86 FR の設計 ID マッピングカード |
| `99_前提制約と本書が約束しないこと.md` | eval・DLL 動的読み替え・マイクロサービス分割 を対象外と明示 |
| `img/` | 図ファイル格納 |

---

## 図一覧

| 図ファイル名（img/ 配下）| 内容 |
|---|---|
| `fig_des_arch_layered.{drawio,svg}` | 3 アプリのレイヤー構成対応図 |
| `fig_des_arch_bounded_context.{drawio,svg}` | 5 境界づけられたコンテキスト |
| `fig_des_arch_plugin_step_engine.{drawio,svg}` | Step エンジン拡張点 |
| `fig_des_arch_outbox_event.{drawio,svg}` | イベント駆動・Outbox 内部フロー |
| `fig_des_arch_mod_catalog.{drawio,svg}` | MOD 依存マップ |
| `fig_des_arch_error_taxonomy.{drawio,svg}` | ERR 分類（3 層）|
