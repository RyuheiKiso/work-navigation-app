# 07 保守記録テンプレ（REC-NNN）

本章は保守記録（REC-NNN）の標準テンプレートを提供する。保守作業開始時にこのテンプレートを複製し、必要なフィールドを記入して使用する。

---

## 1. テンプレートの使用方法

```bash
# テンプレート複製
cp docs/09_運用・保守/保守記録/07_保守記録テンプレ\（REC-NNN\）.md \
   docs/09_運用・保守/保守記録/REC-NNN_YYYY-MM-DD_<概要スラグ>.md

# REC 番号を付録/99 §5 台帳で採番してから使用する
# 台帳登録前の識別子使用は不正とする
```

**本節で確定した方針**
- **全保守作業は本テンプレートを使用して記録する。テンプレート外形式での記録は不正とする。**
- **記録は ALCOA+ Contemporaneous 原則に基づき作業開始時に起票し、各フェーズの完了時に更新する。**
- **記録の事後改ざんは禁止する。記録の誤りは訂正理由と訂正者・訂正日時を付記した上で修正する。**

---

## 2. 共通ヘッダー（全 record_type 共通）

```yaml
# REC-NNN: 保守記録
---
record_id: REC-YYYY-NNN
record_type: bugfix | dependency_update | key_rotation | db_migration | security_patch | sop_revision | capacity_extension | disposal
status: open | in_progress | completed | closed | rollback_completed
priority: P1 | P2 | P3 | P4
related_incident: INC-YYYY-NNN  # 任意（インシデント連動の場合）
related_prob: PROB-NNN          # 任意（問題起票がある場合）
related_adr: ADR-OPS-NNN        # 任意（設計判断 ADR がある場合）
cr_ops_id: CR-OPS-NNN           # 変更要求 ID（必須）
started_at: YYYY-MM-DD
completed_at: YYYY-MM-DD        # completed/closed 時に記入
executed_by: system_admin
reviewed_by: quality_admin      # ALCOA+ 影響時・P1 修正時は必須
```

---

## 3. record_type 別本文テンプレート

### 3-1. bugfix（バグ修正）

```markdown
## 問題概要

<!-- PROB-NNN の title を転記 -->

## 影響範囲

- 影響機能: <!-- API-NNN / TBL-NNN / 画面名 -->
- ALCOA+ 影響: <!-- あり / なし。ありの場合は具体的な原則を記載 -->
- ユーザー影響: <!-- 影響を受けるユーザーの範囲 -->

## 修正内容

<!-- 何をどのように修正したかを具体的に記述 -->

## テスト結果

- [ ] cargo test --release PASS
- [ ] cargo clippy -- -D warnings PASS
- [ ] cargo audit CRITICAL/HIGH 0 件
- [ ] pnpm test PASS
- [ ] hash-chain-verify ERROR 0（ALCOA+ 影響時）
- [ ] STG 5 項目全 PASS

## ステージング確認記録

| 確認項目 | 結果 | 確認日時 |
|---|---|---|
| STG-1 ヘルスチェック | PASS / FAIL | YYYY-MM-DD HH:MM |
| STG-2 DB 接続 | PASS / FAIL | YYYY-MM-DD HH:MM |
| STG-3 ハッシュチェーン | PASS / FAIL | YYYY-MM-DD HH:MM |
| STG-4 Outbox フラッシュ | PASS / FAIL | YYYY-MM-DD HH:MM |
| STG-5 マイグレーション適用 | PASS / FAIL | YYYY-MM-DD HH:MM |

## レビュー記録

- レビュー種別: self_review | quality_admin_approval
- レビュー実施日時: YYYY-MM-DD HH:MM
- クーリング期間: X 日（P3/P4 の場合）
- レビュー結果: PASS / FAIL
- レビュー指摘事項: <!-- なし or 具体的な指摘と対応 -->

## 本番移行記録

- GO/NOGO 判定: GO / NOGO
- NOGO 理由（NOGO の場合）: <!-- 理由 -->
- 移行実施日時: YYYY-MM-DD HH:MM
- 移行後確認: PASS / FAIL
- ロールバック: なし / あり（理由: ）
- git_ref: <!-- コミットハッシュ -->
```

---

### 3-2. dependency_update（依存ライブラリ更新）

