# Glossary (English)

> Corresponding sections: roadmap §28 §3 §11.3.1 §22.1
> Audience: maintainers, contributors, authors of docs / code / UI / API schemas
> Revision cycle: §22.1 semi-annual

This is the canonical English-language glossary that mirrors [`glossary-ja.md`](./glossary-ja.md). The Japanese edition is the primary reference; this document provides authoritative English translations to keep API names, code identifiers, and English-language UI consistent. Synonym co-existence is prohibited (§9.4 / §28).

## 1. Domain terms (core concepts)

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| 作業 | Task | §3.1.1 (11 constituent elements) | Work, Job, Activity, Operation (in this scope) |
| 工程 | Process | §3.6.3 hierarchy upper layer | Procedure (reserved), Operation |
| 手順 | Procedure | sequence unit within a process | Step (reserved), Sequence |
| 動作 | Step | atomic action within a procedure | Action, Move, Operation |
| 完了条件 | Completion Criteria | §3.1.1 verification clause | Done condition, Pass criteria |
| 開始条件 | Precondition | §3.1.1 entry clause | Prerequisite (used elsewhere) |
| 順序情報 | Production Order Sequence | §10.3.2 received from ERP/MES | Plan info, Schedule (in MES sense) |
| 実績 | Record | §10.6 append-only event | History, Log (Audit Log is distinct) |
| アンドン | Andon | TPS notification | Alert, Warning |
| 監査ログ | Audit Log | §11.4.1 immutable | Operation log, History |
| ナビゲーション | Navigation | §3.6.1 formal definition | Guidance, Wayfinding (academic only) |
| フロー | Flow | work flow definition | Workflow (reserved for §17 addons) |
| マスタ | Master Data | §10.2 centrally managed | Reference data, Base data |
| アドオン | Addon | §17 extension module | Plugin, Extension |
| 端末ペアリング | Device Pairing | §14.3 QR-based bootstrap | Device authentication (distinct from §10.5) |
| 端末バインド | Device Binding | §11.4.1 ID-to-device association | Device authentication |

## 2. Manufacturing methods

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| ポカヨケ | Poka-Yoke | §9.3.1 mistake-proofing | Error-proofing |
| 自働化（にんべん） | Jidoka | §9.3.1 stop-on-anomaly | Automation (must distinguish) |
| カイゼン | Kaizen | §9.3.1 continuous improvement | Improvement (must use Kaizen for the cultural concept) |
| ジャストインタイム | Just-In-Time (JIT) | §9.3.1 | Lean stock |
| カンバン | Kanban | §9.3.1 pull signal | Signage |
| 平準化 | Heijunka | §9.3.1 leveling | Smoothing (signal-processing sense) |
| タクトタイム | Takt Time | §9.3.1 | Cycle time (distinct concept) |
| 七つのムダ | Seven Wastes | §9.3.2 | Seven big wastes |
| 5S | 5S | TPS | Tidy-up |
| SMED | SMED | §9.3.1 | Single-digit changeover |
| なぜなぜ分析 | 5 Whys | §9.3.1 | Five-W analysis |
| A3 思考 | A3 Thinking | §9.3.1 | A3 report |
| 三現主義 | Three Reals | §9.3.4 | Genba-Genbutsu-Genjitsu (transliteration only when explicit) |
| OEE | OEE (Overall Equipment Effectiveness) | §9.3.4 | Equipment uptime |

## 3. Formalization, sync, testing

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| 階層型有限状態機械 | HSM (Harel statechart) | §3.4.1 | UML state machine (rejected alternative) |
| 着色 Petri net | CPN | §3.4.1 | Place/Transition net (rejected) |
| TLA+ | TLA+ | §3.4.1 | Alloy (rejected) |
| ラムポートタイムスタンプ | Lamport timestamp | §10.6.1 | Logical clock (overly broad) |
| 追記のみ集合 | G-Set | §10.6.1 | grow-only set (lowercase) |
| LWW レジスタ | LWW-Register | §10.6.1 | Last-write-wins register |
| 不変式 | Invariant | §3.4.2 | Constraint |
| 性質ベーステスト | Property-Based Test | §13.1 | Fuzzing (distinct) |
| ミューテーションテスト | Mutation Testing | §13.4.1 | — |
| カオスエンジニアリング | Chaos Engineering | §13.4.2 | Fault injection (broader) |
| デッドレター | Dead Letter | §10.3.1 | Error queue |

