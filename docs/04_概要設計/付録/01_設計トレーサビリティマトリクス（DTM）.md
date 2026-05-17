# 01 設計トレーサビリティマトリクス（DTM）

本章の責務は、要件 ID と設計 ID の対応を 6 つのマトリクス（M1〜M6）で管理し、概要設計の完全性を保証することである。`04_設計トレーサビリティ枠組み.md` で確定した DTM 構造に基づく。

各マトリクスは概要設計の各サブ章が完成した段階で逐次追記する。現時点（v0.1.0）は採番済み設計 ID を基に初期エントリを記載する。

---

## M1: FR × API（機能要件 → API エンドポイント）

**完全性条件**: Won't 以外の全 88 FR が ≥1 行に出現する

| FR-ID | FR 名称（要約） | API-ID | 関与種別 |
|---|---|---|---|
| FR-NV-001 | 作業開始トリガ（QR/リスト選択） | API-work-orders-001 | 主 |
| FR-NV-001 | 〃 | API-work-execs-001 | 主 |
| FR-NV-002 | プロセス→オペレーション→SOP 階層解決 | API-work-orders-001 | 副 |
| FR-NV-003 | 作業員スキル要件チェック | API-work-execs-001 | 副 |
| FR-NV-004 | ロックステップ進行制御 | API-step-events-001 | 主 |
| FR-NV-005 | 条件分岐 Step（DSL 評価） | API-step-events-001 | 副 |
| FR-NV-006 | カスタム Step（自由入力） | API-step-events-001 | 副 |
| FR-NV-007 | Step 内サブステップ展開 | API-step-events-001 | 副 |
| FR-NV-008 | プレースキーパー保存（中断ポイント） | API-work-execs-003 | 主 |
| FR-NV-009 | 並列 Step（OR 分岐合流） | API-step-events-001 | 副 |
| FR-NV-010 | 進捗バー表示 | API-work-execs-002 | 副 |
| FR-NV-011 | Step スキップ防止 | API-step-events-001 | 副 |
| FR-NV-012 | 残り作業時間推定 | API-work-execs-002 | 副 |
| FR-NV-013 | 多端末同時実行制御 | API-work-execs-002 | 副 |
| FR-EV-001 | Step 完了イベント記録 | API-step-events-001 | 主 |
| FR-EV-002 | 写真証拠取得 | API-evidences-001 | 主 |
| FR-EV-003 | 測定値取得（IoT 計測器） | API-step-events-001 | 副 |
| FR-EV-004 | QR/バーコードスキャン | API-step-events-001 | 副 |
| FR-EV-005 | 証拠メタデータ自動付与 | API-evidences-001 | 副 |
| FR-EV-006 | 数値入力（手動） | API-step-events-001 | 副 |
| FR-EV-007 | 選択肢入力 | API-step-events-001 | 副 |
| FR-EV-008 | テキスト入力 | API-step-events-001 | 副 |
| FR-EV-009 | チェックリスト入力 | API-step-events-001 | 副 |
| FR-EV-010 | 音声メモ | API-evidences-001 | 副 |
| FR-EV-011 | 証拠ファイルの SHA-256 ハッシュ | API-evidences-001 | 副 |
| FR-EV-012 | 証拠ファイルの圧縮・暗号化 | API-evidences-001 | 副 |
| FR-EV-013 | 工具/治具スキャン照合 | API-step-events-001 | 主（step_completed イベント） |
| FR-EV-013 | 〃 | API-master-001 | 副（equipments マスタ参照） |
| FR-ST-001 | 中断理由カテゴリ選択 | API-work-execs-003 | 主 |
| FR-ST-002 | 中断状態保存 | API-work-execs-003 | 主 |
| FR-ST-003 | 中断時の電子サイン | API-electronic-signs-001 | 主 |
| FR-ST-004 | 再開時の本人認証 | API-auth-001 | 副 |
| FR-ST-005 | 再開時のチェックポイント表示 | API-work-execs-004 | 主 |
| FR-ST-006 | 再開可否判定 | API-work-execs-004 | 主 |
| FR-ST-007 | 強制中断（監督権限） | API-work-execs-003 | 副 |
| FR-ST-008 | 中断履歴の閲覧 | API-work-execs-002 | 副 |
| FR-ST-009 | 引継ぎコメント記入 | API-work-execs-003 | 副 |
| FR-ST-010 | 同一 SOP 別作業員引継ぎ | API-work-execs-004 | 副 |
| FR-ST-011 | 中断中の通知抑制 | API-work-execs-003 | 副 |
| FR-ST-012 | 中断状態の自動エクスパイア（24h） | BAT（未採番）→CFG | 副 |
| FR-MA-001 | プロセスマスタ CRUD | API-master-001 | 主 |
| FR-MA-002 | オペレーションマスタ CRUD | API-master-001 | 主 |
| FR-MA-003 | 製品マスタ CRUD | API-master-001 | 主 |
| FR-MA-004 | SOP マスタ作成（ドラフト） | API-master-002 | 主 |
| FR-MA-005 | Step マスタ作成（ドラフト） | API-master-002 | 副 |
| FR-MA-006 | SOP インポート（CSV/Excel） | API-master-002 | 副 |
| FR-MA-007 | SOP プレビュー | API-master-003 | 副 |
| FR-MA-008 | レビュー依頼ワークフロー | API-master-007 | 主 |
| FR-MA-009 | 承認電子サイン | API-master-005 | 主 |
| FR-MA-010 | 公開（有効化日付指定） | API-master-005 | 副 |
| FR-MA-011 | 改訂版作成（旧版凍結） | API-master-002 | 主 |
| FR-MA-012 | 廃止処理（アーカイブ） | API-master-006 | 主 |
| FR-MA-013 | バージョン差分表示 | API-master-001 | 副 |
| FR-MA-014 | ユーザー CRUD | API-master-001 | 副 |
| FR-MA-015 | ロール/スキル割当 | API-master-001 | 副 |
| FR-MA-016 | Step-DAG ビジュアルフロー編集 | API-master-003 | 主（フロールール保存） |
| FR-MA-016 | 〃 | API-master-007 | 副（DAG dry-run 検証） |
| FR-SY-001 | ユーザー認証（JWT） | API-auth-001 | 主 |
| FR-SY-002 | 子機初回同期（全マスタ Pull） | API-sync-001 | 主 |
| FR-SY-003 | マスタ差分同期（増分 Pull） | API-sync-001 | 主 |
| FR-SY-004 | マスタ受信完了応答 | API-sync-001 | 副 |
| FR-SY-005 | Outbox イベント送信（実績 Push） | API-sync-002 | 主 |
| FR-SY-006 | 冪等性キー検証 | API-sync-002 | 副 |
| FR-SY-007 | DLQ 監視 | API-ops-001 | 主 |
| FR-SY-008 | 縮退モード切替 | API-system-001 | 副 |
| FR-SY-009 | 縮退時の OFFLINE 動作継続 | （クライアント処理）| — |
| FR-KZ-001 | アンドン発報 | API-andon-001 | 主 |
| FR-KZ-002 | アンドン応答（監督確認） | API-andon-002 | 主 |
| FR-KZ-003 | アンドン解決記録 | API-andon-002 | 副 |
| FR-KZ-004 | 不適合登録 | API-capa-001 | 主 |
| FR-KZ-005 | CAPA 登録 | API-capa-001 | 主 |
| FR-KZ-006 | CAPA 承認ワークフロー | API-capa-002 | 主 |
| FR-KZ-007 | Kaizen 提案登録 | API-kaizen-001 | 主 |
| FR-KZ-008 | Kaizen 集計（個人別ランキング禁止） | [非実装: BR-BUS-029 による永続的非実装。個別集計APIエンドポイントを設計上排除] | — |
| FR-AU-001 | 電子サイン取得（4 要素） | API-electronic-signs-001 | 主 |
| FR-AU-002 | 電子サイン検証 | API-electronic-signs-002 | 主 |
| FR-AU-003 | 電子サイン履歴閲覧 | API-electronic-signs-003 | 主 |
| FR-AU-004 | 監査ログ閲覧 | API-reports-002 | 副 |
| FR-AU-005 | 監査ログ XES エクスポート | API-reports-002 | 主 |
| FR-AU-006 | ハッシュチェーン検証（週次） | API-system-001（ヘルスチェック兼用） | 副 |
| FR-UI-001 | 多言語表示（日本語・英語） | （クライアント処理・CFG-014）| — |
| FR-UI-002 | やさしい日本語切替 | （クライアント処理）| — |
| FR-UI-003 | 言語切替（端末別保持） | （クライアント処理・CFG-014）| — |
| FR-UI-004 | ダークモード自動切替 | （クライアント処理・CFG）| — |
| FR-UI-005 | フォントサイズ拡大 | （クライアント処理）| — |
| FR-UI-006 | ハイコントラストモード | （クライアント処理）| — |
| FR-UI-007 | 色覚多様性対応 | （クライアント処理）| — |
| FR-UI-008 | 音声読み上げ | （OS 機能）| — |
| FR-UI-009 | タッチターゲット 44dp/72dp | （クライアント実装・CFG-013）| — |
| FR-UI-010 | エラー表示（色＋記号） | （クライアント処理・ERR 連動）| — |
| FR-UI-011 | プログレス通知 | API-work-execs-002 | 副 |

