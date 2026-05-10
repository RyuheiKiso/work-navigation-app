# ADR-0011: 至高品質バックログ

> 提案日: 2026-05-10
> 採用日: -
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §13（テスト）§24（意思決定）§28（観測性）

## Status

Proposed

## Type

Type 2（可逆／§30.1）

## Context

リポジトリ監査により、ロードマップ項目はチェックされているが、**実装コード（13,754 行）に対しドキュメント（16,610 行）が上回り、ガバナンス書類が単独開発者に対して過剰**である状態を確認した。さらに以下の品質劣化要因がある:

- CLAUDE.md の「各行に日本語コメント」規則により、`// 状態を Running に遷移` のような自明言い換えコメントが大量に発生し、可読性を逆に下げている。
- 500 行ファイル制限に違反するファイルが 3 件存在し、CI で強制されていない (auth.rs 531, flow-canvas.tsx 540, navigation-shell.tsx 527)。
- ドメイン層が浅く（backend は 8 ファイル）、製造ナビとして本来必要な ProductionOrder / Process / BOM / Equipment / QualityCriteria 集約が欠落。
- 結合テスト・E2E が無く、「login → task 作成 → 完了 → 監査」が通る証明が無い。
- i18n が 13 言語実装されているが、利用者は存在しない（先回り抽象化）。
- `docs/02_設計/` と `docs/03_設計/` の両方が「設計」と命名され、情報構造が崩れている。
- 直近コミットが「新しい機能を追加」3 連発で、git log が将来の保守に役立たない。

`docs/CLAUDE.md` に「個人開発なので MVP も phase も定めない」と明記されているため、本 ADR は**フェーズ分けせず、依存関係付き優先度バックログ**として品質改善項目を記録する。各項目は他の項目の前提になるかどうかで順序付けされる。

## Decision

以下の品質改善バックログを採用する。各項目は独立に着手可能だが、**先行項目（前提）が解決されていないと後続項目の効果が薄れる**ことを依存関係として明示する。

### A. 規約と土台 (他のすべての前提)

- **A1. CLAUDE.md コメント方針改訂** ✅ 本 ADR と同 PR で適用済。WHY のみ、自明 WHAT 禁止。
- **A2. 自明コメント剥離**: 既存コードから「直下行の言い換え」コメントを除去。AST/heuristic で候補抽出 → 人手レビュー。
- **A3. 500 行上限の CI 強制**: `scripts/check-file-length.sh` を CI に組み込み。既存違反 3 件を分割。
- **A4. Conventional Commits 強制**: commitlint + pre-push hook。`新しい機能を追加` 系を禁止。
- **A5. ドキュメント情報構造再編**: `docs/02_設計/` + `docs/03_設計/` を `docs/design/` に統合し意味のあるサブフォルダ（`glossary/`, `formal-models/`, `ui-audit/`, `design-tokens/`）に再配置。コード/ドキュメント参照を更新。
- **A6. 不要先回り抽象の凍結**: i18n 11 言語ファイル、ガバナンス一式（CODE_OF_CONDUCT/MAINTAINERS/SECURITY/CONTRIBUTING）を `archive/` 配下に退避。ja+en と LICENSE のみを active に保つ。実需要発生時に復活。

### B. 自動強制ゲート (A1〜A4 完了後)

- **B1. Rust lint 厳格化**: `clippy::pedantic` + `clippy::nursery` を deny。CI で `--deny warnings`。
- **B2. TS lint/型 厳格化**: `tsconfig.strict`、`noUncheckedIndexedAccess`、`exactOptionalPropertyTypes`。
- **B3. カバレッジ閾値**: `cargo-llvm-cov` で domain ≥ 90%、usecase ≥ 80%。
- **B4. Mutation testing**: `cargo-mutants` で domain 層 mutation kill ≥ 80%。
- **B5. 依存監査**: `cargo-deny` (license/advisories/duplicate)、`pnpm audit` を CI に。

### C. ドメイン深化 (A 完了後 / B と並行可)

