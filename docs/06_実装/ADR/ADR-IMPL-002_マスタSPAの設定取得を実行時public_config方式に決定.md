# ADR-IMPL-002: マスタ SPA の設定取得を実行時 /public/config 方式に決定

日付: 2026-05-18
状態: 確定
提案者: RyuheiKiso

## 背景

- 詳細設計の記述:
  - `docs/06_実装/12_環境変数とシークレット一覧.md`: `WNAV_FE_MA_API_BASE_URL` / `WNAV_FE_MA_OPENAPI_URL` / `WNAV_FE_MA_SESSION_TIMEOUT_MIN` を `.env` / `vite.config.ts` 経由でビルド時注入する方針が記載されている
  - `docs/05_詳細設計/05_WebAPP詳細設計/`（未作成）: マスタ SPA の設定取得方針が未定義
- 実装開始前に発生した問題・制約:
  - ADR-IMPL-001 により設定の SSoT が `src/infra/config/config.{profile}.yml` に移行した。Vite ビルド時に YAML を読み込む仕組みを作ると、YAML（バックエンド側）と Vite 設定（フロントエンド側）の二重管理が発生する
  - マスタ SPA は IIS + WSL2 上のバックエンドと同一サイト（同一オリジン）に配信されるため、実行時に API エンドポイントから設定を取得できる構成が自然
  - ビルド時 `.env.production` を使うと、環境ごとにビルド成果物が分かれ CI 成果物管理が複雑になる

## 決定

マスタ SPA（React + Vite）は起動時に `GET /api/v1/public/config` を呼び出して設定を取得する。このエンドポイントは `wnav_master_api` が実装し、認証不要・ホワイトリスト方式で非機密設定のみを返す。

## 理由

### 採用した選択肢の根拠

- **SSoT の一元化**: バックエンドが `src/infra/config/config.{profile}.yml` を読んで `/public/config` で公開するため、接続先情報が YAML のみで完結する。フロントエンドのビルド設定に環境変数を別途管理する必要がない
- **1 ビルド・全環境動作**: 環境ごとにビルドを分けなくて済む。IIS への配置後にバックエンドを再起動するだけで設定変更が反映される
- **機密混入防止**: `/public/config` レスポンスは `PublicConfig` 構造体で型を限定するため、`secret_ref` を含むフィールドが構造上含まれない。ビルド時 `.env.production` に機密を誤コミットするリスクがない
- **デプロイ整合性**: バックエンドと SPA が同一デプロイパッケージに含まれるため、設定の版ずれが発生しない

### 却下した代替案

| 代替案 | 却下理由 |
|---|---|
| ビルド時 Vite 環境変数注入（`VITE_*`） | 環境別ビルド成果物が増え、YAML（BE）と `.env.production`（FE）の二重管理が発生する。YAML 変更時に再ビルド・再デプロイが必要 |
| マスタ SPA にも設定画面（ハンディと同方式） | マスタは IIS 同一オリジン上で管理者が使うツールであり、インストールごとに URL を手入力するユースケースはない。バックエンドと同一デプロイ前提の SPA に設定画面は過剰 |

## 影響

- **影響範囲**:
  - `src/backend/crates/wnav_master_api/src/api/public_config.rs`（新規エンドポイント）
  - `src/frontend/master/src/config/`（起動時 fetch ロジック）
  - `vite.config.ts`（`VITE_PUBLIC_CONFIG_URL` 環境変数のみ。既定は相対パス `./api/v1/public/config`）
- **追加のテスト**:
  - `GET /api/v1/public/config` のユニットテスト（ホワイトリスト・スキーマ）
  - Playwright E2E: SPA 起動時に `/api/v1/public/config` が 1 回だけ呼ばれること
- **ドキュメント更新**:
  - `docs/06_実装/12_環境変数とシークレット一覧.md` — `WNAV_FE_MA_*` の取得元を「YAML `frontend.master.*` / `/public/config` 経由」に変更
  - `docs/05_詳細設計/05_WebAPP詳細設計/`（将来作成時）— 本 ADR の決定を反映

## 参照

- 関連 ADR: ADR-IMPL-001（YAML + figment 一元管理）
- 関連 FR-NNN: —（非機能要件・運用性に関する判断）
