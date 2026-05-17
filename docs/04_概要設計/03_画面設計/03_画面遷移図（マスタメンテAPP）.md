# 03 画面遷移図（マスタメンテ APP）

本章の責務は、マスタメンテ APP（SCR-MA-001〜011）の画面遷移（TRN-NNN）を確定することである。

図: fig_des_screen_master_flow（img/ 配下）を参照。

---

## 1. 画面遷移一覧（TRN - マスタメンテ APP 部分）

| TRN-ID | 発生元 SCR | 遷移先 SCR | トリガー | ガード条件 | 副作用 API |
|---|---|---|---|---|---|
| TRN-025 | （ログイン）| SCR-MA-001 | ログイン成功（master_admin ロール）| master_admin ロール | API-auth-001 |
| TRN-026 | SCR-MA-001 | SCR-MA-004 | SOP 編集ボタン | — | API-master-002 |
| TRN-027 | SCR-MA-001 | SCR-MA-002 | オペレーション一覧 | — | — |
| TRN-028 | SCR-MA-002 | SCR-MA-001 | 戻る | — | — |
| TRN-029 | SCR-MA-004 | SCR-MA-005 | CSV/Excel インポートボタン | — | — |
| TRN-030 | SCR-MA-005 | SCR-MA-004 | インポート完了 | プレビュー確認済み | API-master-002 |
| TRN-031 | SCR-MA-004 | SCR-MA-006 | プレビューボタン | — | — |
| TRN-032 | SCR-MA-006 | SCR-MA-004 | 閉じる | — | — |
| TRN-033 | SCR-MA-004 | SCR-MA-007 | レビュー依頼ボタン | Draft 状態 | API-master-004 |
| TRN-034 | SCR-MA-007 | SCR-MA-008 | 品質担当に通知 → 承認画面（quality_admin）| quality_admin ロール | — |
| TRN-035 | SCR-MA-008 | SCR-MA-009 | 電子サイン完了 | sign_id 取得済み | API-master-005 |
| TRN-036 | SCR-MA-009 | SCR-MA-001 | 公開完了 | — | — |
| TRN-037 | SCR-MA-004 | SCR-MA-010 | バージョン差分ボタン | ≥2 版存在 | — |
| TRN-038 | SCR-MA-010 | SCR-MA-004 | 閉じる | — | — |
| TRN-039 | SCR-MA-001 | SCR-MA-011 | 廃止処理ボタン | Published 状態 | API-master-007（dry-run）|
| TRN-040 | SCR-MA-011 | SCR-MA-001 | 廃止確定（電子サイン付き）| sign_id 取得済み | API-master-006 |

付録/99 採番台帳: 次採番値 **TRN-041**。

---

## 2. 承認フローの設計

マスタ承認は master_admin → quality_admin の二段階電子サインフローを経る。

```
master_admin（SCR-MA-007）
  → レビュー依頼送付（API-master-004）
    → quality_admin がログインして SCR-MA-008 を開く
      → dry-run で影響確認（API-master-007）
        → 電子サイン付き承認（API-master-005・API-electronic-signs-001）
          → SCR-MA-009 で公開日付を指定
            → API で公開実行
```

この承認フローは UC-016 のシーケンス図（SEQ-005）として fig_des_seq_master_publish に図示する。

---

**本節で確定した方針**
- **マスタメンテ APP の TRN-025〜040 を確定し、11 画面（SCR-MA）間の遷移・承認フロー・廃止フローを明示した。**
- **承認フローはシーケンス図 SEQ-005 と対応し、quality_admin 専用の承認画面（SCR-MA-008）への遷移を RBAC ガードで保護する設計を確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
