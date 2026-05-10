# Architecture Decision Records (ADR)

> 対応 §: ロードマップ §9.4 §30 §33
> 形式: Michael Nygard 2011 提唱形式（Status／Context／Decision／Consequences）に Type 1／撤退条件を追加
> 対象読者: メンテナ、貢献者、Type 1 決定の影響範囲レビュア
> 改訂サイクル: §22.1 半期サイクル／決定変更時は ADR 追記＋ §24.2 出所表更新

§9.4 重要設計判断は ADR として本ディレクトリに残す。§30 Type 1（不可逆）決定は ADR 必須、Type 2（可逆）決定は通常 PR レビューで足りる。`deferred/` には §33 遅延決定（DEFER-）を保管する。

## 1. インデックス

| ID | タイトル | Type | Status |
| --- | --- | --- | --- |
| [0000](./0000-template.md) | テンプレート | — | — |
| [0001](./0001-license-apache-2-0.md) | ライセンス: Apache-2.0 | Type 1 | Accepted |
| [0002](./0002-formalization-notation.md) | 形式化 notation: HSM／CPN／TLA+ | Type 1 | Accepted |
| [0003](./0003-sync-formal-model.md) | 同期形式モデル: G-Set／LWW／Lamport | Type 1 | Accepted |
| [0004](./0004-storage-encryption-sqlcipher.md) | 端末暗号化: SQLCipher（AES-256） | Type 1 | Accepted |
| [0005](./0005-addon-api-surface-v1.md) | アドオン API surface v1（11 領域） | Type 1 | Accepted |
| [0006](./0006-tech-stack.md) | 技術スタック: Tauri＋React／Rust（tokio）／PostgreSQL／SQLite | Type 1 | Accepted |
| [0007](./0007-auth-id-password-default.md) | 既定認証: ID＋パスワード（高度認証はアドオン化） | Type 1 | Accepted |
| [0008](./0008-ios-out-of-scope.md) | iOS／iPadOS 非対応（閲覧用途を除く） | Type 1 | Accepted |
| [0009](./0009-pr-size-exception-bootstrap.md) | 初期投入における 1 PR ≤ 500 行差分の例外 | Type 2 | Accepted |
| [0010](./0010-hs256-real-hmac.md) | セッショントークン署名の HMAC-SHA256 化 | Type 1 | Accepted |
| [0011](./0011-quality-roadmap.md) | 至高品質バックログ | Type 2 | Proposed |

## 2. 命名規則

```
docs/adr/<連番 4 桁>-<英数字-ハイフン区切り>.md
docs/adr/deferred/DEFER-<YYYYMMDD>-<short>.md
```

- 連番は採番済み番号の最大値 + 1。
- 上書き・再番号付け禁止。

## 3. ADR の必須セクション

§30.2 Type 1 決定のチェックリストを満たすため、各 ADR は次のセクションを含む。

1. **Status**: `Proposed`／`Accepted`／`Deprecated`／`Superseded by ADR-XXXX`
2. **Type**: `Type 1`／`Type 2`
3. **Context**: 決定が必要になった背景・制約・問題
4. **Decision**: 採用した方針
5. **Alternatives**: 代替案 ≥ 2 件と却下理由
6. **Consequences**: 採用結果として生じる正・負の帰結
7. **Type 1 撤退条件**（Type 1 のみ）: §22.3 撤退条件に整合する撤退基準
8. **§24.2 出所表への追記**（Type 1 のみ）: 追記済の有無を Yes／No で記録
9. **References**: ロードマップの該当 §、外部資料

## 4. 受入観点（§30.4）

- ADR 件数が Type 1 決定件数と整合（CI で件数チェック、CI 整備後）。
- Type 1 決定の撤退条件が空欄の ADR を CI で検出。
- §24.2 行追加と Type 1 ADR 追加が同期している。
- 半期に 1 回、Type 2 決定の揺れ頻度を `git log` 解析で集計し、Type 1 化候補を検討（§22.1）。
- Type 1 決定変更時は CHANGELOG と公開ポストモーテム（§2.3）で告知。
