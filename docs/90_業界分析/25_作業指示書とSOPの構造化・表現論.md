# 作業指示書とSOPの構造化・表現論

## 1. 問題の所在

作業ナビゲーションアプリの価値は、提示するコンテンツの質に直結する。どれほど優れた UI や記録機能があっても、手順書 (SOP: Standard Operating Procedure) の内容が曖昧・冗長・理解困難であれば、作業者はナビを無視するか、誤った手順を踏む。本章は **SOP という文書自体の設計論** を扱い、テキスト・図・動画・AR それぞれの適切な使い分け、構造化の手法、版管理の要件を整理する。

[19_電子チェックリストと手順遵守の科学](./19_電子チェックリストと手順遵守の科学.md) が「手順をどう遵守させるか」を扱うのに対し、本章は「手順書をどう書き設計するか」という上流の問いに答える。[23_作業訓練設計とインストラクショナルデザイン](./23_作業訓練設計とインストラクショナルデザイン.md) の ADDIE プロセスがカリキュラム全体を対象とするのに対し、本章は **個々の手順ドキュメント** の構造と表現に特化する。

## 2. SOP の系譜

### 2.1 軍事手順書から産業 SOP へ

標準的な手順書の起源は軍事的な **OPORD (Operation Order)** と **TTP (Tactics, Techniques, and Procedures)** に求められる。WWII 期の TWI (Training Within Industry) プログラムは、製造現場向けにこの考え方を転用し、**Job Instruction (JI)** という 4 ステップ指導法 (準備・提示・実演・確認) を普及させた ([26_多能工とスキル管理・作業者資格](./26_多能工とスキル管理・作業者資格.md) で詳述)。

### 2.2 GMP SOP と規制要求

医薬品・医療機器業界では **GMP (Good Manufacturing Practice)** が SOP の形式・管理方法まで規定する。FDA は SOP に以下を求める:

- 作業手順の目的・スコープ・責任者の明記
- 変更履歴と承認署名 (電子署名可、21 CFR Part 11 準拠)
- 定期レビュー周期 (通常 2–3 年)
- 旧版の廃棄・アーカイブ手続き

### 2.3 ISO 9001 の文書化要求

ISO 9001:2015 は「文書化した情報 (Documented Information)」を「適切な粒度で維持すること」と要求しているが、形式は規定しない。これは 2008 年版の「文書化された手順」という硬直的な要求からの転換であり、**電子手順書・動画手順書・デジタルワークフローを正式に認める** 方向への解釈が定着している。

## 3. 構造化オーサリング

### 3.1 Topic-based Authoring と DITA

**DITA (Darwin Information Typing Architecture)** は OASIS が策定した XML ベースの構造化文書標準 (OASIS, 2005)。コンテンツを **Topic** (独立した最小単位) に分解し、map ファイルで組み合わせる。Topic の 3 種:

| Topic 型 | 目的 | 製造手順への対応 |
|---|---|---|
| Task | 手順 (ステップリスト) | 作業ステップ本体 |
| Concept | 概念説明 | 部品の役割・背景知識 |
| Reference | 参照情報 | 規格値・許容誤差表 |

モジュール化により、同一 Step を複数の手順書で **再利用** でき、改訂コストを削減できる。

### 3.2 S1000D — 防衛・航空宇宙向け標準

**S1000D** は NATO が策定した技術文書規格 (現行 Issue 6.0)。航空機整備マニュアル・防衛装備品の技術手順書に広く使われる。データモジュール (Data Module) 単位で管理し、Common Source DataBase (CSDB) でバージョン・配信先を一元管理する。製造現場の SOP としては過重だが、設計思想 (再利用可能なモジュール + メタデータ) は参考になる。

### 3.3 シンプルな構造化の原則

DITA や S1000D を採用しない場合でも、以下の構造原則は適用できる:

```
SOP 文書の推奨構造
├── メタデータ
│   ├── 文書番号・版番号・有効日
│   ├── 対象工程・対象製品
│   └── 作成者・承認者・レビュー日
├── 目的とスコープ
├── 関連文書・前提条件
├── 必要資材・工具・設備
├── 手順 (Task Topics)
│   ├── ステップ番号
│   ├── 行為 (Action: 動詞 + 目的語)
│   ├── 注意・警告 (Warning/Caution/Note)
│   └── 合否判定基準 (Accept/Reject criteria)
├── 記録事項
└── 改訂履歴
```

