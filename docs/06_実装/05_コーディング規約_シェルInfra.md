# 05 コーディング規約_シェルInfra

## 1. Bash 規約

### シェバン行と安全オプション

すべての Bash スクリプトは以下の 2 行で始める。

```bash
#!/usr/bin/env bash
set -euo pipefail
```

| オプション | 効果 |
|---|---|
| `-e` | コマンドがゼロ以外の終了コードを返した場合にスクリプトを終了する |
| `-u` | 未定義変数を参照した場合にエラーを返す |
| `-o pipefail` | パイプラインのいずれかのコマンドが失敗した場合にパイプライン全体を失敗とする |

### shellcheck CI 統合

すべての Bash スクリプトを `shellcheck` で静的解析する。

```bash
# CI での shellcheck 実行
find . -name "*.sh" -exec shellcheck --severity=warning {} +
```

### 変数の書き方

```bash
#!/usr/bin/env bash
set -euo pipefail

# 変数は二重引用符で囲む（単語分割・グロブ展開を防止する）
readonly DB_HOST="${WNAV_DB_HOST:-localhost}"
readonly DB_PORT="${WNAV_DB_PORT:-5432}"

# 配列のループ
readonly MIGRATION_FILES=("001_create_tables.sql" "002_create_indexes.sql")
for file in "${MIGRATION_FILES[@]}"; do
    echo "Applying migration: ${file}"
done

# コマンド置換は $() を使用する（バッククォート禁止）
readonly CURRENT_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# 比較は [[ ]] を使用する（[ ] は POSIX sh 用・Bash では [[ ]] が安全）
if [[ "${WNAV_ENV}" == "production" ]]; then
    echo "本番環境での実行を検出した"
fi
```

### 関数命名（`verb_noun` snake_case）

```bash
#!/usr/bin/env bash
set -euo pipefail

# 関数名は verb_noun の snake_case 形式で命名する
check_prerequisites() {
    # Docker が起動していることを確認する
    if ! docker info &>/dev/null; then
        echo "ERROR: Docker が起動していない" >&2
        return 1
    fi
    echo "前提条件チェック: OK"
}

run_database_migration() {
    local -r db_url="${1}"
    sqlx migrate run --database-url "${db_url}"
}

verify_database_schema() {
    local -r db_url="${1}"
    # スキーマ検証を実行する
    psql "${db_url}" -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';"
}

# メイン処理
main() {
    check_prerequisites
    run_database_migration "${WNAV_DB_URL}"
    verify_database_schema "${WNAV_DB_URL}"
}

main "$@"
```

**本節で確定した方針**
- **全 Bash スクリプトを `#!/usr/bin/env bash` と `set -euo pipefail` で開始する。**
- **`shellcheck` を CI に統合し、警告レベル以上をエラーとして扱う。**
- **変数は `"${var}"` で二重引用符を付け、比較は `[[ ]]` を使用する。**

---

## 2. PowerShell 規約

### ヘッダーと厳格モード

```powershell
#Requires -Version 7
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

<#
.SYNOPSIS
    IIS アプリケーションプールの冪等セットアップスクリプト。

.DESCRIPTION
    作業ナビゲーションシステムのバックエンド API 用 IIS 設定を適用する。
    何度実行しても同じ結果になる冪等設計となっている。

.PARAMETER SiteName
    IIS サイト名（デフォルト: WNavAPI）

.EXAMPLE
    .\setup_iis.ps1 -SiteName "WNavAPI"
#>
```

### Cmdlet 動詞-名詞形式

