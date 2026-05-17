# 02 RTM 拡張版 — 要件 × 設計識別子マッピング

本章は `03_要件定義/付録/01_要件トレーサビリティマトリクス（RTM）.md` の 8 列構成に、概要設計フェーズで確定した「主 SCR」「主 API」「主 TBL」の 3 列を追加した拡張ビューである。

**位置づけ**: RTM 本体（要件定義/付録/01）は Frozen 状態を維持し、本ファイルが設計識別子との橋渡しを担う。本ファイルを更新する際は同時に `付録/01_DTM.md` の対応マトリクス（M1〜M3）も更新する。

---

## 拡張 RTM 列定義

| 列 | 内容 | 変更 |
|---|---|---|
| BR-ID | 業務要件 ID | 既存 |
| BR タイトル | 業務要件要約 | 既存 |
| FR-ID | 対応機能要件 ID | 既存 |
| NFR-ID | 対応非機能要件 ID | 既存 |
| TST-ID | 対応テスト要件 ID | 既存 |
| 上流リンク | 構想章・計画章 | 既存 |
| MoSCoW | M/S/C/W | 既存 |
| 状態 | Agreed/Frozen | 既存 |
| **主 SCR** | 主要実現画面（最大 3 件） | **追加** |
| **主 API** | 主要実現 API（最大 3 件） | **追加** |
| **主 TBL** | 主要データテーブル（最大 3 件） | **追加** |

---

## 拡張 RTM エントリ（全 36 BR）

