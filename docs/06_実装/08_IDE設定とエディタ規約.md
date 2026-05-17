# 08 IDE設定とエディタ規約

## 1. VS Code 推奨拡張

以下の拡張機能を全開発者がインストールすることを必須とする。`.vscode/extensions.json` で推奨設定を共有する。

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "mtxr.sqltools",
    "mtxr.sqltools-driver-pg",
    "eamodio.gitlens",
    "editorconfig.editorconfig",
    "bradlc.vscode-tailwindcss",
    "expo.vscode-expo-tools",
    "ms-vscode.vscode-typescript-next",
    "usernamehw.errorlens",
    "gruntfuggly.todo-tree",
    "streetsidesoftware.code-spell-checker",
    "streetsidesoftware.code-spell-checker-japanese",
    "visualstudioexptteam.vscodeintellicode",
    "ms-azuretools.vscode-docker"
  ]
}
```

### 拡張機能の詳細（表形式）

| 拡張名 | 拡張 ID | 用途 |
|---|---|---|
| rust-analyzer | `rust-lang.rust-analyzer` | Rust の補完・型チェック・インライン型表示 |
| Even Better TOML | `tamasfe.even-better-toml` | `Cargo.toml` / `rust-toolchain.toml` のシンタックスハイライト |
| ESLint | `dbaeumer.vscode-eslint` | TypeScript の lint（`eslint.fixOnSave` 連動） |
| Prettier | `esbenp.prettier-vscode` | TypeScript/JSON/YAML のフォーマット |
| SQLTools | `mtxr.sqltools` | PostgreSQL への接続・クエリ実行 |
| SQLTools PostgreSQL | `mtxr.sqltools-driver-pg` | SQLTools の PostgreSQL ドライバ |
| GitLens | `eamodio.gitlens` | git blame・コミット履歴インライン表示 |
| EditorConfig | `editorconfig.editorconfig` | `.editorconfig` 設定の適用 |
| Tailwind CSS IntelliSense | `bradlc.vscode-tailwindcss` | master SPA でのクラス補完（Tailwind 採用時） |
| Expo Tools | `expo.vscode-expo-tools` | `app.json` の補完・EAS 操作 |
| TypeScript Next | `ms-vscode.vscode-typescript-next` | 最新 TypeScript 機能の先行サポート |
| Error Lens | `usernamehw.errorlens` | エラー・警告をインラインで表示 |
| Todo Tree | `gruntfuggly.todo-tree` | `TODO`/`FIXME` の一覧表示（Issue 番号確認） |
| Code Spell Checker | `streetsidesoftware.code-spell-checker` | 英語スペルチェック |
| Code Spell Checker Japanese | `streetsidesoftware.code-spell-checker-japanese` | 日本語コメントのスペルチェック補助 |
| IntelliCode | `visualstudioexptteam.vscodeintellicode` | AI 補完（ローカルモデル） |
| Docker | `ms-azuretools.vscode-docker` | `docker-compose.yml` のシンタックス・コンテナ操作 |

**本節で確定した方針**
- **`.vscode/extensions.json` を必須インストール推奨リストとしてバージョン管理に含める。**
- **`rust-analyzer` と `ESLint` を必須拡張とし、インストールなしの開発を禁止する。**
- **`Todo Tree` を使用して未解決の `TODO`/`FIXME` を定期的に確認し、リリース前に解消する。**

---

## 2. settings.json テンプレ

`.vscode/settings.json` をプロジェクト共有設定としてバージョン管理に含める。

```json
{
  // エディタ共通設定
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": "explicit",
    "source.organizeImports": "never"
  },
  "editor.rulers": [100],
  "editor.tabSize": 2,
  "editor.insertSpaces": true,
  "editor.detectIndentation": false,
  "editor.trimAutoWhitespace": true,
  "editor.linkedEditing": true,
  "files.eol": "\n",
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true,

  // rust-analyzer の設定
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true,
  "rust-analyzer.semanticHighlighting.operator.enable": true,

  // ESLint の設定
  "eslint.validate": ["typescript", "typescriptreact"],
  "eslint.format.enable": false,
  "eslint.codeActionsOnSave.mode": "problems",

  // Prettier の設定
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[json]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.tabSize": 4
  },
  "[toml]": {
    "editor.defaultFormatter": "tamasfe.even-better-toml"
  },

  // SQLTools の設定
  "sqltools.connections": [
    {
      "name": "WNav Dev (app_write)",
      "driver": "PostgreSQL",
      "host": "localhost",
      "port": 5432,
      "database": "wnav_dev",
      "username": "wnav_write",
      "usePassword": "Ask on connect"
    },
    {
      "name": "WNav Dev (app_read)",
      "driver": "PostgreSQL",
      "host": "localhost",
      "port": 5432,
      "database": "wnav_dev",
      "username": "wnav_read",
      "usePassword": "Ask on connect"
    }
  ],

  // ファイル除外設定（パフォーマンス向上）
  "files.watcherExclude": {
    "**/target/**": true,
    "**/node_modules/**": true,
    "**/.git/objects/**": true
  },
  "search.exclude": {
    "**/target": true,
    "**/node_modules": true,
    "**/.sqlx": true
  }
}
```

**本節で確定した方針**
- **`editor.formatOnSave: true` を必須設定とし、保存時の自動フォーマットを有効にする。**
- **`rust-analyzer.checkOnSave.command: "clippy"` で保存時に clippy チェックを自動実行する。**
- **SQLTools の接続設定でパスワードを `"usePassword": "Ask on connect"` にし、設定ファイルへの記載を禁止する。**

---

## 3. EditorConfig

プロジェクトルートに `.editorconfig` を配置し、エディタ間の設定差異を排除する。

```ini
# .editorconfig
# https://editorconfig.org