```powershell
#Requires -Version 7
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Cmdlet は PowerShell 承認済み動詞-名詞形式で命名する
function Set-IisApplicationPool {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$PoolName,
        [string]$ManagedRuntimeVersion = ''
    )

    # アプリケーションプールが存在しない場合のみ作成する（冪等設計）
    if (-not (Test-Path "IIS:\AppPools\${PoolName}")) {
        Write-Verbose "アプリケーションプール '${PoolName}' を作成する"
        New-WebAppPool -Name $PoolName
    }

    # マネージドランタイムを設定する（No Managed Code = Rust バイナリ用）
    Set-ItemProperty -Path "IIS:\AppPools\${PoolName}" -Name managedRuntimeVersion -Value $ManagedRuntimeVersion
    Write-Verbose "アプリケーションプール '${PoolName}' を設定した"
}

function Test-IisConfiguration {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$SiteName
    )

    # サイトの存在確認
    return Test-Path "IIS:\Sites\${SiteName}"
}
```

### スクリプト署名（Windows Server 2022）

```powershell
# Windows Server 2022 では RemoteSigned を設定する
# Set-ExecutionPolicy RemoteSigned -Scope LocalMachine

# スクリプト実行前に署名の有効性を確認する
function Test-ScriptSignature {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ScriptPath
    )

    $signature = Get-AuthenticodeSignature -FilePath $ScriptPath
    if ($signature.Status -ne 'Valid') {
        throw "スクリプト署名が無効: ${ScriptPath} (Status: $($signature.Status))"
    }
    Write-Verbose "スクリプト署名: 有効 (${ScriptPath})"
}
```

**本節で確定した方針**
- **全 PowerShell スクリプトを `#Requires -Version 7`・`Set-StrictMode -Version Latest`・`$ErrorActionPreference = 'Stop'` で開始する。**
- **Cmdlet は PowerShell 承認済み動詞-名詞形式（`Set-IisApplicationPool`）で命名する。**
- **スクリプトは冪等設計（`If Not Exists` パターン）とし、複数回実行しても同じ結果を保証する。**

---

## 3. Dockerfile 規約

### マルチステージビルド

```dockerfile
# Stage 1: ビルドステージ（Rust コンパイル環境）
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# 依存関係のキャッシュを最大化するため、Cargo.toml/Cargo.lock を先にコピーする
COPY Cargo.toml Cargo.lock ./
COPY src/backend/Cargo.toml ./src/backend/

# ダミーの main.rs でキャッシュ用にビルドする（2バイナリ分）
RUN mkdir -p src/backend/src && \
    echo 'fn main() {}' > src/backend/src/main.rs && \
    cargo build --release --bin wnav_terminal_api --bin wnav_master_api && \
    rm -f src/backend/src/main.rs

# ソースをコピーしてビルドする
COPY src/backend/src ./src/backend/src
RUN touch src/backend/src/main.rs && \
    SQLX_OFFLINE=true cargo build --release --bin wnav_terminal_api --bin wnav_master_api

# Stage 2: terminal-api 最終イメージ（最小サイズ）
FROM gcr.io/distroless/cc-debian12:nonroot AS terminal_api_runtime

# 非 root ユーザーで実行する（UID 65532 = nonroot）
USER nonroot:nonroot

WORKDIR /app

# terminal-api バイナリのみをコピーする
COPY --from=builder --chown=nonroot:nonroot /app/target/release/wnav_terminal_api ./wnav_terminal_api

# ヘルスチェックを定義する
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/wnav_terminal_api", "--health-check"]

EXPOSE 8080

ENTRYPOINT ["/app/wnav_terminal_api"]

# Stage 3: master-api 最終イメージ（最小サイズ）
FROM gcr.io/distroless/cc-debian12:nonroot AS master_api_runtime

# 非 root ユーザーで実行する（UID 65532 = nonroot）
USER nonroot:nonroot

WORKDIR /app

# master-api バイナリのみをコピーする
COPY --from=builder --chown=nonroot:nonroot /app/target/release/wnav_master_api ./wnav_master_api

# ヘルスチェックを定義する
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/wnav_master_api", "--health-check"]

EXPOSE 8081

ENTRYPOINT ["/app/wnav_master_api"]
```

### ARG でシークレットを受け取らない

