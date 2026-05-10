# マイグレーション移行ガイド（テンプレート）

> 対応 §: ロードマップ §32.4 §15 §10.2 §10.3.1 §17.3
> 対象読者: 導入企業のリリース担当、サブシステムオーナー
> 改訂サイクル: メジャーリリースごと

§32.4 廃止予告プロセスの「削除」段階で同梱する移行ガイドの雛形。本テンプレートを `docs/04_運用/migration-<from>-to-<to>.md`（例: `migration-v1-to-v2.md`）にコピーし、実際の差分を埋めて配布する。

## 1. 概要

| 項目 | 値 |
| --- | --- |
| 移行元バージョン | （例）v1.x.x |
| 移行先バージョン | （例）v2.0.0 |
| リリース予定日 | YYYY-MM-DD |
| 廃止予告開始日 | YYYY-MM-DD（最低 1 マイナー版前、§32.4） |
| アドオン API 削除猶予 | 6 ヶ月（§17.3／§19.3） |
| 影響範囲 | 端末／設定 UI／バックエンド／アドオン／基幹連携 |

## 2. 破壊的変更（BREAKING CHANGE）一覧

| 区分 | 変更前 | 変更後 | 移行手順 |
| --- | --- | --- | --- |
| OpenAPI | （旧スキーマ） | （新スキーマ） | クライアントの再生成（`openapi-typescript`） |
| アドオン API surface | （旧 API 名） | （新 API 名） | アドオン側のコール置換、SemVer メジャーバンプ |
| 監査ログスキーマ | （旧フィールド） | （新フィールド） | 旧スキーマのアーカイブ／新規追記は新スキーマで |
| DB スキーマ | （旧テーブル） | （新テーブル） | 自動マイグレーション（`sqlx migrate run`） |
| 翻訳キー | （旧 ID） | （新 ID） | 翻訳ファイル lint で漏れ検出 |

## 3. 事前準備（移行前チェックリスト）

- [ ] バックアップ取得（PostgreSQL／SQLite）。RPO 目標は §15.2 を参照。
- [ ] 旧バージョンに対する CHANGELOG／本ガイド記載の警告を確認した。
- [ ] 廃止予告期間（最低 1 マイナー版）でユーザー・アドオン開発者に通知した。
- [ ] CI で `openapi-diff` ／ `cargo semver-checks` ／ フィクスチャ往復テストが緑。
- [ ] §32.5 ロールバック手順に従い、移行失敗時の戻し口を確認した。

## 4. 移行手順

### 4.1. サーバ

```bash
# 1. バックアップ
pg_dump -Fc -d wna > /backup/wna-$(date +%F).dump

# 2. 新版イメージで起動（マイグレーション自動実行）
sed -i 's/IMAGE_TAG=v1\..*/IMAGE_TAG=v2.0.0/' /opt/wna/.env
docker compose -f /opt/wna/docker-compose.yml up -d

# 3. ヘルスチェック・互換性確認
curl -fsSL http://localhost/healthz
```

### 4.2. 端末

```bash
# Android
apksigner verify --print-certs work-navigation-app-v2.0.0.apk
adb install -r work-navigation-app-v2.0.0.apk

# Windows
Get-AuthenticodeSignature .\work-navigation-app-v2.0.0.msi
msiexec /i work-navigation-app-v2.0.0.msi /qn
```

端末側 SQLite のマイグレーションはアプリ起動時に自動実行される。失敗時は §10.5.1 と同方針で業務操作を拒否し管理者通知。

### 4.3. アドオン

```bash
# 1. アドオン互換性チェック
wnaddon-cli check ./my-addon.wnaddon --against v2.0.0

# 2. 互換 NG の場合: アドオン側を更新（API 置換／capability 再宣言）
# 3. 互換 OK の場合: そのまま運用継続
```

API 互換性の自動検査は CI（`openapi-diff` ／ `cargo semver-checks`）で実施（CI 整備後）。

### 4.4. 設定（フロー定義）

- フロー定義のスキーマ進化は §15 互換マトリクスに従う。
- 旧フォーマットのフローは新版でも読み込み可能であること（前方互換）。
- 必要に応じてエクスポート → インポートの順で再投入する。

## 5. ロールバック条件

次の場合は §32.5 [`rollback.md`](./rollback.md) に従い直前安定版へ戻す。

- 移行後 24 時間以内に CVSS ≥ 7.0 の脆弱性が検出された。
- 主要機能（作業ナビ／同期／監査ログ）が回帰した。
- 性能 SLO（§31.2）が連続超過した。

## 6. よくあるトラブル

| 症状 | 対処 |
| --- | --- |
| 端末側マイグレーションが失敗 | 直前バックアップから復旧／§14.2 `make doctor` で原因切り分け |
| アドオンが capability エラーで起動しない | manifest.toml の capability 宣言を新 API に合わせる |
| 同期遅延が増えた | DLQ 件数を確認（§31 SLI-07）／ネットワーク帯域を確認 |
| 監査ログ整合違反検知 | §27 F-002 / R-016 の対応／管理者通知発火 |

## 7. 受入観点（§32.4／§32.6）

- すべての破壊的変更が予告 → 警告 → 削除の 3 段階を経ていること。
- 本ガイドが廃止予告期間中に公開されていること。
- ロールバック手順とリンクされていること（[`rollback.md`](./rollback.md)）。
- アドオン API の deprecation が 6 ヶ月猶予を満たすこと（時刻計測テスト、CI 整備後）。
- 移行ガイドの記載漏れは §22.4 是正フロー対象。
