# 11 CI/CDパイプライン設定

本章は GitHub Actions を用いた CI/CD パイプラインの設計・設定仕様を定める。品質ゲートを自動化し、リリースの再現性と安全性を保証することを目的とする。

---

## 1. CI/CD 全体

**図 1: メイン CI/CD ワークフロー**

![図 1 メイン CI/CD ワークフロー](img/fig_cicd_workflow_main.svg)

> 原本: [`img/fig_cicd_workflow_main.drawio`](img/fig_cicd_workflow_main.drawio)

CI/CD パイプラインは GitHub Actions で構成する。主要ワークフローは以下の 2 種類である。

| ワークフロー | トリガー | 対象 | 目的 |
|---|---|---|---|
| `ci.yml` | PR 作成・更新・main へのマージ | 全コンポーネント | 品質ゲート通過の保証 |
| `release.yml` | タグ push（`v*.*.*`） | 全コンポーネント | リリースビルド・配信 |

パイプラインは「コードを push してから本番に届くまでの全工程を自動化し、人間の確認が必要なステップのみ手動介入点を設ける」という原則で設計する。

**本節で確定した方針**
- **GitHub Actions を CI/CD 基盤として採用する**: セルフホスト Runner は対象外と判断する
- **PR 単位で品質ゲートを通過させる**: main ブランチに品質未達のコードを直接 push しない
- **タグ push をリリーストリガーとする**: 意図しないリリースを防ぐ

---

## 2. パイプライン段階

| 段階 | ジョブ名 | 内容 | 失敗時の挙動 |
|---|---|---|---|
| 1 | `lint` | cargo clippy / ESLint / Prettier チェック | 即時停止（後続ジョブをブロック） |
| 2 | `unit-test` | cargo nextest / Jest / Vitest | 即時停止 |
| 3 | `integration-test` | testcontainers-rs + PostgreSQL コンテナ | 即時停止 |
| 4 | `build` | cargo build --release / pnpm build / eas build | 即時停止 |
| 5 | `security-scan` | cargo audit / pnpm audit / SAST | クリティカル脆弱性がある場合のみ停止 |
| 6 | `artifact-push` | Docker image を ghcr.io に push | 即時停止 |
| 7 | `deploy` | staging または prod へのデプロイ | 即時停止（ロールバック通知） |

### 段階の依存関係

```
lint ──┬──→ unit-test ──→ integration-test ──→ build ──→ security-scan ──→ artifact-push ──→ deploy
       │
       └── (並列実行可能: unit-test と integration-test は別ジョブとして並列実行する)
```

`security-scan` は `build` 完了後に実行するが、Medium 以下の脆弱性は警告のみとして継続する。Critical のみを停止条件とする。

**本節で確定した方針**
- **7 段階のパイプラインを定義する**: lint → unit-test → integration-test → build → security-scan → artifact-push → deploy の順序を維持する
- **前段が失敗した場合は後続ジョブをブロックする**: 品質ゲートを迂回できない構造とする
- **security-scan の停止条件は Critical のみとする**: Medium/Low は警告として記録し、運用で対応する

---

## 3. 品質ゲート

**図 2: 品質ゲート判定フロー**

![図 2 品質ゲート判定フロー](img/fig_quality_gate.svg)

> 原本: [`img/fig_quality_gate.drawio`](img/fig_quality_gate.drawio)

品質ゲートは CI パイプラインの `unit-test` / `integration-test` / `security-scan` 段階で自動評価される。

### 3.1 カバレッジ目標

| コンポーネント | ライン カバレッジ | ブランチ カバレッジ | 特記 |
|---|---|---|---|
| backend（Rust） | ≥ 80% | ≥ 70% | ドメインロジック単体 ≥ 95% |
| handy（React Native） | ≥ 75% | ≥ 65% | Step エンジン単体 ≥ 95% |
| master（React） | ≥ 75% | ≥ 65% | — |

カバレッジレポートは `cargo-llvm-cov`（Rust）および `vitest --coverage`（TypeScript）で生成し、GitHub Actions の `actions/upload-artifact` で保存する。

### 3.2 静的解析・セキュリティ

| チェック | ツール | 停止条件 |
|---|---|---|
| Rust 静的解析 | `cargo clippy -- -D warnings` | 警告 1 件以上 |
| Rust セキュリティ監査 | `cargo audit` | Critical 脆弱性 1 件以上 |
| JS/TS セキュリティ監査 | `pnpm audit --audit-level=critical` | Critical 脆弱性 1 件以上 |
| Rust コードフォーマット | `cargo fmt --check` | フォーマット差分あり |
| TS コードフォーマット | `pnpm exec prettier --check` | フォーマット差分あり |

