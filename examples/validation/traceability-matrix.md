# トレーサビリティマトリクス（要求 ↔ 設計 ↔ 試験）

> 対応規格: GMP Annex 11 §4.4／FDA 21 CFR Part 11 §11.10
> 対応 §: ロードマップ §24

URS（要求）→ FS（機能仕様）→ DS（設計仕様）→ IQ／OQ／PQ（試験）の双方向トレースを記録する。
**孤児行（試験ゼロの要求／要求ゼロの試験）は §22.4 是正フロー対象**。

## マトリクス（記入例）

| URS ID | 要求 | FS ID | DS ID | 試験 ID | 状態 |
| --- | --- | --- | --- | --- | --- |
| URS-FUNC-01 | ID＋パスワード認証 | FS-AUTH-01 | DS-AUTH-01（Argon2id+HMAC-SHA256） | OQ-AUTH-01〜05 | OK |
| URS-FUNC-02 | 作業手順のナビ表示 | FS-NAV-01 | DS-NAV-01（HSM＋6 基本機能） | OQ-NAV-01 | OK |
| URS-FUNC-03 | 完了条件 写真／人手 | FS-COMP-01 | DS-COMP-01（CompletionCriteria） | OQ-COMP-01 | OK |
| URS-FUNC-04 | 監査ログ改ざん不可 | FS-AUDIT-01 | DS-AUDIT-01（DB トリガ+INV-07） | OQ-AUDIT-01〜03 | OK |
| URS-NFR-01 | 画面遷移 ≤ 200ms | FS-PERF-01 | DS-PERF-01（§5.2） | OQ-PERF-01 | OK |
| URS-NFR-02 | オフライン 7 日 | FS-OFFLINE-01 | DS-OFFLINE-01（§10.5.1） | OQ-AUTH-04 | OK |
| URS-NFR-05 | WCAG 2.2 AA | FS-A11Y-01 | DS-A11Y-01（§11.2） | OQ-A11Y-01〜03 | OK |

## 受入観点

- **すべての URS** が FS／DS／試験と紐付く（孤児な要求 0）
- **すべての試験** が URS／FS と紐付く（孤児な試験 0）
- 孤児検出時は §22.4 是正フロー起票
