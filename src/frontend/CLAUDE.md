# src/frontend — フロントエンド共通規約

本ファイルは `handy/`（React Native ハンディ APP）と `master/`（React マスタメンテ Web）の両アプリに共通する規約を記す。
各アプリ固有の実装規約は `handy/CLAUDE.md` / `master/CLAUDE.md` が権威となる。

---

## サブディレクトリの責任境界

| ディレクトリ | 対象 | 主な特性 |
|---|---|---|
| `handy/` | 現場作業者向け React Native アプリ | Offline-First 必須・SQLite ローカル記録・72dp タッチ・3 OS 対応 |
| `master/` | マスタ管理者・監査者向け React Web | 常時オンライン前提・サーバー API のみ・静的 SPA・Audit Trail 表示 |
| `shared/`（将来作成） | 両者共通の純粋関数・型定義 | UI コンポーネントは含まない（RN と React で API が異なるため） |

**マスタ管理機能は `master/` のみが持つ。`handy/` はキャッシュ（読み取り専用差分取得）のみ。**
`handy/` から直接マスタを書き換えるエンドポイントを呼んではならない。

---

## 共通技術選定

権威ドキュメント: `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` 第 2・3 節

- **TypeScript**: 両アプリで必須。`tsconfig.json` の `strict: true`、`any` 禁止
- **React エコシステム統一**: handy は React Native、master は React。カスタムフック・型定義・ビジネスロジックは `shared/` を介して再利用する
- **サーバー状態管理**: TanStack Query（react-query）を両アプリで統一して使用する（異なるライブラリを混在させない）
- **クライアント状態管理**: Zustand を第一候補とする（master が大規模になった場合に ADR で再評価）

---

## 共有コンポーネント方針（`shared/` — 将来作成）

| 共有対象 | 共有しない対象 |
|---|---|
| TypeScript 型定義 | UI コンポーネント（RN と React の JSX API が異なる） |
| 純粋関数（バリデーション・計算） | スタイル定義（StyleSheet vs CSS） |
| JSONB ローカライズユーティリティ | デバイス固有 API（カメラ・Keystore 等） |
| カスタムフック（ビジネスロジック層） | |

判断基準: 「React Native と React の両方で import できるか」を必ず確認してから `shared/` に追加する。

---

## 多言語対応

権威ドキュメント: `docs/02_企画/システム化計画/06_データモデル中核設計.md`

- **i18n ライブラリ**: `react-i18next` を handy / master で統一して使用する
- **翻訳ソース**: サーバーの JSONB `{"ja": "...", "en": "...", "zh": "..."}` を動的に取得してローカライズする。静的な `.json` ファイルにマスタ翻訳をハードコードしない
- **フォールバック**: キーが存在しない場合は `ja` にフォールバックし、最終的にキー文字列を表示する

---

## アクセシビリティ最小ライン

- コントラスト比: WCAG 2.1 AA 相当（通常テキスト 4.5:1 以上・大テキスト 3:1 以上）
- `handy/`: `accessibilityLabel` / `accessibilityRole` を全インタラクティブ要素に付与
- `master/`: `aria-*` 属性を全インタラクティブ要素に付与、キーボードナビゲーション保証

---

## テスト方針共通骨子

| テスト種別 | `handy/` | `master/` |
|---|---|---|
| ユニット | Jest | Vitest |
| コンポーネント | Testing Library (React Native) | Testing Library (React) |
| E2E | Detox | Playwright |

Testing Library の API は両者でほぼ共通のため、テストの書き方を揃える。
モックは外部依存（ネットワーク・DB・デバイス API）のみに限定し、ビジネスロジックはモックしない。

---

## 共通コーディング規約

- **命名**: コンポーネントは PascalCase、関数・変数は camelCase、定数は UPPER_SNAKE_CASE、型は PascalCase
- **ファイル構成**: 1 ファイル 1 コンポーネントを基本とする
- **副作用**: `useEffect` の依存配列を省略しない。`eslint-plugin-react-hooks` を必ず有効にする
- **非同期**: `async/await` を使用し、`.then()` チェーンを混在させない
- **エラー境界**: 画面単位で `ErrorBoundary` を必ず設ける

---

## 参照ドキュメント

- `docs/02_企画/システム化計画/07_技術スタック選定根拠.md` — 第 2 節（handy）・第 3 節（master）技術選定 ADR
- `docs/02_企画/システム化計画/08_品質特性と非機能要件方針.md` — Glanceable / アクセシビリティ方針
- `docs/02_企画/システム化計画/19_マスタ編集者体験設計（オーサリング UX とガバナンス）.md` — master の編集・承認 UX
- `src/CLAUDE.md` — frontend/backend/infra 横断の共通原則

---

最終更新: 2026-05-17
次回見直しトリガー: `src/frontend/shared/` 作成時、または両アプリの初回コード commit 時