### 3.3 ドキュメント品質

docs/ ディレクトリに変更がある場合、以下のチェックを追加実行する。

```bash
# drawio-lint で図ファイルの規約違反を検出する
drawio-lint docs/**/*.drawio

# svg-postcheck で SVG エクスポートの品質を確認する
svg-postcheck docs/**/*.svg
```

`drawio-lint` / `svg-postcheck` の ERROR が 1 件でもある場合はパイプラインを停止する。

**本節で確定した方針**
- **カバレッジ目標を CI で自動評価する**: 目標未達はパイプライン停止とする
- **Clippy 警告 0 を必須とする**: `-- -D warnings` フラグで警告をエラーとして扱う
- **docs 変更時は drawio-lint / svg-postcheck を必須実行する**: 図品質の自動保証

---

## 4. GitHub Actions ワークフロー定義

### 4.1 ci.yml（PR トリガー）

```yaml
name: CI

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Rust ツールチェーンをセットアップする
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      # Cargo キャッシュを復元する
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      # pnpm をセットアップする
      - uses: pnpm/action-setup@v3
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      # Rust 静的解析を実行する
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
        working-directory: src/backend

      # Rust フォーマットチェックを実行する
      - name: Rustfmt
        run: cargo fmt --check
        working-directory: src/backend

      # TypeScript Lint を実行する
      - name: ESLint (handy)
        run: pnpm install --frozen-lockfile && pnpm lint
        working-directory: src/frontend/handy

      - name: ESLint (master)
        run: pnpm install --frozen-lockfile && pnpm lint
        working-directory: src/frontend/master

  unit-test:
    name: Unit Test
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      # cargo-nextest をインストールして並列実行する
      - name: Install nextest
        uses: taiki-e/install-action@nextest

      # Rust 単体テストを実行する
      - name: Cargo Nextest
        run: cargo nextest run --all-features
        working-directory: src/backend

      # TypeScript 単体テストを実行する
      - uses: pnpm/action-setup@v3
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Jest (handy)
        run: pnpm install --frozen-lockfile && pnpm test --coverage
        working-directory: src/frontend/handy

      - name: Vitest (master)
        run: pnpm install --frozen-lockfile && pnpm test --coverage
        working-directory: src/frontend/master

      # カバレッジレポートを保存する
      - uses: actions/upload-artifact@v4
        with:
          name: coverage-reports
          path: |
            src/backend/coverage/
            src/frontend/handy/coverage/
            src/frontend/master/coverage/

  integration-test:
    name: Integration Test
    runs-on: ubuntu-latest
    needs: lint
    services:
      # PostgreSQL コンテナを起動する（testcontainers の代わりにサービスコンテナを使用する）
      postgres:
        image: postgres:17
        env:
          POSTGRES_PASSWORD: test_password
          POSTGRES_DB: wnav_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      # マイグレーションを適用する
      - name: Run migrations
        env:
          DATABASE_URL: postgres://postgres:test_password@localhost:5432/wnav_test
        run: cargo sqlx migrate run
        working-directory: src/backend

      # 統合テストを実行する
      - name: Integration tests
        env:
          DATABASE_URL: postgres://postgres:test_password@localhost:5432/wnav_test
          WNAV_TERMINAL_PORT: 18080
          WNAV_MASTER_PORT: 18081
        run: cargo nextest run --test '*' --all-features
        working-directory: src/backend

  security-scan:
    name: Security Scan
    runs-on: ubuntu-latest
    needs: [unit-test, integration-test]
    steps:
      - uses: actions/checkout@v4

      # Rust セキュリティ監査を実行する
      - name: Cargo Audit
        uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      # pnpm セキュリティ監査を実行する
      - uses: pnpm/action-setup@v3
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: pnpm audit (handy)
        run: pnpm install --frozen-lockfile && pnpm audit --audit-level=critical
        working-directory: src/frontend/handy

      - name: pnpm audit (master)
        run: pnpm install --frozen-lockfile && pnpm audit --audit-level=critical
        working-directory: src/frontend/master
```

### 4.2 release.yml（タグ push トリガー）

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

env:
  REGISTRY: ghcr.io
  # monorepo のため両バイナリで同一の SemVer タグを使用する
  IMAGE_TERMINAL_API: ${{ github.repository_owner }}/wnav-terminal-api
  IMAGE_MASTER_API: ${{ github.repository_owner }}/wnav-master-api

