# apps/config-ui

> 対応 §: ロードマップ §7.2 §10.2 §11.1 §11.2

React + TypeScript で実装する設定 Web UI。マスタメンテナンス／作業フロー設定を担う。
§4.4「非エンジニア操作性 ◎」を満たす UI を、`docs/02_設計/設定UI監査.md` の 14 観点で検証する。

## クリーンアーキテクチャ層

| ディレクトリ | 層 |
| --- | --- |
| `src/domain/` | `Flow` Aggregate／`FlowNode`／`FlowEdge` |
| `src/usecase/` | `PublishFlowUseCase` |
| `src/adapter/` | `HttpFlowGateway`（バックエンド REST 経由） |
| `src/presentation/` | `FlowEditor` 等の UI コンポーネント |

## 開発・実行

```bash
# 開発サーバ
pnpm --filter @wna/config-ui dev

# テスト
pnpm --filter @wna/config-ui test

# 型検査
pnpm --filter @wna/config-ui typecheck
```

## 受入観点

- §10.2.4「非エンジニアがサンプル参照のみで 5 分以内に最初のフローを発行できる」を実機検証する。
- 任意時点へのロールバックが操作 3 ステップ以内（§10.2.4）。
- §10.2.2 の 14 観点を `docs/02_設計/設定UI監査.md` チェックリストで網羅する。