- **C1. `ProductionOrder` 集約**: 受注→生産指示。状態 Released/InProgress/Done/Cancelled。不変条件: 数量 > 0、Cancelled 終端。
- **C2. `Process` 集約**: 順序付き Step、必要設備、所要標準時間。Step は DAG（循環不可）。
- **C3. `BillOfMaterials` 集約**: 親子関係の有限木、員数 > 0、自己参照禁止。
- **C4. `Equipment` 集約**: 稼働状態、保全期日、能力。稼働中保全禁止。
- **C5. `QualityCriteria` 値オブジェクト**: 数値範囲/ブール/写真証跡の代数和。評価関数 total。
- **C6. Refinement types 拡張**: `parse, don't validate` パターンを全 ID 型に。
- **C7. Domain events**: `enum DomainEvent` をすべての mutation API から返す。
- **C8. Property-based test**: `proptest` で「任意の遷移列で Lamport 単調」など不変条件を検証。
- **C9. Repository 契約テスト**: in-memory / Postgres 両方で同一 trait 試験を実行。

### D. 製品としての真実性 (C と並行可)

- **D1. E2E HTTP テスト**: `testcontainers` + `axum-test` で `login → create order → start task → complete with evidence → audit query` を 1 本通す。
- **D2. Frontend E2E**: Playwright で Tauri ビルド済み bundle に対して同シナリオ。
- **D3. 観測性**: `tracing` + OpenTelemetry exporter、構造化ログ、Prometheus metrics、`request_id` 貫通。
- **D4. レイテンシ予算**: criterion で domain hot path < 100µs、HTTP P99 < 80ms (oha) を CI 計測。
- **D5. DB 健全性**: migration の rollback パス整備、tx isolation 明示、`sqlx prepare` を CI で diff。
- **D6. Addon サンドボックス**: capability token 設計を ADR 化、WASI 制限を integration test で確認。
- **D7. Supply chain**: `cargo-vet` 導入、SLSA Level 2 相当の build provenance。

### E. 持続運用 (恒久)

- **E1. ADR ライフサイクル運用**: proposed/accepted/deprecated を機械的に検証。
- **E2. 不要抽象の復活基準**: i18n 復活は「実ユーザーの当該言語要請」、ガバナンス復活は「外部コントリビュータ出現」を契機とする。
- **E3. モデル検査** (発展): TLA+ で Task HSM と Lamport 不変条件を機械検査（ROI が見えてから）。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| Alt 1: 何もしない | 既存ロードマップの項目消化のみ続ける | 監査で品質劣化要因が判明しており、放置は技術的負債を加速する。「至高」の方針に反する。 |
| Alt 2: 全面書き直し | リポジトリを破棄し再構築 | コードの基盤（バックエンド 12.5K 行、レイヤ分離、テスト）は既に堅実であり、破棄は過剰反応。改善で十分。 |
| Alt 3: フェーズ分割 | Phase 0〜4 で段階的に実施 | `docs/CLAUDE.md` の「個人開発なので phase を定めない」規約に反する。本 ADR ではバックログ + 依存関係の形を採る。 |

## Consequences

- **正の帰結**:
  - 自動ゲート整備により、以後のコードが既存と同じノイズ・違反を再生産しなくなる。
  - ドメイン深化により「製造ナビ」と名乗れる中身を持つ。
  - E2E と観測性により、ロードマップ完了 ≠ 動く製品の乖離が解消される。
- **負の帰結**:
  - A5/A6（ドキュメント再編・不要抽象凍結）で git mv が大量に発生し、git blame が一段階見にくくなる（`--follow` で追跡可）。
  - 一時的に lint/coverage 閾値違反で CI が落ちる期間が発生する。
- **影響範囲**: ロードマップ §13, §24, §28、CLAUDE.md、CI 全般、`services/backend/crates/domain/`、`apps/`、`docs/`。

## References

- ロードマップ §13（テスト戦略）§24（意思決定記録）§28（観測性とリリースエンジニアリング）
- 関連 ADR: ADR-0002（形式化記法）, ADR-0006（技術スタック）, ADR-0009（PR サイズ例外）
- CLAUDE.md（リポジトリルート）
- docs/CLAUDE.md（個人開発・phase 不採用の規約）
