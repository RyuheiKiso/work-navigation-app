# 10 SOPマスタ改訂記録テンプレ

本章は SOP（Standard Operating Procedure）マスタの改訂フロー・改訂記録テンプレートを確定する。MNT-040・NFR-OPS-012/013 に基づく quality_admin 承認義務・教育完了確認義務を実体化する。

図: fig_mnt_sop_revision（img/ 配下）を参照

---

## 1. SOP 改訂の定義と対象

### 1-1. SOP の定義

本システムにおける SOP は以下を含む。

| SOP 種別 | 格納場所 | 例 |
|---|---|---|
| 運用手順書 | `../運用手順/` | 週次・月次・バックアップ手順 |
| 障害対応手順書 | `../障害対応/` | P1〜P4 対応・ランブック |
| 保守記録手順書 | 本ディレクトリ | 01〜12・99 章 |
| 製造作業標準 | `../../03_要件定義/` 参照 | 製造現場の作業標準（SCR 経由）|

### 1-2. 改訂が必要なケース

- 手順の変更・追加・削除（軽微な誤字修正を除く）
- 規制改訂（21 CFR Part 11 / FDA Guidance の改版）への対応
- インシデント・ポストモーテムの改善処置として手順を変更する場合
- 依存するシステムのバージョンアップにより手順が変わる場合

**本節で確定した方針**
- **SOP の変更は全件 CR-OPS-NNN を起票してから改訂フロー（§2）を開始する。口頭での SOP 変更合意は不正とする。**
- **軽微な誤字修正（手順の内容を変えない）は CR-OPS-NNN 起票を要しないが、変更理由を git commit メッセージに記録する。**
- **製造現場の作業標準変更は quality_admin との合意を必須とし、改訂フロー全段階を踏む。**

---

## 2. SOP 改訂の 6 段階フロー

### Stage 1: 改訂申請受付

**実施者**: system_admin または quality_admin

```yaml
# CR-OPS-NNN 起票
cr_ops_id: CR-OPS-NNN
change_type: sop_revision
title: <改訂する SOP 名・改訂内容の要約>
requested_by: system_admin | quality_admin
requested_at: YYYY-MM-DD
revision_target_file: <対象 SOP ファイルのパス>
revision_reason: <改訂理由（規制対応 / インシデント改善 / 機能変更 / 等）>
related_inc: INC-YYYY-NNN  # インシデント起因の場合
related_pm: PM-YYYY-NNN    # ポストモーテム改善の場合
```

**本ステージの完了条件**: CR-OPS-NNN が付録/99 §6 台帳に登録されている。

### Stage 2: ドラフト作成

**実施者**: system_admin

- 対象 SOP ファイルをブランチ `sop/CR-OPS-NNN-<slug>` で修正する
- 変更点を明示するため git diff を CR-OPS-NNN に添付する
- 影響する下流手順・チェックリスト（CHK-NNN）を特定して同時改訂する

```bash
# ドラフトブランチ作成
git checkout -b sop/CR-OPS-NNN-<slug>

# SOP ファイル修正後、変更差分を確認
git diff main...HEAD -- docs/09_運用・保守/
```

**本ステージの完了条件**: ブランチに変更がコミットされ、変更差分が CR-OPS-NNN に記録されている。

### Stage 3: セルフレビュー（7 日クーリング）

**実施者**: system_admin（ドラフト作成者と同一）

- ドラフトコミット後 7 日経過してからセルフレビューを実施する（04 章 §2-1 と同一ルール）
- 以下の観点でレビューを実施する

| レビュー観点 | 確認内容 |
|---|---|
| 手順の明確性 | 手順が曖昧なく実行可能か |
| ALCOA+ 整合 | 改訂後も ALCOA+ 原則が維持されるか |
| 規制準拠 | 21 CFR Part 11 / NFR 要件との整合性 |
| 他手順との整合 | 関連 SOP・CHK-NNN との整合性 |
| 教育要件 | 改訂内容の教育が必要か（§4 参照）|

```yaml
# Stage 3 完了記録
stage3_review_at: YYYY-MM-DD HH:MM
stage3_reviewer: system_admin
stage3_cooling_days: XX  # ドラフトから何日後か（7 以上が必要）
stage3_result: PASS | FAIL
stage3_notes: <指摘事項・対応内容>
```

**本ステージの完了条件**: セルフレビュー実施日がドラフトコミット日から 7 日以上経過しており、レビュー結果が PASS である。

### Stage 4: quality_admin 承認

**実施者**: quality_admin

- ドラフトと Stage 3 レビュー記録を quality_admin に提出する
- quality_admin は内容を確認し承認または差し戻しを決定する
- MNT-040 に基づき、全 SOP 改訂は quality_admin 承認が必須である