注: FR-UI/FR-SY-009 等のクライアント完結機能は API 不要（「クライアント処理」と明記）。

---

## M2: FR × TBL（機能要件 → 物理テーブル × CRUD）

**完全性条件**: データ操作を含む全 FR が ≥1 行に出現する

| FR-ID | TBL-ID | 物理テーブル名 | CRUD |
|---|---|---|---|
| FR-NV-001 | TBL-005 | work_executions | C |
| FR-NV-001 | TBL-006 | work_orders | R |
| FR-NV-002 | TBL-007 | sops | R |
| FR-NV-002 | TBL-008 | steps | R |
| FR-NV-002 | TBL-004 | master_versions | R |
| FR-NV-004 | TBL-001 | work_events | C |
| FR-NV-004 | TBL-005 | work_executions | R |
| FR-EV-001 | TBL-001 | work_events | C |
| FR-EV-002 | TBL-009 | evidence_files | C |
| FR-EV-003 | TBL-010 | measurements | C |
| FR-EV-011 | TBL-009 | evidence_files | C（hash 更新） |
| FR-ST-001 | TBL-011 | suspensions | C |
| FR-ST-001 | TBL-005 | work_executions | U（状態更新） |
| FR-ST-004 | TBL-005 | work_executions | U（状態更新） |
| FR-MA-001 | TBL-021 | processes | C,R,U |
| FR-MA-002 | TBL-022 | operations | C,R,U |
| FR-MA-003 | TBL-023 | products | C,R,U |
| FR-MA-004 | TBL-007 | sops | C |
| FR-MA-004 | TBL-004 | master_versions | C |
| FR-MA-005 | TBL-008 | steps | C,U |
| FR-MA-009 | TBL-002 | electronic_signs | C（承認サイン） |
| FR-MA-009 | TBL-004 | master_versions | U（状態更新: Published） |
| FR-MA-012 | TBL-004 | master_versions | U（状態更新: Archived） |
| FR-SY-001 | TBL-016 | users | R |
| FR-SY-001 | TBL-032 | auth_logs | C |
| FR-SY-002 | TBL-007 | sops | R |
| FR-SY-002 | TBL-008 | steps | R |
| FR-SY-005 | TBL-003 | outbox_events | C,U（status 更新） |
| FR-SY-007 | TBL-003 | outbox_events | R（DLQ フィルタ） |
| FR-KZ-001 | TBL-012 | andon_alerts | C |
| FR-KZ-002 | TBL-012 | andon_alerts | U（acknowledged） |
| FR-KZ-004 | TBL-013 | nonconformities | C |
| FR-KZ-005 | TBL-014 | capas | C |
| FR-KZ-006 | TBL-014 | capas | U（承認） |
| FR-KZ-007 | TBL-015 | kaizen_proposals | C |
| FR-AU-001 | TBL-002 | electronic_signs | C |
| FR-AU-002 | TBL-002 | electronic_signs | R |
| FR-AU-003 | TBL-002 | electronic_signs | R |
| FR-AU-005 | TBL-001 | work_events | R（XES エクスポート） |
| FR-AU-006 | TBL-031 | hash_chain_blocks | R |
| FR-MA-014 | TBL-016 | users | C,R,U |
| FR-MA-015 | TBL-019 | user_roles | C,R,U |
| FR-MA-015 | TBL-020 | user_skills | C,R,U |
| FR-MA-016 | TBL-030 | step_flow_rules | C,R,U |
| FR-MA-016 | TBL-008 | steps | R（ノード参照） |
| FR-MA-016 | TBL-004 | master_versions | R（バージョン整合性） |
| FR-EV-013 | TBL-025 | equipments | R（scan_code 検索） |
| FR-EV-013 | TBL-026 | instruments | R（calibration_due_date AND 照合） |
| FR-EV-013 | TBL-001 | work_events | C（payload.scan_verifications 記録） |