jobs:
  build-and-push:
    name: Build and Push Docker Images
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4

      # Docker Buildx をセットアップする
      - uses: docker/setup-buildx-action@v3

      # ghcr.io にログインする
      - name: Login to ghcr.io
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      # terminal-api のイメージメタデータを生成する
      - name: Extract metadata (terminal-api)
        id: meta-terminal
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_TERMINAL_API }}
          tags: |
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=raw,value=latest,enable={{is_default_branch}}

      # master-api のイメージメタデータを生成する
      - name: Extract metadata (master-api)
        id: meta-master
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_MASTER_API }}
          tags: |
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=raw,value=latest,enable={{is_default_branch}}

      # terminal-api Docker イメージをビルドして push する
      - name: Build and push (terminal-api)
        uses: docker/build-push-action@v5
        with:
          context: src/backend
          build-args: |
            BIN=wnav_terminal_api
          push: true
          tags: ${{ steps.meta-terminal.outputs.tags }}
          labels: ${{ steps.meta-terminal.outputs.labels }}
          cache-from: type=gha,scope=terminal-api
          cache-to: type=gha,mode=max,scope=terminal-api

      # master-api Docker イメージをビルドして push する
      - name: Build and push (master-api)
        uses: docker/build-push-action@v5
        with:
          context: src/backend
          build-args: |
            BIN=wnav_master_api
          push: true
          tags: ${{ steps.meta-master.outputs.tags }}
          labels: ${{ steps.meta-master.outputs.labels }}
          cache-from: type=gha,scope=master-api
          cache-to: type=gha,mode=max,scope=master-api

  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    needs: build-and-push
    environment: staging
    steps:
      - name: Deploy to staging
        env:
          IMAGE_TAG: ${{ github.ref_name }}
          WNAV_PROFILE: staging
          # 機密のみ GitHub Secrets から注入する（非機密設定は config.staging.yml で管理）
          WNAV_DB_PASSWORD_WRITE: ${{ secrets.STAGING_DB_PASSWORD_WRITE }}
          WNAV_DB_PASSWORD_EVENT_INSERT: ${{ secrets.STAGING_DB_PASSWORD_EVENT_INSERT }}
          WNAV_DB_PASSWORD_READ: ${{ secrets.STAGING_DB_PASSWORD_READ }}
          WNAV_BE_JWT_SECRET: ${{ secrets.STAGING_JWT_SECRET }}
          WNAV_BE_JWT_PUBLIC_KEY: ${{ secrets.STAGING_JWT_PUBLIC_KEY }}
          WNAV_BE_WEBHOOK_SECRET: ${{ secrets.STAGING_WEBHOOK_SECRET }}
          WNAV_TERMINAL_DATABASE_URL: ${{ secrets.STAGING_TERMINAL_DATABASE_URL }}
          WNAV_MASTER_DATABASE_URL: ${{ secrets.STAGING_MASTER_DATABASE_URL }}
        run: |
          # staging 環境に SSH してデプロイコマンドを実行する
          echo "Deploying terminal-api $IMAGE_TAG to staging..."
          echo "Deploying master-api $IMAGE_TAG to staging..."

  create-release:
    name: Create GitHub Release
    runs-on: ubuntu-latest
    needs: deploy-staging
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          draft: false
```

**本節で確定した方針**
- **ci.yml は PR トリガー・release.yml はタグ push トリガーとする**: トリガーを分離して責務を明確にする
- **ジョブは依存関係を `needs` で明示する**: 実行順序を宣言的に管理する
- **ghcr.io を Docker イメージレジストリとして採用する**: GitHub との統合が簡単で追加費用が不要
- **2 バイナリは同一の SemVer タグを使用する**: monorepo 構成のため両バイナリのバージョンは常に一致させる
- **Dockerfile の `--bin` 引数でビルドバイナリを切り替える**: `wnav_terminal_api` / `wnav_master_api` をそれぞれ `BIN` ビルド引数で指定する

---

## 5. キャッシュ戦略

ビルド時間を短縮するため、以下のキャッシュを `actions/cache@v4` で管理する。

| キャッシュ対象 | キャッシュキー | 効果 |
|---|---|---|
| `~/.cargo/registry` / `~/.cargo/git` | `Cargo.lock` のハッシュ | Rust クレートのダウンロード省略 |
| `target/` | `Cargo.lock` + ソースのハッシュ | インクリメンタルビルドの活用 |
| `~/.pnpm-store` | `pnpm-lock.yaml` のハッシュ | npm パッケージのダウンロード省略 |
| Docker レイヤ | `type=gha`（GitHub Actions Cache） | Docker ビルドの中間レイヤ再利用 |

```yaml
# Cargo キャッシュの設定例
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

