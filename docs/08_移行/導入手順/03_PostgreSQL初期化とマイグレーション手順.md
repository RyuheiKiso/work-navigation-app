# 03 PostgreSQL 初期化とマイグレーション手順

本章の責務は、INST-A4-c（DB 初期化・マイグレーション）に基づき、本番環境への初回導入時に PostgreSQL コンテナを起動し、sqlx マイグレーションを適用して、システムが利用可能な DB 状態を確立することである。開発環境向けの手順は `06_実装/10_デプロイ手順.md` が権威ソースであり、本章は本番初回導入に固有の手順のみを記載する。

---

## 1 本章の責務と `06_実装/10_デプロイ手順.md` との境界

### 1-1. 本章のスコープ

本章が扱う手順は「本番環境への初回導入」に限定する。以下の表でスコープを明確にする。

| 作業種別 | 権威ソース | 本章での扱い |
|---|---|---|
| 開発環境の DB 初期化 | `06_実装/06_開発環境構築手順.md` | 対象外と判断する |
| 本番以外の CI/CD での自動マイグレーション | `06_実装/10_デプロイ手順.md` §3 | 対象外と判断する |
| **本番環境への初回 PostgreSQL 起動** | **本章 §2** | **対象** |
| **本番環境への初回 sqlx migrate run** | **本章 §3** | **対象** |
| **初期データ（システムロール等）の確認** | **本章 §4** | **対象** |
| 通常デプロイ時のマイグレーション | `06_実装/10_デプロイ手順.md` §8 | 対象外と判断する |

**MIG-X-123**: 本章の手順は本番環境への初回導入時に 1 回のみ実施する。通常デプロイ時のマイグレーションは `06_実装/10_デプロイ手順.md` の手順に従うことを確定する。（DES-MIG-015 対応）

### 1-2. 前提となる環境

本章の手順を開始する前に、以下の前提が全て満たされていることを確認する。

| 前提項目 | 確認方法 | 参照手順 |
|---|---|---|
| WSL2 + Docker が起動済み | `wsl -d Ubuntu-24.04 -- docker ps` | 01 章 §4 |
| `docker-compose.prod.yml` が `/opt/wnav/` に配置済み | `ls /opt/wnav/docker-compose.prod.yml` | 01 章 §4 |
| `.env.prod` が `/opt/wnav/` に配置済み | `ls /opt/wnav/.env.prod` | 01 章 §4 |
| バックアップ用ディレクトリが作成済み | `ls /backup/` | 01 章 §6 |

---

**本節で確定した方針**
- 本章は本番初回導入専用の手順書であり、開発環境や通常デプロイのマイグレーション手順は `06_実装/10_デプロイ手順.md` に従うことを確定する。
- 本章の手順は本番環境への初回導入時に 1 回のみ実施することを確定する。
- 本章の手順開始前に 01 章の WSL2 + Docker 環境構築が完了していることを前提条件とすることを確定する。

---

## 2 PostgreSQL コンテナの起動

### 2-1. `docker-compose.prod.yml` での PostgreSQL の起動

**MIG-X-123**: PostgreSQL 17 コンテナを `docker-compose.prod.yml` で定義された設定に従って起動する。（DES-MIG-015 対応）

`docker-compose.prod.yml` の PostgreSQL サービス定義が以下の通りであることを確認する。

```yaml
# docker-compose.prod.yml（PostgreSQL 部分）
services:
  postgres:
    image: postgres:17-alpine
    restart: always
    environment:
      - POSTGRES_DB=${WNAV_DB_NAME}
      - POSTGRES_USER=${WNAV_DB_USER}
      - POSTGRES_PASSWORD=${WNAV_DB_PASSWORD}
      - PGDATA=/var/lib/postgresql/data/pgdata
    volumes:
      - prod_pgdata:/var/lib/postgresql/data
      - /backup:/backup
    ports:
      - "127.0.0.1:5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${WNAV_DB_USER} -d ${WNAV_DB_NAME}"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  prod_pgdata:
    name: wnav_prod_pgdata
```

PostgreSQL のみを先に起動する。

```bash
cd /opt/wnav

# PostgreSQL コンテナのみ起動する
docker compose -f docker-compose.prod.yml up -d postgres

# 起動状態を確認する（Status が healthy になるまで待機する）
docker compose -f docker-compose.prod.yml ps postgres
```

`Status` が `healthy` になるまで最大 60 秒待機する。60 秒を超えてもヘルスチェックが通過しない場合は `docker compose logs postgres` でエラーを確認する。

### 2-2. 初期ユーザー・パスワード・データベース名の設定