---

## M3: UC × SCR（ユースケース → 画面）

**完全性条件**: 全 23 UC が ≥1 SCR に紐づく

| UC-ID | UC 名称（要約） | SCR-ID | 関与種別 |
|---|---|---|---|
| UC-001 | 作業開始 | SCR-HA-001 | 必須 |
| UC-001 | 〃 | SCR-HA-002 | 必須 |
| UC-001 | 〃 | SCR-HA-003 | 条件（QR 起動時）|
| UC-001 | 〃 | SCR-HA-004 | 必須 |
| UC-002 | Step 完了（ロックステップ） | SCR-HA-005 | 必須 |
| UC-002 | 〃 | SCR-HA-006 | 条件（条件分岐） |
| UC-002 | 〃 | SCR-HA-007 | 条件（カスタム入力）|
| UC-003 | 条件分岐 Step | SCR-HA-006 | 必須 |
| UC-004 | カスタム Step | SCR-HA-007 | 必須 |
| UC-005 | 多言語切替 | SCR-HA-015 | 必須 |
| UC-006 | 写真証拠取得 | SCR-HA-008 | 必須 |
| UC-007 | 測定値取得 | SCR-HA-009 | 必須 |
| UC-008 | QR/バーコードスキャン | SCR-HA-003 | 必須 |
| UC-009 | 電子サイン | SCR-HA-010 | 必須 |
| UC-010 | 作業中断 | SCR-HA-011 | 必須 |
| UC-011 | 作業再開 | SCR-HA-012 | 必須 |
| UC-012 | アンドン発報 | SCR-HA-013 | 必須 |
| UC-013 | 不適合登録 | SCR-HA-014 | 必須 |
| UC-014 | プロセス/オペレーションマスタ登録 | SCR-MA-001 | 必須 |
| UC-014 | 〃 | SCR-MA-002 | 必須 |
| UC-015 | SOP/Step マスタ登録 | SCR-MA-004 | 必須 |
| UC-015 | 〃 | SCR-MA-005 | 条件（CSV インポート） |
| UC-016 | マスタ承認・公開 | SCR-MA-007 | 必須 |
| UC-016 | 〃 | SCR-MA-008 | 必須 |
| UC-017 | マスタ改訂・廃止 | SCR-MA-010 | 条件（差分確認） |
| UC-017 | 〃 | SCR-MA-011 | 必須 |
| UC-018 | ユーザー/ロール管理 | SCR-MC-002 | 必須 |
| UC-018 | 〃 | SCR-MC-003 | 必須 |
| UC-019 | 子機初回同期 | SCR-HA-015 | 必須（縮退表示） |
| UC-020 | マスタ差分同期 | （バックグラウンド処理）| — |
| UC-021 | Outbox 実績同期 | （バックグラウンド処理）| — |
| UC-022 | サーバー障害時の縮退モード | SCR-HA-015 | 必須 |
| UC-022 | 〃 | SCR-MC-001 | 条件（ダッシュボード） |
| UC-026 | 工具/治具スキャン照合 | SCR-HA-003 | 必須 |

