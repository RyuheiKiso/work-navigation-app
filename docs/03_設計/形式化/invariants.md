# 不変式集

> 対応 §: ロードマップ §3.4.2 §10.6.2 §27 F-002 §29 R-016 §31.1 SLI-06 §31.2 SLO-06
> 対象読者: 形式検証担当、CI 整備者
> 改訂サイクル: §22.1 半期サイクル

形式化の検証結果として保証する不変式を本書で一覧する。各不変式は HSM／CPN／TLA+ のいずれか、または複数で表現される。CI 整備時には本書の不変式が **すべて緑** であることをリリースブロッカー条件とする。

## 1. 一覧

| ID | 名称 | 表現 | 紐付き |
| --- | --- | --- | --- |
| INV-01 | `Inv_NoEventLoss` | TLA+（[`sync.tla`](./sync.tla)）／CPN（[`cpn-sync.md`](./cpn-sync.md)） | §10.6.2／§27 F-002／§29 R-016／§31 SLI-06 |
| INV-02 | `Inv_LWWDeterministic` | TLA+／CPN | §10.6.2／§27 F-002 |
| INV-03 | `Inv_BoundedDLQ` | TLA+／CPN | §10.6.2／§31.2 SLO-07 |
| INV-04 | `Inv_ServerAuthorityMaster` | CPN | §10.3.6 RACI／§10.6.1 |
| INV-05 | `Inv_TaskHSMReachability` | sismic（HSM 到達可能性） | §10.1／§3.4.2 |
| INV-06 | `Inv_NoDeadlockTask` | sismic（デッドロック検出） | §10.1／§3.4.2 |
| INV-07 | `Inv_AuditAppendOnly` | TLA+（追加モジュールで） | §11.4.1／§10.6 |
| INV-08 | `Inv_LamportMonotonic` | TLA+（[`sync.tla`](./sync.tla) 内 ProduceEvent ガード） | §10.6.1 |

## 2. 不変式の詳細

### INV-01: `Inv_NoEventLoss`

「生産された `record` 種別イベントは、端末バッファ／送信キュー／ネットワーク／サーバ受信箱／G-Set／デッドレターのいずれかに必ず存在する」

- **検証手段**: TLC（[`sync.tla`](./sync.tla)）。
- **失敗時の意味**: 同期競合下で実績消失（§29 R-016／§27 F-002）。
- **リリースブロッカー**: §27 F-002 AP=H により Yes。

### INV-02: `Inv_LWWDeterministic`

「ユーザー設定の LWW-Register は、受領済みイベントのうち (lamport_ts, device_id) lex 順で最大の値に等しい」

- **検証手段**: TLC／CPN。
- **失敗時の意味**: 端末ごとに異なる値が観測され、行動の不一致が発生する。
- **リリースブロッカー**: Yes（§10.6.1 決定性要件）。

### INV-03: `Inv_BoundedDLQ`

「デッドレターキューの大きさは無限増殖しない」

- **検証手段**: TLC（バウンド制約）／本番では §31.2 SLO-07（DLQ 遷移率 ≤ 0.1%）で運用上の境界を維持。
- **失敗時の意味**: 監査負荷の暴騰、再投入の遅延。
- **リリースブロッカー**: No（運用 SLO で吸収）。

### INV-04: `Inv_ServerAuthorityMaster`

「マスタ（製品・工程・設備）と作業フロー定義は、サーバを単一権威として書込みされ、端末からの直接書込みは存在しない」

- **検証手段**: CPN プレース不変式。
- **失敗時の意味**: §10.3.6 RACI の責任二重化、データ整合性破綻。
- **リリースブロッカー**: Yes。

### INV-05: `Inv_TaskHSMReachability`

「[`hsm-task.puml`](./hsm-task.puml) に定義された全状態が初期状態から到達可能である」

- **検証手段**: sismic／PlantUML 検査。
- **失敗時の意味**: 不到達ノードの存在は §10.2.1 受入観点違反。

### INV-06: `Inv_NoDeadlockTask`

「[`hsm-task.puml`](./hsm-task.puml) はデッドロックを持たない（任意状態から終端へ到達可能）」

- **検証手段**: sismic デッドロック検出。
- **失敗時の意味**: 中断・再開の不能、業務停止。

### INV-07: `Inv_AuditAppendOnly`

「監査ログは追記のみであり、削除・編集により多重度が減少しない」

- **検証手段**: TLA+ 追加モジュール（本セッションでは未作成）。
- **失敗時の意味**: §11.4.1 改ざん耐性要件違反、規制適合不能。

### INV-08: `Inv_LamportMonotonic`

「各端末の Lamport timestamp は単調増加する」

- **検証手段**: TLA+（`ProduceEvent` のガード `clock[d] < MaxClock`、`nextTs == clock[d] + 1`）。
- **失敗時の意味**: LWW 決定性の前提が崩れる。

## 3. 検証実行手順（将来 CI 統合）

```bash
# TLA+ モデル検査
java -cp tla2tools.jar tlc2.TLC docs/03_設計/形式化/sync.tla \
  -config docs/03_設計/形式化/sync.cfg \
  -workers auto

# CPN（Access/CPN または CPN Tools の CLI モード）
access-cpn docs/03_設計/形式化/cpn-sync.cpn --query invariants.q

# HSM（sismic）
sismic-bdd docs/03_設計/形式化/hsm-task.yaml \
  --reachability --no-deadlock
```

## 4. 受入観点（§3.4.2／§10.6.2）

- INV-01〜INV-08 が CI で常時緑であること（コード初期実装後）。
- INV-01／INV-02／INV-03 が proptest（性質ベーステスト）でも実装層から検証されていること。
- 失敗時は §22.4 是正フロー起動。
- §27 FMEA／§29 リスク登録簿との双方向リンクが本書で維持されていること。
