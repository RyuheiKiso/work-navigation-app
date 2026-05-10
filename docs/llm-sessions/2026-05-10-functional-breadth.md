# LLM セッション記録: 2026-05-10 機能拡充（auth／idempotency／追加アドオン／業界テンプレ）

> 対応 §: ロードマップ §18.5.1 §18.5.2 §22.1
> 対象読者: メンテナ、§22.1 サイクルでモデル世代交代の影響評価を行うレビュア
> モデル: Claude Opus 4.7（1M context）／identifier: `claude-opus-4-7[1m]`

§18.5.1「LLM 出力の記録規約」に従い、本セッションのプロンプト要旨と意思決定経路を保存する。
PII を含まず、公開許容範囲のみを記録する。

## 1. セッションサマリ

| 項目 | 値 |
| --- | --- |
| 日付 | 2026-05-10（連続 3 セッション目） |
| モデル | Claude Opus 4.7（1M context） |
| 主たる目的 | コード骨格に対する **機能拡充**（auth／idempotency／追加サンプルアドオン／業界テンプレ／追加 scripts／PR-size 例外 ADR） |
| 入力 | 既存リポジトリ全体（前 2 セッション成果物を含む）／`docs/01_企画/ロードマップ.md` |
| 出力 | 認証ドメイン（domain/auth.rs／production_order.rs／usecase/login.rs／usecase/receive_order.rs／adapter/argon2_hasher.rs／adapter/memory_idempotency_store.rs）／追加サンプルアドオン 2 件（slack-notify／opc-ua-bridge）／業界テンプレ（automotive 3 ファイル）／scripts 3 種（build-tokens／build-diagrams／competitor-watch）／ADR-0009 |

## 2. 主要な意思決定経路

| 決定 | 採用理由 | 却下案 | 関連 § |
| --- | --- | --- | --- |
| 機能拡充の優先順位を §27 FMEA AP=H 駆動で決定 | F-005／F-006／F-008 をリリースブロッカーとして優先 | 受入観点リスト全網羅／競合スコア駆動 | §27 §29 |
| Argon2id を `argon2` crate（PHC 標準）で実装 | RFC 9106 準拠／OS Keystore 連携と直交 | bcrypt／scrypt／自前 SHA-256 | §10.5.1 §11.4.2 |
| Idempotency は trait 抽象 + メモリ実装 + PostgreSQL 実装は次セッション | 24h 窓ロジックは trait と実装で分離 | PostgreSQL 直接実装で時間延長 | §10.3.1 §27 F-005 |
| サンプルアドオンは Slack（Notify）／OPC UA（外部接続）の 2 系統で §17.7「最低 3 種」を完成 | 既存 hello-step と合わせて代表ユースケースを網羅 | Microsoft Teams／Modbus／物理ランプ | §17.7 §19.4.2 |
| 業界テンプレは自動車のみ（IATF 16949 を最初に）。医薬／食品／電子は次セッション | IATF 16949 §8.5.1 タクト管理が最も基礎的 | 4 業界並行整備 | §10.2.1 §3.1.5.4 §12 |
| ADR-0009 は Type 2（可逆）。永久ルール変更ではなく初期投入限定の例外 | レビュアビリティ研究（Bacchelli & Bird 2013）と整合 | Type 1（永久ルール撤廃）／暗黙運用 | §9.4 §30 |

## 3. 沈黙の妥協（§2.2）チェック

| 観点 | 結果 | コメント |
| --- | --- | --- |
| 決定の出所が明記されているか | Yes | 各 ADR ／本書で対応 § を明記。新規独自決定（ADR-0009）は §24.2 追記不要（Type 2） |
| 受入観点が記載されているか | Yes | 各テスト／業界テンプレ末尾／ADR の Consequences で網羅 |
| §24.2 への追記が必要か | No | 本セッションの決定は ADR-0006（技術スタック）／ADR-0007（既定認証）／ADR-0005（API surface v1）の枠内に収まる |

## 4. ハルシネーション対策（§18.5.1）

- 数値（Argon2id PHC 上限 200 文字／IdempotencyKey 上限 128 文字／24h 窓）はロードマップ §21 脚注または NIST SP 800-63B／RFC 9106 から引用。
- argon2 crate の API（`Argon2::default()`／`hash_password`／`verify_password`）は実装言語の `argon2` 0.5 系の公開仕様に準拠。
- OPC UA の擬似実装は将来 `opcua` crate を組み込む前提のスタブ。実通信を本セッションで模倣しない。

## 5. 経年劣化対策（§18.5.2）

- argon2／sqlx／axum／tokio／tauri などの依存バージョンは `Cargo.toml` に明示し、§22.1 半期サイクルで再評価する。
- Slack Webhook URL の取得 API（`getConfig("slack.webhook.url")`）は §17.3 ConfigRead capability の規約に従う。Slack 側仕様変更には影響されない（本アプリ → ホスト → 外部の経路）。

## 6. 未着手（次セッション以降）

- F-008（暗号鍵漏洩）対策の実装: SQLCipher 統合 + OS Keystore（Android）／DPAPI（Windows） 連携
- F-002（同期競合決定性）の実装: Lamport クロック付き G-Set 実装と PostgreSQL 永続化
- F-004（アドオン暴走）の Wasmtime 統合: addon ホストランタイム（services/backend/crates/adapter または専用 crate）
- 業界テンプレ追加: 医薬／食品／電子
- 多言語化（§11.3.1）: ロケールファイル整備（ja／en 初版以降）

## 7. 受入観点（§18.5.3）

- 本書が `docs/llm-sessions/` に保存されていること（達成）
- 直近 30 日で `docs/llm-sessions/` の更新が 2 件以上（前回 2026-05-10-docs-bootstrap、本書）
- ADR-0009 で「規律例外」が出所付きで記録されていること（達成）
- 連続 3 セッションの累積成果物が CHANGELOG `[Unreleased]` に集約されていること（Phase F でこれから更新）
