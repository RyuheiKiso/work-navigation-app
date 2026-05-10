# services/backend

> 対応 §: ロードマップ §7.3 §8 §9.1 §10.3 §10.6 §11.4 §13.2

Rust + tokio + axum で実装するバックエンドサーバ。クリーンアーキテクチャ（§9.1）に従い、5 crate に物理分割している。

## crate 構成

| crate | 層 | 責務 | 依存方向 |
| --- | --- | --- | --- |
| `domain` | 最内層（純粋） | Aggregate／Entity／Value Object／Repository trait（§3.1.1） | 標準ライブラリのみ |
| `usecase` | ユースケース層 | アプリケーションサービス（Start Task／Append Record） | `domain` のみ |
| `adapter` | ゲートウェイ層 | PostgreSQL 実装、DTO 変換 | `domain`／`usecase` |
| `presentation` | プレゼンテーション層 | axum REST API ルータ／ハンドラ | `usecase` のみ（`adapter` には直接依存しない／DI 経由） |
| `infrastructure` | インフラ層 | `tokio::main`／DI コンテナ／環境変数読込 | 上記すべて |

依存方向は内→外のみ。`scripts/lint-rust-deps.sh`（将来）で逆向き依存検出を CI に組み込む。

## 起動

```bash
# docker compose（推奨）
docker compose up -d

# ローカル直接実行
DATABASE_URL=postgres://wna:wna_dev_password@localhost:5432/wna \
RUST_LOG=info \
cargo run -p wna-infrastructure
```

## マイグレーション

`migrations/` 配下に sqlx の SQL マイグレーションを配置する。実行は `infrastructure` 起動時に自動。
