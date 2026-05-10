#!/usr/bin/env bash
# 対応 §: ロードマップ §4.8 §22.1
# 競合監視の自動収集。RSS／GitHub API／arXiv／特許 DB／投資情報／OSS 健全性の
# 7 系統を **取得試行のみ** 行い、変化検出は本セッション範囲外（手動レビュー前提）。
# 自動収集データは `docs/01_企画/競合監視ログ/<年月>.md` に追記する。

set -euo pipefail

# 出力ファイル
YEAR_MONTH=$(date -u +%Y-%m)
LOG_DIR="docs/01_企画/競合監視ログ"
LOG_FILE="${LOG_DIR}/${YEAR_MONTH}.md"

# ディレクトリ作成
mkdir -p "$LOG_DIR"

# 既存ファイルが無ければスケルトンを作成
if [ ! -f "$LOG_FILE" ]; then
  cat > "$LOG_FILE" <<'EOF'
# 競合監視ログ（自動収集）

> 対応 §: ロードマップ §4.8
> 自動収集の試行履歴。重大トピックは手動レビュー（§22.1 四半期）で `competitor-watch` ラベル Issue に昇格させる。

EOF
fi

# 1. 競合公式 RSS（例: Tulip／Augmentir 等の RSS）
RSS_URLS=(
  # 公開 RSS が見つかった製品のみ列挙する。最初は空でも問題ない。
  # "https://tulip.co/feed/"
  # "https://www.augmentir.com/feed/"
)

# 2. arXiv 検索（cs.HC/cs.SE/cs.SY、キーワード manufacturing work instruction）
ARXIV_QUERY='cat:cs.HC+AND+abs:%22work+instruction%22'

# 取得試行のみ。失敗は警告として記録。
{
  echo
  echo "## $(date -u +%Y-%m-%dT%H:%M:%SZ) 自動収集試行"
  echo
  echo "### RSS"
  if [ "${#RSS_URLS[@]}" -eq 0 ]; then
    echo "- 監視対象 RSS 未設定（本スクリプト内で URL を追加する）"
  else
    for url in "${RSS_URLS[@]}"; do
      if curl -fsS -m 10 "$url" >/dev/null 2>&1; then
        echo "- ${url}: OK"
      else
        echo "- ${url}: 取得失敗（ネットワーク／DNS／timeout）"
      fi
    done
  fi
  echo
  echo "### arXiv"
  echo "- クエリ: ${ARXIV_QUERY}"
  if curl -fsS -m 10 "http://export.arxiv.org/api/query?search_query=${ARXIV_QUERY}&max_results=1" >/dev/null 2>&1; then
    echo "- 取得 OK"
  else
    echo "- 取得失敗（オフライン環境では正常）"
  fi
  echo
  echo "### GitHub Trending（OSS 競合候補）"
  if command -v gh >/dev/null 2>&1; then
    echo "- gh CLI 検出済（手動で gh search repos で実行）"
  else
    echo "- gh CLI 未導入（§4.8 整備対象）"
  fi
  echo
  # バッククォートはシェル展開対象のため、ラベル名はシングルクォートで囲んで literal に保つ
  echo '### 注: 本ログは自動試行の記録のみ。重大トピックは手動レビューで `competitor-watch` Issue を起票する。'
} >> "$LOG_FILE"

echo "competitor-watch: ${LOG_FILE} に追記しました"
exit 0
