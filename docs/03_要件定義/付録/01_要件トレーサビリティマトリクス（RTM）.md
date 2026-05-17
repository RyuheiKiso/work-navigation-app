# 01 要件トレーサビリティマトリクス（RTM）

本章の責務は、本書（要件定義書）の全要件間のトレーサビリティを 5 段構造（Why → What → BR → FR/NFR → TST）で管理する RTM の目的・構造・サンプル行・更新規約を確定することである。RTM は「要件の根拠が追跡できること」を ALCOA+ の Traceable 原則として実現する手段である。

---

## 1. RTM の目的と構造

### 1-1. RTM の目的

RTM（Requirements Traceability Matrix）は以下の目的で管理する。

- **前方トレーサビリティ（Forward）**: 上流（構想・計画）の意思決定が、下流の BR → FR/NFR → TST の各要件に反映されているかを確認する
- **後方トレーサビリティ（Backward）**: 各要件が上流のどの意思決定から生まれたかを追跡し、「なぜこの要件があるか」を説明できる状態を維持する
- **変更影響分析**: ある要件を変更した場合に影響する下流要件を特定し、変更の波及範囲を管理する
- **受入テスト網羅性の確認**: 全 BR・FR・NFR が対応する TST 受入基準に紐づいていることを確認する

図: fig_rtm_overview（img/ 配下）を参照。

### 1-2. RTM の 5 段構造

本書の RTM は以下の 5 段トレーサビリティチェーンで管理する（要件定義書 04 章 §4 の 6 段トレーサビリティを RTM 表では 5 段に集約する）。

| 段 | 要素 | 文書 | 説明 |
|---|---|---|---|
| 段 1 | Why（構想） | `システム化構想/` | なぜこの機能が必要か（課題・価値命題・倫理スタンス） |
| 段 2 | What（計画） | `システム化計画/` 各章 | 何をスコープとするか（機能スコープ・アーキテクチャ原則） |
| 段 3 | BR | `業務要件/` サブ | 業務フロー・ユースケースから導かれる業務的要件 |
| 段 4 | FR / NFR | `機能要件/` / `非機能要件/` サブ | 検証可能な機能・品質目標 |
| 段 5 | TST | `テスト・受入要件/` サブ | 受入基準・テスト種別・受入シナリオ |

### 1-3. RTM 表の列定義

RTM の各行は以下の列を持つ。

| 列 | 内容 |
|---|---|
| BR-ID | 業務要件 ID（`BR-BUS-NNN` または `BR-SC-NNN`）|
| BR タイトル | 業務要件の要約タイトル |
| FR-ID（複数可）| 対応する機能要件 ID（カンマ区切りで複数記載可）|
| NFR-ID（複数可）| 対応する非機能要件 ID（カンマ区切りで複数記載可）|
| TST-ID（複数可）| 対応する受入基準 ID（カンマ区切りで複数記載可）|
| 上流リンク（計画章）| 対応するシステム化計画の章番号（例: 計画 06 章）|
| MoSCoW | Must / Should / Could / Won't |
| 状態 | Draft / In Review / Agreed / Frozen / 廃止 |

---

**本節で確定した方針**
- RTM を 5 段トレーサビリティ（Why → What → BR → FR/NFR → TST）で管理することを確定する。
- RTM 表の 8 列（BR-ID・BR タイトル・FR-ID・NFR-ID・TST-ID・上流リンク・MoSCoW・状態）を確定する。
- 前方・後方両方向のトレーサビリティを維持し、変更影響分析・受入テスト網羅性確認に使用することを確定する。

---

## 2. RTM 表（サンプル行）

### 2-1. RTM サンプル 30 件以上

以下の RTM は本書の要件サブが完成した時点で全件を記載する。現時点では各サブカテゴリの代表的な要件をサンプルとして示す。

