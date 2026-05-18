# src/frontend/master — マスタメンテ Web 実装規約

React で構築するマスタ管理者・監査者向け Web アプリの実装規約。
横断共通原則は `src/CLAUDE.md`、フロントエンド共通規約は `src/frontend/CLAUDE.md` が権威。
本ファイルはマスタメンテ Web 固有の規約を記す。

権威ドキュメント: `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` 第 3 節

---

## 技術スタック

| 要素 | 採用 |
|---|---|
| フレームワーク | React + Vite + TypeScript（strict mode） |
| UI コンポーネントライブラリ | Ant Design または MUI（実装開始マイルストーン初期に ADR として確定） |
| サーバー状態管理 | TanStack Query（handy と統一） |
| API クライアント | OpenAPI Generator（backend の utoipa 生成スキーマから自動生成） |
| 認証 | JWT RS256（TTL 8h）+ リフレッシュトークン |
| ビルド成果物 | 静的ファイル（IIS または axum から配信・Node.js サーバー不要） |

---

## 対象ユーザーと機能スコープ

| ロール | 主な機能 |
|---|---|
| マスタ管理者 | SOP マスタ・工程マスタ・ユーザーマスタ・設備マスタの CRUD |
| 監査者 | Audit Trail 閲覧・進捗ダッシュボード・未同期記録の識別・時点参照 |

**ハンディ APP のユーザー（現場作業者）はこのアプリを使用しない。**
RBAC 6 ロールで認可を実装し、ロール外の操作は画面・API 両層で禁止する。

---

## マスタ編集 UX 規約

権威ドキュメント: `docs/02_企画/システム化計画/19_マスタ編集者体験設計（オーサリング UX とガバナンス）.md`

- **即時公開禁止**: 編集後は必ずレビュー・承認フローを経由してから公開する
- **差分プレビュー必須**: 変更前後の差分を承認前に表示する
- **影響範囲可視化**: 変更対象マスタを参照中の作業指示・進行中の作業を事前に表示する
- **物理削除禁止**: マスタの削除は論理削除（`deleted_at` タイムスタンプ）のみ。削除後も Audit Trail から参照可能にする

---

## 時点参照の UI 表現

権威ドキュメント: `docs/02_企画/システム化計画/06_データモデル中核設計.md`

- マスタ表示時は「現在版」と「指定時点版」を切り替えられるコントロールを設ける
- Audit Trail では作業実施時点のマスタ版を固定表示する（現在版で過去の作業を書き換えない）
- 時点指定は UTC で管理し、表示はユーザーのタイムゾーンに変換する

---

## 未同期記録の識別表示

権威ドキュメント: `docs/02_企画/システム化計画/05_アーキテクチャ原則.md` 第 1 節

- 「記録済（未同期）」フラグ付きレコードはテーブル・詳細画面で別色（警告色）とアイコンで識別する
- 未同期期間（`server_received_at` と `client_recorded_at` の差分）を表示する
- 監査者が「事後記録」と誤解しないよう、未同期の理由（ネットワーク切断）を明示する

---

## 認証・認可

権威ドキュメント: `docs/02_企画/システム化計画/15_セキュリティ深堀り.md`

- **JWT RS256**: TTL 8h。秘密鍵はバックエンドのみが保有する
- **リフレッシュトークン**: アクセストークンと分離して管理する
- **トークン保管**: `httpOnly` Cookie に保管する。`localStorage` / `sessionStorage` への保管は禁止（XSS 対策）
- **RBAC**: 権限チェックは画面コンポーネント層（表示制御）と API 呼び出し層（実行制御）の両方で行う

---

## データフェッチ規約

- **TanStack Query 必須**: サーバー状態のキャッシュ・ローディング・エラー状態を一元管理する
- **楽観的更新禁止**: 監査対象データの更新は、サーバーでの確定後に UI を更新する（楽観的更新で誤った状態を表示することは監査信頼性を損なう）
- **ポーリング**: Audit Trail の更新検知にはポーリング（例: 30 秒間隔）またはサーバー送信イベント（SSE）を使用する

