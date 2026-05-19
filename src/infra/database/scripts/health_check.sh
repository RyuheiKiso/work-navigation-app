#!/bin/bash
# health_check.sh — PostgreSQL ヘルスチェックスクリプト
# 権威ドキュメント:
#   docs/04_概要設計/01_システム方式設計/03_配置設計（Active_Standby・単一建屋内冗長）.md §3-1
#
# 目的: PostgreSQL が接続を受け付けられる状態にあるかを確認する
#       Docker HEALTHCHECK または監視スクリプトから呼び出す
#
# 正常終了: exit 0（PostgreSQL が接続可能）
# 異常終了: exit 1（PostgreSQL が接続不可）

set -euo pipefail

# pg_isready で PostgreSQL の起動状態を確認する
# 環境変数が設定されていない場合はデフォルト値を使用する
pg_isready \
    -U "${POSTGRES_USER:-postgres}" \
    -d "${POSTGRES_DB:-postgres}" \
    -h "${PGHOST:-localhost}" \
    -p "${PGPORT:-5432}"