| BR-ID | BR タイトル | FR-ID | NFR-ID | TST-ID | 上流リンク（計画章）| MoSCoW | 状態 |
|---|---|---|---|---|---|---|---|
| BR-BUS-001 | 工程ごとの作業ナビゲーション実行 | FR-NV-001, FR-NV-002 | NFR-PRF-001 | TST-001, TST-002 | 計画 04 章 UC-A | Must | Agreed |
| BR-BUS-002 | ロックステップ進行（前 Step 未完了で次 Step に進めない）| FR-NV-002 | NFR-AVL-001 | TST-001, TST-005 | 計画 06 章 ALCOA+ Complete | Must | Agreed |
| BR-BUS-003 | クリティカルステップでの証拠写真必須化 | FR-NV-005, FR-EV-002 | NFR-SEC-001 | TST-001, TST-005 | 計画 06 章 ALCOA+ Attributable | Must | Agreed |
| BR-BUS-004 | 作業完了記録への ALCOA+ 準拠タイムスタンプ付与 | FR-EV-001, FR-EV-007 | NFR-SEC-002 | TST-005 | 計画 06 章 ALCOA+ Contemporaneous | Must | Agreed |
| BR-BUS-005 | 全作業記録の Append-only 保全（改ざん防止）| FR-EV-008 | NFR-SEC-003 | TST-005 | 計画 06 章 ALCOA+ Original | Must | Agreed |
| BR-BUS-006 | SHA-256 ハッシュチェーンによる改ざん検知 | FR-EV-002 | NFR-SEC-003 | TST-005 | 計画 06 章 ALCOA+ Consistent | Must | Agreed |
| BR-BUS-007 | Wi-Fi 断絶時の作業継続（Offline-First）| FR-SY-004, FR-ST-012 | NFR-AVL-002 | TST-003, TST-007 | 計画 05 章 Offline-First | Must | Agreed |
| BR-BUS-008 | Outbox Pattern による実績自動送信 | FR-SY-002 | NFR-AVL-002 | TST-007 | 計画 12 章 Outbox Pattern | Must | Agreed |
| BR-BUS-009 | SOP マスタの Draft-First 管理 | FR-MA-001, FR-MA-005 | — | TST-001 | 計画 19 章 Draft First | Must | Agreed |
| BR-BUS-010 | SOP バージョン参照整合性（廃止版も参照可能）| FR-MA-006, FR-MA-014 | NFR-SEC-004 | TST-001, TST-005 | 計画 06 章 ALCOA+ Enduring | Must | Agreed |
| BR-BUS-011 | アンドン発報・ヒヤリハット記録 | FR-ST-007, FR-ST-009 | NFR-PRF-002 | TST-001, TST-002 | 計画 09 章 CAPA | Must | Agreed |
| BR-BUS-012 | CAPA 起票・フォローアップ・クローズ管理 | FR-KZ-003 | — | TST-002 | 計画 09 章 CAPA | Must | Agreed |
| BR-BUS-013 | ロール別アクセス制御（RBAC）| FR-AU-002 | NFR-SEC-005 | TST-004 | 計画 15 章 RBAC | Must | Agreed |
| BR-BUS-014 | 電子サイン（電子承認）の付与と記録 | FR-EV-005 | NFR-SEC-006 | TST-002, TST-005 | 計画 06 章 ALCOA+ Attributable | Must | Agreed |
| BR-BUS-015 | マスタ改訂後の全端末への即時反映 | FR-MA-005, FR-SY-001 | NFR-PRF-003 | TST-001 | 計画 04 章 KPI:SOP 改訂反映時間 | Must | Agreed |
| BR-BUS-016 | トレサビ照会（ロット/品番から作業記録を検索）| FR-KZ-007 | NFR-PRF-004 | TST-002 | 計画 04 章 UC-G | Must | Agreed |
| BR-BUS-017 | JWT 認証・セッション管理 | FR-AU-001, FR-AU-005 | NFR-SEC-007 | TST-004 | 計画 15 章 JWT | Must | Agreed |
| BR-BUS-018 | 子機モード: 親機からのマスタ同期（READ-ONLY）| FR-SY-001 | NFR-AVL-003 | TST-007 | 計画 12 章子機モード | Must | Agreed |
| BR-BUS-019 | マスタ初期投入（工程・作業・Step・ユーザー）| — | — | TST-006 | 計画 13 章データ移行 | Must | Agreed |
| BR-BUS-020 | カスタム Step タイプ定義（拡張 Step エンジン）| FR-MA-010, FR-NV-007 | — | TST-001 | 計画 18 章 Step エンジン | Should | Agreed |
| BR-BUS-021 | 測定値入力 Step（上下限値バリデーション）| FR-EV-003, FR-NV-006 | — | TST-002 | 計画 18 章カスタム Step | Must | Agreed |
| BR-BUS-022 | QR コードスキャン Step | FR-EV-004 | — | TST-002 | 計画 18 章カスタム Step | Must | Agreed |
| BR-BUS-023 | 多言語表示（日本語・英語）| FR-UI-001, FR-NV-008 | NFR-PRT-001 | TST-001 | 計画 P05 外国人技能実習生 | Should | Agreed |
| BR-BUS-024 | ダッシュボード（稼働率・完了率・遅延アラート）| FR-KZ-008 | NFR-PRF-005 | TST-001 | 計画 09 章健全性指標 | Should | Agreed |
| BR-BUS-025 | 紙フォールバックへの縮退（第 3 縮退）| FR-ST-012, FR-SY-004 | NFR-AVL-004 | TST-003 | 計画 09 章縮退運転 | Must | Agreed |
| BR-BUS-026 | バックアップ・リストア | — | NFR-AVL-005 | TST-003 | 計画 08 章 RTO/RPO | Must | Agreed |
| BR-BUS-027 | IT 担当 1 名での初期構築（GUI ウィザード）| — | NFR-AVL-006 | PRJ-065-04 | 計画 09 章 GUI ウィザード | Must | Agreed |
| BR-BUS-028 | 行動データ用途三限定（作業支援・記録保全・改善活動）| — | NFR-ETH-001 | TST-001 | 計画 01 章倫理境界 | Must | Agreed |
| BR-BUS-029 | 個人別生産性ランキング機能の非実装（禁止実装要件）| — | NFR-ETH-002 | TST-004 | 計画 01 章倫理スタンス | Must（禁止）| Agreed |
| BR-BUS-030 | 校正機器・校正証明書の記録（計量法対応）| FR-MA-004, FR-EV-006 | — | TST-005 | 計画 R-R02 計量法 | Should | Agreed |
| BR-BUS-031 | 工具・治具・計測器のスキャン照合（ポカヨケ） | FR-EV-013, FR-EV-004 | — | テスト計画要 | 計画 03 章 §3 ポカヨケ対応 | Must | Agreed |
| BR-SC-001 | シナリオ: SOP ナビゲーション完全フロー | FR-NV-001〜FR-NV-013 | NFR-PRF-001 | TST-002, 受入シナリオ A | 計画 04 章 UC-A | Must | Agreed |
| BR-SC-002 | シナリオ: 証拠記録と電子サイン | FR-EV-001〜FR-EV-012 | NFR-SEC-001〜NFR-SEC-007 | TST-002, 受入シナリオ B | 計画 04 章 UC-C | Must | Agreed |
| BR-SC-003 | シナリオ: マスタ改訂 | FR-MA-001〜FR-MA-015 | — | TST-001, 受入シナリオ C | 計画 04 章 UC-D | Must | Agreed |
| BR-SC-004 | シナリオ: アンドン・CAPA クローズ | FR-ST-007〜FR-ST-010, FR-KZ-001〜FR-KZ-008 | NFR-PRF-002 | TST-002, 受入シナリオ D | 計画 04 章 UC-E | Must | Agreed |
| BR-SC-005 | シナリオ: Outbox オフライン同期 | FR-SY-001〜FR-SY-009 | NFR-AVL-002 | TST-007, 受入シナリオ E | 計画 12 章 Outbox | Must | Agreed |

