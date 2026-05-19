# src/infra/database

PostgreSQL 16 データベースのマイグレーション・コンテナ設定・運用スクリプト一式を管理するディレクトリ。

## ディレクトリ構成

```
src/infra/database/
├── Dockerfile                          # postgres:16-alpine ベースイメージ
├── .dockerignore
├── README.md                           # 本ファイル
├── config/
│   ├── postgresql.conf                 # Active ノード向けの PostgreSQL 設定
│   ├── postgresql.standby.conf         # Standby ノード向けの追加設定
│   ├── pg_hba.conf                     # 接続認証設定（scram-sha-256 + RESTRICT）
│   ├── pg_ident.conf                   # アイデンティティマップ（デフォルト空）
│   └── recovery.conf.template          # フェイルオーバ時の昇格テンプレート
├── docker-entrypoint-initdb.d/
│   ├── 00_create_databases.sh          # 環境変数 WNAV_DATABASES で指定した DB を作成
│   ├── 01_create_login_roles.sql       # ログインロール作成（パスワードは環境変数で設定）
│   └── 02_create_app_roles.sql         # グループロール作成 + ログインロール追加
├── migrations/                         # sqlx migrate 形式 SQL（61 本）
│   ├── V20260517120000__create_extensions.sql
│   ├── ...（V20260517120060 まで連番）
│   └── V20260517120060__seed_step_type_definitions.sql
├── scripts/
│   ├── apply_migrations.sh             # マイグレーション実行ラッパー
│   ├── backup.sh                           # BAT-005 日次バックアップ（AES-256-GCM 暗号化・90日保持）
│   ├── restore.sh                          # リストア（5 ステップ）
│   ├── failover.sh                         # Active/Standby フェイルオーバ
│   ├── wal_archive.sh                      # WAL アーカイブ（postgresql.conf archive_command から呼び出し）
│   ├── evidence_rsync.sh                   # 証拠ファイル NAS 同期（日次 03:00 JST）
│   ├── partition_create_monthly.sql        # 翌月 work_events パーティション作成（月次 25 日）
│   ├── cold_archive.sh                     # 61 ヶ月超パーティションのコールドアーカイブ
│   ├── verify_hash_chain.sql               # BAT-001 ハッシュチェーン整合性検証
│   ├── pii_anonymize.sql                   # BAT-004 退職ユーザー PII 匿名化
│   ├── refresh_mv_daily_work_summary.sql   # BAT-010 mv_daily_work_summary REFRESH CONCURRENTLY
│   ├── idempotency_keys_gc.sql             # idempotency_keys 24h TTL ガベージコレクション
│   ├── case_locks_expire.sql               # BAT-013 case_locks ハートビートタイムアウト
│   ├── work_assignments_expire.sql         # BAT-015 作業指示期限切れ
│   ├── rework_cost_aggregate.sql           # BAT-011 リワーク原価集計
│   ├── apply_migrations.sh                 # sqlx migrate 実行ラッパー
│   └── health_check.sh                     # pg_isready ヘルスチェック
└── json-schemas/                       # JSONB 列の JSON Schema 定義（15 本）
    ├── instruction_text.v1.schema.json
    ├── judgment_condition.v1.schema.json
    ├── required_scans.v1.schema.json
    ├── multilingual_name.v1.schema.json
    ├── external_key.v1.schema.json
    ├── step_flow_rule_definition.v1.schema.json
    └── payload.*.v1.schema.json（9 種）
```

## コンテナ起動

```bash
# 開発環境（wnav_dev データベースを作成）
docker build -t wnav-postgres:16 src/infra/database/
docker run -d \
    --name wnav_db \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=CHANGE_ME \
    -e WNAV_DATABASES=wnav_dev \
    -p 127.0.0.1:5432:5432 \
    --network wnav_backend \
    -v wnav_postgres_data:/var/lib/postgresql/data \
    -v wnav_wal_archive:/wal_archive \
    wnav-postgres:16
```

## マイグレーション実行

```bash
# 環境変数を設定する
export DATABASE_URL="postgres://wnav_admin:CHANGE_ME@localhost:5432/wnav_dev"
export MIGRATION_DIR="src/infra/database/migrations"

# dry-run で確認してから適用する
./src/infra/database/scripts/apply_migrations.sh

# または sqlx-cli を直接使用する
sqlx migrate run --source src/infra/database/migrations
```

## 環境変数

| 変数名 | 必須 | デフォルト | 説明 |
|---|---|---|---|
| `DATABASE_URL` | ○ | — | sqlx migrate 用の接続 URL |
| `POSTGRES_USER` | ○ | — | PostgreSQL スーパーユーザー名 |
| `POSTGRES_PASSWORD` | ○ | — | PostgreSQL スーパーユーザーパスワード |
| `WNAV_DATABASES` | — | `wnav_dev` | 作成するデータベース名（スペース区切りで複数可） |
| `BACKUP_ENCRYPTION_KEY` | ○（本番） | — | AES-256-GCM バックアップ暗号化キー |
| `EVIDENCE_SOURCE` | — | `/data/evidence` | 証拠ファイルのソースディレクトリ |
| `NAS_TARGET` | ○（本番） | — | rsync 転送先（例: `nas.internal:/backup/evidence`） |
| `PGHOST` | — | `localhost` | ヘルスチェック・バックアップ用ホスト |
| `PGPORT` | — | `5432` | PostgreSQL ポート |

## ボリューム規約