```dockerfile
# 禁止: ARG で機密情報を受け取る（docker history で閲覧可能になる）
# ARG DATABASE_PASSWORD
# ENV DATABASE_PASSWORD=${DATABASE_PASSWORD}

# 推奨: シークレットは実行時の環境変数または Docker Secrets で渡す
# docker run -e DATABASE_PASSWORD="..." または
# docker run --secret id=db_password wnav-api:latest
```

**本節で確定した方針**
- **Dockerfile はマルチステージビルドを必須とし、最終イメージは `distroless` または `alpine` の最小版を使用する。**
- **最終イメージは `USER nonroot` で非 root 実行を必須とし、root 実行を禁止する。**
- **`ARG` で機密情報を受け取ることを禁止し、実行時の環境変数または Docker Secrets で渡す。**

---

## 4. docker-compose 規約

```yaml
# docker-compose.yml
services:
  # サービス名は snake_case
  postgres:
    image: postgres:17-bookworm
    environment:
      POSTGRES_DB: wnav_dev
      POSTGRES_USER: postgres
      # パスワードは Secrets スタンザから取得する（ハードコード禁止）
      POSTGRES_PASSWORD_FILE: /run/secrets/postgres_password
    volumes:
      - wnav_postgres_data:/var/lib/postgresql/data
      - ./infra/database/init:/docker-entrypoint-initdb.d
    ports:
      - "127.0.0.1:5432:5432"  # ループバックのみにバインドする
    networks:
      - backend_network
    secrets:
      - postgres_password
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres -d wnav_dev"]
      interval: 10s
      timeout: 5s
      retries: 5

  wnav_terminal_api:
    build:
      context: .
      dockerfile: src/backend/Dockerfile.terminal
    environment:
      WNAV_TERMINAL_DATABASE_URL_FILE: /run/secrets/terminal_db_url
      WNAV_BE_JWT_PUBLIC_KEY_FILE: /run/secrets/jwt_public_key
      RUST_LOG: "wnav_terminal_api=info,tower_http=debug"
    ports:
      - "127.0.0.1:8080:8080"
    networks:
      - backend_network
      - frontend_network
    depends_on:
      postgres:
        condition: service_healthy
    secrets:
      - terminal_db_url
      - jwt_public_key

  wnav_master_api:
    build:
      context: .
      dockerfile: src/backend/Dockerfile.master
    environment:
      WNAV_MASTER_DATABASE_URL_FILE: /run/secrets/master_db_url
      WNAV_BE_JWT_PUBLIC_KEY_FILE: /run/secrets/jwt_public_key
      RUST_LOG: "wnav_master_api=info,tower_http=debug"
    ports:
      - "127.0.0.1:8081:8081"
    networks:
      - backend_network
      - frontend_network
    depends_on:
      postgres:
        condition: service_healthy
    secrets:
      - master_db_url
      - jwt_public_key

networks:
  # ネットワークを明示的に分離する
  backend_network:
    driver: bridge
  frontend_network:
    driver: bridge

volumes:
  # ボリューム名は project_name_purpose 形式
  wnav_postgres_data:
    driver: local

secrets:
  postgres_password:
    file: ./secrets/postgres_password.txt
  terminal_db_url:
    file: ./secrets/terminal_db_url.txt
  master_db_url:
    file: ./secrets/master_db_url.txt
  jwt_public_key:
    file: ./secrets/jwt_public_key.pem
```

**本節で確定した方針**
- **サービス名を `snake_case` で命名し、ネットワークを `backend_network`/`frontend_network` に明示的に分離する。**
- **シークレットは `secrets:` スタンザで管理し、`environment:` への直接記載を禁止する。**
- **ポートは `127.0.0.1:<port>` 形式でループバックにバインドし、全インターフェースへの公開を禁止する。**
- **`wnav_terminal_api`（8080）と `wnav_master_api`（8081）をそれぞれ独立したサービスとして定義し、2バイナリ構成を維持する。**

---

## 5. IIS 設定スクリプト

