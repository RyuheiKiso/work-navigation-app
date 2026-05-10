#!/usr/bin/env bash
# 対応 §: ロードマップ §28 §28.4
# §28 用語集の同義語禁止規約を CI で機械的に検査する。
# 主要用語に対する禁止同義語が文書／コード上に紛れ込んでいないことを確認する。

set -euo pipefail

# 検査対象（ドキュメント＋コード）
INCLUDE_PATHS=(
  "docs"
  "apps"
  "services"
  "addon-sdk"
  "examples"
  "README.md"
  "CHANGELOG.md"
  "CONTRIBUTING.md"
  "MAINTAINERS.md"
)

# 例外（用語集本体は対比表として禁止語を含むため除外）
EXCLUDE_PATHS=(
  "docs/02_設計/glossary-ja.md"
  "docs/02_設計/glossary-en.md"
  "docs/01_企画/メモ.txt"
  "docs/01_企画/ロードマップ.md"
  "docs/01_企画/競合比較表.md"
)

# 禁止語ペア（"禁止語|代替語"）
# §28 用語集と一致させる。既存ロードマップに存在する文脈はホワイトリストで個別除外する設計。
PROHIBITED=(
  "暗黙の妥協|沈黙の妥協（§2.2）"
  "隠れた妥協|沈黙の妥協（§2.2）"
  "プラグイン|アドオン（§17）"
  "ワークフロー|フロー（§28）"
  "改善案|カイゼン（§28／§9.3.1）"
)

# 違反件数
violations=0

# 検査用 grep を構築する
# `--include` で対象ファイルを絞らず、自動除外を find で行う方式
for pair in "${PROHIBITED[@]}"; do
  banned="${pair%%|*}"
  alt="${pair##*|}"
  # 検索を実行
  for path in "${INCLUDE_PATHS[@]}"; do
    [ -e "$path" ] || continue
    # grep -R は --exclude をサポートする
    while IFS= read -r line; do
      # 行が空ならスキップ
      [ -z "$line" ] && continue
      # 例外ファイル判定
      file="${line%%:*}"
      skip=0
      for ex in "${EXCLUDE_PATHS[@]}"; do
        if [ "$file" = "$ex" ]; then
          skip=1
          break
        fi
      done
      [ "$skip" -eq 1 ] && continue
      # 違反として出力
      echo "::warning file=${file}::禁止用語『${banned}』が混入しています（代替: ${alt}）"
      violations=$((violations + 1))
    done < <(grep -R -n -F "$banned" "$path" 2>/dev/null || true)
  done
done

# 集計
if [ "$violations" -gt 0 ]; then
  echo "glossary-lint: ${violations} 件の警告"
  # 個人開発・段階導入のため CI fail はしない（STRICT=1 で強制）
  if [ "${STRICT:-0}" = "1" ]; then
    exit 1
  fi
else
  echo "glossary-lint: OK"
fi
exit 0
