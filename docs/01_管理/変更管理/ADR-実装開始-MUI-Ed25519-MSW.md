# ADR-IMPL-001: フロントエンド実装開始時の保留ADR確定

日付: 2026-05-19
状態: 確定

## 背景

`src/frontend/master/CLAUDE.md` および `src/frontend/terminal/CLAUDE.md` に「実装開始マイルストーン初期に ADR として確定」と明記されていた保留事項が3件あった。フロントエンドの全量実装を開始するにあたり、これらを確定する。

## 決定

### 1. master/ UI コンポーネントライブラリ: MUI (Material UI) を採用

**却下した代替案**: Ant Design

**理由**:
- MUI は `@mui/x-data-grid`（大量レコードの DataGrid 表示）・`@mui/x-date-pickers`（時点参照コントロール）等の enterprise コンポーネントが充実している
- MUI v6 の `sx` プロパティと `ThemeProvider` により `shared/design-tokens` の Steel Navy / Electric Blue ブランドカラーを一元注入できる
- SCR-MC-001 OperationDashboard 等でのグラフは recharts と独立しているためライブラリ選択に依存しない
- Ant Design は中国 CDN 依存のデフォルト設定が工場内 LAN 環境では問題になる可能性があった

### 2. terminal/ 電子署名アルゴリズム: Ed25519 を採用

**却下した代替案**: ECDSA P-256

**理由**:
- Ed25519 は署名・検証が高速（ハンディ端末の低スペック CPU を考慮）
- `@noble/curves/ed25519` は純粋 TypeScript で実装されており、ネイティブモジュール追加不要（Expo OTA 更新の恩恵を最大限受けられる）
- Rust バックエンド側も `ed25519-dalek` クレートで標準対応
- FIPS 認証が不要な環境（工場内 LAN、ISO 9001 規格は Ed25519 を排除していない）であることを確認済み

**実装方法**:
- 秘密鍵は `expo-secure-store` 経由で Android Keystore / iOS Keychain / Windows DPAPI に保管
- 署名結果は WorkEvent の `payload.signature` に 64 バイト hex として格納
- バックエンドが `ed25519-dalek` で検証

### 3. API クライアント戦略: MSW + 手書き OpenAPI 3.1 を採用

**却下した代替案**:
- Backend スタブも同 PR で実装（スコープ膨張リスク）
- OpenAPI スキーマのみ手書きで MSW と型を共有（選択）

**理由**:
- バックエンドは `src/backend/CLAUDE.md` のみ存在し Rust コードは未実装のため、utoipa 生成の `openapi.json` は存在しない
- `docs/05_詳細設計/03_API詳細設計/` に 39 エンドポイントが完全仕様化されているため、手書き OpenAPI 3.1 YAML で型契約を先に確立できる
- `openapi-typescript` で型を生成し `openapi-fetch` で型安全なクライアントを提供
- MSW ハンドラを `shared/mocks/` に集約し master（msw/browser）と terminal（msw/native）の両方が同一ハンドラを再利用
- バックエンド実装後は `shared/openapi/wnav-openapi.yaml` を utoipa 出力に差し替えるだけで移行完了

## 影響

- `src/frontend/shared/openapi/wnav-openapi.yaml` が API 契約の一時的権威となる
- バックエンド実装時に utoipa の出力と差分比較して YAML を更新すること
- MUI v7 へのメジャーアップグレード時は ThemeProvider の破壊的変更を確認する
- Ed25519 から ECDSA P-256 への変更が必要な場合は `crypto/ed25519.ts` の差し替えのみで済むよう `KeystoreAdapter` を抽象化してある