---

## M4: UC × SEQ（ユースケース → シーケンス図）

**完全性条件**: 業務根幹 UC（UC-001/002/009/012/014/016/019/020）は ≥1 SEQ を持つ

| UC-ID | SEQ-ID | 図ファイル名 |
|---|---|---|
| UC-001 / UC-002 | SEQ-001 | fig_des_seq_step_execution |
| UC-006 / UC-009 | SEQ-002 | fig_des_seq_evidence_sign |
| UC-020 / UC-021 | SEQ-003 | fig_des_seq_outbox_sync |
| UC-010 / UC-011 | SEQ-004 | fig_des_seq_suspend_resume |
| UC-014 / UC-016 | SEQ-005 | fig_des_seq_master_publish |
| UC-020 | SEQ-006 | fig_des_seq_webhook_dlq |
| UC-019 | SEQ-008 | fig_des_seq_master_pull |
| （バッチ）| SEQ-007 | fig_des_seq_hash_chain_verify |
| UC-026 | SEQ-009 | 工具/治具スキャン照合シーケンス図（将来追加） |

---

## M5: NFR × 設計章（非機能要件 → 概要設計章）

**完全性条件**: 全 NFR がいずれかの設計章に出現、または「下流委任」「対象外」が明示される

（主要項目のみ初期記載。各サブ完成後に全件追記）

