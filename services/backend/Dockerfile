# 対応 §: ロードマップ §11.7.2 §14.1 §14.2 §19.3
# multi-stage build で SCI（§11.7.3）を抑え、最終イメージは distroless 相当にする。

# ---- builder stage ---------------------------------------------------------
# 安定版の Rust（slim）を使ってビルドする
FROM rust:1.78-slim AS builder

# 必要な開発ライブラリをインストールする（OpenSSL 等）
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 作業ディレクトリ
WORKDIR /work

# 先に Cargo manifest をコピーすることで依存解決をキャッシュする
COPY Cargo.toml ./
COPY services/backend/crates/domain/Cargo.toml services/backend/crates/domain/Cargo.toml
COPY services/backend/crates/usecase/Cargo.toml services/backend/crates/usecase/Cargo.toml
COPY services/backend/crates/adapter/Cargo.toml services/backend/crates/adapter/Cargo.toml
COPY services/backend/crates/presentation/Cargo.toml services/backend/crates/presentation/Cargo.toml
COPY services/backend/crates/infrastructure/Cargo.toml services/backend/crates/infrastructure/Cargo.toml
COPY addon-sdk/rust/Cargo.toml addon-sdk/rust/Cargo.toml
COPY addon-sdk/examples/hello-step/Cargo.toml addon-sdk/examples/hello-step/Cargo.toml
COPY addon-sdk/examples/slack-notify/Cargo.toml addon-sdk/examples/slack-notify/Cargo.toml
COPY addon-sdk/examples/opc-ua-bridge/Cargo.toml addon-sdk/examples/opc-ua-bridge/Cargo.toml
COPY examples/cli-client/Cargo.toml examples/cli-client/Cargo.toml

# ソースをコピーする
COPY services services
COPY addon-sdk addon-sdk
COPY examples examples

# リリースビルド
RUN cargo build --release -p wna-infrastructure

# ---- runtime stage ---------------------------------------------------------
# 軽量実行イメージ
FROM debian:bookworm-slim AS runtime

# 必要最小限のランタイム依存
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates wget && \
    rm -rf /var/lib/apt/lists/*

# 非 root ユーザー
RUN useradd -m -u 10001 wna

# 作業ディレクトリ
WORKDIR /app

# 成果物をコピー
COPY --from=builder /work/target/release/wna-backend /usr/local/bin/wna-backend
COPY services/backend/migrations /app/migrations

# ユーザー切替
USER wna

# 公開ポート
EXPOSE 8080

# エントリポイント
ENTRYPOINT ["/usr/local/bin/wna-backend"]
