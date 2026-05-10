# ADR-0006: 技術スタック: Tauri＋React／Rust（tokio）／PostgreSQL／SQLite

> 提案日: 2026-05-09
> 採用日: 2026-05-09
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §7 §8 §11.7 §29 R-007

## Status

Accepted

## Type

Type 1（基盤フレームワークの変更はコード全面リライトを伴うため不可逆）

## Context

- メモ L23-27 が技術スタックを明示している:
  - 端末: tauri(react)
  - 端末ローカル DB: SQLite
  - 設定 UI: react
  - バックエンド: rust(tokio)
  - サーバ DB: PostgreSQL
- §11.7「グリーンソフトウェア」と §4.9「完全 OSS／TCO 優位」の両立、§17 アドオン WASM サンドボックスとの親和性、§14 配布の容易性が同時に求められる。
- 個人開発の維持コスト（§19.5）から複数言語ランタイムの並存は避けたい。

## Decision

| レイヤ | 採用技術 |
| --- | --- |
| 端末アプリ | Tauri + React |
| 端末ローカル DB | SQLite（SQLCipher 暗号化、ADR-0004） |
| 設定 Web UI | React |
| バックエンド | Rust（tokio） |
| サーバ DB | PostgreSQL |
| コンテナ | Docker |

技術選定理由・代替案・撤退条件は本 ADR で版管理する。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| Electron + Node.js | クロスプラットフォーム | バイナリサイズ大／RSS 大／§11.7.2 Rust の効率性と不整合 |
| Flutter | Dart 単一スタック | iOS／Android が主／メモ L13 の Android／Windows と部分整合 |
| Java Spring + Android Java | エンタープライズ慣行 | RSS 大／§11.7.2 と不整合／メモ L26 と不整合 |
| Go + React | Go の保守容易さ | tokio の async ／ Rust の所有権モデルが §10.6 同期形式モデルと親和 |
| .NET MAUI + ASP.NET | Windows 親和 | Apache-2.0 ライセンスとのライセンス整合に注意／§4.4 OSS TCO の主張弱化 |

## Consequences

- **正の帰結**:
  - Tauri により Android＋Windows で統一 UX（§4.4 対応端末 ◎）。
  - Rust で型安全・メモリ安全・性能（§11.7.2 RSS 30〜70% 低）。
  - tokio の async モデルが §10.6 同期競合の表現と親和。
  - WASM／Wasmtime（§17）と Rust の親和性（`wasm32-wasi`）。
- **負の帰結**:
  - Rust の学習曲線（外部貢献者の参入障壁、§19.4 で教育コンテンツ整備）。
  - Tauri は Electron に比べエコシステムが若い（プラグインの選択肢が少ない）。
- **影響範囲**: §7／§8（プロダクト構成）、§9.4（コーディング規律）、§11.7（サステナビリティ）、§13.2（テストツール）、§14（配布）、§17（アドオン基盤）、§29 R-007（重大破壊的変更）。

## Type 1 撤退条件

- Tauri／Rust／React のいずれかが OSS としてメンテナンス停止（§29 R-007）。
- 上記いずれかが Apache-2.0 と非互換なライセンスへ後退した場合（§19.5.3 と整合）。
- 個人開発の維持コストが構造的に成立しなくなった場合（§22.3 撤退条件）。
- 撤退時の代替: ADR で再評価し、各レイヤを独立に置換可能な構成（例: Rust → Go、React → SolidJS）を再判断。

## §24.2 出所表への追記

- 追記済: Yes（§24.2 「サポート OS 下限」「ディスプレイ最小」等の周辺項目に既に記載）

## References

- ロードマップ §7 §8 §11.7 §17 §29
- Tauri: <https://tauri.app/>
- Rust／tokio: <https://tokio.rs/>
- PostgreSQL: <https://www.postgresql.org/>
- SQLite／SQLCipher: ADR-0004
- 関連 ADR: ADR-0004（端末暗号化）／ADR-0005（アドオン API）
- 関連リスク: R-007
