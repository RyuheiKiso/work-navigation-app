# ADR-0003: 同期形式モデル: G-Set／LWW-Register／Lamport

> 提案日: 2026-05-09
> 採用日: 2026-05-09
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §10.6 §10.6.1 §10.6.2 §3.4.1 §27 F-002 §29 R-016 §31 SLI-06 SLO-06

## Status

Accepted

## Type

Type 1（CRDT 種別の変更は端末↔サーバ間の永続データ移行を伴うため不可逆）

## Context

- §10.6 同期競合方針（端末優先・サーバ優先・LWW・追記のみ）は文字面では運用可能だが、形式モデルが定まっていないと §27 F-002（同期競合解決の決定性破綻）／§29 R-016（同期破壊）が顕在化しうる。
- 沈黙の妥協（§2.2）を回避するため、採用する CRDT 種別と Lamport クロックの扱いを **初期決定** として固定する。
- 形式検証は §3.4.1 TLA+（[`sync.tla`](../03_設計/形式化/sync.tla)）と §13.2 proptest の組み合わせで担う。

## Decision

| エンティティ群 | 形式モデル | 検証 |
| --- | --- | --- |
| 作業実績（イベント） | G-Set（追記のみ Set, Shapiro et al. 2011） | TLA+ 不変式 `Inv_NoEventLoss`（INV-01）|
| 添付メディア | LWW-Element-Set（不可逆／追記） | ハッシュ照合（§10.4.5）＋ proptest |
| マスタ（製品・工程・設備） | サーバ単一権威。CRDT 不要 | 単一書込み口 + Idempotency-Key |
| 作業フロー定義 | サーバ単一権威＋端末ローカルキャッシュ | バージョン番号でキャッシュ整合 |
| ユーザー設定 | LWW-Register（device_id, lamport_ts） | Lamport timestamp の単調性検証（INV-08）|
| デッドレターキュー | サーバ単一権威 | 監査ログ追記不変 |

- **クロック**: 全イベントに Lamport timestamp を付与（壁時計とは独立）。壁時計（端末／サーバ UTC）は §20.2 に従い別途記録。
- **競合解決の決定性**: LWW 適用時は (lamport_ts, device_id) の lex 順で必ず決定。`device_id` は端末初回登録時に発行する UUID v7。
- **デッドレター遷移条件**: 24h 経過した未解決競合／形式モデル外の例外。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| 全エンティティを LWW で統一 | 単純 | 作業実績の消失リスク。追記のみが必要 |
| OT（Operational Transformation） | リアルタイム編集向けの伝統的方式 | 構造化編集に向くが、同期境界が広く性能とのトレードオフが本用途と不整合 |
| 集中ロック（CP 系） | 単一権威で同期 | オフライン継続不能（§10.6 §6.4 受入観点と不整合） |
| ベクタークロック | 因果関係を完全保持 | 実装複雑度が高い／決定性確保のためには Lamport で十分 |
| HLB（Hybrid Logical Clock） | 物理＋論理 | 必要性なし（壁時計は §20.2 で別記録） |

## Consequences

- **正の帰結**: 作業実績の消失ゼロが TLA+ で機械検証可能（INV-01）。LWW の決定性が一意に定まる（INV-02）。学術文献で根拠が明確（§4.9 圧倒候補機能）。
- **負の帰結**: 端末 UUID v7 発行・Lamport ts の永続化が必要。実装コストが集中ロック方式より高い。
- **影響範囲**: §10.6.1（同期形式モデル）、§3.4.1（TLA+ ファイル整備）、§13.2（proptest）、§31 SLI-06／SLO-06／SLO-07（同期競合 SLO）、§27 F-002／§29 R-016。

## Type 1 撤退条件

- TLA+／proptest で `Inv_NoEventLoss` が継続的に違反する（モデルバグでなく実装バグでない場合）→ より強いモデル（OT／CRDT for sequence）への移行を検討。
- Lamport timestamp の単調性が破綻する OS／ランタイムが普及（理論上は不発生）。
- 撤退時の代替: ADR で再評価し、ベクタークロック＋OT の組合せを検討。

## §24.2 出所表への追記

- 追記済: Yes（§24.2「同期形式モデル（CRDT 種別＋Lamport） → §10.6.1」行）

## References

- ロードマップ §10.6 §10.6.1 §10.6.2 §3.4.1 §3.4.2
- Shapiro, M. et al. *Conflict-Free Replicated Data Types*. INRIA Research Report, 2011.
- Lamport, L. *Time, Clocks, and the Ordering of Events in a Distributed System*. CACM, 1978.
- 関連 ADR: ADR-0002（形式化 notation）
- 関連 FMEA: F-002
- 関連リスク: R-016
- 関連 SLI: SLI-06／SLO-06