**本節で確定した方針**
- **Cargo.lock / pnpm-lock.yaml をキャッシュキーとする**: 依存関係変更時に自動でキャッシュを更新する
- **Docker レイヤキャッシュは GHA キャッシュを使用する**: セルフホスト Registry は使用しない
- **target/ ディレクトリをキャッシュする**: Rust のインクリメンタルビルドでビルド時間を削減する

---

## 6. シークレット注入

### 6.1 GitHub Actions Secrets の使用

```yaml
# ワークフロー内でのシークレット参照例（2 バイナリ対応）
env:
  # terminal-api 固有のシークレット
  WNAV_TERMINAL_DATABASE_URL: ${{ secrets.WNAV_TERMINAL_DATABASE_URL }}
  # master-api 固有のシークレット
  WNAV_MASTER_DATABASE_URL: ${{ secrets.WNAV_MASTER_DATABASE_URL }}
  # 両バイナリ共通のシークレット（同一の GitHub Secret から注入する）
  WNAV_TERMINAL_JWT_SECRET: ${{ secrets.WNAV_JWT_SECRET }}
  WNAV_TERMINAL_JWT_PUBLIC_KEY: ${{ secrets.WNAV_JWT_PUBLIC_KEY }}
  WNAV_MASTER_JWT_SECRET: ${{ secrets.WNAV_JWT_SECRET }}
  WNAV_MASTER_JWT_PUBLIC_KEY: ${{ secrets.WNAV_JWT_PUBLIC_KEY }}
```

シークレットは `Settings > Secrets and variables > Actions` で管理する。環境別シークレットは `staging` / `prod` 環境（`environment`）に紐付けて管理し、最小権限原則に従う。

### 6.2 OIDC Workload Identity（将来検討）

GitHub Actions OIDC を使用した Workload Identity Federation は、静的なシークレットを使わずにクラウドリソースにアクセスできる仕組みである。現在は Windows Server 2022 / WSL2 へのデプロイが主体のため適用外と判断するが、将来クラウドリソースを利用する場合は ADR-IMPL-NNN として記録し採用を検討する。

**本節で確定した方針**
- **シークレットは GitHub Actions Secrets で管理する**: ハードコードおよびログへの出力を禁止する
- **環境別シークレットは environment スコープで分離する**: staging と prod で異なるシークレットを使用する
- **OIDC Workload Identity は将来検討とする**: 現フェーズでは対象外と判断する

---

## 7. アーティファクト管理

| アーティファクト | 保存先 | 保持期間 | タグ命名規則 |
|---|---|---|---|
| Docker image (terminal-api) | ghcr.io/ryuhei-kiso/wnav-terminal-api | 30 日（`latest` は永続） | `v{version}` / `latest` |
| Docker image (master-api) | ghcr.io/ryuhei-kiso/wnav-master-api | 30 日（`latest` は永続） | `v{version}` / `latest` |
| カバレッジレポート | GitHub Actions artifacts | 30 日 | PR 番号 + コミットハッシュ |
| テスト結果（JUnit XML） | GitHub Actions artifacts | 30 日 | PR 番号 + コミットハッシュ |
| EAS ビルド成果物 | EAS servers | 30 日 | eas.json の `buildNumber` |

```bash
# Docker イメージのタグ命名例（monorepo のため両バイナリで同一タグを使用する）
ghcr.io/ryuhei-kiso/wnav-terminal-api:v1.0.0    # バージョンタグ（変更不可）
ghcr.io/ryuhei-kiso/wnav-terminal-api:v1.0      # マイナーバージョンタグ（パッチで更新）
ghcr.io/ryuhei-kiso/wnav-terminal-api:latest    # 最新安定版（main ブランチのみ）

ghcr.io/ryuhei-kiso/wnav-master-api:v1.0.0      # バージョンタグ（変更不可）
ghcr.io/ryuhei-kiso/wnav-master-api:v1.0        # マイナーバージョンタグ（パッチで更新）
ghcr.io/ryuhei-kiso/wnav-master-api:latest      # 最新安定版（main ブランチのみ）
```

