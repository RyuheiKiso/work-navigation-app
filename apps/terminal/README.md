# apps/terminal

> 対応 §: ロードマップ §7.1 §10.1 §10.6 §11.4.2

Tauri 2.x + React + TypeScript で実装する端末作業ナビアプリ（Android／Windows）。
`src/` に React 側、`src-tauri/` に Rust 側を配置する。

## クリーンアーキテクチャ層

| ディレクトリ | 層 | 例 |
| --- | --- | --- |
| `src/domain/` | 最内層 | `Task`／`TaskId`／`DeviceId`／`LamportTimestamp`／`Repository` interface |
| `src/usecase/` | ユースケース | `StartTaskUseCase` |
| `src/adapter/` | ゲートウェイ | `TauriTaskRepository`（Tauri command 経由） |
| `src/presentation/` | プレゼンテーション | `TaskCard` ほか UI コンポーネント |

## 開発・実行

```bash
# 端末アプリ（Vite 単体）の開発サーバ
pnpm --filter @wna/terminal dev

# Tauri 同梱の実機相当（Rust 側＋ Web フロント）
pnpm --filter @wna/terminal tauri:dev

# 単体テスト
pnpm --filter @wna/terminal test

# 型検査
pnpm --filter @wna/terminal typecheck
```

## ビルド

```bash
# プロダクションビルド（Web フロント）
pnpm --filter @wna/terminal build

# Tauri アプリのバンドル（Android／Windows 等）
pnpm --filter @wna/terminal tauri:build
```

## ビルドターゲットと WSL2 注意

本アプリのターゲットは **Android／Windows のみ**（§6.2 ADR-0008）。Linux デスクトップ向けビルドは想定しないため、`src-tauri` は **ルートの Rust ワークスペースから除外**（ルート `Cargo.toml` の `workspace.exclude`）してある。

WSL2／Linux 環境で `cargo check apps/terminal/src-tauri` を試行すると、Tauri の Linux 依存（`libdbus-1-dev`／`pkg-config`／`gtk` 系）が解決できず失敗する。これは想定通り。

実機ビルドは:

- **Android**: `pnpm --filter @wna/terminal tauri android build`（Android NDK 必須）
- **Windows**: `pnpm --filter @wna/terminal tauri build`（Windows ホスト or WiX 必須）

の手順で行う。

## 依存関係の方向

`presentation → usecase → domain`。`adapter` は `domain` を実装する形で差し込む（依存逆転）。
JSON 設定ファイル（`tauri.conf.json`／`package.json`）はコメント不可のため、本書で補助情報を提供する。
