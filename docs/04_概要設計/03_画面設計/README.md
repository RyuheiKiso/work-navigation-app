# 03_画面設計

IPA 共通フレーム 2013「2.4 ヒューマンインタフェース設計タスク」に対応する。

要件定義フェーズで確定した 35 画面（SCR-HA-001〜015・SCR-MA-001〜011・SCR-MC-001〜009）の遷移設計・ワイヤーフレーム・デザインシステム・アクセシビリティ・多言語設計を確定する。デザイントーン「テック×洗練（Precision SaaS）」のもと、ブランドと機能の両立を目指す。

---

## 担当する上流要件

| 種別 | 範囲 |
|---|---|
| 画面 SCR | SCR-HA/MA/MC 全 35 件 |
| FR-NV | FR-NV 全 13 件（ナビゲーション）|
| FR-UI | FR-UI 全 11 件（UI/UX）|
| FR-ST | FR-ST 全 12 件（中断・再開 UI）|
| NFR-UX | NFR-UX 全 43 件（ユーザビリティ・A11y）|

---

## 章構成

| ファイル | 目的 |
|---|---|
| `README.md` | 本書 |
| `00_本書の位置づけと識別子規約.md` | IPA 対応・識別子（SCR/TRN/FRG/CMP-CMN）確定・UI不変原則10条 |
| `01_画面一覧と機能配置.md` | 35 画面の責務・機能・データ・FR 対応 |
| `02_画面遷移図（ハンディAPP）.md` | SCR-HA-001〜015 の遷移と TRN-NNN |
| `03_画面遷移図（マスタメンテAPP）.md` | SCR-MA-001〜011 の遷移 |
| `04_画面遷移図（管理コンソール）.md` | SCR-MC-001〜009 の遷移 |
| **`05A_ブランドアイデンティティとデザイン原則.md`** | **WorkNav ブランド・キーカラー・ロゴ・モチーフ・ボイス&トーン（新規）** |
| `05_共通UIコンポーネントとデザインシステム.md` | デザイントークン全網羅・FRG-001〜040 汎用部品・CMP カタログ・マイクロインタラクション |
| `06_情報設計とナビゲーション原則.md` | 1画面1Step・フィードバック視覚仕様・トランジション規定・三角配置禁止 |
| `07_ワイヤーフレーム（ハンディAPP・15画面分）.md` | SCR-HA ハイファイ風ワイヤーフレーム（代表3画面詳細含む）|
| `08_ワイヤーフレーム（マスタメンテAPP・11画面分）.md` | SCR-MA ハイファイ風ワイヤーフレーム（代表2画面詳細含む）|
| `09_ワイヤーフレーム（管理コンソール・9画面分）.md` | SCR-MC ハイファイ風ワイヤーフレーム（代表3画面詳細含む）|
| `10_アクセシビリティ方式設計.md` | WCAG 2.1 AA・コントラスト40ペア・フォーカスリング意匠・両立原則 |
| `11_多言語・国際化方式.md` | i18n・翻訳キー・やさしい日本語・フォントフォールバック |
| `12_入力デバイスと外部周辺機器UX.md` | スキャナ・BLE 計測器・カメラ UX・サウンド/ハプティック仕様 |
| `13_画面×FR×UCマトリクス.md` | 35 × 86 × 22 の三軸クロス |
| `99_前提制約と本書が約束しないこと.md` | 担保事項16項目・対象外7項目・drawio一括検証手順 |
| `img/` | 図ファイル格納（drawio + SVG）|

---

## 図一覧

### 画面遷移・ナビゲーション

| 図ファイル名 | 内容 |
|---|---|
| `fig_des_screen_handy_flow.{drawio,svg}` | ハンディ APP 画面遷移 |
| `fig_des_screen_master_flow.{drawio,svg}` | マスタメンテ APP 画面遷移 |
| `fig_des_screen_console_flow.{drawio,svg}` | 管理コンソール画面遷移 |

### ブランド・デザインシステム（新規 18 枚）

| 図ファイル名 | 内容 |
|---|---|
| `fig_des_brand_logomark.{drawio,svg}` | ロゴシンボル（テセラ 6 片）幾何構造 |
| `fig_des_brand_logo_variants.{drawio,svg}` | ロゴバリアント（カラー/モノクロ/クリアスペース）|
| `fig_des_brand_motif.{drawio,svg}` | モチーフボキャブラリ |
| `fig_des_tokens_color_scale.{drawio,svg}` | カラートークン階調 |
| `fig_des_tokens_color_dark.{drawio,svg}` | ダーク/nightShift トークン対応表 |
| `fig_des_tokens_type_scale.{drawio,svg}` | タイポグラフィスケール |
| `fig_des_tokens_radius_space.{drawio,svg}` | 角丸 + スペーシング |
| `fig_des_tokens_elevation.{drawio,svg}` | エレベーション shadow 0-5 |
| `fig_des_tokens_motion.{drawio,svg}` | duration / easing 曲線 |
| `fig_des_components_catalog_buttons.{drawio,svg}` | Button / IconButton カタログ |
| `fig_des_components_catalog_forms.{drawio,svg}` | Input / Select / Checkbox 等カタログ |
| `fig_des_components_catalog_feedback.{drawio,svg}` | Toast / Snackbar / Banner 等カタログ |
| `fig_des_components_catalog_navigation.{drawio,svg}` | Tabs / Pagination / Breadcrumb 等カタログ |
| `fig_des_components_catalog_dataviz.{drawio,svg}` | DataTable / Sparkline / SliGauge 等カタログ |
| `fig_des_focus_ring_spec.{drawio,svg}` | フォーカスリング寸法・コントラスト |
| `fig_des_input_sound_haptic_matrix.{drawio,svg}` | イベント × 音 × 振動 × 視覚対応表 |
| `fig_des_input_ble_state.{drawio,svg}` | BLE 接続 5 状態ビジュアル |
| `fig_des_a11y_contrast_matrix.{drawio,svg}` | AA/AAA コントラスト検証マトリクス |