```yaml
# Stage 4 完了記録
stage4_approved_by: quality_admin
stage4_approved_at: YYYY-MM-DD HH:MM
stage4_result: APPROVED | REJECTED
stage4_rejection_reason: <差し戻しの場合の理由>
```

**本ステージの完了条件**: quality_admin が CR-OPS-NNN の `approved_by` に署名（承認日時を記録）している。

### Stage 5: MasterVersion PUBLISHED 化

**実施者**: system_admin

- quality_admin 承認後、ブランチを main にマージする
- SOP ファイルのヘッダーに版数・承認日・承認者を更新する
- CHANGELOG を更新する

```bash
# main にマージ
git checkout main
git merge --no-ff sop/CR-OPS-NNN-<slug>
git tag -a "sop-CR-OPS-NNN" -m "SOP revision CR-OPS-NNN approved by quality_admin"
git push origin main --tags

# SOP ファイルのヘッダー更新例
# | 版 | 日付 | 変更者 | 変更内容 |
# | X.Y.Z | YYYY-MM-DD | system_admin | CR-OPS-NNN 適用 |
```

**本ステージの完了条件**: main ブランチにマージ済みで、SOP ファイルの版数が更新されている。

### Stage 6: 教育完了確認

**実施者**: system_admin

- 改訂内容が製造現場のオペレーターに影響する場合は教育を実施する
- 教育完了を EDU-NNN として記録する（EDU 識別子は `03_要件定義/非機能要件/` の EDU 要件に準拠）
- system_admin / quality_admin が実施者のみの場合は教育確認は対象外と判断する

```yaml
# Stage 6 完了記録
stage6_edu_required: true | false
stage6_edu_target: <教育対象（オペレーター / system_admin / quality_admin）>
stage6_edu_completed_at: YYYY-MM-DD
stage6_edu_confirmed_by: quality_admin
```

**本ステージの完了条件**: 教育が不要（`edu_required: false`）または教育完了確認（`edu_confirmed_by` に記録）済みである。

**本節で確定した方針**
- **SOP 改訂の 6 段階フロー（申請 → ドラフト → セルフレビュー → 承認 → 公開 → 教育）は全段階を省略なく踏む。**
- **Stage 3 のセルフレビューは 7 日クーリングを必須とし、Stage 4 の quality_admin 承認は MNT-040 に基づき必須とする。**
- **Stage 5 のマージ後に git tag を付与し、SOP の特定版数を永続的に参照可能にする（ALCOA+ Enduring 原則）。**

---

## 3. 改訂記録テンプレート（REC-NNN 種別: sop_revision）

```yaml
# REC-NNN SOP 改訂記録（07 章テンプレートに追加するフィールド）
sop_revision_target: <対象 SOP ファイルパス>
sop_current_version: X.Y.Z
sop_new_version: X.Y.Z+1
cr_ops_id: CR-OPS-NNN

# 各 Stage の完了記録
stage1_cr_raised_at: YYYY-MM-DD
stage2_draft_committed_at: YYYY-MM-DD
stage3_self_review_at: YYYY-MM-DD
stage3_cooling_days: XX
stage4_approved_by: quality_admin
stage4_approved_at: YYYY-MM-DD
stage5_published_at: YYYY-MM-DD
stage5_git_tag: sop-CR-OPS-NNN
stage6_edu_required: true | false
stage6_edu_completed_at: YYYY-MM-DD

# 完了宣言
all_stages_completed: true | false
completed_at: YYYY-MM-DD
```

**本節で確定した方針**
- **REC-NNN の `all_stages_completed: true` が SOP 改訂完了の唯一の条件とする。**
- **改訂記録は ALCOA+ Contemporaneous 原則に基づき各 Stage 完了時に即座に更新する（事後記録は不正）。**
- **改訂記録は SOP 廃止後も 7 年間保全する（ALCOA+ Enduring 原則・MNT-041）。**

---

## 4. 改訂履歴台帳

| REC-NNN | CR-OPS-NNN | 対象 SOP | 改訂前版 | 改訂後版 | 承認者 | 有効化日 | 教育完了日 |
|---|---|---|---|---|---|---|---|
| - | - | - | - | - | - | - | - |

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.2.c・4.2.2.d — 修正実施・保守レビューとしての SOP 改訂
- 21 CFR Part 11 §11.10(c)（コンピュータによる記録の保護）— SOP 版数管理の規制根拠
- FDA 21 CFR Part 820 §820.40（文書管理）— 承認・改訂・配布の要件

### 関連
- [`../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)
- NFR-OPS-012/013（`../../03_要件定義/非機能要件/`）

---

## 版数履歴

| 版 | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | RyuheiKiso | 初版（6 段階改訂フロー・改訂記録テンプレート確定）|