**MIG-X-124**: `.env.prod` に設定する本番環境の DB 接続情報を確定する。設定値は以下の規則に従う。（DES-MIG-015 対応）

`.env.prod` テンプレートを以下の内容で作成する（実際の値はここに記載しない）。

```bash
# .env.prod テンプレート
# 本番専用の強固なパスワードを設定すること（16 文字以上・英大文字・英小文字・数字・記号を含む）
WNAV_DB_NAME=wnav_prod
WNAV_DB_USER=wnav_admin
WNAV_DB_PASSWORD=<本番専用パスワード>

# バックアップ専用ユーザー（SELECT 権限のみ）
WNAV_DB_BACKUP_USER=wnav_backup
WNAV_DB_BACKUP_PASSWORD=<バックアップ専用パスワード>

# sqlx 用接続 URL
DATABASE_URL=postgresql://wnav_admin:${WNAV_DB_PASSWORD}@localhost:5432/wnav_prod
```

`.env.prod` は以下の権限設定で保護する。

```bash
chmod 600 /opt/wnav/.env.prod
chown wnav:wnav /opt/wnav/.env.prod
```

### 2-3. ローカルバインド確認（5432 ポート）

PostgreSQL は `127.0.0.1:5432` にのみバインドし、外部ネットワークからの直接アクセスを禁止する。バインド設定を確認する。

```bash
# ポート 5432 のバインド先を確認する（127.0.0.1 のみが表示されることを確認する）
docker compose -f docker-compose.prod.yml exec postgres \
    pg_isready -h localhost -p 5432 -U ${WNAV_DB_USER}

# Windows 側からの確認（外部からアクセスできないことを確認する）
# WSL2 内から実行する
ss -tlnp | grep 5432
```

外部ネットワーク（0.0.0.0:5432）へのバインドが存在する場合は `docker-compose.prod.yml` のポートバインディングを `127.0.0.1:5432:5432` に修正する。

---

**本節で確定した方針**
- PostgreSQL は `postgres:17-alpine` イメージを使用することを確定する。
- PostgreSQL のポートバインドは `127.0.0.1:5432` のみとし、外部アクセスを禁止することを確定する。
- `.env.prod` のファイル権限は `600`（所有者のみ読み書き可能）に設定することを確定する。

---

## 3 sqlx migrate の実行

### 3-1. マイグレーションファイルの確認

**MIG-X-125**: マイグレーション実行前に、適用対象のマイグレーションファイル一覧と現在の DB 状態を確認する。（DES-MIG-016 対応）

```bash
# リポジトリのマイグレーションファイル一覧を確認する
ls -la /opt/wnav/migrations/

# 期待する出力例（ファイル名はバージョン番号_説明.sql 形式）
# 20240101000000_initial_schema.up.sql
# 20240101000000_initial_schema.down.sql
# 20240201000000_add_work_events.up.sql
# 20240201000000_add_work_events.down.sql
# ...

# sqlx で適用済みマイグレーションの確認（初回実行前は空）
sqlx migrate info --database-url "postgresql://wnav_admin:${WNAV_DB_PASSWORD}@localhost:5432/wnav_prod"
```

`sqlx migrate info` の出力で、全マイグレーションが `Not Applied` 状態であることを確認してから次の手順を実施する。

### 3-2. `sqlx migrate run` の実行

**MIG-X-126**: マイグレーションを実行する。実行前にトランザクションが正常にコミットされることを確認する。（DES-MIG-016 対応）

```bash
# マイグレーションの dry-run を実施する（実際の変更は行わない）
sqlx migrate run --dry-run \
    --database-url "postgresql://wnav_admin:${WNAV_DB_PASSWORD}@localhost:5432/wnav_prod"

# dry-run でエラーがなければ本番のマイグレーションを実行する
sqlx migrate run \
    --database-url "postgresql://wnav_admin:${WNAV_DB_PASSWORD}@localhost:5432/wnav_prod"

# 実行ログの例（正常終了時）
# Applied 20240101000000_initial_schema.up.sql (1.234s)
# Applied 20240201000000_add_work_events.up.sql (0.567s)
# ...
```

マイグレーション実行時の出力を全て記録し、`C:\wnav-install\records\migration_run.log` に保存する。

### 3-3. マイグレーション適用記録の確認

マイグレーション完了後、`_sqlx_migrations` テーブル（sqlx が自動作成）でアプリケーション結果を確認する。

```bash
# _sqlx_migrations テーブルで適用済みマイグレーションを確認する
docker compose -f docker-compose.prod.yml exec postgres \
    psql -U wnav_admin -d wnav_prod -c "
      SELECT version, description, installed_on, success
      FROM _sqlx_migrations
      ORDER BY version;
    "
```