| NFR-ID | 概要設計章 | 実現方法要約 |
|---|---|---|
| NFR-AVL-001 | 01/08（可用性）| 99.5% SLO を MET-003 で計測、Active-Standby（01/03）|
| NFR-AVL-011 | 01_システム §03 | Active-Standby 手動切替 |
| NFR-AVL-015 | 08_運用 §04 | WAL ストリーミング PITR（BAT-006）|
| NFR-AVL-020 | 02_ソフトウェア §09 | SQLite Offline-First Outbox（BAT-002）|
| NFR-PRF-001 | 04_データ §06 | work_events の (case_id, ts) B-Tree IDX（IDX-001）|
| NFR-PRF-002 | 02_ソフトウェア §01 | Rust async/await による非同期処理 |
| NFR-PRF-010 | 02_ソフトウェア §10 | Outbox Consumer（BAT-002）指数バックオフ |
| NFR-SEC-010 | 07_セキュリティ §03 | JWT RS256・RSA 4096bit（KEY-001）|
| NFR-SEC-020 | 07_セキュリティ §02 | RBAC PRM マトリクス（PRM-NNN）|
| NFR-SEC-040 | 04_データ §05 | Append-only DB ロール（MOD-IN-001）|
| NFR-DQ-001 | 04_データ §05 | ハッシュチェーン（TBL-031, BAT-001）|
| NFR-UX-009 | 03_画面 §05 | タッチターゲット 72dp（CFG-013）|
| NFR-AVL-010 | 99（対象外）| 地理冗長: 対象外と判断する（中小製造業コスト制約）|
| NFR-PRF-003 | 05_WebAPP詳細設計/03_SopFlowEditor 詳細設計 §15 | シミュレーション 200ms 以内（クライアント完結）|

---

## M6: BR-BUS × API/ERR（業務ルール → バリデーション/エラー）

**完全性条件**: 全 46 BR-BUS が ≥1 行に出現、または「[非実装]」が明示される