---

**本節で確定した方針**
- RTM サンプルを BR-BUS-001〜031・BR-SC-001〜005 の 36 行として確定する。
- 全行に BR-ID・FR-ID・NFR-ID・TST-ID・上流リンク・MoSCoW・状態の 7 列を付与することを確定する。
- 禁止実装要件（BR-BUS-029）も RTM に含め、Must（禁止）として明示することを確定する。
- BR-BUS-031（工具/治具スキャン照合）を追加し、RTM の BR 行数を 36 に拡張した。

---

## 3. RTM の更新規約

### 3-1. RTM 更新のタイミング

RTM は以下のタイミングで更新する。

| 変更種別 | 更新すべき RTM 箇所 | 更新タイミング |
|---|---|---|
| 新規 BR 追加 | 新規行を追加。対応する FR/NFR/TST を同時に記録 | BR 追加と同時 |
| 新規 FR/NFR 追加 | 対応する BR 行の FR-ID/NFR-ID 列に追記 | FR/NFR 追加と同時 |
| 新規 TST 追加 | 対応する BR 行の TST-ID 列に追記 | TST 追加と同時 |
| 要件の変更 | 変更された要件に関連する全行を更新 | 要件変更プロセス（PRJ-036）完了後 |
| 要件の廃止 | 廃止された要件の行の「状態」を「廃止」に変更。廃止要件を参照する他行も確認 | 要件廃止と同時 |

### 3-2. RTM 更新の確認方法

RTM の完全性は以下の観点で確認する。

- **全 BR が RTM に存在するか**: `業務要件/` サブの全 BR-BUS-NNN・BR-SC-NNN が RTM に存在すること
- **全 FR が RTM に存在するか**: `機能要件/` サブの全 FR-NNN が RTM の少なくとも 1 行の FR-ID 列に含まれること
- **全 TST が RTM に存在するか**: TST-001〜007 が RTM の少なくとも 1 行の TST-ID 列に含まれること
- **RTM に TST 紐づきのない BR がないか**: TST-ID 列が空白の BR 行が存在しないこと

---

**本節で確定した方針**
- RTM の更新タイミングを変更種別ごとに確定し、変更と同時の更新を義務とする。
- RTM の完全性確認を全 BR・全 FR・TST-001〜007 の全件が RTM に含まれることで確認することを確定する。
- TST 紐づきのない BR が存在しないことを RTM 完全性の条件として確定する。

---

## 参照業界分析

### 必須

- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md) — RTM の 5 段トレーサビリティ構造と ALCOA+ Traceable 原則の接続根拠

### 関連

- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../../../90_業界分析/22_規制別トレーサビリティ要件詳論.md) — RTM による要件追跡と規制当局監査への対応の関係
- [`90_業界分析/30_国内製造業IT調達慣行とSI構造.md`](../../../../90_業界分析/30_国内製造業IT調達慣行とSI構造.md) — 将来チーム移行時の RTM 活用（設計根拠の追跡）の根拠