```markdown
## 更新対象

| パッケージ名 | 更新前バージョン | 更新後バージョン | 更新理由 |
|---|---|---|---|
| <!-- crate/package 名 --> | <!-- before --> | <!-- after --> | <!-- CVE / 定期更新 / EOL --> |

## CVE 対処（CVE 対処を含む場合）

| CVE ID | CVSS スコア | 対処内容 | ロールバック条件 |
|---|---|---|---|
| CVE-YYYY-NNNNN | <!-- スコア --> | <!-- パッチ適用 / バージョン固定 --> | <!-- ロールバック基準 --> |

## テスト結果

- [ ] cargo test --release PASS
- [ ] cargo audit CRITICAL/HIGH 0 件
- [ ] pnpm test PASS
- [ ] pnpm audit HIGH/CRITICAL 0 件
- [ ] Docker ビルド成功
- [ ] STG 5 項目全 PASS

## 更新台帳記録

- [ ] 08_依存ライブラリ更新履歴テンプレ.md を更新した
- 更新日時: YYYY-MM-DD

## 本番移行記録

<!-- bugfix テンプレートの「本番移行記録」セクションと同一 -->
```

---

### 3-3. key_rotation（鍵ローテーション）

```markdown
## 対象鍵

| KEY-ID | 鍵名称 | ローテーション理由 | 周期 |
|---|---|---|---|
| KEY-NNN | <!-- 鍵名称 --> | <!-- 定期 / 漏洩疑惑 / 手動 --> | <!-- 90 日 / 365 日 / 2 年 --> |

## ローテーション手順記録

- 新鍵生成日時: YYYY-MM-DD HH:MM
- 新鍵 fingerprint: <!-- 鍵の fingerprint（SHA-256）-->
- 旧鍵 fingerprint: <!-- 旧鍵の fingerprint -->
- 猶予期間（移行期間）: YYYY-MM-DD 〜 YYYY-MM-DD
- 完全失効確認日時: YYYY-MM-DD HH:MM
- 旧鍵廃棄完了: YYYY-MM-DD HH:MM

## 影響確認

- [ ] JWT 認証が新鍵で正常動作することを確認
- [ ] 旧 JWT トークンの無効化を確認（猶予期間終了後）
- [ ] バックアップ暗号化が新鍵で動作することを確認（KEY-004 の場合）
- [ ] ハッシュチェーンへの影響なし（KEY-008 の場合は必須確認）

## 台帳更新記録

- [ ] 09_鍵ローテーション履歴テンプレ.md を更新した
- 更新日時: YYYY-MM-DD
```

---

### 3-4. db_migration（DB マイグレーション）

```markdown
## マイグレーション概要

- マイグレーションファイル名: migrations/YYYYMMDDHHMMSS_<説明>.sql
- 対象テーブル: <!-- TBL-NNN 一覧 -->
- 変更内容: <!-- カラム追加 / インデックス追加 / パーティション追加 等 -->

## 事前確認

- [ ] ロールバックスクリプト（.down.sql）を作成した
- [ ] STG 環境で apply → revert → apply の一往復を確認した
- [ ] アーカイブ対象データへの影響なし

## マイグレーション実施記録

- STG 適用日時: YYYY-MM-DD HH:MM
- 本番適用日時: YYYY-MM-DD HH:MM
- 適用後のスキーマ version_num: <!-- version_num -->
- 本番適用確認: PASS / FAIL

## ロールバック記録（ロールバック実施時のみ）

- ロールバック実施日時: YYYY-MM-DD HH:MM
- ロールバック理由: <!-- 理由 -->
- ロールバック後の version_num: <!-- version_num -->
```

---

## 4. status 遷移

```
open          → （02 章 PROB 分析完了）→ in_progress
in_progress   → （03 章 修正完了）→ completed
completed     → （04 章 レビュー完了）→ ready_for_deployment
              → （05 章 本番移行成功）→ closed
              → （05 章 ロールバック）→ rollback_completed
rollback_completed → （再修正 REC 起票）→ 新 REC-NNN が open
```

**本節で確定した方針**
- **status を `closed` にするのは本番移行成功確認（05 章 §3）の完了後のみとする。**
- **`rollback_completed` ステータスの REC-NNN は再修正の REC-NNN から `related_rec` で参照され、同一問題の追跡を可能にする。**
- **REC-NNN は作成後 7 年間保全する（ALCOA+ Enduring 原則・MNT-041）。**

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.2.a〜f（保守プロセス全タスク）
- 21 CFR Part 11（電子記録・電子署名）— 保守記録の 7 年保全・真正性の規制根拠
- ALCOA+ 原則（FDA Guidance）— 記録フィールド設計の根拠

### 関連
- [`../../90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)

---

## 版数履歴

| 版 | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | RyuheiKiso | 初版（8 種 record_type テンプレート確定）|
