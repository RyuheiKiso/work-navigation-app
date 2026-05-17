# 03_画面設計

IPA 共通フレーム 2013「2.4 ヒューマンインタフェース設計タスク」に対応する。

要件定義フェーズで確定した 35 画面（SCR-HA-001〜015・SCR-MA-001〜011・SCR-MC-001〜009）の遷移設計・ワイヤーフレーム・共通 UI コンポーネント・アクセシビリティ・多言語設計を確定する。

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
| `00_本書の位置づけと識別子規約.md` | IPA 対応・SCR/NAV/FRG/CMP 識別子確定 |
| `01_画面一覧と機能配置.md` | 35 画面の責務・機能・データ・FR 対応 |
| `02_画面遷移図（ハンディAPP）.md` | SCR-HA-001〜015 の遷移と TRN-NNN |
| `03_画面遷移図（マスタメンテAPP）.md` | SCR-MA-001〜011 の遷移 |
| `04_画面遷移図（管理コンソール）.md` | SCR-MC-001〜009 の遷移 |
| `05_共通UIコンポーネントとデザインシステム.md` | CMP カタログ・デザイントークン |
| `06_情報設計とナビゲーション原則.md` | 1画面1Step・現在地常時表示・監督呼ぶボタン |
| `07_ワイヤーフレーム（ハンディAPP・15画面）.md` | SCR-HA 論理レイアウト |
| `08_ワイヤーフレーム（マスタメンテAPP・11画面）.md` | SCR-MA 論理レイアウト |
| `09_ワイヤーフレーム（管理コンソール・9画面）.md` | SCR-MC 論理レイアウト |
| `10_アクセシビリティ方式設計.md` | WCAG 2.1 AA・色覚多様性・コントラスト |
| `11_多言語・国際化方式.md` | i18n・翻訳キー・やさしい日本語 |
| `12_入力デバイスと外部周辺機器UX.md` | スキャナ・BLE 計測器・カメラ UX |
| `13_画面×FR×UCマトリクス.md` | 35 × 86 × 22 の三軸クロス |
| `99_前提制約と本書が約束しないこと.md` | 個人別ダッシュボード・監視カメラを対象外と明示 |
| `img/` | 図ファイル格納 |

---

## 図一覧

| 図ファイル名（img/ 配下）| 内容 |
|---|---|
| `fig_des_screen_handy_flow.{drawio,svg}` | ハンディ APP 画面遷移 |
| `fig_des_screen_master_flow.{drawio,svg}` | マスタメンテ APP 画面遷移 |
| `fig_des_screen_console_flow.{drawio,svg}` | 管理コンソール画面遷移 |
| `fig_des_screen_wire_step_execution.{drawio,svg}` | Step 実行画面ワイヤフレーム |
| `fig_des_screen_wire_suspension.{drawio,svg}` | 中断画面ワイヤフレーム |
| `fig_des_screen_offline_state.{drawio,svg}` | オフライン状態の画面 |
| `fig_des_screen_matrix.{drawio,svg}` | 画面 × FR マトリクス |