| BR-BUS-ID | BR-BUS 名称（要約） | API-ID | バリデーション層 | ERR-ID | 違反時処理 |
|---|---|---|---|---|---|
| BR-BUS-001 | 直前 Step 未完了時は次 Step 表示禁止 | API-step-events-001 | アプリ層 | ERR-BIZ-001 | 409 ブロック |
| BR-BUS-002 | スキル要件未充足で Step 開始禁止 | API-work-execs-001 | アプリ層 | ERR-BIZ-006 | 403 ブロック |
| BR-BUS-003 | 必須証拠未取得で Step 完了禁止 | API-step-events-001 | アプリ層 | ERR-BIZ-002 | 422 ブロック |
| BR-BUS-004 | 公開状態の SOP のみ実行可 | API-work-execs-001 | アプリ層 | ERR-BIZ-003 | 409 ブロック |
| BR-BUS-005 | 並列 Step 合流 | API-step-events-001 | アプリ層 | ERR-BIZ-001 | 409 ブロック |
| BR-BUS-006 | サブステップ展開 | API-step-events-001 | アプリ層 | ERR-BIZ-001 | 409 ブロック |
| BR-BUS-007 | 校正期限プレフライト | API-step-events-001 | アプリ層 | ERR-BIZ-004 | 409 ブロック |
| BR-BUS-008 | SOP 版凍結後変更禁止 | API-master-003 | DB 制約 | ERR-BIZ-005 | 409 ブロック |
| BR-BUS-009 | 承認済版の再承認禁止 | API-master-005 | アプリ層 | ERR-BIZ-005 | 409 ブロック |
| BR-BUS-010 | 電子サイン 4 要素必須 | API-electronic-signs-001 | アプリ層 | ERR-VAL-001 | 422 ブロック |
| BR-BUS-011 | 電子サイン時の本人認証再入力 | API-electronic-signs-001 | アプリ層 | ERR-AUTH-001 | 401 ブロック |
| BR-BUS-012 | SOP 承認は品質担当ロール限定 | API-master-005 | RBAC | ERR-AUTH-004 | 403 ブロック |
| BR-BUS-013 | マスタ廃止時も電子サイン必須 | API-master-006 | アプリ層 | ERR-VAL-001 | 422 ブロック |
| BR-BUS-014 | 改ざん検出時は即時アラート | （BAT-001 検出後）| バッチ | ERR-DB-003 | 500 + アラート |
| BR-BUS-020 | JSON Logic 準拠の DSL / 拡張 Step エンジン（FR-MA-016）| API-master-003 | アプリ層 | ERR-VAL-003, ERR-VAL-024 | 422 ブロック |
| BR-BUS-021 | 演算子ホワイトリスト方式 | API-master-003 | アプリ層 | ERR-VAL-003 | 422 ブロック |
| BR-BUS-022 | ネスト最大 5 段 | API-master-003 | アプリ層 | ERR-VAL-003 | 422 ブロック |
| BR-BUS-023 | 副作用禁止（参照のみ） | API-master-003 | アプリ層 | ERR-VAL-003 | 422 ブロック |
| BR-BUS-024 | 評価タイムアウト 1 秒 | API-step-events-001 | アプリ層 | ERR-SYS-001 | 500 |
| BR-BUS-025 | ドラフト時にプレビュー必須 | API-master-007 | アプリ層 | ERR-BIZ-001 | 409 ブロック |
| BR-BUS-029 | **個人別ランキング非実装** | [非実装: BR-BUS-029 永続的非実装。個人集計APIエンドポイントを設計上排除。07_セキュリティ §08 で技術的拒否設計確定] | — | — | — |
| BR-BUS-030 | 数値範囲チェック（UCUM 単位） | API-step-events-001 | アプリ層 | ERR-VAL-002 | 422 |
| BR-BUS-031 | 必須項目チェック | 全書込み API | アプリ層 | ERR-VAL-001 | 422 |
| BR-BUS-032 | 文字数・形式チェック | 全書込み API | アプリ層 | ERR-VAL-004 | 422 |
| BR-BUS-033 | 写真解像度チェック（1280×720 以上） | API-evidences-001 | アプリ層 | ERR-VAL-003 | 422 |
| BR-BUS-034 | QR 形式チェック（GS1 準拠） | API-step-events-001 | アプリ層 | ERR-VAL-003 | 422 |
| BR-BUS-035 | 時刻チェック（過去禁止） | 全書込み API | アプリ層 | ERR-VAL-003 | 422 |
| BR-BUS-036 | マスタ存在チェック | 全書込み API | DB 制約 | ERR-DB-002 | 409 |
| BR-BUS-040 | RBAC 6 ロール × 機能マトリクス | 全 API | RBAC ミドルウェア | ERR-AUTH-004 | 403 |
| BR-BUS-041 | スキルレベル不足時作業開始禁止 | API-work-execs-001 | アプリ層 | ERR-BIZ-006 | 403 |
| BR-BUS-042 | 監督権限による緊急バイパス（電子サイン要） | API-work-execs-003 | アプリ層 | ERR-AUTH-001 | 401 |
| BR-BUS-043 | 端末別ロール制限 | API-master-001 | RBAC | ERR-AUTH-004 | 403 |
| BR-BUS-044 | 退職者の即時無効化 | API-master-014（ユーザー CRUD） | アプリ層 | ERR-AUTH-001 | 401 |
| BR-BUS-045 | パスワード再発行は system_admin 権限 | API-master-014 | RBAC | ERR-AUTH-004 | 403 |
| BR-BUS-046 | 誤工具・誤治具ハードブロック | API-step-events-001 | アプリ層（StepEngine.canAdvanceToStep） | ERR-VAL-006 | 422 ブロック |