---

## OpenAPI クライアント生成

バックエンドの `utoipa` が生成する OpenAPI 3.1 スキーマから型安全な API クライアントを生成する。

```bash
# バックエンドのスキーマ生成後に実行
openapi-generator-cli generate \
  -i ../../backend/openapi.json \
  -g typescript-fetch \
  -o src/api/generated/
```

生成ファイル（`src/api/generated/`）は手動で編集しない（再生成で上書きされる）。
カスタマイズが必要な場合は `src/api/` に薄いラッパーを作成する。

---

## テスト

| テスト種別 | ツール | 必須シナリオ |
|---|---|---|
| ユニット | Vitest | ビジネスロジック・日付フォーマット・ローカライズ |
| コンポーネント | Testing Library (React) | フォーム送信・権限による表示制御・未同期フラグ表示 |
| E2E | Playwright | マスタ編集→承認フロー・Audit Trail の時点参照・楽観的更新が起きないこと |

Audit Trail 表示の正確性は E2E でカバーする（ユニットテストだけでは時刻処理・フラグ表示の統合バグを検出しにくい）。

---

## ディレクトリ構成案（暫定）

```
src/frontend/master/
  pages/        # ページコンポーネント（React Router）
  features/     # 機能単位ディレクトリ（sop/ user/ equipment/ audit/ 等）
  api/          # OpenAPI 生成クライアント + 薄いラッパー
  auth/         # JWT 管理・RBAC ヘルパー
  audit/        # Audit Trail 表示・時点参照コントロール
  i18n/         # react-i18next 設定・JSONB ローカライズ
  components/   # 共通 UI（未同期アイコン・差分ビューア・版切替コントロール等）
```

実装開始時に整合性を確認し、変更する場合は ADR に記録する。

---

## Docker

### Dockerfile の配置

`src/frontend/master/Dockerfile` に配置する。ビルドコンテキストは `src/frontend/master/` とする。

### マルチステージビルド構成

```dockerfile
# Stage 1: ビルダー（Vite ビルド）
FROM node:22-slim AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build        # dist/ に静的ファイルを出力

# Stage 2: 静的配信（nginx）
FROM nginx:alpine AS runtime
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
```

- ランタイムイメージに Node.js・ソースコード・`node_modules` を含めない
- `node_modules/` と `dist/` は `.dockerignore` で除外する

### nginx.conf の最小要件

```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    index index.html;

    # SPA のフォールバック（React Router 対応）
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

### 注意事項

- ビルド成果物は静的ファイルのみ。Node.js サーバーをランタイムイメージに含めない（`ビルド成果物: 静的ファイル` 規約に従う）
- `docker-compose.yml` 側でバックエンド URL を環境変数（`VITE_API_BASE_URL` 等）として渡す。コードにハードコードしない
- IIS または axum から配信する本番環境では Docker を使わない場合がある。`npm run build` の成果物（`dist/`）をそのままデプロイすれば十分

---

## 参照ドキュメント

- `docs/02_企画/システム化計画/06_データモデル中核設計.md` — 時点参照固定・マスタ物理削除禁止
- `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` — React 選定根拠・静的 SPA 配信設計
- `docs/02_企画/システム化計画/08_品質特性と非機能要件方針.md` — RBAC 6 ロール・セキュリティ方針
- `docs/02_企画/システム化計画/15_セキュリティ深堀り.md` — JWT / RBAC 詳細
- `docs/02_企画/システム化計画/19_マスタ編集者体験設計（オーサリング UX とガバナンス）.md` — 編集・承認 UX
- `src/CLAUDE.md` — 横断共通原則
- `src/frontend/CLAUDE.md` — フロントエンド共通規約

---

最終更新: 2026-05-18 (Docker セクション追加)
次回見直しトリガー: React 実装開始時、または UI ライブラリ（Ant Design / MUI）を ADR で確定した時
