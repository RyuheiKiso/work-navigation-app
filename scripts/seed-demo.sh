#!/usr/bin/env bash
# 対応 §: ロードマップ §14.2 §10.2.1（テンプレ） §4.9 §19.4.2
# デモシード一括投入スクリプト。§14.2 ワンコマンド初期化を満たす。

set -euo pipefail

PG_CONTAINER="${PG_CONTAINER:-wna-postgres}"

echo "== work-navigation-app デモシード =="

# 1. PostgreSQL 起動確認
if ! docker ps --format '{{.Names}}' | grep -q "$PG_CONTAINER"; then
  echo "[1/4] PostgreSQL を起動"
  docker compose up -d postgres
  sleep 5
fi
docker exec "$PG_CONTAINER" pg_isready -U wna -t 10 >/dev/null

# 2. デモユーザ投入
echo "[2/4] デモユーザ投入（alice / bob）"
PHC='$argon2id$v=19$m=19456,t=2,p=1$eqsyBNgnGLfArY7lREIZJQ$vhW6FgEEmSMHMdL43tmfQmmLZ6J5wJKms/zuCejdryg'
docker exec "$PG_CONTAINER" psql -U wna -d wna -q -c "
INSERT INTO credentials (user_id, display_name, enabled, password_hash) VALUES
  ('alice', 'Alice Operator', true, '${PHC}'),
  ('bob', 'Bob 班長', true, '${PHC}'),
  ('charlie', 'Charlie 生産技術', true, '${PHC}')
ON CONFLICT (user_id) DO UPDATE SET password_hash = EXCLUDED.password_hash;" >/dev/null

# 3. デモ Task 投入
echo "[3/4] デモ Task 投入（demo-task-001 〜 003）"
docker exec "$PG_CONTAINER" psql -U wna -d wna -q -c "
INSERT INTO tasks (id, state, device_id, lamport, completion_criteria) VALUES
  ('demo-task-001', 'Idle', 'terminal-001', 0, 'manual'),
  ('demo-task-002', 'Idle', 'terminal-001', 0, 'photo'),
  ('demo-task-003', 'Idle', 'terminal-002', 0, 'manual')
ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state;" >/dev/null

# 4. ペアリング QR の生成
echo "[4/4] 端末ペアリング QR を出力"
bash scripts/qr-pair.sh terminal-001 || echo "（qr-pair.sh が未生成。次フェーズで作る）"

cat <<'EOF'

== ✓ デモシード完了 ==
ログイン情報:
  alice / hello-world  （オペレータ）
  bob / hello-world    （班長）
  charlie / hello-world（生産技術）

URL:
  端末:    http://localhost:1420
  設定UI:  http://localhost:1421
  バック:  http://localhost:8080

EOF
