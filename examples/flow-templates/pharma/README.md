# 医薬品 業界フローテンプレート

> 対応 §: ロードマップ §10.2.1 §3.1.5.4 §12（FDA 21 CFR Part 11／EU GMP Annex 11）
> 対象規格: GMP Annex 11／21 CFR Part 11 電子記録・電子署名／ICH Q9 リスク管理
> ライセンス: Apache-2.0

## 概要

GxP 環境（医薬品製造）向けの最小フローテンプレート。
バリデート済み手順の厳密実行・電子記録・電子署名を §3.1.1 完了条件で表現する。

## 同梱ファイル

| ファイル | 用途 |
| --- | --- |
| `batch-record.yaml` | バッチ製造記録（ISA-88 batch model 整合） |
| `change-control.yaml` | 変更管理（GMP Annex 11 §10） |
| `oos-investigation.yaml` | OOS（Out of Specification）調査フロー |

## 業界語感（§3.1.5.4 整合）

- 「作業」= バリデート済み **手順**（validated procedure）
- 「工程」= ユニットオペレーション（unit operation、ISA-88）
- 「手順」= SOP の各ステップ
- 「動作」= 計量・混合・充填 などの個別操作

## 必須機能との対応

| 規格項目 | 本アプリの該当機能 |
| --- | --- |
| 21 CFR Part 11 §11.10(e) 監査証跡 | §11.4.1 監査ログ追記不変（INV-07） |
| 21 CFR Part 11 §11.50 電子署名 | §10.5 電子署名（拡張認証アドオン経由） |
| GMP Annex 11 §6 監査証跡 | 同上 |
| GMP Annex 11 §9 監査と検査 | §22 改訂サイクル |
| バリデーション IQ／OQ／PQ | `examples/validation/`（次セッション） |
