# GxP バリデーションテンプレート

> 対応 §: ロードマップ §4.9（OSS で空白の差別化軸）§12 §27（FMEA）§19.4
> 対象規格: FDA 21 CFR Part 11／EU GMP Annex 11／ICH Q9
> ライセンス: Apache-2.0（本リポジトリ）／CC BY 4.0（記入式テンプレ部）

§4.9「圧倒候補機能」の 1 つ「**GxP バリデーション支援テンプレ同梱（IQ／OQ／PQ）**」の実体。
OSS で IQ/OQ/PQ テンプレを同梱しているプロダクトは事実上空白であり、本ディレクトリは
本プロジェクトの差別化軸として導入企業のバリデーション工数を直接削減する。

## 含まれるテンプレ

| ファイル | 用途 | 所要時間目安 |
| --- | --- | --- |
| [`vmp.md`](./vmp.md) | バリデーションマスタープラン（VMP） | 1 日 |
| [`urs.md`](./urs.md) | ユーザ要件仕様（URS） | 半日 |
| [`fs.md`](./fs.md) | 機能仕様（FS） | 半日 |
| [`ds.md`](./ds.md) | 設計仕様（DS） | 半日 |
| [`risk-assessment.md`](./risk-assessment.md) | リスク評価（ICH Q9） | 1 日 |
| [`iq-protocol.md`](./iq-protocol.md) | インストールクオリフィケーション（IQ） | 半日／環境 |
| [`oq-protocol.md`](./oq-protocol.md) | オペレーショナルクオリフィケーション（OQ） | 1 日／環境 |
| [`pq-protocol.md`](./pq-protocol.md) | パフォーマンスクオリフィケーション（PQ） | 2 日／環境 |
| [`traceability-matrix.md`](./traceability-matrix.md) | 要求 ↔ 設計 ↔ 試験 トレース表 | 半日 |
| [`change-control.md`](./change-control.md) | 変更管理手順 | 半日 |
| [`periodic-review.md`](./periodic-review.md) | 定期レビュー手順 | 半年に 1 回 |

## ライセンス・帰属

- 文書テンプレ自体: **CC BY 4.0**（自由改変・商用利用可、出典明記）
- 記入後の **導入企業のバリデーションパッケージ** は導入企業が著作権を持つ
- 本テンプレを参照／引用した場合は `© 2026 work-navigation-app contributors / CC BY 4.0` を明記

## 運用

1. 導入企業は本ディレクトリのテンプレをコピーし、自社用語に合わせて記入する
2. §22.1 改訂サイクル（半期）でテンプレの追従を検証
3. work-navigation-app の機能変更時は §32.4 廃止予告プロセスに従い、本テンプレも更新

## 受入観点（§12 規制適合）

- 各テンプレの「対応規格条項」が明記されていること
- 21 CFR Part 11 / GMP Annex 11 に対応する条項にトレース可能であること
- IQ/OQ/PQ の各プロトコル雛形が、本アプリの実機構成（PostgreSQL／Tauri／WASM ランタイム）に整合していること