## 4. Carroll の Minimalism

**John M. Carroll** は *The Nurnberg Funnel: Designing Minimalist Instruction for Practical Computer Skill* (1990) でミニマリズム設計を提唱した。元来はコンピュータ利用マニュアルを対象としていたが、製造手順書設計に転用できる。

### 4.1 4 原則

| 原則 | 内容 | 製造手順への適用 |
|---|---|---|
| 行動志向 (Action-oriented) | ユーザーが読むより実行することを優先 | 「〇〇することによって」ではなく「〇〇せよ」 |
| エラーを学習機会に | エラー発生を予測し、対処手順を埋め込む | 「〇〇となった場合は △△ ステップへ」 |
| 最小化 | 必要な情報だけ。説明の過剰を避ける | 背景説明はリンク先へ、本文はステップのみ |
| 足場 (Scaffolding) | 学習進行に応じて情報を追加 | 熟練者向けの略式版と初心者向け詳細版を分ける |

### 4.2 手順書の典型的過剰

製造現場の手順書が長くなる原因は多くが「読まれないコンテンツ」の蓄積:

- 法令・規格の引用全文 (リンクに置き換える)
- 「〜であることに注意する」型の抽象的注意書き
- 担当者の過去トラブル対策が無制限に追加されたアドオン

Minimalism に基づくレビューで 30–50% の削減が可能なケースが多い。

## 5. Mayer のマルチメディア学習認知理論

**Richard Mayer** は *Multimedia Learning* (2001, 2nd ed. 2009) で、学習者の認知系に基づくマルチメディア設計原則を提唱した。ダブルチャンネル仮定 (視覚系・聴覚系) と限定容量仮定 (ワーキングメモリ) に基づく。

### 5.1 主要原則と製造手順への含意

| 原則 | 定義 | 含意 |
|---|---|---|
| Modality 効果 | テキスト+図より音声+図が有効 | 動画ナレーションは字幕より優位 |
| Redundancy 効果 | 音声+図への画面テキスト追加は妨害 | 動画に同文テキストを重ねない |
| Coherence 原則 | 無関係な音楽・装飾は認知負荷を増す | シンプルな図解を優先 |
| Segmenting 原則 | 長いコンテンツは区切りを入れて学習者ペース化 | ステップごとに次へ進むUI |
| Contiguity 原則 | テキストと対応図は近接配置 | 説明文は対応する図の隣 |

### 5.2 動画手順書の適用限界

動画は曖昧さが少なく強力だが:

- 更新コストが高い (製品仕様変更のたびに再撮影)
- 検索性が低い (特定のステップを探しにくい)
- 設備ノイズ環境では音声が届かない (→ 字幕必須)
- 外国人作業者・言語多様性に弱い

静止画 + テキストと動画を **ステップの複雑さ** に応じて使い分けるハイブリッドが現実的である。

## 6. Job Aid 設計

**Job Aid (作業支援ツール)** は「人が記憶しなくてよいよう、作業の時点で情報・手順を提供するツール」(Rossett & Schafer, *Job Aids and Performance Support*, 2007)。チェックリスト・フローチャート・図解ポスター・タブレットナビゲーション画面はすべて Job Aid の一形態である。

### 6.1 EPSS (Electronic Performance Support System)

**EPSS** (Gery, 1991) は「作業中に必要な情報・ガイダンス・学習機会をその場で提供するシステム」。作業ナビゲーションアプリは EPSS の典型例である。Gery はサポートの深さで 3 種を区別した:

```
External EPSS  : 作業とは別の参照ツール (PDF マニュアル)
Extrinsic EPSS : 作業ツールに統合 (ポップアップヒント)
Intrinsic EPSS : ガイダンスが作業フローに溶け込んでいる (ナビ本体)
```

本アプリが目指すのは **Intrinsic EPSS** — 手順提示と記録入力が一体化し、別途マニュアルを参照しなくて済む状態である。

### 6.2 手順提示モードの選択肢

