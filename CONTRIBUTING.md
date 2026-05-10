# 貢献ガイド

> 対応 §: ロードマップ §19.2 §19.4 §9.4 §2.2 §30
> 対象読者: 本リポジトリへの PR・Issue を提出するすべての貢献者
> 改訂サイクル: §22.1 半期サイクル

本書は work-navigation-app への貢献手続きを定める。本プロジェクトは個人開発から始まる OSS であり、§19.4 の段階に従って外部貢献者を取り込む。

## 1. 行動規範（Code of Conduct）

すべての貢献者は [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)（Contributor Covenant v2.1）に同意したうえで参加する。違反対応の SLA は 7 日（§19.4.4）。

## 2. ライセンスと DCO（Developer Certificate of Origin）

本プロジェクトは Apache License 2.0（[`LICENSE`](./LICENSE)）で配布する。新規 PR は **Developer Certificate of Origin 1.1** に署名すること。コミットメッセージ末尾に次の行を含める。

```
Signed-off-by: Random J Developer <random@developer.example.org>
```

`git commit -s` で自動付与できる。DCO 署名のない PR は CI で fail する（§19.3）。

## 3. PR を作る前に

1. [`docs/01_企画/ロードマップ.md`](./docs/01_企画/ロードマップ.md) を読み、本プロジェクトの「妥協禁止」「世界一」「沈黙の妥協禁止」（§1 §2 §2.2）の姿勢を理解する。
2. 該当する受入観点（ロードマップ各章末尾）を確認する。
3. 既存 Issue／Discussions に重複が無いか検索する。`good-first-issue` ラベルは初回貢献者向け。
4. 大きい変更は **事前に Issue または Discussions で議論** する（§19.4.2）。

## 4. ブランチ・コミット規約

- ブランチ名: `feat/<short>`／`fix/<short>`／`docs/<short>`／`refactor/<short>`／`test/<short>`／`chore/<short>`。
- コミットメッセージは **Conventional Commits 1.0.0**（§9.4）。例: `feat(navigator): 中断・再開を HSM 状態として表現`。
- 1 PR ≤ 500 行差分（§9.4／§9.6）。超過時は責務単位で分割する。

## 5. PR 受入基準（§19.3／§19.4）

PR は次の条件をすべて満たすこと。

- [ ] 既存テスト緑／カバレッジ低下なし（ドメイン層 ≥ 90%、§13.3／§21 注 7）。
- [ ] `clippy --deny warnings`／`eslint --max-warnings 0` 緑（§9.6）。
- [ ] DCO 署名済み。
- [ ] Conventional Commits 準拠。
- [ ] 1 PR ≤ 500 行差分。
- [ ] 受入観点（ロードマップ該当章）の達成が説明欄に記載されている。
- [ ] **PR テンプレートの「沈黙の妥協」チェック 3 項（§2.2）** にすべて Yes か根拠が記載されている。
- [ ] **Type 1 決定**（§30.1）を含む場合、ADR 追加と §24.2 への追記がセットで含まれる。
- [ ] FMEA／リスク登録簿に影響する場合、対応行が更新されている（§27 §29）。
- [ ] アクセシビリティ・国際化・規制適合のいずれかに影響する場合、該当章の受入観点を再評価している。

メンテナ 1 名以上のレビュー承認で merge 可能。`MAINTAINERS.md` を参照。

## 6. ラベル運用

| ラベル | 用途 |
| --- | --- |
| `good-first-issue` | 初回貢献者向け |
| `help-wanted` | 外部支援を歓迎 |
| `silent-compromise` | §2.2 検出時に CI が自動付与 |
| `competitive-score` | §4.6.3 競合スコア反論プロセス |
| `competitor-watch` | §4.8 競合監視自動化 |
| `regression` | §22.4 是正フロー |
| `competitive-gap` | §22.4 競合との差分検出 |
| `risk-new` | §29.3 新規リスク起票 |
| `llm-error` | §18.5.1 LLM 出力誤りの是正 |
| `anti-goal` | §26 アンチゴール抵触 |
| `governance/<期>` | §22.1 改訂サイクル記録 |

## 7. 文書のみの変更

- ドキュメントのみの PR でも本書の規約と PR テンプレートを適用する。
- `docs/CLAUDE.md` の「アスキー図禁止」を遵守する（drawio／PNG 推奨）。
- ロードマップ §24.2 に関わる新規独自決定を含む文書追加は、該当行の追加もセットで提出する。

## 8. コードのみの変更（将来）

本セッション時点ではコードは存在しないが、将来コード貢献が始まった際は次を遵守する。

- 各行の上に日本語コメント（ルート `CLAUDE.md`）。
- 1 ファイル ≤ 500 行（ルート `CLAUDE.md` ／§9.4）。
- クリーンアーキテクチャ／DDD／TDD（ルート `CLAUDE.md`／§9.1）。
- パブリック API には rustdoc／TSDoc 必須（§9.4／§9.6）。

## 9. ローカル開発（将来手順の予約）

将来 `make doctor`／`docker compose up`／Tauri ビルド手順を本節に追記する（§14.2）。本セッション時点では手順は未整備。

## 10. セキュリティ脆弱性の報告

セキュリティ脆弱性は GitHub Issue で公開しない。[`SECURITY.md`](./SECURITY.md) に従い GitHub Security Advisories から報告する。

## 11. 学術界・教育目的での貢献

- §19.4.2 に従い、HCI／CSCW／ICSE／IEEE TII への CFP 提案を歓迎する。
- 学生 Capstone プロジェクトのテーマは [`docs/community/capstone-themes.md`](./docs/community/capstone-themes.md) を参照。

## 12. 参考リンク

- ロードマップ: [`docs/01_企画/ロードマップ.md`](./docs/01_企画/ロードマップ.md)
- 用語集: [`docs/02_設計/glossary-ja.md`](./docs/02_設計/glossary-ja.md)
- ADR: [`docs/adr/`](./docs/adr/)
- リスク登録簿: [`docs/04_運用/risk-register.md`](./docs/04_運用/risk-register.md)
- FMEA: [`docs/04_運用/fmea.md`](./docs/04_運用/fmea.md)
