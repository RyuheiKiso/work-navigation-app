# wnav バックエンド Dockerfile
#
# マルチステージビルドで 2 バイナリ（wnav_terminal_api / wnav_master_api）を生成する。
# ランタイムイメージに Rust ツールチェーン・ソースコードを含めない（最小化）。
#
# ビルドコンテキストは src/backend/ とする:
#   docker build -t wnav-backend ./src/backend
#
# バイナリの切り替えは docker-compose.yml の command: で行う:
#   command: /usr/local/bin/wnav_terminal_api   # ポート 8080
#   command: /usr/local/bin/wnav_master_api      # ポート 8081

# ─────────────────────────────────────────────────────────────────────────────
# Stage 1: builder — cargo build --release でバイナリを生成する
# ─────────────────────────────────────────────────────────────────────────────
FROM rust:1.85-slim AS builder
WORKDIR /app

# ビルドに必要なシステムパッケージをインストールする（native-tls のリンカ依存）
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 依存クレートのキャッシュ最適化のために先に Cargo.toml / Cargo.lock をコピーする。
# ソースコード（crates/）を変更してもここまでのレイヤはキャッシュが有効になる。
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# SQLX_OFFLINE=true 用のクエリキャッシュをコピーする。
# ディレクトリが空でも COPY は成功するため、.sqlx prepare 未実行でもビルドできる。
# .sqlx/ が存在する場合は必ずコピーしてオフラインモードでの型検証を有効にする。
COPY .sqlx/ .sqlx/

# cargo registry / target をマウントキャッシュに置くことで再ビルド時間を短縮する。
# バイナリはキャッシュマウント外にコピーしてから次ステージに渡す。
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    SQLX_OFFLINE=true cargo build --release --bin wnav_terminal_api && \
    SQLX_OFFLINE=true cargo build --release --bin wnav_master_api && \
    # マウントキャッシュは後続ステージから参照できないため、ここでコピーする
    cp target/release/wnav_terminal_api /wnav_terminal_api && \
    cp target/release/wnav_master_api /wnav_master_api

# ─────────────────────────────────────────────────────────────────────────────
# Stage 2: runtime — 必要最小限のランタイム環境を構築する
# ─────────────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# TLS ルート証明書（reqwest / native-tls が必要）のみインストールする
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 非 root ユーザーで実行することでコンテナ侵害時の影響範囲を最小化する
RUN useradd --no-create-home --uid 10001 wnav
USER wnav

# builder ステージから静的リンク済みバイナリだけをコピーする
# （Rust ツールチェーン・ソースコード・中間ファイルはランタイムイメージに含まない）
COPY --from=builder /wnav_terminal_api /usr/local/bin/wnav_terminal_api
COPY --from=builder /wnav_master_api /usr/local/bin/wnav_master_api

# 両バイナリが使用するポートを宣言する（docker-compose.yml の command: で切り替える）
# 8080: wnav_terminal_api（工場 LAN / ハンディ端末向け）
# 8081: wnav_master_api（管理 LAN / マスタメンテ向け）
EXPOSE 8080 8081

# ヘルスチェック: terminal_api の /healthz エンドポイントを確認する。
# WNAV_PROFILE が未設定の場合、バイナリは exit 78 で即座に終了する（fail-fast 設計）。
# --start-period=10s はコネクションプール初期化を待つための猶予時間。
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/healthz || exit 1

# 起動するバイナリは docker-compose.yml の command: で指定する。
# 例:
#   terminal_api コンテナ: command: /usr/local/bin/wnav_terminal_api
#   master_api コンテナ:   command: /usr/local/bin/wnav_master_api
#
# WNAV_PROFILE と機密値（DB パスワード・JWT 鍵）は
# docker-compose.yml の environment: または Docker secrets で注入する。
# イメージに機密値を埋め込んではならない。
