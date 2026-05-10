# CPN 同期モデル概要

> 対応 §: ロードマップ §3.4.1 §10.6 §10.6.1 §10.6.2
> 対象: 並行・同期・順序情報の流れ
> notation: 着色 Petri net（CPN, Jensen & Kristensen 2009）
> 検証ツール（将来）: CPN Tools／Access/CPN

`*.cpn` ファイル（CPN Tools のバイナリ／XML 形式）を直接コミットする前段階として、本書で **モデル構成の意図** をテキストで記述する。CPN Tools が整備された後、本書を `cpn-sync.cpn` の README として保持する。

## 1. モデルの目的

§10.6 同期競合戦略を、**端末複数台 × サーバ 1 台 × オフライン期間 × 競合発生** の組み合わせ下で正しく動作させることを CPN で表現する。本モデルで証明したい性質は §10.6.2 受入観点と完全一致する。

## 2. プレース（Place）

| プレース名 | カラー | 意味 |
| --- | --- | --- |
| `terminal_event_buffer` | `Event = {device_id, lamport_ts, payload}` | 端末側で発生した未送信イベント |
| `terminal_send_queue` | `Event` | 同期送信キュー |
| `network_link` | `Event` | ネットワーク経路上のイベント（遅延・喪失モデル化） |
| `server_inbox` | `Event` | サーバ受領済み未処理イベント |
| `server_g_set` | `Set[Event]` | G-Set（追記のみ、§10.6.1 作業実績） |
| `server_lww_register` | `Map[key, (lamport_ts, device_id, value)]` | LWW-Register（§10.6.1 ユーザー設定） |
| `dead_letter_queue` | `Event` | デッドレター（§10.3.1 §10.6.1） |
| `master_data` | `MasterRecord` | サーバ単一権威マスタ |
| `flow_definition` | `FlowVersion` | サーバ単一権威フロー定義 |

## 3. 遷移（Transition）

| 遷移名 | 入力プレース | 出力プレース | ガード |
| --- | --- | --- | --- |
| `produce_event` | （外部入力） | `terminal_event_buffer` | 端末で作業実績または設定変更 |
| `enqueue_send` | `terminal_event_buffer` | `terminal_send_queue` | バッファ非空 |
| `transmit` | `terminal_send_queue` | `network_link` | ネットワーク到達可 |
| `network_drop` | `network_link` | （消滅） | 確率的喪失（カオス演習相当、§13.4.2） |
| `network_delay` | `network_link` | `network_link` | 遅延注入 |
| `receive` | `network_link` | `server_inbox` | サーバオンライン |
| `merge_g_set` | `server_inbox` | `server_g_set` | event.kind == "record"（追記のみ） |
| `merge_lww` | `server_inbox` | `server_lww_register` | event.kind == "user_setting" かつ (lamport_ts, device_id) lex 順で大きい |
| `dead_letter_promotion` | `server_inbox` | `dead_letter_queue` | event.age ≥ 24h かつ未解決 |
| `master_write` | `master_data` | `master_data` | サーバ単一書込口 |
| `flow_write` | `flow_definition` | `flow_definition` | サーバ単一書込口 |

## 4. 不変式（CPN プロパティ）

CPN Tools の Place Invariant／State Space Analysis で次を確認する。

- `Inv_NoEventLoss`: `record` 種別のイベントは `terminal_event_buffer` ＋ `terminal_send_queue` ＋ `network_link` ＋ `server_inbox` ＋ `server_g_set` ＋ `dead_letter_queue` の合計多重度が単調非減少（喪失しない）。
- `Inv_LWWDeterministic`: 任意の終端到達状態において、同一 key に対する `server_lww_register` の値は (lamport_ts, device_id) で決定的に定まる。
- `Inv_BoundedDLQ`: `dead_letter_queue` は無限増殖しない（DLQ 遷移率 ≤ 0.1%、§31.2 SLO-07 と整合）。
- `Inv_ServerAuthorityMaster`: `master_data`／`flow_definition` は端末からの直接書込みを許さない。

## 5. CPN Tools での検証手順（将来）

```
1. cpn-sync.cpn を CPN Tools で開く
2. Tools → State Space → Calculate State Space
3. Query → 上記 Inv_* を P/T 不変式として登録
4. CTL: AG (record_in_system_count >= record_produced_count)
5. 結果 OK ならコミット。NG なら §22.4 是正フロー
```

## 6. 関連ドキュメント

- 不変式詳細: [`invariants.md`](./invariants.md)
- TLA+ 仕様: [`sync.tla`](./sync.tla)
- HSM（並列領域として接続）: [`hsm-task.puml`](./hsm-task.puml)

## 7. 受入観点

- §3.1.1 の 11 構成要素のうち「状態」「例外」「時間」「入出力」が本モデル上に表現されている。
- §10.6.2 で要求された不変式が CPN プロパティとして登録され、半期サイクルで再検証可能。
- §27 FMEA F-002／§29 R-016 と本モデルが双方向参照されている。
