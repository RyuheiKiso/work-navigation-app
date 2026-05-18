# ADR-IMPL-003: ハンディ APP の接続情報を端末内設定画面と QR セットアップで管理

日付: 2026-05-18
状態: 確定
提案者: RyuheiKiso

## 背景

- 詳細設計の記述:
  - `docs/06_実装/12_環境変数とシークレット一覧.md`: `WNAV_FE_HA_API_BASE_URL` の取得元は `.env` / `app.config.js`（Expo ビルド時注入）と記載
  - `docs/08_移行/導入手順/07_ハンディAPP配布手順（統合）.md` L160: 「ハンディ APP の設定画面で本番サーバー URL を入力する」と既に明記
  - `docs/08_移行/導入手順/11_ロールバック実行手順.md` L142: 「各端末のハンディ APP 設定画面からサーバー URL を削除する」
- 実装開始前に発生した問題・制約:
  - ハンディ端末（Android / iOS / Windows タブレット）は MDM 非管理を想定しており、ビルド時の環境変数注入では配布済み APK/IPA を端末ごとに差別化できない
  - ADR-IMPL-001 により設定の SSoT が YAML に移行したが、端末ランタイムは YAML を直接読めない
  - Offline-First 原則（`src/CLAUDE.md` §1）を満たすため、端末はサーバー URL が自律的に保持されている必要がある

## 決定

ハンディ APP はアプリ内設定画面を持ち、端末ごとにサーバー URL・端末 ID を入力する。非機密設定は `AsyncStorage` に、JWT・端末固有鍵は `Expo SecureStore` に保存する。マスタ管理者は `frontend.handy.default_api_base_url` 等から QR コードを生成し、初回セットアップ時に端末でスキャンすることで一括入力できる。

## 理由

### 採用した選択肢の根拠

- **MDM 不要**: 専用 MDM を導入しなくても、QR スキャンで接続先を端末に配布できる。小規模製造現場への導入障壁を下げる
- **Offline-First 整合性**: 端末がサーバー URL をローカルに保持するため、初回セットアップ後はオフラインでも設定の参照に依存しない
- **既存手順書との整合**: `docs/08_移行/導入手順/07_ハンディAPP配布手順（統合）.md` が既に「設定画面で URL を入力」を前提に書かれており、方式変更なく手順を流用できる
- **機密の端末内隔離**: JWT など機密は `Expo SecureStore`（Android Keystore / iOS Keychain / Windows DPAPI）に格納し、YAML や `.env` とは独立したライフサイクルで管理される

### 却下した代替案

| 代替案 | 却下理由 |
|---|---|
| `app.config.js` にビルド時注入 | 端末ごとに接続先が異なる場合に対応できない（工場ごとに別 URL を持つ可能性）。端末配布後の URL 変更に再ビルドが必要 |
| 起動時にサーバーから設定 fetch | 初回起動前にサーバー URL 自体が不明なため chicken-and-egg 問題が発生する |
| `.env` ファイルを端末に配置 | Android / iOS はファイルシステムへのアクセスが制限されており実現困難。Windows タブレットのみ有効で OS 間統一できない |

## 影響

- **影響範囲**:
  - `src/frontend/handy/app/setup/`（新規・セットアップ画面）
  - `src/frontend/handy/app/(config)/`（設定画面・再編集）
  - `src/frontend/handy/src/storage/deviceConfig.ts`（AsyncStorage + SecureStore 読み書き）
  - `src/backend/crates/wnav_master_api/`（QR 生成エンドポイント・将来追加）
- **追加のテスト**:
  - Detox E2E: 初回起動でセットアップ画面が表示されること
  - Detox E2E: サーバー URL 入力後に疎通確認（`GET /healthz`）が行われること
  - ユニットテスト: SecureStore への JWT 保存・読み出し
- **ドキュメント更新**:
  - `docs/06_実装/12_環境変数とシークレット一覧.md` — `WNAV_FE_HA_API_BASE_URL` の取得元を「YAML `frontend.handy.default_api_base_url`（QR 生成用原本）/ 端末内 AsyncStorage（実行時参照）」に変更
  - `docs/08_移行/導入手順/07_ハンディAPP配布手順（統合）.md` — QR セットアップ手順の詳細を追記
  - `docs/08_移行/導入手順/07a_Android配布手順…` / `07b_iOS配布手順…` / `07c_Windows配布手順…` — QR スキャン手順追記

## 参照

- 関連 ADR: ADR-IMPL-001（YAML + figment 一元管理）
- 関連 FR-NNN: —（移行・運用要件に関する判断）
- 上流文書: `docs/08_移行/導入手順/07_ハンディAPP配布手順（統合）.md`（本 ADR の決定を前提として既に記載あり）