```powershell
#Requires -Version 7
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

<#
.SYNOPSIS
    WNav マスタメンテ SPA の IIS 設定を冪等適用する。
#>

[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$SiteName = 'WNavMaster',
    [string]$AppPoolName = 'WNavMasterPool',
    [string]$PhysicalPath = 'C:\inetpub\wnav\master',
    [int]$Port = 443,
    [switch]$DryRun
)

function Set-WNavAppPool {
    [CmdletBinding()]
    param([string]$PoolName)

    Import-Module WebAdministration -ErrorAction Stop

    if (-not (Test-Path "IIS:\AppPools\${PoolName}")) {
        Write-Verbose "アプリケーションプール '${PoolName}' を作成する"
        New-WebAppPool -Name $PoolName | Out-Null
    }

    # No Managed Code: React SPA は静的ファイル配信のため CLR 不要
    Set-ItemProperty -Path "IIS:\AppPools\${PoolName}" `
        -Name managedRuntimeVersion `
        -Value ''

    # アイドルタイムアウトを無効にする（起動コストなし）
    Set-ItemProperty -Path "IIS:\AppPools\${PoolName}" `
        -Name processModel.idleTimeout `
        -Value '00:00:00'

    Write-Verbose "アプリケーションプール '${PoolName}': 設定完了"
}

function Set-WNavSite {
    [CmdletBinding()]
    param(
        [string]$SiteName,
        [string]$AppPoolName,
        [string]$PhysicalPath,
        [int]$Port
    )

    # 物理パスが存在することを確認する
    if (-not (Test-Path $PhysicalPath)) {
        New-Item -ItemType Directory -Path $PhysicalPath -Force | Out-Null
    }

    if (-not (Test-Path "IIS:\Sites\${SiteName}")) {
        Write-Verbose "IIS サイト '${SiteName}' を作成する"
        New-WebSite -Name $SiteName `
            -PhysicalPath $PhysicalPath `
            -Port $Port `
            -ApplicationPool $AppPoolName | Out-Null
    } else {
        # 既存サイトの設定を更新する（冪等）
        Set-ItemProperty -Path "IIS:\Sites\${SiteName}" `
            -Name physicalPath `
            -Value $PhysicalPath
    }

    Write-Verbose "IIS サイト '${SiteName}': 設定完了"
}

# メイン処理
if ($DryRun) {
    Write-Output "DryRun モード: 実際の変更は行わない"
    return
}

Set-WNavAppPool -PoolName $AppPoolName
Set-WNavSite -SiteName $SiteName -AppPoolName $AppPoolName `
             -PhysicalPath $PhysicalPath -Port $Port
Write-Output "IIS 設定の適用が完了した"
```

**本節で確定した方針**
- **IIS 設定スクリプトは冪等設計（存在確認 → 作成 or 更新）とし、複数回の実行を安全にする。**
- **`New-WebSite`/`New-WebAppPool` には `-DryRun` スイッチを実装し、CI での検証を可能にする。**
- **アプリケーションプールのマネージドランタイムを空文字（No Managed Code）に設定し、Rust バイナリ/React SPA に適用する。**

---

## 6. nginx 設定

共通のプロキシパラメータは `/etc/nginx/conf.d/wnav_proxy_params.conf` に切り出し、各 `location` ブロックから `include` する。これにより設定の重複を排除する。

```nginx
# /etc/nginx/conf.d/wnav_proxy_params.conf
# 全 location ブロックで共通のプロキシ設定（include で再利用する）
proxy_http_version 1.1;
proxy_set_header Host              $host;
proxy_set_header X-Real-IP         $remote_addr;
proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;
proxy_set_header Connection        "";  # keepalive のため Connection ヘッダをクリア
proxy_connect_timeout 10s;
proxy_send_timeout    30s;
proxy_read_timeout    30s;
```