### ワイヤーフレーム — ハンディ APP（HA）

| 図ファイル名 | 内容 |
|---|---|
| `fig_des_screen_ha_001.{drawio,svg}` | SCR-HA-001 ログイン |
| `fig_des_screen_ha_002.{drawio,svg}` | SCR-HA-002 ホーム/作業選択 ★ハイファイ |
| `fig_des_screen_ha_003.{drawio,svg}` | SCR-HA-003 QR スキャン起動 |
| `fig_des_screen_ha_004.{drawio,svg}` | SCR-HA-004 SOP 一覧 |
| `fig_des_screen_ha_005.{drawio,svg}` | SCR-HA-005 Step 実行（標準）★ハイファイ |
| `fig_des_screen_ha_006.{drawio,svg}` | SCR-HA-006 Step 実行（条件分岐）|
| `fig_des_screen_ha_007.{drawio,svg}` | SCR-HA-007 Step 実行（カスタム入力）|
| `fig_des_screen_ha_008.{drawio,svg}` | SCR-HA-008 写真撮影 |
| `fig_des_screen_ha_009.{drawio,svg}` | SCR-HA-009 測定値入力 |
| `fig_des_screen_ha_010.{drawio,svg}` | SCR-HA-010 電子サイン入力 |
| `fig_des_screen_ha_011.{drawio,svg}` | SCR-HA-011 中断 |
| `fig_des_screen_ha_012.{drawio,svg}` | SCR-HA-012 再開 |
| `fig_des_screen_ha_013.{drawio,svg}` | SCR-HA-013 アンドン発報 ★ハイファイ |
| `fig_des_screen_ha_014.{drawio,svg}` | SCR-HA-014 不適合登録 |
| `fig_des_screen_ha_015.{drawio,svg}` | SCR-HA-015 設定 |

### ワイヤーフレーム — マスタメンテ APP（MA）

| 図ファイル名 | 内容 |
|---|---|
| `fig_des_screen_ma_001.{drawio,svg}` | SCR-MA-001 プロセス一覧 |
| `fig_des_screen_ma_002.{drawio,svg}` | SCR-MA-002 オペレーション一覧 |
| `fig_des_screen_ma_003.{drawio,svg}` | SCR-MA-003 製品一覧 |
| `fig_des_screen_ma_004.{drawio,svg}` | SCR-MA-004 SOP 編集 ★ハイファイ |
| `fig_des_screen_ma_005.{drawio,svg}` | SCR-MA-005 SOP インポート |
| `fig_des_screen_ma_006.{drawio,svg}` | SCR-MA-006 SOP プレビュー |
| `fig_des_screen_ma_007.{drawio,svg}` | SCR-MA-007 レビュー依頼 |
| `fig_des_screen_ma_008.{drawio,svg}` | SCR-MA-008 承認サイン ★ハイファイ |
| `fig_des_screen_ma_009.{drawio,svg}` | SCR-MA-009 公開設定 |
| `fig_des_screen_ma_010.{drawio,svg}` | SCR-MA-010 バージョン差分 |
| `fig_des_screen_ma_011.{drawio,svg}` | SCR-MA-011 廃止処理 |

### ワイヤーフレーム — 管理コンソール（MC）

| 図ファイル名 | 内容 |
|---|---|
| `fig_des_screen_mc_001.{drawio,svg}` | SCR-MC-001 運用ダッシュボード ★ハイファイ（新規）|
| `fig_des_screen_mc_002.{drawio,svg}` | SCR-MC-002 ユーザー管理（新規）|
| `fig_des_screen_mc_003.{drawio,svg}` | SCR-MC-003 ロール/スキル管理 ★ハイファイ（新規）|
| `fig_des_screen_mc_004.{drawio,svg}` | SCR-MC-004 監査ログ閲覧 ★ハイファイ（新規）|
| `fig_des_screen_mc_005.{drawio,svg}` | SCR-MC-005 XES エクスポート（新規）|
| `fig_des_screen_mc_006.{drawio,svg}` | SCR-MC-006 バックアップ状況（新規）|
| `fig_des_screen_mc_007.{drawio,svg}` | SCR-MC-007 DLQ 監視（新規）|
| `fig_des_screen_mc_008.{drawio,svg}` | SCR-MC-008 ハッシュチェーン検証（新規）|
| `fig_des_screen_mc_009.{drawio,svg}` | SCR-MC-009 帳票出力（新規）|

> ★ハイファイ: 実カラートークン・角丸・elevation・アイコンを反映した高忠実度ワイヤーフレーム
> 新規: MC 図面は本改訂で新規作成（旧来は未生成）