root = true

# 全ファイルのデフォルト設定
[*]
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true
indent_style = space
indent_size = 2

# Rust: インデントサイズは 4 スペース
[*.rs]
indent_size = 4
max_line_length = 100

# TOML: インデントサイズは 2 スペース
[*.toml]
indent_size = 2

# Markdown: 末尾スペースは一部で有効な構文（改行）のため除外
[*.md]
trim_trailing_whitespace = false

# Windows バッチファイル: CRLF が必要
[*.bat]
end_of_line = crlf

# PowerShell: LF（WSL2/Linux 互換のため）
[*.ps1]
end_of_line = lf

# SQL: インデントサイズは 4 スペース
[*.sql]
indent_size = 4

# YAML: インデントサイズは 2 スペース
[*.yml]
indent_size = 2
[*.yaml]
indent_size = 2
```

**本節で確定した方針**
- **`.editorconfig` で `end_of_line = lf` を全ファイルに適用し、CRLF の混入を排除する。**
- **Rust は `indent_size = 4`・TypeScript/YAML は `indent_size = 2` で統一する。**
- **`insert_final_newline = true` を設定し、ファイル末尾の改行を保証する。**

---

## 4. .gitattributes

```gitattributes
# .gitattributes
# テキストファイルの行末を LF に正規化する
* text=auto eol=lf

# Windows バッチファイルは CRLF を維持する
*.bat text eol=crlf
*.cmd text eol=crlf

# バイナリファイル（差分・マージを無効化する）
*.drawio binary
*.svg binary
*.png binary
*.jpg binary
*.jpeg binary
*.gif binary
*.ico binary
*.ttf binary
*.woff binary
*.woff2 binary
*.pdf binary
*.zip binary
*.tar.gz binary

# Rust のロックファイルはユニオンマージ戦略を使用する
# （複数ブランチの依存変更をマージするとき）
Cargo.lock merge=union

# pnpm のロックファイルはユニオンマージ戦略を使用する
pnpm-lock.yaml merge=union

