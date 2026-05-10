# ADR-0002: 形式化 notation: HSM／CPN／TLA+

> 提案日: 2026-05-09
> 採用日: 2026-05-09
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §3.4.1 §10.6.1 §27 F-001 F-002 §29 R-016

## Status

Accepted

## Type

Type 1（採用 notation を変更すると過去の検証結果が無効化されるため、不可逆）

## Context

- §3.4 分析方法論の最終ステップは「形式化」であり、§3.1 作業の正式定義と §10.6 同期戦略を機械検証可能な形で固定する必要がある。
- 沈黙の妥協（§2）を防ぐため、採用する notation を **初期決定** として固定し、§22.1 で再評価する。
- §27 FMEA F-001／F-002／§29 R-016（同期破壊）の検出度 D を構造的に下げるためには、状態機械・並行モデル・不変式の 3 層を別の notation で扱うのが効率的。

## Decision

| 対象 | notation |
| --- | --- |
| 単一作業の状態 | **階層型有限状態機械（HSM, Harel statecharts 1987）** |
| 並行・同期・順序情報の流れ | **着色 Petri net（CPN, Jensen & Kristensen 2009）** |
| データ・整合性 | **TLA+（Lamport 1999）** |
| 動的トポロジ（プロセス代数） | π計算／CSP は **不採用** |

形式化成果物は `docs/03_設計/形式化/` に配置する（HSM 図／CPN モデル `.cpn`／TLA+ `.tla`）。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| UML state machine（HSM の代替） | UML 標準の状態機械 | 階層・並行を表現する場合に冗長。Harel statecharts のほうが簡潔 |
| プレース変遷ネット（CPN の代替） | 色情報なしの Petri net | 属性（カラー）表現が困難 |
| Alloy（TLA+ の代替） | 関係論理ベース | バウンデッド過ぎる（無限状態空間の不変性証明が困難） |
| π計算 | プロセス代数の動的チャネル生成 | 本アプリ層に動的チャネル生成中心の処理がない／過剰 |
| CSP | 通信プロセス代数 | 同上 |

## Consequences

- **正の帰結**: HSM／CPN／TLA+ の各 notation がそれぞれ得意領域で表現され、機械検証可能。学術界からも引用可能（§4.9 圧倒候補機能「ナビゲーションの形式化」）。
- **負の帰結**: 3 つの記法を学習する必要があり、貢献者の参入障壁が高い。教育コンテンツ（§25）と SDK チュートリアル（§17）でフォローする。
- **影響範囲**: §3.4.1（採用宣言）、§3.4.2（受入観点）、§10.6.1（同期形式モデル）、§13.2（テストツール）、§27 FMEA（検出度 D の改善手段）、§31 SLI-06（監査ログ追記不変性違反検知数）。

## Type 1 撤退条件

- TLA+ ／ CPN Tools のいずれかが OSS としてメンテナンス停止し、代替が無い場合。
- 学術界の主流 notation が変化し、HSM／CPN／TLA+ がデファクトを失った場合（§22.1 半期レビューで判定）。
- 撤退時の代替: 単一の形式化フレームワーク（例: Coq/Isabelle）への一本化を ADR で再判断する。

## §24.2 出所表への追記

- 追記済: Yes（§24.2「形式化 notation 確定（HSM／CPN／TLA+） → §3.4.1」行）

## References

- ロードマップ §3.4.1 §3.4.2 §10.6.1 §10.6.2
- Harel, D. *Statecharts: A Visual Formalism for Complex Systems*. Science of Computer Programming, 1987.
- Jensen, K., Kristensen, L. M. *Coloured Petri Nets — Modelling and Validation of Concurrent Systems*. Springer, 2009.
- Lamport, L. *Specifying Systems*. Addison-Wesley, 2002（TLA+ の標準教科書）。
- 関連 ADR: ADR-0003（同期形式モデル）
- 関連 FMEA: F-001／F-002
- 関連リスク: R-016