```nginx
# /etc/nginx/conf.d/wnav.conf
# WNav バックエンド API のリバースプロキシ設定（WSL2 上の nginx）
# 2バイナリ構成: wnav_terminal_api（8080）/ wnav_master_api（8081）

upstream terminal_upstream {
    # wnav_terminal_api: ハンディ端末向け作業イベント記録 API（8080）
    server 127.0.0.1:8080;
    keepalive 32;
}

upstream master_upstream {
    # wnav_master_api: マスタメンテナンス API（8081）
    server 127.0.0.1:8081;
    keepalive 32;
}

# HTTP → HTTPS リダイレクト
server {
    listen 80;
    server_name wnav.example.local;
    return 301 https://$host$request_uri;
}

# HTTPS サーバー
server {
    listen 443 ssl http2;
    server_name wnav.example.local;

    # TLS 設定（Let's Encrypt または自己署名証明書）
    ssl_certificate     /etc/ssl/certs/wnav.pem;
    ssl_certificate_key /etc/ssl/private/wnav.key;
    ssl_protocols       TLSv1.3;
    ssl_ciphers         ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-CHACHA20-POLY1305;
    ssl_prefer_server_ciphers on;
    ssl_session_cache   shared:SSL:10m;
    ssl_session_timeout 1d;

    # HSTS: 1 年間 HTTPS を強制する
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # セキュリティヘッダー
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'" always;

    # gzip 圧縮（API レスポンスと静的ファイル）
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types application/json text/plain text/css application/javascript;

    # terminal-api: 作業イベント記録（ハンディ端末向け）
    # /api/v1/events/ と /api/v1/terminal/ を terminal_upstream（8080）にルーティングする
    location /api/v1/events/ {
        proxy_pass http://terminal_upstream;
        include /etc/nginx/conf.d/wnav_proxy_params.conf;
    }

    location /api/v1/terminal/ {
        proxy_pass http://terminal_upstream;
        include /etc/nginx/conf.d/wnav_proxy_params.conf;
    }

    # master-api: SOP・マスタデータ管理（マスタメンテ向け）
    # /api/v1/sop/ と /api/v1/master/ を master_upstream（8081）にルーティングする
    location /api/v1/sop/ {
        proxy_pass http://master_upstream;
        include /etc/nginx/conf.d/wnav_proxy_params.conf;
    }

    location /api/v1/master/ {
        proxy_pass http://master_upstream;
        include /etc/nginx/conf.d/wnav_proxy_params.conf;
    }

    # ヘルスチェックエンドポイント（terminal-api / master-api）
    location /healthz/terminal {
        proxy_pass http://terminal_upstream/healthz;
        access_log off;
    }

    location /healthz/master {
        proxy_pass http://master_upstream/healthz;
        access_log off;
    }
}
```

**本節で確定した方針**
- **TLS は TLSv1.3 のみを許容し、TLSv1.2 以下を対象外と判断する。**
- **HSTS（`Strict-Transport-Security: max-age=31536000`）を全レスポンスに付与する。**
- **`X-Frame-Options: DENY` を設定し、クリックジャッキングに対応する。**
- **nginx の upstream を `terminal_upstream`（8080）と `master_upstream`（8081）に分割し、パスプレフィックスでルーティングする。**
- **`/api/v1/events/` および `/api/v1/terminal/` は `terminal_upstream` へ、`/api/v1/sop/` および `/api/v1/master/` は `master_upstream` へプロキシする。**
- **共通のプロキシヘッダ・タイムアウト設定は `wnav_proxy_params.conf` に切り出して `include` し、各 `location` ブロックでの重複を排除する。**

---

## 7. 環境変数命名

### 命名規則

`WNAV_<SCOPE>_<KEY>` 形式で統一する。

| SCOPE | 対象 |
|---|---|
| `BE` | バックエンド（Rust axum API）|
| `FE_HA` | フロントエンド ハンディ APP（React Native）|
| `FE_MA` | フロントエンド マスタメンテ（React SPA）|
| `INFRA` | インフラ（Docker/nginx/IIS）|
| `DB` | データベース（PostgreSQL）|

### 環境変数一覧（具体例）