# 生成ファイルは差分表示から除外する（diff tool では参照可能）
*.min.js -diff
src/frontend/master/src/api/generated/** linguist-generated=true
```

**本節で確定した方針**
- **`.gitattributes` で `* text=auto eol=lf` を設定し、改行コードを LF に統一する。**
- **`.drawio` と `.svg` を `binary` として扱い、テキスト差分でのマージを禁止する。**
- **`Cargo.lock` と `pnpm-lock.yaml` に `merge=union` を設定し、依存更新のコンフリクトを最小化する。**

---

## 5. デバッグ設定

`.vscode/launch.json` でデバッグ設定を定義する。

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "🦀 Debug wnav-api (Rust)",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/target/debug/wnav-api",
      "args": [],
      "cwd": "${workspaceFolder}/src/backend",
      "env": {
        "RUST_LOG": "wnav_api=debug,tower_http=trace",
        "WNAV_BE_LISTEN_ADDR": "0.0.0.0:8080"
      },
      "sourceLanguages": ["rust"],
      "preLaunchTask": "🦀 cargo build (debug)"
    },
    {
      "name": "📱 Expo Go (QR scan)",
      "type": "node",
      "request": "attach",
      "port": 19000,
      "cwd": "${workspaceFolder}/src/frontend/handy",
      "sourceMaps": true,
      "trace": false
    },
    {
      "name": "🌐 Attach to master (Node debug)",
      "type": "node",
      "request": "attach",
      "port": 9229,
      "cwd": "${workspaceFolder}/src/frontend/master",
      "sourceMaps": true,
      "outFiles": [
        "${workspaceFolder}/src/frontend/master/dist/**/*.js"
      ]
    }
  ]
}
```

**本節で確定した方針**
- **`launch.json` を `.vscode/` に配置してバージョン管理し、デバッグ設定を開発者間で共有する。**
- **Rust デバッガは `lldb`（CodeLLDB 拡張）を使用し、Rust 固有の型（`Vec<T>`・`Option<T>`）を正しく表示する。**
- **デバッグ設定の環境変数は開発環境の値のみを含め、本番のシークレットを記載しない。**

---

## 6. タスク定義

`.vscode/tasks.json` でよく使うタスクを定義する。

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "🦀 cargo nextest run",
      "type": "shell",
      "command": "cargo nextest run",
      "options": {
        "cwd": "${workspaceFolder}/src/backend"
      },
      "group": "test",
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": "$rustc"
    },
    {
      "label": "🦀 cargo build (debug)",
      "type": "shell",
      "command": "cargo build",
      "options": {
        "cwd": "${workspaceFolder}/src/backend"
      },
      "group": "build",
      "problemMatcher": "$rustc"
    },
    {
      "label": "📦 pnpm test (master)",
      "type": "shell",
      "command": "pnpm test",
      "options": {
        "cwd": "${workspaceFolder}/src/frontend/master"
      },
      "group": "test"
    },
    {
      "label": "📦 pnpm test (handy)",
      "type": "shell",
      "command": "pnpm test",
      "options": {
        "cwd": "${workspaceFolder}/src/frontend/handy"
      },
      "group": "test"
    },
    {
      "label": "🗄️ sqlx migrate run",
      "type": "shell",
      "command": "sqlx migrate run",
      "options": {
        "cwd": "${workspaceFolder}/src/backend"
      },
      "problemMatcher": []
    },
    {
      "label": "📱 expo start",
      "type": "shell",
      "command": "expo start",
      "options": {
        "cwd": "${workspaceFolder}/src/frontend/handy"
      },
      "isBackground": true,
      "problemMatcher": {
        "pattern": {
          "regexp": "^.*$",
          "message": 1
        },
        "background": {
          "activeOnStart": true,
          "beginsPattern": "Starting Metro Bundler",
          "endsPattern": "Metro waiting"
        }
      }
    },
    {
      "label": "🐳 docker compose up",
      "type": "shell",
      "command": "docker compose up -d",
      "options": {
        "cwd": "${workspaceFolder}"
      },
      "problemMatcher": []
    }
  ]
}
```

**本節で確定した方針**
- **`tasks.json` にテスト・ビルド・マイグレーション・起動コマンドを定義し、ターミナルコマンドの暗記を不要にする。**
- **`expo start` はバックグラウンドタスクとして設定し、起動確認後に他のタスクを実行できるようにする。**
- **タスク名にアイコンプレフィックスを付けて視認性を確保する（🦀=Rust / 📦=Node / 🗄️=DB / 📱=Expo / 🐳=Docker）。**

---

## 7. JetBrains 補足

VS Code を推奨エディタとするが、JetBrains IDE を使用する場合の補足設定を記載する。

### RustRover（Rust バックエンド）