| ボリューム名 | マウント先 | 用途 |
|---|---|---|
| `wnav_postgres_data` | `/var/lib/postgresql/data` | PostgreSQL データ領域 |
| `wnav_wal_archive` | `/wal_archive` | WAL アーカイブ（BAT-002） |
| `wnav_evidence_files` | `/data/evidence` | 証拠ファイル一時保存 |

## DB ロール構成

### グループロール（NOLOGIN）

| ロール名 | 権限 | 用途 |
|---|---|---|
| `app_event_writer` | Append-only テーブルへの INSERT/SELECT | 作業ログ記録 |
| `app_read_write` | 更新可テーブルへの CRUD | マスタ操作 |
| `app_admin` | 全権限 + CREATEROLE | マイグレーション・ユーザー管理 |
| `app_event_insert` | case_locks への INSERT/UPDATE/DELETE | 端末排他制御（例外） |

### ログインロール

| ロール名 | グループ | 用途 |
|---|---|---|
| `wnav_admin` | app_admin | マイグレーション・管理操作 |
| `wnav_write` | app_read_write | バックエンド API（マスタ操作） |
| `wnav_event_insert` | app_event_writer + app_event_insert | バックエンド API（作業ログ記録） |
| `wnav_read` | app_read_write（SELECT のみ） | 監査・ダッシュボード |
| `wnav_backup` | pg_read_all_data | バックアップ専用 |

## バックアップ

```bash
# 日次バックアップ（AES-256-GCM 暗号化、/backup/db/ に保存、7 世代保持）
BACKUP_ENCRYPTION_KEY=<key> ./src/infra/database/scripts/backup.sh

# リストア
BACKUP_ENCRYPTION_KEY=<key> ./src/infra/database/scripts/restore.sh /backup/db/work_nav_YYYYMMDD.dump.gz.enc
```

## フェイルオーバ

Active/Standby 構成での Standby 昇格（RTO 目標: 60 分）:

```bash
./src/infra/database/scripts/failover.sh
```

詳細手順: `docs/09_運用・保守/運用手順/10_ActiveStandby切替手順（OPS-PROC-010）.md`

## BAT ジョブ一覧

| BAT-ID | スクリプト | 実行タイミング | 説明 |
|---|---|---|---|
| BAT-001 | `verify_hash_chain.sql` | 週次（月曜 03:00 JST） | ハッシュチェーン整合性検証 |
| BAT-004 | `pii_anonymize.sql` | 日次 01:00 JST | 退職ユーザー PII 匿名化 |
| BAT-005 | `backup.sh` | 日次 02:00 JST | 日次フルバックアップ（90 日保持） |
| BAT-010 | `refresh_mv_daily_work_summary.sql` | 日次 06:00 / 週次月 07:00 / 月次 1 日 07:00 | mv_daily_work_summary REFRESH CONCURRENTLY |
| BAT-011 | `rework_cost_aggregate.sql` | 日次 03:00 JST | リワーク原価集計 |
| BAT-013 | `case_locks_expire.sql` | 毎分 | case_locks ハートビートタイムアウト |
| BAT-015 | `work_assignments_expire.sql` | 5 分毎 | 作業指示期限切れ遷移 |
| — | `wal_archive.sh` | 連続（archive_command） | WAL アーカイブ（postgresql.conf から呼び出し） |
| — | `evidence_rsync.sh` | 日次 03:00 JST | 証拠ファイル NAS 同期 |
| — | `partition_create_monthly.sql` | 毎月 25 日 02:00 JST | 翌月 work_events パーティション作成 |
| — | `cold_archive.sh` | 月次（手動トリガ） | 61 ヶ月超パーティションのコールドアーカイブ |
| — | `idempotency_keys_gc.sql` | 毎時 | idempotency_keys 24h TTL 削除 |

## 検証チェックリスト

マイグレーション適用後に以下を確認する:

```sql
-- テーブル数（57 テーブル + 12 work_events パーティション + _sqlx_migrations = 70 以上）
SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public';

-- ビュー数（通常ビュー 7 + マテリアライズドビュー 1 = 8）
SELECT COUNT(*) FROM pg_views WHERE schemaname = 'public';
SELECT COUNT(*) FROM pg_matviews WHERE schemaname = 'public';

-- ロール確認
SELECT rolname FROM pg_roles WHERE rolname LIKE 'app_%' ORDER BY rolname;

-- Append-only 保証確認（0 行が正常）
SELECT grantee, privilege_type
FROM information_schema.role_table_grants
WHERE table_name = 'work_events'
  AND grantee IN ('app_event_writer', 'app_read_write', 'PUBLIC')
  AND privilege_type IN ('UPDATE', 'DELETE');

-- シードデータ確認
SELECT role_name FROM roles ORDER BY role_id;

-- トリガ確認
SELECT trigger_name FROM information_schema.triggers
WHERE trigger_name = 'trg_disposition_distinct_signers';
```

## 関連ドキュメント

| ドキュメント | 内容 |
|---|---|
| `docs/05_詳細設計/01_データベース詳細設計/` | DDL・インデックス・ビュー・マイグレーション設計の権威ソース |
| `docs/04_概要設計/04_データ設計/` | テーブルカタログ・論理 ER・ハッシュチェーン設計 |
| `docs/04_概要設計/01_システム方式設計/03_配置設計*.md` | Active/Standby 構成・Docker 設定 |
| `docs/04_概要設計/08_運用方式設計/04_バックアップ・リストア方式.md` | バックアップ手順の詳細 |
| `docs/09_運用・保守/運用手順/10_ActiveStandby切替手順.md` | フェイルオーバ手順の詳細 |
| `docs/01_管理/ADR/ADR-012〜017.md` | 本ディレクトリの設計判断（不整合解決記録） |
