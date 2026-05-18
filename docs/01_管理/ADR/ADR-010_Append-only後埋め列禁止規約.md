# ADR-010: Append-only テーブルへの後埋め列禁止規約

**状態**: 採択
**日付**: 2026-05-18
**担当サブ**: 05_詳細設計/01_データベース詳細設計

## 背景

TBL-011 suspensions テーブルに `resumed_at TIMESTAMPTZ NULL` および `resume_sign_id UUID NULL` が定義されていた。この 2 列は INSERT 後に再開時刻が確定した際に UPDATE で埋まる設計を前提としていたが、suspensions は Append-only テーブル（`REVOKE UPDATE, DELETE ON suspensions FROM app_event_writer`）として宣言されており、UPDATE ができない。この矛盾により、resumed_at / resume_sign_id は永続的に NULL となる実質的デッドカラムが発生していた。

また、上流（`03_要件定義/機能要件/12_データ要件（論理）.md` L260）は「WorkExecution の status 遷移更新は work_resumed イベントで代替する」とイベントソーシング方式を明示しており、suspensions に再開情報を保持する意図は上流設計にも存在しない。

この問題を受けて、Append-only テーブルの設計ルールを明文化する必要が生じた。

## 決定事項

**Append-only テーブルには INSERT 後に値が変わる列（後埋め列）を一切置かない。**

- 中断・再開のような状態遷移は work_events のイベント（activity='work_suspended' / 'work_resumed'）で表現する（イベントソーシング）
- Append-only テーブルの定義: `REVOKE UPDATE, DELETE` が付与されたテーブル（work_events / electronic_signs / evidence_files / measurements / suspensions / hash_chain_blocks / auth_logs / external_key_bindings）
- 全列が INSERT 時に確定すること（NULL でない確定値、または NULL が「情報なし」という定常状態を意味すること）

**唯一の例外**: outbox_events.status の UPDATE のみ許可する。これは Transactional Outbox パターンの送信状態管理（PENDING → SENDING → SENT）に必要であり、`app_read_write` ロールに限定して明示的に許可する。

## 根拠（代替案との比較表を含む）

| 代替案 | メリット | デメリット | 採否 |
|---|---|---|---|
| 後埋め列を許可し REVOKE UPDATE を解除（C案）| suspensions に再開情報を局所化できる | Append-only 原則が崩れ改ざん検知保証が弱まる。ALCOA+ Original 要件との整合が不明確になる | 却下 |
| 新規 resumptions テーブルを追加（B案）| suspensions と resumptions を対称に保てる | テーブル数増加。work_events にも work_resumed イベントが存在するため再開情報が 2 か所に分散し冗長になる | 却下 |
| resumed_at / resume_sign_id を削除し work_events で代替（A案・採択案）| イベントソーシング思想に完全一致。上流 12_データ要件 L260「status 遷移は work_events で代替」設計命題と整合する。複雑性が下がる | 再開情報取得に work_events JOIN が必要（VW-001 等のビューで吸収可能）| **採択** |

## 結果

- TBL-011 suspensions から resumed_at / resume_sign_id / fk_suspensions_resume_sign / ck_suspensions_resumed_after_suspended を削除
- 再開時刻・再開電子サインは work_events.activity='work_resumed' の payload（suspension_id 参照・resume_sign_id を含む）から取得する
- REVOKE 設計と DDL の整合が回復し、Append-only 原則が全対象テーブルで統一された

## 参照

- [`ADR-007_per_case_id_genesis採用.md`](ADR-007_per_case_id_genesis採用.md) — イベントソーシング基本方針
- [`ADR-008_補正レコード継続規則.md`](ADR-008_補正レコード継続規則.md) — Append-only + 補正の方針
- `03_要件定義/機能要件/05_UC記述_中断再開とアンドン.md` — UC-011 事後条件（work_resumed イベント記録）

## 更新履歴

| 日付 | 変更 | 変更者 |
|---|---|---|
| 2026-05-18 | 初版（DB 詳細設計レビューで suspensions の Append-only 矛盾を発見し対応）| RyuheiKiso |
