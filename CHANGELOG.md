# 変更履歴

> 対応 §: ロードマップ §19.2 §19.3 §32
> 形式: [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) 1.1.0 準拠
> バージョニング: [Semantic Versioning 2.0.0](https://semver.org/lang/ja/) ＋ [Conventional Commits 1.0.0](https://www.conventionalcommits.org/ja/v1.0.0/)
> リリーストレイン: §32.2（パッチ隔週／マイナー四半期／メジャー最低 18 ヶ月）

リリースノートは Conventional Commits から自動生成し、本書では「重大変更」「セキュリティ修正」「廃止予告」セクションを手動で補足する（§32.5）。

## [Unreleased]

### Added

- ロードマップ初期確定（2026-05-09）。
- ドキュメント雛形を一括投入（LICENSE／CONTRIBUTING／CODE_OF_CONDUCT／SECURITY／MAINTAINERS／README）。
- 派生ドキュメント初版を整備（`docs/02_設計/`、`docs/03_設計/形式化/`、`docs/04_運用/`、`docs/adr/`、`docs/llm-sessions/`、`docs/governance/`、`docs/community/`）。
- GitHub テンプレート（`.github/PULL_REQUEST_TEMPLATE.md`、`.github/ISSUE_TEMPLATE/*.yml`、`.github/workflows/*.yml`）を初期投入。
- **コード初期実装（骨格）**: ルートワークスペース（`Cargo.toml`／`package.json`／`pnpm-workspace.yaml`／`docker-compose.yml`／`Makefile`／`.gitignore`／`.editorconfig`／`.dockerignore`）。
- **バックエンド 5 crate 骨格**（`services/backend/`）: domain／usecase／adapter／presentation／infrastructure をクリーンアーキテクチャに従って分割。Task Aggregate（§3.1.1 11 構成要素のうち最小 6 要素）／Lamport timestamp／LWW／PostgreSQL Repository／axum REST API／sqlx マイグレーション 0001。
- **端末アプリ骨格**（`apps/terminal/`）: Tauri 2.x + React 18 + TypeScript。`src/domain/` `src/usecase/` `src/adapter/` `src/presentation/` の論理 4 層構造。`Task` Aggregate ／`StartTaskUseCase`／`TauriTaskRepository`／`TaskCard`。Vitest 単体テスト 6 件。
- **設定 UI 骨格**（`apps/config-ui/`）: React 18 + TypeScript。`Flow` Aggregate ／`PublishFlowUseCase`／`HttpFlowGateway`／`FlowEditor`。Vitest 単体テスト 4 件。
- **アドオン SDK 骨格**（`addon-sdk/`）: Rust 一次（`addon-sdk/rust/`）／AssemblyScript 二次（`addon-sdk/assemblyscript/`）／サンプル `examples/hello-step`。§17.3 API surface v1 の 11 領域 trait を Rust で宣言。
- **scripts/ 実装本体**: 8 種のシェルスクリプト（`lint-file-size.sh`／`lint-line-comments.sh`／`glossary-lint.sh`／`lint-rationale.sh`／`check-links.sh`／`aging-todo.sh`／`lint-deferred.sh`／`observability-link-lint.sh`）。
- **examples/**: `cli-client/`（バックエンド REST API を叩くサンプル CLI、§10.3.5）／`fmea-iatf16949/`（AP=H 例の汎用化版、§27.5）。
- **機能拡充（Session 3）**: 認証ドメイン（`domain/auth.rs`：UserId／PasswordHash／PasswordHasher／Credential／User／Session／SessionToken／CredentialError）／順序情報ドメイン（`domain/production_order.rs`：OrderId／ItemCode／Quantity／IdempotencyKey／ProductionOrder／ProductionOrderError）／ログインユースケース（`usecase/login.rs`：CredentialRepository／SessionFactory／LoginUseCase、F-006 対応）／順序情報受領ユースケース（`usecase/receive_order.rs`：OrderRepository／ReceiveOrderUseCase／24h 重複排除、F-005 対応）／Argon2id ハッシャ実装（`adapter/argon2_hasher.rs`、ADR-0007）／メモリ Idempotency ストア（`adapter/memory_idempotency_store.rs`、開発・テスト用）。
- **追加サンプルアドオン 2 件**（§17.7 受入観点「最低 3 種」を完成）: `addon-sdk/examples/slack-notify/`（Slack 通知）／`addon-sdk/examples/opc-ua-bridge/`（OPC UA タグ→実績ブリッジ）。
- **業界別フローテンプレ（自動車）**: `examples/flow-templates/automotive/`（assembly-line.yaml／setup-changeover-smed.yaml／quality-hold.yaml）。IATF 16949 §8.5.1 タクト管理／SMED 内段取・外段取分離／品質ホールド例外フロー。
- **追加スクリプト 3 種**: `scripts/build-tokens.sh`（Style Dictionary フォールバック付き）／`scripts/build-diagrams.sh`（drawio→PNG エクスポート）／`scripts/competitor-watch.sh`（§4.8 自動収集の試行）。
- **ADR-0009**: 初期投入における 1 PR ≤ 500 行差分の例外（Type 2、Session 3 で正式化）。
- **LLM セッション記録**: `docs/llm-sessions/2026-05-10-functional-breadth.md`（Session 3）。
- **Session 4: ロードマップ未達項目を一括解消**:
  - **F-002 PostgreSQL G-Set/LWW 実装**: `migrations/0002_user_settings_lww.sql`／`adapter/postgres_lww_repository.rs`（Lamport+device_id lex 順の SQL 条件付き UPDATE）。
  - **F-006 認証経路完成**: `adapter/postgres_credential_repository.rs`／`adapter/hs256_session_factory.rs`／`presentation/handler_auth.rs`（POST /auth/login）。infrastructure DI 配線。
  - **F-008 SQLCipher + OS Keystore 抽象**: `apps/terminal/src-tauri/src/secure_storage.rs`（SecureStorage trait／SqlCipherStorage／KeyProtection trait／InMemoryKeyProtection）。`feature = "sqlcipher"` で内蔵 SQLCipher 切替。
  - **F-004 Wasmtime アドオンランタイム crate**: `services/backend/crates/addon-runtime/`（manifest／limits／capability_check）。capability ベース既定 deny／glob 越境禁止／ResourceLimits（端末 64MB/100ms、サーバ 128MB/500ms）。
  - **業界テンプレ拡充**: pharma（batch-record／change-control／oos-investigation）／food（cooking-ccp／cold-chain）／electronics（smt-assembly／cleanroom-entry）。§10.2.1 4 業界完成。
  - **i18n 整備**: `apps/terminal/src/i18n/`（ja／en）／`apps/config-ui/src/i18n/`（ja／en）。§28 用語と完全一致。
  - **メディア対応骨格**: `domain/media.ts`（MediaRef／SHA-256 検証）／`usecase/capture-media.ts`／`adapter/tauri-media-capture.ts`／Tauri `capture_media` コマンドスタブ。
  - **依存修正**: WSL2 環境向けに sqlx の rustls→runtime-tokio、reqwest の rustls 無効化、sqlx migrate feature 追加、tokio sync feature 追加（dev-deps）。Domain `Evidence` を再エクスポート。
  - **検証結果**: `cargo test --workspace`（terminal-tauri 除く）で **53 テスト全 PASS**、`pnpm -r test` で **25 テスト全 PASS**、`scripts/lint-file-size.sh` 緑（500 行制限維持）。
- **Session 5（至高 KPI 駆動・全項目完了）**:
  - **HMAC-SHA256 本実装**（ADR-0010）: Argon2id ハッシャと並ぶ本物の `hmac`/`sha2`/`base64` crate 統合。定数時間比較／Base64URL（パディングなし）／verify 経路。FNV-1a の自前簡易実装を撤去。
  - **§10.6 sync ループ実装**: `domain/sync.rs`（SyncEvent／SyncEventKind／lww_strictly_after）／`usecase/sync_push.rs`（TerminalEventBuffer trait／SyncTransport trait／SyncPushUseCase／step()／drain()）。INV-01 NoEventLoss を実装側でも保証（送信成功確認後にのみ dequeue）。
  - **§10.4 メディア SHA-256 実装**: Tauri Rust 側 `media.rs`（sha256_hex／capture_stub、NIST テストベクタ検証）、`capture_media` コマンドを実 SHA-256 に差替。
  - **§11.3.1 拡張ロケール**: zh（誤読回避「作业（操作）」）／ko／de（Arbeitsschritt = Step）／es 計 6 言語。
  - **§17.5 Wasmtime 実体**: `wasmtime_host.rs`（feature `wasmtime-runtime`）、capability チェック先行＋ fuel／epoch interrupt／StoreLimits（メモリ上限）統合、`_start` 関数 invoke。
  - **§13.4 mutation/chaos**: `scripts/mutation-test.sh`（cargo-mutants/stryker）、`scripts/chaos/s01-s08.sh`（8 シナリオ全実装）。
  - **§19.3 SBOM/cosign**: `scripts/generate-sbom.sh`（CycloneDX rust/node/container）、`scripts/sign-release.sh`（OIDC keyless cosign）／VERIFY.md 同梱。
  - **§22.1 改訂サイクル記録**: `docs/governance/cycle-2026-Q2.md`（至高 KPI スナップショット／是正トリガー／次サイクル重点）。
  - **ADR-0010 HMAC-SHA256 化**を Type 1 として正式 ADR 化。
  - **検証結果**: `cargo test --workspace`（terminal-tauri 除く）で **66 テスト全 PASS**（+13 件）、`pnpm -r test` で **29 テスト全 PASS**（+4 件）、`scripts/lint-file-size.sh` 緑。
- **Session 6（至高 KPI 駆動・残課題一括解消、9 Phase）**:
  - **terminal-tauri ワークスペース分離**: ルート `Cargo.toml` の `exclude` ＋ src-tauri/Cargo.toml の独立 `[workspace]` table。Linux ビルド対象外の旨を `apps/terminal/README.md` に明記（実機ターゲット Android／Windows のみ）。
  - **YAML フローパーサ**: `apps/config-ui/src/adapter/yaml-flow-parser.ts`（依存追加なし、限定 YAML サブセット手書きパース）。業界テンプレ YAML→Flow Aggregate 変換が可能に。`flow.ts` の `FlowNode` 型を YAML スキーマに整合させ、completion_criteria／standard_time_seconds／stress／ccp_thresholds／smed／capabilities_required／andon_severity／mes_integration を追加。
  - **メディアストレージライフサイクル**（§10.4.4）: `domain/storage-lifecycle.ts`（80%警告／90%ブロック、純粋関数判定、§20.1「人を責めない」表現テスト含む）。
  - **ハンズフリー音声コマンド**（§10.4.3）: `domain/voice-command.ts`（6 既定セット start/complete/suspend/back/memo/capture、9 言語の認識エイリアス、部分一致対応）。
  - **Grafana SLO ダッシュボード**（§31.5）: `docs/04_運用/grafana/dashboard-slo.json`（11 パネル、SLI-01〜10＋エラー予算ゲージ、Prometheus PromQL）。
  - **DR drill スクリプト**（§15.2）: `scripts/dr-drill.sh`（small/medium/large の RPO/RTO 目標別、pg_dump→DROP→restore→計測→自動レポート）。
  - **§11.3.1 残 7 言語 i18n**: vi（ベトナム）／th（タイ）／id（インドネシア）／fr（フランス）／pt（ポルトガル）／ar（アラビア、RTL）／he（ヘブライ、RTL）。`isRtl()` ／`RTL_LOCALES` 関数を追加。**計 13 ロケール**で §11.3.1 拡張計画を完了。
  - **教育コンテンツ骨格**（§25）: `docs/04_運用/education/README.md` ＋ 4 ペルソナ向けスクリプト（operator-5min／lead-monitoring／prod-eng-handson／sysadmin-sdk）、CC BY 4.0。
  - **lint-line-comments.sh 精度改善**: awk regex を再設計（チェーン継続／else/match ブランチ／属性行を除外）、awk warning ゼロ化、295 件警告 → 0 件。
  - **検証結果（Session 6 末）**: Rust **66 テスト全 PASS**（保持）、TypeScript **58 テスト全 PASS**（terminal 42 + config-ui 16、+29 件）、`scripts/lint-file-size.sh` 緑、`scripts/lint-line-comments.sh` OK（精度改善後）。
- **削除**:
  - `docs/01_企画/メモ.txt`: §18.4 規定に従い削除。§24.1 メモ追跡可能性マトリクスで L1〜L71 全行が反映済みであることを確認済み（§18.4 受入観点充足）。

### Changed

- `README.md` を §19.4.1 規約に従い「5 分で動かす／概念／ロードマップへの導線」を冒頭に配置するよう改訂。
- `.github/workflows/ci.yml` を Phase 6 scripts と接続（lint-file-size／lint-line-comments／glossary-lint／lint-rationale／check-links／lint-deferred／observability-link-lint）。

### Deprecated

- なし。

### Removed

- `docs/01_企画/メモ.txt`: §18.4 規定により削除（§24.1 全行反映済み）。

### Fixed

- なし。

### Security

- なし。

### Migration

- なし（初回コード骨格投入のため）。

### 規律例外

- **§9.4「1 PR ≤ 500 行差分」例外**: 本初期投入（コード骨格＋ドキュメント 60 ファイル超）は性質上分割困難であるため、§22.4 の事前承認を経て一括投入する。再発防止策は不要（初期投入の特殊性として受容）。次回以降の PR は通常の §9.4 規律に従う。

---

## バージョン番号運用

| 種別 | 例 | 含む変更 |
| --- | --- | --- |
| MAJOR | `1.0.0` | 破壊的変更。§32.4 廃止予告を経たもの。LTS 起点 |
| MINOR | `1.1.0` | 後方互換な機能追加 |
| PATCH | `1.1.1` | バグ修正・セキュリティ修正 |
| プレリリース | `1.0.0-rc.1` | リリース候補 |
| ビルドメタ | `1.0.0+sha.abc1234` | ビルド識別 |

## リリーストレイン（§32.2）

| トレイン | 周期 | 内容 |
| --- | --- | --- |
| パッチ | 隔週（偶数週金曜） | バグ修正・セキュリティ修正のみ |
| マイナー | 四半期（§22.1 と同期） | 後方互換な機能追加 |
| メジャー | 最低 18 ヶ月間隔 | 破壊的変更（§32.4 予告必須） |
| ホットフィックス | 即時 | CVSS ≥ 7.0 で 72 時間以内 |

## 廃止予告プロセス（§32.4）

| 段階 | 期間 | 措置 |
| --- | --- | --- |
| 予告 | 公開後最低 1 マイナー版 | 本書／ドキュメント／実行時警告 |
| 警告強化 | 次マイナー版 | UI／ログ／`@deprecated` 注釈／`Deprecation` ヘッダ |
| 削除 | メジャー版 | 移行ガイドを `docs/04_運用/migration-<from>-to-<to>.md` に同梱 |
| アドオン API 削除猶予 | 6 ヶ月（§17.3／§19.3） | 同上 |
