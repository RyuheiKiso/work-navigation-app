# ADR-012: PostgreSQL 16-alpine 採用

日付: 2026-05-19
状態: 確定

## 背景

docs 内で PostgreSQL 16（詳細設計・配置設計）と PostgreSQL 17（コーディング規約 §1・導入手順 §2-1）の記述が混在していた。
具体的には以下のドキュメントで不整合が発生していた。

- `docs/05_詳細設計/01_データベース詳細設計/00_本書の位置づけと識別子規約.md` §3-1: 「PostgreSQL 16 を対象」と明示
- `docs/04_概要設計/01_システム方式設計/03_配置設計（Active_Standby・単一建屋内冗長）.md`: `postgres:16-alpine` を指定
- `docs/06_実装/04_コーディング規約_SQL.md` §1: 「PostgreSQL 17」と記述
- `docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md` §2-1: `postgres:17-alpine` を使用

この不整合は、コンテナイメージ選定・CI サービス設定・Dockerfile のベースイメージ指定に矛盾をもたらしていた。

## 決定

PostgreSQL 16（`postgres:16-alpine`）を採用する。

## 理由

- `docs/05_詳細設計/01_データベース詳細設計/00_本書の位置づけと識別子規約.md` §3-1 が明示的に「PostgreSQL 16 を対象」と宣言しており、詳細設計が権威ソースとなる
- `docs/04_概要設計/01_システム方式設計/03_配置設計（Active_Standby・単一建屋内冗長）.md` も `postgres:16-alpine` を Docker Compose サービス定義で指定している
- 上流設計（概要設計・詳細設計）が PostgreSQL 16 で統一されており、コーディング規約 §1 の「PostgreSQL 17」は上流設計との整合性に欠ける記述である
- PostgreSQL 16 は本執筆時点で安定した LTS 相当バージョンであり、alpine イメージによりコンテナサイズを最小化できる
- PostgreSQL 16 で提供される機能（RANGE パーティション・ストリーミングレプリケーション・pg_stat_statements）は本システムの全要件を充足する

## 影響

- `src/infra/database/Dockerfile` のベースイメージ: `postgres:16-alpine`
- CI の postgres サービス: `postgres:16`
- Docker Compose の postgres イメージ: `postgres:16-alpine`
- `docs/06_実装/04_コーディング規約_SQL.md` §1 は次回改訂時に PG 16 に修正する
- `docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md` §2-1 の `postgres:17-alpine` 記述は次回改訂時に PG 16 に修正する