```bash
# バックエンド: terminal-api（wnav_terminal_api / 8080）
# app_event_insert ロール（INSERT 専用）と app_read ロールを使用する
WNAV_TERMINAL_DATABASE_URL="postgres://app_event_insert:...@localhost:5432/wnav"
WNAV_TERMINAL_DATABASE_URL_READ="postgres://app_read:...@localhost:5432/wnav"
WNAV_BE_JWT_PUBLIC_KEY_PATH="/etc/wnav/jwt/public.pem"
WNAV_TERMINAL_LISTEN_ADDR="0.0.0.0:8080"
WNAV_TERMINAL_RUST_LOG="wnav_terminal_api=info,tower_http=debug"
WNAV_TERMINAL_IDEMPOTENCY_CACHE_TTL_SECS="86400"

# バックエンド: master-api（wnav_master_api / 8081）
# app_write ロール（SELECT/INSERT/UPDATE）と app_read ロールを使用する
WNAV_MASTER_DATABASE_URL="postgres://app_write:...@localhost:5432/wnav"
WNAV_MASTER_DATABASE_URL_READ="postgres://app_read:...@localhost:5432/wnav"
WNAV_MASTER_LISTEN_ADDR="0.0.0.0:8081"
WNAV_MASTER_RUST_LOG="wnav_master_api=info,tower_http=debug"
WNAV_MASTER_IDEMPOTENCY_CACHE_TTL_SECS="86400"

# ハンディ APP
WNAV_FE_HA_API_BASE_URL="https://wnav.example.local/api/v1"
WNAV_FE_HA_OFFLINE_TIMEOUT_SECS="300"
WNAV_FE_HA_SYNC_INTERVAL_SECS="30"
WNAV_FE_HA_EXPO_PROJECT_ID="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

# マスタメンテ
WNAV_FE_MA_API_BASE_URL="https://wnav.example.local/api/v1"
WNAV_FE_MA_POLLING_INTERVAL_MS="30000"

# インフラ
WNAV_INFRA_NGINX_CERT_PATH="/etc/ssl/certs/wnav.pem"
WNAV_INFRA_NGINX_KEY_PATH="/etc/ssl/private/wnav.key"
WNAV_INFRA_IIS_SITE_NAME="WNavMaster"
WNAV_INFRA_IIS_APP_POOL_NAME="WNavMasterPool"

# データベース
WNAV_DB_HOST="localhost"
WNAV_DB_PORT="5432"
WNAV_DB_NAME="wnav"
WNAV_DB_WRITE_USER="app_write"
WNAV_DB_EVENT_USER="app_event_insert"
WNAV_DB_READ_USER="app_read"
```

**本節で確定した方針**
- **環境変数は `WNAV_<SCOPE>_<KEY>` 形式で命名し、プレフィックスなしの変数名を禁止する。**
- **DB 接続情報を 1 つの `DATABASE_URL` にまとめず、3 ロール分を別々の変数として定義する。**
- **シークレットを含む変数は `_FILE` サフィックスを付け（例: `WNAV_BE_DATABASE_URL_WRITE_FILE`）、ファイルパスで渡す形式を推奨する。**

---

## 8. シークレット取り扱い

### ソースコード禁止

```bash
# 禁止: ソースコードへのシークレットの記載
DATABASE_PASSWORD="P@ssw0rd!Wnav2026"

# 禁止: .env ファイルをバージョン管理に含める
git add .env  # 禁止

# 推奨: .env.example のみバージョン管理し、実際の値は記載しない
# .env.example（バージョン管理対象）
WNAV_DB_WRITE_USER=app_write
WNAV_DB_WRITE_PASSWORD=  # 実際の値は記載しない
```

### `git log --all` からの除外

```bash
# .gitignore に必ず含める
.env
.env.local
.env.*.local
*.pem
*.key
*.p12
*.pfx
secrets/
.secrets/
```

誤ってシークレットを commit した場合は `git filter-branch` または `git filter-repo` で履歴から除去し、シークレットを即座にローテーションする。