| BR-BUS-015 | マスタ改訂後の全端末への即時反映 | API-master-010 | アプリ層 | ERR-BIZ-007 | 409 ブロック |
| BR-BUS-016 | トレサビ照会（ロット/品番から作業記録を検索）| API-reports-002 | アプリ層 | ERR-BIZ-008 | 404 |
| BR-BUS-017 | JWT 認証・セッション管理 | API-auth-001 | RBAC | ERR-AUTH-001 | 401 |
| BR-BUS-018 | 子機モード: 親機からのマスタ同期（READ-ONLY）| API-master-001 | アプリ層 | ERR-EXT-002 | 503 + リトライ |
| BR-BUS-019 | マスタ初期投入（工程・作業・Step・ユーザー）| API-master-001〜003 | アプリ層 | ERR-VAL-001 | 422 ブロック |
| BR-BUS-026 | バックアップ・リストア | BAT-001（スケジューラ）| バッチ層 | ERR-SYS-004 | P2 アラート + 再試行 |
| BR-BUS-027 | IT 担当 1 名での初期構築（GUI ウィザード）| —（初期構築 UI）| アプリ層 | ERR-SYS-003 | 500 + ウィザード停止 |
| BR-BUS-028 | 行動データ用途三限定 | [非実装: API エンドポイント設計上不存在。07_セキュリティ §08 で技術的拒否設計確定] | — | ERR-BIZ-005（用途外 API 呼出し）| 403 |

---

**本節で確定した方針**
- **DTM 6 マトリクス（M1〜M6）の初期エントリを記載し、概要設計の各サブ完成とともに全行を埋めることを完成条件とする。**
- **BR-BUS-029（個人別ランキング）は「[非実装]」として明記し、削除せずに永続管理する。**
- **FR-UI 系の「クライアント処理」と明記した行は API 不要であることを明示し、サーバーサイド API との責務分離を確定する。**
- **FR-MA-016（Step-DAG フロー編集）および FR-EV-013（工具/治具スキャン照合）を DTM 全 6 マトリクスに反映し、要件→API→TBL→ERR の追跡可能性を確立した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)