## 4. Psychology and UX

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| 認知負荷 | Cognitive Load | §9.2.1 §9.2.2 | Mental load |
| ナッジ | Nudge | §9.2.3 | Push, Encouragement |
| デフォルト効果 | Default Effect | §9.2.3 | Initial-value effect |
| 損失回避 | Loss Aversion | §9.2.3 | Loss avoidance (incorrect) |
| ピーク・エンドの法則 | Peak-End Rule | §9.2.3 | Peak-terminal rule |
| ゴール・グラディエント効果 | Goal Gradient Effect | §9.2.3 | Goal-slope effect |
| メンタルモデル | Mental Model | §9.2.3 | Image model |
| ダーク・パターン | Dark Pattern | §9.2.4 | Deceptive design (partial overlap, not adopted) |
| 段階的開示 | Progressive Disclosure | §3.6.4 §9.2.2 | Incremental reveal |
| アンドン経路 | Andon Path | §17 notification capability | Alert path |

## 5. Project governance

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| 至高 | Supremacy | §2 attitude of continuous improvement | Perfection, Strongest |
| 沈黙の妥協 | Silent Compromise | §2.2 | Implicit compromise, Hidden compromise |
| 圧倒候補機能 | Killer Feature | §4.9 | Killer functionality |
| 受入観点 | Acceptance Criteria | per-chapter pass/fail indicators | Acceptance basis |
| 追跡可能性マトリクス | Traceability Matrix | §24 | Cross-reference table |
| アンチゴール | Anti-Goal | §26 | Non-goal |
| 撤退条件 | Exit Criteria | §22.3 | Withdrawal basis |
| Type 1 決定 | Type 1 Decision | §30.1 irreversible | — |
| Type 2 決定 | Type 2 Decision | §30.1 reversible | — |
| 遅延決定 | Deferred Decision | §33 explicit hold | Undecided (must distinguish from silent compromise) |
| ADR | Architecture Decision Record | §9.4 | — |

## 6. Release and compatibility

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| リリーストレイン | Release Train | §32.2 fixed-cadence releases | Train releases (verbose) |
| 廃止予告 | Deprecation Notice | §32.4 | Discontinuation announcement |
| 互換性レーン | Compatibility Lane | §32.3 | Compatibility track |
| ホットフィックス | Hotfix | §32.2 | Emergency fix |
| LTS | LTS (Long-Term Support) | §19.3 | Long-term release |

## 7. Observability and SRE

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| SLI | Service Level Indicator | §31.1 | Service quality indicator |
| SLO | Service Level Objective | §31.2 | Service quality goal |
| エラー予算 | Error Budget | §31.3 | Error tolerance |
| バーンレート | Burn Rate | §31.4 | Consumption rate |
| 監査追記不変性 | Append-Only Immutability | §11.4.1 §10.6 | Tamper-resistant log (overlap) |

## 8. Data protection

| Japanese | English | Definition (§) | Banned synonyms |
| --- | --- | --- | --- |
| 個人特定情報 | PII (Personally Identifiable Information) | §11.4.1 §20.3 | Personal data (broader, GDPR sense) |
| 越境データ移転 | Cross-Border Data Transfer | §20.3 not implemented in core | International data transfer (overly generic) |
| 端末暗号化 | Device-Side Encryption | §11.4.2 SQLCipher AES-256 | Edge encryption |
| 鍵保護 | Key Protection | §11.4.2 OS Keystore / DPAPI | Key management (broader scope) |

## 9. Sync with the Japanese edition

This document MUST be updated whenever [`glossary-ja.md`](./glossary-ja.md) is changed. CI checks (`scripts/glossary-lint.sh`, when available) will fail on unsynchronized rows. Translation drift is treated as a §22.4 corrective trigger.
