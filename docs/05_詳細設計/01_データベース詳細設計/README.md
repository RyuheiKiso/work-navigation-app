# 01 データベース詳細設計

本サブ（`01_データベース詳細設計/`）は IPA 共通フレーム 2013「**2.5.3 ソフトウェアコンポーネントデータの詳細設計**」を完全に担当する。`04_概要設計/04_データ設計/` で確定した物理テーブル一覧（TBL-001〜035）・列定義・インデックス方針・パーティション方式をコーディング直前仕様まで精緻化し、PostgreSQL 16 上で実行可能な DDL を全件定義する。

---

## 章構成と IPA 2.5.3 タスク対応

| ファイル | タイトル | IPA 2.5.3 タスク対応 |
|---|---|---|
| `00_本書の位置づけと識別子規約.md` | IPA 位置づけ・識別子規約・TBL カバレッジ | 2.5.3 開始条件確認・設計文書識別 |
| `01_マスタ系テーブルDDL（TBL-004〜028）.md` | マスタ系 CREATE TABLE 全文 | 2.5.3 データコンポーネント定義 |
| `02_トランザクション系テーブルDDL（TBL-001〜003・029〜035）.md` | トランザクション系 CREATE TABLE 全文 | 2.5.3 データコンポーネント定義 |
| `03_インデックス詳細設計（IDXカタログ）.md` | IDX-001〜016 CREATE INDEX 全文 | 2.5.3 アクセスパス・格納構造 |
| `04_ビュー・マテリアライズドビュー設計（VWカタログ）.md` | VW-001〜008 CREATE VIEW 全文 | 2.5.3 データアクセス層定義 |
| `05_JSONBスキーマ定義（TBL-030_step_flow_rules等）.md` | JSONB 列の JSON Schema 全定義 | 2.5.3 データ構造詳細 |
| `06_パーティション・アーカイブ詳細設計.md` | 月次パーティション・アーカイブ階層 | 2.5.3 格納・保存方式 |
| `07_マイグレーションスクリプト設計.md` | sqlx migrate 設計・命名規約 | 2.5.3 データ移行手順 |
| `08_シードデータ・テストフィクスチャ設計.md` | 本番シード・テストフィクスチャ設計 | 2.5.3 テストデータ定義 |
| `99_前提制約と本書が約束しないこと.md` | スコープ外明示 | 2.5.3 完了条件定義 |

---

## TBL カバレッジ

本サブは TBL-001〜035 の全 35 テーブルを対象とする。

| TBL-ID | テーブル名 | 掲載ファイル |
|---|---|---|
| TBL-001 | work_events | 02 |
| TBL-002 | electronic_signs | 02 |
| TBL-003 | outbox_events | 02 |
| TBL-004 | master_versions | 01 |
| TBL-005 | work_executions | 02 |
| TBL-006 | work_orders | 02 |
| TBL-007 | sops | 01 |
| TBL-008 | steps | 01 |
| TBL-009 | evidence_files | 02 |
| TBL-010 | measurements | 02 |
| TBL-011 | suspensions | 02 |
| TBL-012 | andon_alerts | 02 |
| TBL-013 | nonconformities | 02 |
| TBL-014 | capas | 02 |
| TBL-015 | kaizen_proposals | 02 |
| TBL-016 | users | 01 |
| TBL-017 | roles | 01 |
| TBL-018 | skills | 01 |
| TBL-019 | user_roles | 01 |
| TBL-020 | user_skills | 01 |
| TBL-021 | processes | 01 |
| TBL-022 | operations | 01 |
| TBL-023 | products | 01 |
| TBL-024 | lots | 01 |
| TBL-025 | equipments | 01 |
| TBL-026 | instruments | 01 |
| TBL-027 | external_key_bindings | 02 |
| TBL-028 | work_patterns | 01 |
| TBL-029 | step_type_definitions | 01 |
| TBL-030 | step_flow_rules | 01 |
| TBL-031 | hash_chain_blocks | 02 |
| TBL-032 | auth_logs | 02 |
| TBL-033 | devices | 01 |
| TBL-034 | device_sync_states | 01 |
| TBL-035 | idempotency_keys | 02 |

---

## 完成判定基準

| 確認項目 | 判定基準 |
|---|---|
| DDL | 全 35 TBL に CREATE TABLE 全文が存在する |
| IDX | IDX-001〜016 に CREATE INDEX 全文が存在する |
| VW | VW-001〜008 に CREATE VIEW / MATERIALIZED VIEW 全文が存在する |
| JSONB Schema | 全 JSONB 列に JSON Schema（type・required・properties）が存在する |
| パーティション | work_events 月次パーティション手順が runnable な SQL で記述されている |
| マイグレーション | sqlx migrate の up/down 命名規約と互換性ルールが明記されている |

---

**本節で確定した方針**
- **本サブ（01_データベース詳細設計）は IPA 2.5.3 のスコープを全て担当し、TBL-001〜035 全 35 テーブルの CREATE TABLE 全文を成果物として確定する。**
- **識別子は DDL-NNN（TBL-NNN と 1:1）・IDX-NNN・VW-NNN の 3 系統を使用し、04_概要設計の体系を継承する。**
- **PostgreSQL 16・UUID v7（`gen_random_uuid()` を使用）・TIMESTAMPTZ 統一・Append-only ロール制御を全 DDL に適用する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../90_業界分析/21_電子記録の法規制とALCOA+.md)

### 関連
- [`04_概要設計/04_データ設計/02_物理テーブル一覧（TBLカタログ）.md`](../../04_概要設計/04_データ設計/02_物理テーブル一覧（TBLカタログ）.md)