| BR-ID | BR タイトル | FR-ID（代表）| MoSCoW | 主 SCR | 主 API | 主 TBL |
|---|---|---|---|---|---|---|
| BR-BUS-001 | 工程ごとの作業ナビゲーション実行 | FR-NV-001/002 | Must | SCR-HA-002/004 | API-work-orders-001 | TBL-006/007 |
| BR-BUS-002 | ロックステップ進行（前 Step 未完了で次 Step に進めない）| FR-NV-004 | Must | SCR-HA-005 | API-step-events-001 | TBL-001 |
| BR-BUS-003 | クリティカルステップでの証拠写真必須化 | FR-EV-002 | Must | SCR-HA-008 | API-evidences-001 | TBL-009 |
| BR-BUS-004 | 作業完了記録への ALCOA+ タイムスタンプ付与 | FR-EV-001 | Must | SCR-HA-005 | API-step-events-001 | TBL-001 |
| BR-BUS-005 | 全作業記録の Append-only 保全（改ざん防止）| FR-EV-008 | Must | — | API-step-events-001 | TBL-001 |
| BR-BUS-006 | SHA-256 ハッシュチェーンによる改ざん検知 | FR-AU-006 | Must | SCR-MC-008 | BAT-003 | TBL-001/031 |
| BR-BUS-007 | Wi-Fi 断絶時の作業継続（Offline-First）| FR-SY-004 | Must | SCR-HA-002〜014 | — | TBL-003 |
| BR-BUS-008 | Outbox Pattern による実績自動送信 | FR-SY-002 | Must | SCR-HA-015 | API-outbox-001 | TBL-003 |
| BR-BUS-009 | SOP マスタの Draft-First 管理 | FR-MA-004/005 | Must | SCR-MA-004 | API-master-003 | TBL-007/008 |
| BR-BUS-010 | SOP バージョン参照整合性（廃止版も参照可能）| FR-MA-013 | Must | SCR-MA-010 | API-master-013 | TBL-004/007 |
| BR-BUS-011 | アンドン発報・ヒヤリハット記録 | FR-KZ-001 | Must | SCR-HA-013 | API-andon-001 | TBL-012 |
| BR-BUS-012 | CAPA 起票・フォローアップ・クローズ管理 | FR-KZ-004 | Must | SCR-HA-014 | API-capa-001 | TBL-013 |
| BR-BUS-013 | ロール別アクセス制御（RBAC）| FR-AU-001 | Must | SCR-MC-002/003 | API-auth-001 | TBL-016/019 |
| BR-BUS-014 | 電子サイン（電子承認）の付与と記録 | FR-AU-001 | Must | SCR-HA-010/SCR-MA-008 | API-electronic-signs-001 | TBL-002 |
| BR-BUS-015 | マスタ改訂後の全端末への即時反映 | FR-MA-010 | Must | SCR-MA-009 | API-master-010 | TBL-004/007 |
| BR-BUS-016 | トレサビ照会（ロット/品番から作業記録を検索）| FR-AU-004 | Must | SCR-MC-004 | API-reports-002 | TBL-001/032 |
| BR-BUS-017 | JWT 認証・セッション管理 | FR-SY-001 | Must | SCR-HA-001 | API-auth-001 | TBL-016/032 |
| BR-BUS-018 | 子機モード: 親機からのマスタ同期（READ-ONLY）| FR-SY-001 | Must | — | API-master-001 | TBL-007/008 |
| BR-BUS-019 | マスタ初期投入（工程・作業・Step・ユーザー）| FR-MA-001/002/003 | Must | SCR-MA-001〜003 | API-master-001〜003 | TBL-021/022/023 |
| BR-BUS-020 | カスタム Step タイプ定義（拡張 Step エンジン）| FR-NV-006, FR-MA-016 | Should | SCR-MA-004/SCR-HA-007 | API-master-003 | TBL-029/TBL-030 |
| BR-BUS-021 | 測定値入力 Step（上下限値バリデーション）| FR-EV-003 | Must | SCR-HA-009 | API-step-events-001 | TBL-010 |
| BR-BUS-022 | QR コードスキャン Step | FR-EV-004 | Must | SCR-HA-003 | API-work-orders-001 | TBL-006 |
| BR-BUS-023 | 多言語表示（日本語・英語）| FR-UI-001 | Should | SCR-HA-015 | —（クライアント処理）| TBL-034 |
| BR-BUS-024 | ダッシュボード（稼働率・完了率・遅延アラート）| FR-SY-002 | Should | SCR-MC-001 | API-ops-001 | TBL-001/032 |
| BR-BUS-025 | 紙フォールバックへの縮退（第 3 縮退）| FR-SY-004 | Must | SCR-HA-015 | —（縮退時）| TBL-034 |
| BR-BUS-026 | バックアップ・リストア | NFR-AVL | Must | SCR-MC-006 | —（BAT-001）| TBL-026 |
| BR-BUS-027 | IT 担当 1 名での初期構築（GUI ウィザード）| — | Must | —（初期構築 UI）| — | TBL-016/021 |
| BR-BUS-028 | 行動データ用途三限定（作業支援・記録保全・改善）| — | Must | [非実装: 07_セキュリティ §08 で技術的拒否設計確定] | — | — |
| BR-BUS-029 | 個人別生産性ランキング機能の非実装 | — | Must（禁止）| [非実装: 永続的非実装。個人集計 API エンドポイントを設計上排除] | — | — |
| BR-BUS-030 | 校正機器・校正証明書の記録（計量法対応）| FR-EV-006 | Should | SCR-HA-009 | API-step-events-001 | TBL-010 |
| BR-BUS-031 | 工具・治具・計測器のスキャン照合（ポカヨケ） | FR-EV-013 | Must | SCR-HA-003 | API-step-events-001 | TBL-025/TBL-026 |
| BR-SC-001 | シナリオ: SOP ナビゲーション完全フロー | FR-NV-001〜013 | Must | SCR-HA-001〜015 | API-work-execs-001〜005 | TBL-001/006/007 |
| BR-SC-002 | シナリオ: 証拠記録と電子サイン | FR-EV-001〜012 | Must | SCR-HA-008/009/010 | API-evidences-001/signs-001 | TBL-002/009/010 |
| BR-SC-003 | シナリオ: マスタ改訂 | FR-MA-001〜016 | Must | SCR-MA-001〜011 | API-master-001〜010 | TBL-004/007/008/030 |
| BR-SC-004 | シナリオ: アンドン・CAPA クローズ | FR-KZ-001〜008 | Must | SCR-HA-013/014 | API-andon-001/capa-001 | TBL-012/013 |
| BR-SC-005 | シナリオ: Outbox オフライン同期 | FR-SY-001〜009 | Must | SCR-HA-015 | API-outbox-001/master-001 | TBL-003/007 |

---

**本節で確定した方針**
- **本ファイルは要件 RTM 本体（03_要件定義/付録/01）に「主 SCR・主 API・主 TBL」の 3 列を追加した拡張ビューであり、要件 RTM 本体は変更しない。**
- **拡張 RTM の全エントリは概要設計の各サブが完成した後に全量追記し、DTM M1〜M3 との整合性を確保する。**
- **BR-BUS-029（禁止 BR）は「[非実装]」として保持し、設計・実装フェーズで当該機能が追加されないことを文書で宣言する。**
- **拡張 RTM に BR-BUS-031（ポカヨケ）を追加し、要件 RTM 本体との対応を 36 行に拡張した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