**本節で確定した方針**
- **Docker image は ghcr.io に push して 30 日間保持する**: ロールバック用の前バージョンイメージを保持する
- **タグ命名は `v{semver}` で統一する**: `latest` タグは main ブランチのビルドにのみ付与する
- **monorepo のため両バイナリで同一の SemVer タグを使用する**: terminal-api と master-api は常に同じバージョンタグで管理する
- **カバレッジ・テスト結果は artifacts として 30 日保持する**: 品質トレンドの確認を可能にする

---

## 8. リリースワークフロー

**図 3: リリースワークフロー**

![図 3 リリースワークフロー](img/fig_cicd_workflow_release.svg)

> 原本: [`img/fig_cicd_workflow_release.drawio`](img/fig_cicd_workflow_release.drawio)

リリースフローは以下の順序で実行される。

```
1. git tag v1.0.0 でタグを打つ
2. git push origin v1.0.0 でタグを push する
3. release.yml が自動発火する
4. wnav-terminal-api / wnav-master-api の Docker イメージをビルドして ghcr.io に push する（同一 SemVer タグを付与）
5. staging 環境に自動デプロイして両バイナリの動作確認する
6. GitHub Releases を自動作成する（draft: false）
7. 手動確認後に prod へのデプロイコマンドを実行する（セルフレビュー必須）
```

```bash
# リリースタグを作成してリモートに push する
git tag -a v1.0.0 -m "Release v1.0.0: 初回リリース"
git push origin v1.0.0
```

**本節で確定した方針**
- **リリースは git tag push で開始する**: ブランチ push ではなくタグ push をトリガーとする
- **staging への自動デプロイを release.yml に含める**: 本番デプロイ前の検証を自動化する
- **prod への最終デプロイは手動コマンドで実施する**: 自動化の範囲は staging までとする

---

## 9. 失敗時のアラート

CI パイプラインが失敗した場合、以下のアラートを発する。

```yaml
# パイプライン失敗時に GitHub Issue を自動作成する
- name: Create issue on failure
  if: failure()
  uses: actions/github-script@v7
  with:
    script: |
      await github.rest.issues.create({
        owner: context.repo.owner,
        repo: context.repo.repo,
        title: `CI Failure: ${context.workflow} on ${context.ref}`,
        body: `## CI パイプライン失敗\n\n` +
              `- ワークフロー: ${context.workflow}\n` +
              `- ブランチ/タグ: ${context.ref}\n` +
              `- コミット: ${context.sha}\n` +
              `- 実行 URL: ${context.serverUrl}/${context.repo.owner}/${context.repo.repo}/actions/runs/${context.runId}\n`,
        labels: ['ci-failure', 'bug']
      });
```

加えて、GitHub のメール通知設定でパイプライン失敗時にメール通知を受け取る。

**本節で確定した方針**
- **パイプライン失敗時に GitHub Issue を自動作成する**: 失敗を見落とさないようにする
- **メール通知を併用する**: Issue 作成のみに依存しない通知体制とする
- **失敗 Issue には実行 URL を含める**: 原因調査を容易にする

---

## 10. セルフホスト Runner を採用しない理由

| 観点 | 評価 |
|---|---|
| ビルド速度 | GitHub-hosted runner（ubuntu-latest）でのビルド時間は 15 分以内を見込む。個人開発のコミット頻度では許容範囲内 |
| コスト | GitHub Free プランの月 2,000 分（パブリックリポジトリは無制限）で十分 |
| 運用負荷 | セルフホスト Runner の維持・セキュリティパッチ適用が追加の運用負荷となる |
| セキュリティ | GitHub-hosted runner は各ジョブ後にクリーンアップされるため、シークレットの持越しリスクがない |

個人開発・単一インスタンス構成において、GitHub-hosted runner の速度と利便性は十分であり、セルフホスト Runner の複雑性を正当化できない。採用しないことを ADR-IMPL-001 として記録する。

**本節で確定した方針**
- **セルフホスト Runner は対象外と判断する**: GitHub-hosted runner で十分な速度を確保できる
- **GitHub Free プランの範囲内でパイプラインを設計する**: 将来の有料化を前提としない
- **採用しない理由を ADR-IMPL-001 に記録する**: 将来の再検討の根拠を残す

---

## 参照業界分析

### 必須
- [`90_業界分析/07_スマートファクトリーと作業のデジタル化.md`](../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/29_競合製品と作業ナビ・MES・eBR市場.md`](../../90_業界分析/29_競合製品と作業ナビ・MES・eBR市場.md)
- [`90_業界分析/30_国内製造業IT調達慣行とSI構造.md`](../../90_業界分析/30_国内製造業IT調達慣行とSI構造.md)
