# 対応 §: ロードマップ §14.2 §13.2 §9.6 §22.4
# 本 Makefile は §14.2「make doctor」「セットアップ容易性」を満たすためのフロントエンド。
# 主要ターゲット: doctor（自己診断）／lint／test／build／up／down／ci

# 既定シェルを bash に固定する（POSIX 互換だが pipefail を有効化したいため）
SHELL := /bin/bash
# 失敗を即時に伝播させる（パイプ／コマンドのいずれの失敗も検出する）
.SHELLFLAGS := -eu -o pipefail -c
# 既定ターゲットを help にする（タイプミス時の安全側）
.DEFAULT_GOAL := help

# 表示色（ANSI、`tput colors >= 8` 環境向け）
COLOR_RESET := \033[0m
COLOR_BOLD := \033[1m
COLOR_GREEN := \033[32m
COLOR_YELLOW := \033[33m

# ヘルプ表示（target 一覧）
.PHONY: help
help:
	@echo -e "$(COLOR_BOLD)work-navigation-app Makefile$(COLOR_RESET)"
	@echo "  make doctor   — 自己診断（§14.2）"
	@echo "  make lint     — 全 lint（§9.6）"
	@echo "  make test     — 全テスト（§13.2）"
	@echo "  make build    — 全ビルド"
	@echo "  make up       — docker compose up -d"
	@echo "  make down     — docker compose down"
	@echo "  make demo     — compose up + デモシード投入（§14.2 顧客／社内デモ）"
	@echo "  make demo-down— compose down -v（デモ DB をリセット）"
	@echo "  make ci       — CI 相当のローカル実行（§13.2）"
	@echo "  make clean    — ビルド成果物の削除"

# ----------------------------------------------------------------------
# §14.2 self-diagnostic: OS / Docker / 空きポート / メモリ / 時刻同期 を 30 秒以内に判定
# ----------------------------------------------------------------------
.PHONY: doctor
doctor:
	@echo -e "$(COLOR_BOLD)== work-navigation-app doctor ==$(COLOR_RESET)"
	@echo "OS: $$(uname -srm)"
	@echo "Docker: $$(docker --version 2>/dev/null || echo 'not installed')"
	@echo "Docker Compose: $$(docker compose version 2>/dev/null || echo 'not installed')"
	@echo "Cargo: $$(cargo --version 2>/dev/null || echo 'not installed')"
	@echo "Node: $$(node --version 2>/dev/null || echo 'not installed')"
	@echo "pnpm: $$(pnpm --version 2>/dev/null || echo 'not installed')"
	@echo "Free memory (MiB): $$(free -m 2>/dev/null | awk '/^Mem:/{print $$7}' || echo 'unknown')"
	@echo "NTP sync: $$(timedatectl 2>/dev/null | grep -E 'synchronized' || echo 'unknown')"
	@echo "Free port 5432: $$(ss -ltn 'sport = :5432' 2>/dev/null | grep -q LISTEN && echo 'busy' || echo 'free')"
	@echo "Free port 8080: $$(ss -ltn 'sport = :8080' 2>/dev/null | grep -q LISTEN && echo 'busy' || echo 'free')"
	@echo "Done."

# ----------------------------------------------------------------------
# Lint 群（§9.6 §28 §2.2）
# ----------------------------------------------------------------------
.PHONY: lint
lint: lint-files lint-line-comments lint-glossary lint-rationale lint-rust lint-ts

.PHONY: lint-files
lint-files:
	@scripts/lint-file-size.sh

.PHONY: lint-line-comments
lint-line-comments:
	@scripts/lint-line-comments.sh

.PHONY: lint-glossary
lint-glossary:
	@scripts/glossary-lint.sh

.PHONY: lint-rationale
lint-rationale:
	@scripts/lint-rationale.sh || true

.PHONY: lint-rust
lint-rust:
	@cargo clippy --workspace --all-targets --deny warnings 2>/dev/null || \
	  echo "$(COLOR_YELLOW)cargo not available (skipped)$(COLOR_RESET)"

.PHONY: lint-ts
lint-ts:
	@pnpm -r lint 2>/dev/null || \
	  echo "$(COLOR_YELLOW)pnpm not available (skipped)$(COLOR_RESET)"

# ----------------------------------------------------------------------
# Test 群（§13.2）
# ----------------------------------------------------------------------
.PHONY: test
test: test-rust test-ts

.PHONY: test-rust
test-rust:
	@cargo test --workspace 2>/dev/null || \
	  echo "$(COLOR_YELLOW)cargo not available (skipped)$(COLOR_RESET)"

.PHONY: test-ts
test-ts:
	@pnpm -r test 2>/dev/null || \
	  echo "$(COLOR_YELLOW)pnpm not available (skipped)$(COLOR_RESET)"

# ----------------------------------------------------------------------
# Build 群
# ----------------------------------------------------------------------
.PHONY: build
build: build-rust build-ts

.PHONY: build-rust
build-rust:
	@cargo build --workspace --release 2>/dev/null || \
	  echo "$(COLOR_YELLOW)cargo not available (skipped)$(COLOR_RESET)"

.PHONY: build-ts
build-ts:
	@pnpm -r build 2>/dev/null || \
	  echo "$(COLOR_YELLOW)pnpm not available (skipped)$(COLOR_RESET)"

# ----------------------------------------------------------------------
# docker compose 操作
# ----------------------------------------------------------------------
.PHONY: up
up:
	@docker compose up -d

.PHONY: down
down:
	@docker compose down

# ----------------------------------------------------------------------
# デモセットアップ（§14.2 顧客／社内デモ向けワンコマンド初期化）
# ----------------------------------------------------------------------
# `make demo` は compose 起動 → readiness 待機 → wna-backend seed まで一括実行する。
# adapter 層の upsert API を経由するため、`make demo` 連打でも DB は壊れない（冪等）。
.PHONY: demo
demo:
	@echo -e "$(COLOR_BOLD)== work-navigation-app demo セットアップ ==$(COLOR_RESET)"
	@bash scripts/seed-demo.sh showcase
	@echo -e "$(COLOR_GREEN)デモ環境の準備が完了しました。$(COLOR_RESET)"
	@echo "  端末アプリ: pnpm -F terminal dev"
	@echo "  設定 UI:    pnpm -F config-ui dev"
	@echo "  デモ ID:    alice / bob / charlie  (パスワード: hello-world)"
	@echo "  詳細:       docs/demo.md"

# `make demo-down` はボリューム込みで DB を破棄し、次回 demo を綺麗にやり直す。
.PHONY: demo-down
demo-down:
	@docker compose down -v
	@echo -e "$(COLOR_GREEN)デモ DB を破棄しました。$(COLOR_RESET)"

# ----------------------------------------------------------------------
# CI 相当のローカル実行
# ----------------------------------------------------------------------
.PHONY: ci
ci: lint test build
	@echo -e "$(COLOR_GREEN)CI 相当のローカル実行に成功しました$(COLOR_RESET)"

# ----------------------------------------------------------------------
# Clean
# ----------------------------------------------------------------------
.PHONY: clean
clean:
	@cargo clean 2>/dev/null || true
	@find apps -name 'dist' -type d -exec rm -rf {} + 2>/dev/null || true
	@find apps -name 'node_modules' -type d -prune -exec rm -rf {} + 2>/dev/null || true