| モード | 特徴 | 向く作業 |
|---|---|---|
| Linear (線形) | ステップを 1 つずつ順に提示 | 毎回同じ順序の組立・点検 |
| Decision Tree | 分岐条件に応じて次ステップが変わる | 診断・異常対応・機種切り替え |
| Reference | 全ステップ一覧参照 (熟練者向け) | 頻繁に実施して記憶している工程 |
| Parallel | 複数ステップを並行して表示 | 左右対称工程・2 人作業 |

## 7. 版管理と有効性確認

### 7.1 版管理の基本

ISO 9001:2015 条項 7.5 は「文書化した情報」の管理として以下を要求する:

- **識別** — 番号・タイトル・版番号・日付での一意識別
- **入手可能性** — 使用時点で適切な版が参照できること
- **保護** — 改ざん・意図しない変更の防止
- **配布・アクセス・検索・使用** — 権限に応じた参照制御
- **保管・保存** — 可読性の維持
- **変更管理** — 変更内容・変更者・変更日の記録
- **保管期間と廃棄** — 旧版の不使用状態への移行

### 7.2 Effective Date 管理

製造現場で重要なのは「**いつから新版が有効か**」の明確化。

- 製品ロットとの紐付け: 新版手順は「ロット番号 XXXXXX 以降に適用」
- 教育完了確認 (Training Completion Check): 新版で作業する前に関連者が訓練を完了したことの記録
- 経過措置期間: 旧版と新版が混在する期間を設定しない (混乱の原因)

### 7.3 旧版廃棄と電子管理

電子システムでは「旧版が誤って参照されないこと」が特に重要:

- 配信システムが常に最新版を表示 (Pull 型)
- 旧版は閲覧可能だが「廃止」ラベルを明示
- 印刷を禁止 or 印刷時に「管理外文書」ウォーターマークを自動付与

## 8. 批判と限界

### 8.1 文書化の限界 — 暗黙知問題

[05_暗黙知と技能伝承](./05_暗黙知と技能伝承.md) で示した Polanyi の命題「我々は語れる以上のことを知っている」は SOP 設計の根本的な限界を示す。熟練工が「なんとなく締め付けが足りない感じ」で感知するトルク感覚は文書化不可能であり、SOP に書ける内容は **形式知化できた部分のみ** である。

職能水準が上がるほど SOP は「出発点」に過ぎなくなり、文書化そのものの価値は逓減する。

### 8.2 過度な構造化のオーバーヘッド

DITA 等の厳格な構造化は、初期導入コストと継続的な維持コストが高い。変化の速い製品ライン (試作・小ロット品種) では構造化オーサリングのオーバーヘッドが内容更新の速度を妨げる。**構造化の度合いは、手順書の量・更新頻度・再利用ニーズ** に照らして選択する必要がある。

### 8.3 Carroll Minimalism のリスク

情報の最小化は、熟練者には効率的だが、初学者が「背景知識なしに手順だけ見ている」状態に陥る危険もある。特に安全上重要な手順では、注意書きを削りすぎることで重大なエラーを招く場合がある。Minimalism は「不要な情報の削減」であり「必要な安全情報の削減」ではないことを区別する。

---

## 参考文献

- Carroll, J. M. *The Nurnberg Funnel: Designing Minimalist Instruction for Practical Computer Skill*. MIT Press, 1990.
- Mayer, R. E. *Multimedia Learning* (2nd ed.). Cambridge University Press, 2009.
- Gery, G. *Electronic Performance Support Systems: How and Why to Remake the Workplace Through the Strategic Application of Technology*. Weingarten Publications, 1991.
- Rossett, A., & Schafer, L. *Job Aids and Performance Support: Moving From Knowledge in the Classroom to Knowledge Everywhere*. Pfeiffer, 2007.
- OASIS. *Darwin Information Typing Architecture (DITA) Version 1.3*. OASIS Standard, 2015.
- ASD-STE100. *Simplified Technical English*. ASD, Issue 8, 2021.
- ISO 9001:2015 *Quality Management Systems — Requirements*. ISO, 2015.
- Hackos, J. T. *Information Development: Managing Your Documentation Projects, Portfolio, and People*. Wiley, 2007.
- Sweller, J., van Merriënboer, J. J. G., & Paas, F. "Cognitive architecture and instructional design." *Educational Psychology Review*, 10(3), 1998.
- International Aerospace Quality Group. *AS9100 Rev D: Quality Management Systems — Requirements for Aviation, Space, and Defense Organizations*. SAE International, 2016.