全マイグレーションの `success` カラムが `TRUE` であることを確認する。`FALSE` が存在する場合は `sqlx migrate revert` でロールバックしてから原因を調査する。

```bash
# マイグレーションのロールバック手順（エラー発生時のみ実行する）
sqlx migrate revert \
    --database-url "postgresql://wnav_admin:${WNAV_DB_PASSWORD}@localhost:5432/wnav_prod"

# ロールバック後に再実行する前に必ずエラー原因を特定する
```

---

**本節で確定した方針**
- マイグレーション実行前に `--dry-run` オプションで差分を確認することを確定する。
- `_sqlx_migrations` テーブルで全マイグレーションの `success = TRUE` を確認してから次の手順に進むことを確定する。
- マイグレーション失敗時は `sqlx migrate revert` でロールバックし、エラー原因を特定してから再実行することを確定する。

---

## 4 初期データの確認

### 4-1. システムロール・初期設定値の確認

**MIG-X-127**: マイグレーション後に初期データが正しく存在することを確認する。（DES-MIG-017 対応）

初期マイグレーションには以下のデータが含まれる。

```bash
# システムロールの確認
docker compose -f docker-compose.prod.yml exec postgres \
    psql -U wnav_admin -d wnav_prod -c "
      SELECT role_code, role_name, created_at
      FROM system_roles
      ORDER BY role_code;
    "
```

期待する出力（システムロール 6 種が存在すること）を確認する。

| ロールコード | ロール名 | 確認観点 |
|---|---|---|
| operator | オペレーター | ハンディ作業者 |
| line_leader | ラインリーダー | ラインの統括 |
| master_admin | マスタ管理者 | マスタデータ管理 |
| quality_admin | 品質管理者 | SOP 承認・品質管理 |
| system_admin | システム管理者 | システム全体管理 |
| viewer | 閲覧者 | 参照のみ |

```bash
# システム設定値の確認
docker compose -f docker-compose.prod.yml exec postgres \
    psql -U wnav_admin -d wnav_prod -c "
      SELECT key, value, description
      FROM system_settings
      ORDER BY key;
    "
```

### 4-2. DB 接続の疎通確認

**MIG-X-128**: アプリケーション（axum バックエンド）からの DB 接続が確立できることを確認する。（DES-MIG-017 対応）

```bash
# バックエンドコンテナを起動する
docker compose -f docker-compose.prod.yml up -d backend

# ヘルスチェックエンドポイントで DB 疎通を確認する
curl -f http://localhost:8080/healthz

# 期待するレスポンス
# {"status":"ok","db":"connected","version":"<バージョン>"}
```

ヘルスチェックで `"db":"connected"` が返ることを DB 疎通確認の合格基準とする。`"db":"disconnected"` または接続エラーが返った場合は `docker compose logs backend` でエラーを確認する。

### 4-3. マイグレーション完了の記録テンプレート

```
===============================================================
 PostgreSQL 初期化・マイグレーション完了記録
===============================================================
システム名    : 作業ナビゲーション・トレサビシステム
ドキュメント番号 : WNAV-DB-INIT-PROD-001
版数         : 1.0
===============================================================

1. 実施環境
   対象環境: 本番（Production）
   PostgreSQL バージョン: <docker exec で確認したバージョン>
   DB 名: wnav_prod

2. マイグレーション実行結果
   実施日時:
   実施者:
   適用したマイグレーション数: <件数>
   最新マイグレーションバージョン: <バージョン番号>
   全マイグレーション success=TRUE: [ ] 確認済み

3. 初期データ確認結果
   システムロール件数: <件数>（期待値: 6 件）
   system_settings 件数: <件数>
   DB 疎通確認（GET /healthz）: [ ] 合格

4. 実施者署名
   氏名: ___________________
   日付: ___________________
   署名: ___________________
===============================================================
```

---

**本節で確定した方針**
- マイグレーション後のシステムロール 6 種の存在確認を初期化完了の必要条件とすることを確定する。
- `GET /healthz` で `"db":"connected"` が返ることを DB 疎通確認の合格基準とすることを確定する。
- マイグレーション完了記録テンプレートへの実施者署名をもって INST-A4-c 完了と判定することを確定する。

---

## 参照業界分析

### 必須
- [`../../90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`../../90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)
- [`../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md`](../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md)

---

## 更新履歴

| バージョン | 日付 | 変更内容 | 担当者 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | 初版 | RyuheiKiso |