```
Preferences > Build, Execution, Deployment > Cargo
  - Cargo project: /home/ryuhei_kiso/github/work-navigation-app/src/backend/Cargo.toml
  - Toolchain: stable
  - Rustfmt: 有効（Reformat on Save: ON）
  - Clippy: 有効（Check in background: ON）

Preferences > Editor > Code Style > Rust
  - Hard wrap at: 100 columns
```

### WebStorm（TypeScript フロントエンド）

```
Preferences > Languages & Frameworks > TypeScript
  - TypeScript version: Project（pnpm-workspace の TypeScript を使用）
  - Enable TypeScript service: ON
  - tsconfig.json: プロジェクトのファイルを指定

Preferences > Editor > Inspections > TypeScript
  - Implicit any type: Error
  - Type mismatch: Error

ESLint の設定:
  - Automatic ESLint configuration: ON
  - Run eslint --fix on save: ON
```

### 検索・置換の正規表現（JetBrains 共通）

| 操作 | 正規表現パターン |
|---|---|
| 未使用の `unwrap()` を検出 | `\.unwrap\(\)` |
| `any` 型を検出 | `: any\b` |
| 日本語コメントのない `pub fn` を検出 | `(?<!\/\/[^\n]*)\n(pub fn )` |

**本節で確定した方針**
- **JetBrains IDE でも Rustfmt の `Hard wrap at: 100` を設定し、VS Code と同一の行幅制限を適用する。**
- **WebStorm の ESLint 設定で `Run eslint --fix on save` を有効にし、VS Code との動作を統一する。**
- **IDE 固有の設定ファイル（`.idea/`）は `.gitignore` に追加し、バージョン管理対象から除外する。**

---

## 8. ターミナル設定

### WSL2 プロファイル（Windows Terminal）

```json
{
  "profiles": {
    "list": [
      {
        "name": "Ubuntu-24.04 (WNav Dev)",
        "source": "Windows.Terminal.Wsl",
        "commandline": "wsl.exe -d Ubuntu-24.04",
        "startingDirectory": "//wsl$/Ubuntu-24.04/home/ryuhei_kiso/github/work-navigation-app",
        "font": {
          "face": "JetBrains Mono",
          "size": 13
        },
        "colorScheme": "One Half Dark",
        "cursorShape": "bar"
      }
    ]
  }
}
```

### JetBrains Mono フォント

JetBrains Mono は以下の利点があるため、開発環境の標準フォントとして採用する。

- Rust の生存期間記号（`'`）・トレイト境界（`::`）・クロージャ（`|`）が読みやすい
- コーディングリガチャ（`!=`・`>=`・`<=`・`->`・`=>`）をサポート
- 日本語・漢字の混在コメントとの可読性が高い

```bash
# Ubuntu 24.04 での JetBrains Mono インストール
mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
wget https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip
unzip JetBrainsMono-2.304.zip
fc-cache -fv
```

### 配色テーマ推奨

| テーマ | 特性 | 推奨場面 |
|---|---|---|
| **One Half Dark** | コントラストが高く視認性良好 | 通常開発 |
| **Solarized Dark** | 目の疲れが少ない | 長時間作業 |
| VS Code Dark+ | VS Code との視認性統一 | VS Code 移行時 |

```bash
# zsh の設定（プロンプトとエイリアス）
# ~/.zshrc に追加する
alias cr="cargo run"
alias ct="cargo nextest run"
alias cf="cargo fmt"
alias cc="cargo clippy -- -D warnings"
alias pd="pnpm dev"
alias pt="pnpm test"
alias es="expo start"
alias dcu="docker compose up -d"
alias dcd="docker compose down"
```

**本節で確定した方針**
- **JetBrains Mono をターミナル・IDE の標準フォントとして採用し、コーディングリガチャを有効にする。**
- **Windows Terminal のプロファイルの `startingDirectory` をプロジェクトルートに設定する。**
- **`~/.zshrc` にエイリアスを設定し、頻繁に使用するコマンドの入力コストを削減する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)

### 関連
- [`90_業界分析/08_人間工学と作業負荷.md`](../../90_業界分析/08_人間工学と作業負荷.md)
- [`90_業界分析/12_認知工学と状況認識.md`](../../90_業界分析/12_認知工学と状況認識.md)
