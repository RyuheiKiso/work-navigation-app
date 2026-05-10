# ADR-0005: アドオン API surface v1（11 領域）

> 提案日: 2026-05-09
> 採用日: 2026-05-09
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §17 §17.3 §17.4 §17.5 §32 §27 F-004 §29 R-008

## Status

Accepted

## Type

Type 1（公開 API は破壊的変更時に 6 ヶ月の移行猶予を伴う／§17.3 §32.4。設計時の領域確定は不可逆）

## Context

- メモ L65「導入企業のシステム開発部門がアドオン機能を開発できるようにアドオンもサポートする」に従い、§17 でアドオン拡張の境界を定義する。
- §4.4「拡張性・アドオン ◎」の根拠を、API surface の範囲・capability・サンドボックス制約の 3 点で固定する必要がある。
- 最初から API 領域を絞らないと、捕えどころのない巨大 surface になり、§32 互換性・廃止予告の運用が破綻する。

## Decision

v1 API surface を **11 領域** に確定する（§17.3 整合）。

| 領域 | 提供 API（抜粋） | 必須 capability |
| --- | --- | --- |
| 作業情報読取 | `getCurrentTask()`／`listSteps()`／`getContext()` | `task.read` |
| 作業実績書込 | `appendRecord()`／`markStep(id, evidence)` | `task.write` |
| メディア | `attachMedia(taskId, mediaRef)`／`readMedia(ref)` | `media.write`／`media.read` |
| HTTP アウトバウンド | `fetch(url, options)` ホワイトリスト URL のみ | `net.outbound:<host>` |
| ストレージ KV | `kv.get(key)`／`kv.set(key, value)` 名前空間別 | `storage:<namespace>` |
| 通知 | `notify(channel, message)` | `notify:<channel>` |
| UI 拡張 | `registerPanel(slot, component)` | `ui.extend:<slot>` |
| ロギング | `log(level, message)` | 既定許可 |
| 設定 | `getConfig(key)` 公開設定のみ | `config.read` |
| 暗号 | `crypto.sign(payload)`／`crypto.verify(...)` | `crypto.sign` |
| 時刻 | `now()`（サーバ時刻同期済み） | 既定許可 |

- 破壊的変更は MAJOR バンプ＋ 6 ヶ月の移行猶予（§19.3／§17.3）。SemVer 準拠。
- `addon-sdk/` 配下で型定義（`addon.d.ts`／`addon.rs`）を版管理。
- 権限モデルは capability ベース（§17.4）／サンドボックスは Wasmtime（§17.5）。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| API surface を最小化（5 領域以下） | 保守コスト最小 | OPC UA／MQTT／BI／通知の代表ユースケースをカバーできない |
| API surface を最大化（プラグインに任意の API） | 自由度最大 | 互換性・サンドボックス制約が運用破綻 |
| プロセスベース IPC（gRPC） | 言語非依存 | 起動コスト高／§14 セットアップ容易性に逆行 |
| Tauri Native Plugin（OS ネイティブ） | 性能最大 | サンドボックス困難／capability 制御不能 |

## Consequences

- **正の帰結**: アドオン開発者が API 領域を予測しやすい。capability ベース権限により最小権限原則を構造的に強制（§17.4）。WASM サンドボックスにより 0-day 影響範囲を限定（§17.5）。
- **負の帰結**: 11 領域全体のテスト・互換維持コスト。§19.3 LTS 18 ヶ月に従い、長期サポートが必要。
- **影響範囲**: §17.3（API surface）、§17.4（権限モデル）、§17.5（サンドボックス）、§32.3（互換性レーン）、§32.4（廃止予告）、§13.1 セキュリティテスト、§27 F-004。

## Type 1 撤退条件

- 11 領域のうち、利用率が 1 期で 0% の領域があれば §22.4 是正フローで領域削除を検討（§32.4 廃止予告 6 ヶ月）。
- 新たな代表ユースケース（例: 視覚 AI／音声 LLM）が普及し、現行 API では表現不能な場合、追加領域を ADR で起票。
- 撤退時の代替: 削除対象領域の機能はコア API への取り込みまたはアドオン外の連携（OPC UA 等）に委譲する。

## §24.2 出所表への追記

- 追記済: Yes（§24.2「アドオン API surface v1（11 領域） → §17.3」行）

## References

- ロードマップ §17 §17.3 §17.4 §17.5 §32
- WASI / Wasmtime: <https://wasmtime.dev/>
- 関連 ADR: ADR-0006（技術スタック）
- 関連 FMEA: F-004
- 関連リスク: R-008
