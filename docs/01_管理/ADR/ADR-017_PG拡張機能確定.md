# ADR-017: PG 拡張機能確定

日付: 2026-05-19
状態: 確定

## 背景

PostgreSQL の拡張機能として、複数のドキュメントで異なるリストが記述されていた。

`docs/05_詳細設計/01_データベース詳細設計/` のマイグレーション SQL 群では以下の拡張機能が CREATE EXTENSION されていた。
- `pgcrypto`: SHA-256 ハッシュ生成（ハッシュチェーン用）
- `uuid-ossp`: UUID v4 生成（主キー用）
- `pg_stat_statements`: クエリ統計収集（パフォーマンス監視用）

一方、一部のドキュメントでは `pg_trgm`（全文検索）・`btree_gist`（GiST インデックス）の使用が言及されており、インストールの要否が不明確だった。
また `postgresql.conf` の `shared_preload_libraries` に `pg_stat_statements` を含める必要があるが、その設定が明示されていなかった。

## 決定

ver1.0.0 で使用する PG 拡張機能を以下の 3 つに確定する。

| 拡張機能 | 用途 | 必要な `shared_preload_libraries` 設定 |
|---|---|---|
| `pgcrypto` | SHA-256 ハッシュ生成（`digest()` 関数）・ハッシュチェーン完全性保証 | 不要（動的ロード可） |
| `uuid-ossp` | `uuid_generate_v4()` による UUID 主キー生成 | 不要（動的ロード可） |
| `pg_stat_statements` | スロークエリ統計・クエリパフォーマンス分析 | 必要（`postgresql.conf` に追記） |

`pg_trgm` および `btree_gist` は ver1.0.0 では採用しない（将来の全文検索・地理空間検索要件が確定した時点で ADR を追加する）。

## 理由

- `pgcrypto` は SHA-256 ハッシュチェーン（`src/CLAUDE.md` §4「SHA-256 ハッシュチェーン」原則）の実現に必須であり、`digest(data, 'sha256')` 関数を提供する
- `uuid-ossp` は全テーブルの主キー型として UUID v4 を採用する設計決定（詳細設計 TBL 定義）に対応するために必須である
- `pg_stat_statements` はスロークエリ監視（`log_min_duration_statement = 1000ms`）に加え、クエリキャッシュ効率の改善・インデックス最適化判断の根拠データを提供するため採用する
- PostgreSQL 16-alpine には `pgcrypto` / `uuid-ossp` / `pg_stat_statements` が全て同梱されており、追加インストール不要で使用できる
- `pg_trgm` / `btree_gist` は現時点の機能要件に対応する使用箇所がなく、不要な拡張機能のインストールはセキュリティ攻撃面積の拡大を招くため採用しない

## 影響

- `src/infra/database/config/postgresql.conf` に `shared_preload_libraries = 'pg_stat_statements'` を設定する
- `src/infra/database/migrations/` の最初のマイグレーション（`V20260517120000__create_extensions.sql`）で `CREATE EXTENSION IF NOT EXISTS pgcrypto` / `uuid-ossp` / `pg_stat_statements` を実行する
- `src/infra/database/docker-entrypoint-initdb.d/` での拡張機能作成は行わない（マイグレーションに委任する）
- `pg_stat_statements.max = 10000` / `pg_stat_statements.track = all` を `postgresql.conf` に設定し、全クエリを追跡対象とする