### docker history からの隠蔽

```dockerfile
# 禁止: ARG/ENV でシークレットをビルド時に埋め込む（docker history で閲覧可能）
# ARG DB_PASSWORD
# ENV DB_PASSWORD=${DB_PASSWORD}

# 推奨: BuildKit の --secret オプションを使用する（ファイルシステムに残らない）
# docker build --secret id=db_password,src=./secrets/db_password.txt .
RUN --mount=type=secret,id=db_password \
    DB_PASSWORD=$(cat /run/secrets/db_password) \
    sqlx migrate run
```

### Windows DPAPI と HashiCorp Vault の使い分け

| 場面 | 選択 | 理由 |
|---|---|---|
| Windows Server 2022 ローカル環境 | Windows DPAPI | OS 組み込み・追加インフラ不要 |
| 複数サーバー・チーム共有 | HashiCorp Vault | 一元管理・監査ログ・動的シークレット |
| 開発者ローカル | `.env` ファイル（個人管理） | セットアップ簡便性（.gitignore 必須） |

**本節で確定した方針**
- **シークレットをソースコード・Dockerfile・`docker history` から完全に排除する。**
- **誤って commit されたシークレットは即座に履歴から除去し、シークレットをローテーションする。**
- **本番環境のシークレット管理は Windows DPAPI または HashiCorp Vault を使用し、`.env` ファイルを禁止する。**

---

## 9. ファイルパーミッション

```bash
#!/usr/bin/env bash
set -euo pipefail

# 秘密鍵ファイル: 所有者のみ読み書き可（秘密鍵はグループ・その他へのアクセスを禁止する）
chmod 600 /etc/wnav/jwt/private.pem
chmod 600 /etc/wnav/tls/wnav.key

# 設定ファイル: 所有者読み書き + グループ読み取り
chmod 640 /etc/wnav/config.toml
chmod 640 /etc/nginx/conf.d/wnav.conf

# 実行スクリプト: 所有者フルアクセス + グループ読み取り・実行
chmod 750 /opt/wnav/scripts/setup_iis.sh
chmod 750 /opt/wnav/scripts/run_migration.sh

# サービス起動スクリプトのパーミッション確認
check_permissions() {
    local -r file="${1}"
    local -r expected_perm="${2}"
    local -r actual_perm
    actual_perm="$(stat -c '%a' "${file}")"
    if [[ "${actual_perm}" != "${expected_perm}" ]]; then
        echo "ERROR: ${file} のパーミッションが不正: expected=${expected_perm}, actual=${actual_perm}" >&2
        return 1
    fi
}

# umask の設定（新規ファイルのデフォルトパーミッション: 640/750）
umask 027
```

### パーミッション一覧

| 対象 | パーミッション | 理由 |
|---|---|---|
| 秘密鍵（`.pem`, `.key`） | `600` | 所有者のみ読み書き可 |
| 設定ファイル（`.toml`, `.conf`, `.env`） | `640` | グループ読み取り可（サービスアカウント用） |
| 実行スクリプト（`.sh`, `.ps1`） | `750` | グループ実行可・その他禁止 |
| ログファイル | `640` | グループ読み取り可（監査ツール用） |
| データディレクトリ | `750` | グループ実行可（ディレクトリ参照用） |

**本節で確定した方針**
- **秘密鍵は `600`・設定ファイルは `640`・スクリプトは `750` のパーミッションを設定する。**
- **`umask 027` をサービス起動スクリプトに設定し、新規ファイルの過剰な権限付与を防止する。**
- **CI でファイルパーミッションの検証スクリプトを実行し、デプロイ後の権限変更を検出する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/13_安全文化と安全管理システム.md`](../../90_業界分析/13_安全文化と安全管理システム.md)

### 関連
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)
- [`90_業界分析/38_災害・BCP・緊急時手順と作業継続.md`](../../90_業界分析/38_災害・BCP・緊急時手順と作業継続.md)
