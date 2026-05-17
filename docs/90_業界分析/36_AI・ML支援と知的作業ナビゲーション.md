# AI・ML支援と知的作業ナビゲーション

## 1. 製造業における AI・ML の位置づけ

製造業への AI・機械学習 (ML) の統合は、Industry 4.0 の核心的要素として位置づけられている。Kagermann らが提唱した Industry 4.0 コンセプト (Kagermann, Wahlster & Helbig, 2013) は、サイバー-フィジカルシステム (CPS) による製造プロセスのデジタル化・インテリジェント化を基本思想とし、センサデータの収集、リアルタイム解析、自律的意思決定の連鎖を製造 AI の実装フレームワークとして確立した。この枠組みにおいて、作業支援 AI は人間オペレーターと生産システムを媒介する認知補助装置として機能する。

製造 AI の応用領域は大きく四つに分類できる。第一は予知保全 (Predictive Maintenance) であり、設備センサデータから故障前兆を検出し、計画外停止を防ぐ。第二は品質検査 (Quality Inspection) であり、画像認識や振動解析により不良品を自動検出する。第三はプロセス最適化 (Process Optimization) であり、生産条件のリアルタイム調整による歩留まり改善を目指す。第四は作業支援 (Work Assistance) であり、本アプリが対象とする領域として、作業員へのナビゲーション・推奨・異常警告を提供する (Lee, Bagheri & Kao, 2015)。

市場規模については、McKinsey Global Institute (2022) が製造業における AI の年間経済価値を 1.4〜3.3 兆ドルと推計しており、World Economic Forum (2023) は「Manufacturing AI」の採用企業が 5 年間で 50% 以上のコスト削減を報告していると指摘する。ただしこれらの推計は方法論上の一貫性に課題があり、批判的に参照する必要がある (後述)。国内では経済産業省・IPA が「AI 利活用ガイドライン」(2019) を公表し、製造 AI 導入の指針を示している。

本アプリの文脈では、AI・ML は作業ナビゲーションを受動的なステップ表示から、作業者の状態・履歴・環境を考慮した動的な支援へと進化させる技術基盤である。ただし、製造現場における AI の実装は技術的実現可能性のみならず、規制要件、データ品質、作業者の信頼・受容性など多面的な評価が必要であり、段階的かつ慎重な導入アプローチが求められる。

## 2. 異常検知の手法

異常検知は製造 AI の最も成熟した応用領域の一つであり、統計的手法から深層学習まで幅広い手法が実用化されている。本節ではその体系を概観し、作業ナビゲーションへの適用可能性を検討する。

統計的プロセス管理 (SPC) に基づく手法は、製造品質管理の基礎として確立された最も古典的なアプローチである。Shewhart の X̄-R 管理図 (Shewhart, 1931) は工程平均と範囲の変動を監視し、Western Electric ルールに基づく異常判定を行う。CUSUM (累積和管理図) と EWMA (指数重み付き移動平均) は、小さな平均シフトの検出に優れ、Shewhart 管理図が検出困難な微小変化を捉える (Montgomery, 2020)。これらの手法は計算コストが低く、解釈が容易で、製造現場での受容性が高いが、多変量・非線形な工程条件への対応が困難である。

機械学習ベースの異常検知手法は、教師なし学習を中心に発展している。Isolation Forest (Liu, Ting & Zhou, 2008) は各データ点の「孤立させやすさ」を分離木の深さで定量化し、外れ値を効率的に検出する。One-Class SVM (OCSVM) は正常データのみで訓練し、新たなデータが正常分布から外れているかを判定する (Schölkopf et al., 2001)。Autoencoder は入力を低次元表現に圧縮・再構築するニューラルネットワークであり、再構成誤差が大きいデータを異常として検出する。LSTM Autoencoder は時系列データへの対応に優れ、製造プロセスの時間依存パターンを学習する (Malhotra et al., 2016)。

画像認識を用いた外観検査は、近年最も急速に普及した製造 AI の応用例である。CNN アーキテクチャ (LeCun et al., 1998 を起源とする) は製造欠陥の視覚的特徴を自動的に抽出し、ResNet (He et al., 2016)・EfficientNet (Tan & Le, 2019)・YOLOv8 (Ultralytics, 2023) などの発展により、小型デバイス上での高精度・低遅延の欠陥検出が実現されている。工場現場でのエッジ AI 実装では TensorFlow Lite、ONNX Runtime、MediaPipe などの軽量推論エンジンが活用される。

振動・音響解析は設備の異常状態を非侵襲的に検出する手法であり、FFT (高速フーリエ変換) によって時系列データを周波数域に変換し、特定周波数成分の変化から軸受摩耗や不平衡を検出する。Mel Spectrogram + CNN アプローチは音響信号の時間-周波数表現に対して画像認識を適用し、設備異音の自動分類に有効である (Koizumi et al., 2020)。

## 3. 推奨次アクション (Prescriptive Analytics)

製造 AI の進化は「何が起きているか (Descriptive)」→「なぜ起きたか (Diagnostic)」→「次に何が起きるか (Predictive)」→「何をすべきか (Prescriptive)」という四段階のアナリティクス成熟度モデルに沿って理解される (Lepenioti et al., 2020)。作業ナビゲーションの文脈では、Prescriptive Analytics が最も直接的な価値を提供するが、その実装難易度は最も高い。

ルールベースの専門家システム (Rule-Based Expert System) は Prescriptive Analytics の最もシンプルな形態であり、「条件 A かつ条件 B ならば行動 C を推奨する」という IF-THEN ルールの集合として実装される。製造現場での経験的知識を明示的にルール化できる領域では高い信頼性と説明可能性を持つ。一方、条件の組み合わせが増加するにつれてルールの管理が複雑化し、未経験の状況への対応が困難になるという限界がある (Jackson, 1999)。

コンテキスト対応推奨は、現在の作業状態、過去のエラー履歴、作業者のスキルレベルを組み合わせて動的に推奨内容を生成する高度なアプローチである。例えば、特定の工程で過去に複数回エラーを発生させた作業者に対しては、通常の作業手順に加えて注意喚起と詳細ガイダンスを自動的に追加表示する。このアプローチは 26 章 (多能工・スキル管理) の知識表現と統合することで、より的確な個別化支援を実現できる。

強化学習 (Reinforcement Learning) の製造応用は研究段階にあるが、シミュレーション環境での最適制御政策学習に成果が出つつある。DeepMind の AlphaCode や AlphaFold の成功を受け、製造工程の逐次的意思決定問題への RL 適用が試みられているが、実際の製造環境でのオンライン学習は安全性・信頼性の観点から慎重なアプローチが必要である (Waschneck et al., 2018)。

## 4. LLM (Large Language Model) と作業支援

大規模言語モデル (LLM) の製造業への適用は 2023 年以降急速に議論が拡大している。GPT-4 (OpenAI, 2023)、Claude (Anthropic, 2024)、Gemini (Google DeepMind, 2024) などの基盤モデルは、自然言語での質問応答、手順書の要約・翻訳、トラブルシューティング支援において顕著な能力を示している。製造現場における活用シナリオとしては、設備保全マニュアルの Q&A、品質異常の原因分析支援、新規作業者向けの教育対話などが挙げられる (Cheng et al., 2024)。

RAG (Retrieval-Augmented Generation) は LLM の適用において最も実用的なアーキテクチャとして確立されつつある。RAG は社内手順書・設備マニュアル・品質記録をベクトル DB に格納し、ユーザーの質問に関連する文書を検索した上で LLM への入力コンテキストとして提供することで、LLM を社内知識で補強する (Lewis et al., 2020)。Fine-tuning との比較において、RAG はモデルを追加学習するコスト (GPU 計算資源・ラベリング工数) を回避しつつ知識の更新が容易であり、製造 IT の観点からは維持コストと柔軟性の面で優れる。Fine-tuning は特定の文体・専門用語への適応には有効であるが、知識の更新のたびに再学習が必要となる。

Prompt Engineering は LLM の出力品質を向上させるための技術群であり、Few-Shot Prompting、Chain-of-Thought Prompting (Wei et al., 2022)、ReAct フレームワーク (Yao et al., 2023) などが製造文脈でも有効性が示されている。特に Chain-of-Thought は複雑な品質判断や手順の論理的説明において推論の透明性を高める。

LLM のハルシネーション (hallucination) は製造現場での最大のリスクファクターである。LLM が存在しない手順・規格・材質を確信を持って誤告知する現象は、製造品質への直接的な悪影響をもたらし得る。Mckenna et al. (2023) はハルシネーションの発生頻度が知識の稀少性と強く相関することを示しており、一般的なウェブデータに少ない社内固有の製造知識ではリスクが特に高い。RAG アーキテクチャはハルシネーションを軽減するが完全には防止できず、生成された回答に対する「信頼度スコアリング」と「人間による確認フロー」の設計が不可欠である。

## 5. ローカル・エッジ AI と社内 LAN 制約

本アプリは社内オンプレミス・社内 LAN 専用というアーキテクチャ制約を持つため、クラウド AI サービス (OpenAI API、Google Vertex AI 等) の利用は前提とできない。この制約はセキュリティ・コンプライアンスの観点からは利点でもあるが、AI 機能の実装において別のアプローチが必要となる。

オンプレミス LLM の現実的な選択肢として、Ollama (オープンソースの LLM 実行フレームワーク) による Llama 3 (Meta, 2024)、Mistral (Mistral AI, 2023)、Phi-3 (Microsoft, 2024) などのオープンソース LLM のローカル実行が挙げられる。これらのモデルは GGUF (GPT-Generated Unified Format) 量子化フォーマットにより、4 bit または 8 bit 量子化でモデルサイズを大幅に削減し、GPU なし CPU のみでの推論を可能にする。ただし、CPU 推論のトークン生成速度は GPU 実装と比較して 10〜50 倍程度低速であり、リアルタイム性が要求される作業ナビゲーションへの組み込みでは応答速度の現実的評価が不可欠である (Dettmers et al., 2023)。

Vector DB (ベクトルデータベース) は RAG アーキテクチャの核心コンポーネントであり、文書をベクトル表現で格納し、近似最近傍探索 (ANN) により意味的類似度に基づく検索を実現する。本アプリのバックエンドが Rust + PostgreSQL 環境であることを考慮すると、pgvector (PostgreSQL 拡張) は追加インフラを要さずに Vector DB 機能を導入できる選択肢として有望である。Qdrant (オープンソースの専用 Vector DB、Rust 実装) は高性能な ANN 検索が必要な場合の選択肢となる。

**表 1: オンプレミス LLM 実装オプションの比較**

| モデル | パラメータ数 | GGUF Q4 サイズ | CPU 推論速度 (概算) | 主な強み |
|--------|------------|--------------|-----------------|--------|
| Llama 3 8B | 8B | 約 5 GB | 10〜20 tok/s | バランス型、日本語対応 |
| Mistral 7B | 7B | 約 4 GB | 15〜25 tok/s | 指示追従性が高い |
| Phi-3 Mini | 3.8B | 約 2 GB | 25〜40 tok/s | 軽量・推論特化 |
| Llama 3 70B | 70B | 約 40 GB | 1〜3 tok/s (要大容量RAM) | 最高品質 |

タブレット端末上での Edge ML については、Android の Neural Networks API (NNAPI)、TensorFlow Lite、ONNX Runtime Mobile が主要な実行環境である。画像分類・物体検出のような比較的軽量なモデルはタブレット上で実用的な速度で動作するが、LLM クラスのモデルをタブレット単体で動かすことは現時点では現実的でない。作業者支援のための AI 推論は、社内 LAN 上のサーバ (CPU/GPU 搭載) で実行し、タブレットはクライアントとして推論結果を受信する分散アーキテクチャが適切である。

## 6. 人間と AI の協働 (Human-AI Teaming)

AI が製造現場に導入される際に最も重要でありながら最も見落とされやすい課題の一つが、人間と AI の認知システムの統合である。Hollnagel と Woods (2005) は「ジョイント認知システム (Joint Cognitive Systems)」の概念を提唱し、人間と機械が単一の認知ユニットとして機能するシステム設計の重要性を示した。この視点では、AI はツールではなく「認知パートナー」として位置づけられ、その設計はシステム全体の認知的安全性に影響を与える。

Endsley の状況認識 (Situation Awareness) モデル (Endsley, 1995) は Level 1 (要素の認知)、Level 2 (状況の理解)、Level 3 (将来状態の予測) の三段階で SA を定義し、製造現場における作業者の認知状態の評価に広く適用されている (12 章参照)。AI 支援を導入する際、Level 2・3 の SA の一部を AI が肩代わりすることで短期的な効率は向上するが、作業者自身の SA 維持能力が低下するリスク、いわゆる「スキルの腐食 (Skill Atrophy)」が懸念される。Parasuraman ら (2000) はこの問題を「自動化バイアス (Automation Bias)」として体系化し、人間がオートメーションの誤りを見逃す傾向を実験的に示した。

Mode Confusion は自動化システムで顕著に観察される認知障害であり、オペレーターが現在の自動化モードを正確に把握できなくなる状態を指す。製造ナビゲーション AI が複数の動作モード (通常ガイド / 異常対応 / 訓練モード) を持つ場合、モードの遷移条件と現在状態の明示的な表示設計が作業安全性の観点から不可欠である (Sarter, Woods & Billings, 1997)。

XAI (Explainable AI) は AI の意思決定理由を人間が理解できる形式で提示する技術分野であり、製造 AI においては「なぜこの作業手順を推奨するのか」「なぜこのデータを異常と判定したのか」を説明する能力が、作業者の適切な信頼校正 (Calibrated Trust) に不可欠である (Gunning et al., 2019)。LIME (Local Interpretable Model-Agnostic Explanations)・SHAP (SHapley Additive exPlanations) などの事後説明手法が産業応用の実用段階に入りつつある。

## 7. 規制下の AI

AI 規制の国際的枠組みは 2024〜2025 年にかけて急速に具体化しており、製造 AI の開発・導入に直接の影響を与える。EU AI Act (欧州議会, 2024) は AI システムをリスクレベルで分類し、High Risk カテゴリには厳格な要件を課す。製造品質管理 AI の中でも、製品の安全性に影響する判断を下す AI (例: 医療機器・自動車部品の出荷可否判定 AI) は High Risk に分類され、適合性評価・透明性確保・人間監視の義務化が適用される可能性がある (AI Act, Article 10-15)。EU AI Act は 2026 年から順次適用が開始される。

**表 2: AI 関連規制・規格と製造業への適用範囲**

| 規制・規格 | 主管 | 対象 | 製造 AI への関連性 |
|-----------|------|------|-----------------|
| EU AI Act (2024) | 欧州連合 | EU 市場向け AI システム | High Risk 分類の可能性 |
| FDA AI/ML-Based SaMD Action Plan (2021) | FDA | 医療機器ソフトウェア | 医療機器製造 QMS との AI 統合 |
| IEC 62304 | IEC | 医療機器ソフトウェア ライフサイクル | 医療機器製造向けソフトウェア開発 |
| ISO/IEC 42001 (2023) | ISO/IEC | AI 管理システム | 製造企業の AI ガバナンス枠組み |
| ISO/IEC 23053 | ISO/IEC | ML を用いた AI システム | ML モデル開発プロセス |

FDA の AI/ML-Based SaMD (Software as a Medical Device) Action Plan (2021) は医療機器製造における AI の規制要件を先行して具体化しており、「Predetermined Change Control Plan」の概念により、モデルの継続的改善と規制対応の整合を図ろうとしている。IEC 62304 は医療機器ソフトウェアのライフサイクル管理を規定し、製薬機械・医療機器製造者にとって AI 実装の規制文脈を規定する。

ISO/IEC 42001 (2023) は AI 管理システムの要求事項を規定した初の国際規格であり、ISO 9001・ISO 27001 と類似したマネジメントシステムアプローチで AI のガバナンス・リスク管理・継続改善を組織に求める。製造業での AI 導入拡大に伴い、大手製造業企業では ISO/IEC 42001 認証取得の動きが始まりつつある (ISO/IEC, 2023)。

## 8. データ品質とモデルの現実

製造 AI の実装において、技術的な手法の選択よりも根本的な課題として立ちはだかるのがデータ品質の問題である。Domingos (2012) の「データの重要性」に関する指摘は製造 AI の文脈でも普遍的に妥当であり、良質なデータなくして優れたモデルは生まれない。製造現場のデータは多様なシステム (MES、ERP、SCADA、紙帳票) に分散し、フォーマットが不統一で、欠損値・外れ値・重複が頻繁に存在する (Kusiak, 2017)。

ラベリングコストは教師あり学習モデルの最大の阻害要因の一つである。不良品の外観検査 AI を訓練するためには、正常品・不良品の画像にカテゴリラベルを付与する作業が必要であるが、品質専門家の工数を大量に消費する。さらに、Class Imbalance (クラス不均衡) の問題として、製造現場では正常品が不良品を 1000:1 を超える比率で上回ることが多く、通常の学習アルゴリズムでは不良品クラスへの適合が困難になる。SMOTE (Synthetic Minority Over-sampling Technique)、Focal Loss、Anomaly Detection アプローチなどの手法によって対処する必要がある (Chawla et al., 2002)。

Concept Drift は実運用環境での ML モデルの精度低下を引き起こす主要因であり、学習時と推論時のデータ分布が時間経過とともに乖離する現象を指す。製造現場では設備の更新・原材料ロット変更・季節変動・作業者交代などにより Concept Drift が発生する。これに対応するための MLOps (Machine Learning Operations) パイプラインは、継続的なデータ収集、モデル性能モニタリング、再訓練・再デプロイを自動化または半自動化する仕組みである (Sculley et al., 2015)。

**表 3: 製造 ML モデル開発における主要課題と対応手法**

| 課題 | 具体的問題 | 対応手法例 |
|------|-----------|-----------|
| データ品質 | 欠損値・外れ値・フォーマット不統一 | データパイプライン整備、品質ルール定義 |
| ラベリングコスト | 専門家工数・時間の大量消費 | Active Learning、弱教師あり学習 |
| Class Imbalance | 正常品 >> 不良品の比率不均衡 | SMOTE、Focal Loss、Anomaly Detection |
| Concept Drift | 設備変更・材料変更によるモデル劣化 | ドリフト検出 + 自動再訓練 (MLOps) |
| 説明可能性 | ブラックボックス問題 | SHAP、LIME、Attention Visualization |
| データセキュリティ | 製造秘密・個人情報の保護 | Federated Learning、差分プライバシー |

MLOps の実装においては、モデルの性能指標を継続的に計測し、閾値を下回った場合に再訓練パイプラインをトリガーする仕組みが中核となる。本アプリの PostgreSQL バックエンドは作業ログデータの蓄積基盤として機能しており、作業ログ (21 章参照) から継続的に ML 特徴量を生成し、モデルの性能監視と更新に活用するアーキテクチャが長期的に有望である。

## 9. 本アプリへの含意

製造 AI の実装において最も重要な原則の一つは「段階的な導入」であり、過度に複雑な AI から着手するのではなく、価値が明確で信頼性の高いシンプルな手法から開始し、実績を積み上げながら徐々に高度化することが推奨される。本アプリについては以下の段階的実装シナリオを提案する。

フェーズ 1 (初期実装) としてルールベース推奨を実装することが最も堅実なアプローチである。品質担当者・熟練作業者の経験知識を IF-THEN ルールとして明示化し、「特定の工程で前工程結果が規格外であれば警告を表示」「特定の作業者が過去 N 回同じ工程でエラーを発生させていれば注意喚起を追加」といったルールを管理コンソールから設定可能にする。このアプローチは AI 専門知識なしに品質担当者が管理・改善できるため、現場への定着が高い。

フェーズ 2 (中期実装) として pgvector を活用した手順書 RAG Q&A を実装する。既存の PostgreSQL 環境に pgvector 拡張を追加し、PDF・Word 形式の作業標準書・設備マニュアルをチャンク分割・ベクトル化して格納する。作業者がタブレット上で「この材料の締め付けトルクは?」と自然言語で質問すると、関連手順書のチャンクが検索され、LLM (Ollama サーバ上の軽量モデル) が回答を生成する。社内 LAN 内で完結するため、機密情報の外部漏洩リスクがない。ハルシネーション対策として、生成された回答には必ず参照した手順書の文書名・ページ番号を併記し、作業者が原文を確認できる設計とする。

フェーズ 3 (長期実装) として Edge ML による異常検知をタブレット画像認識として実装する。作業者がカメラで撮影した部品画像から、TensorFlow Lite モデルによる欠陥・誤組み付けの検出をリアルタイムで実行する。モデルは社内サーバで訓練し、ONNX または TensorFlow Lite 形式でタブレットに配布する。Concept Drift 対応として、作業者が「AI の判定に同意しない」とフラグを立てたケースをラベル付きデータとして収集し、モデル再訓練データに活用する。

LLM を活用した Q&A 機能は社内 Ollama サーバを前提とした実装順序を推奨する。具体的には: (1) Ollama サーバ (Linux ホスト + Docker Compose) の社内 LAN 設置、(2) pgvector の PostgreSQL 拡張有効化と Embedding モデルの選定 (multilingual-e5-large 等)、(3) 手順書の Ingestion パイプライン構築 (PDF 解析 → チャンク分割 → ベクトル生成 → PostgreSQL 格納)、(4) Rust/Axum バックエンドへの検索・生成エンドポイント追加、(5) タブレットアプリへの Q&A UI 組み込み、の順序が適切である。

## 10. 批判と限界

### 10.1 製造 AI の ROI 実証の困難と市場推計の信頼性

製造 AI の ROI (投資対効果) は、ベンダーやコンサルティング会社の報告書で誇張されやすい領域である。McKinsey・WEF 等が公表する市場規模推計や削減効果の数値は、定義・方法論・サンプリングが開示されないことが多く、独立した検証が困難である。実際の製造現場での AI 導入事例を系統的にレビューした Wuest ら (2016) は、多くの成功事例が大企業・特定プロセス産業に偏っており、中小製造業や装置工業以外への一般化可能性が限られることを指摘している。AI 導入の意思決定においては、ベンダー提供のケーススタディではなく、自社の業務特性・データ保有状況・技術力を冷静に評価することが不可欠である。

### 10.2 ブラックボックス問題と現場作業者の信頼

深層学習モデルの意思決定プロセスは本質的に不透明であり、「なぜこの判定をしたか」を説明できないブラックボックス性は、製造現場での受容に最大の障壁となる。作業者が AI の推奨を信頼して従うためには、推奨の根拠が理解可能な形で提示される必要があり、XAI 技術はこの問題への対応策として注目されている。しかし、SHAP・LIME 等の現行 XAI 手法が生成する説明は必ずしも「因果的説明」ではなく、「相関に基づく事後的な近似説明」に過ぎないという根本的な限界がある (Rudin, 2019)。真に解釈可能なモデルを使用するか、ブラックボックスモデルの不完全な説明を受け入れるかのトレードオフは、未解決の研究課題である。

### 10.3 誤推奨の責任帰属と法的課題

AI が誤った作業手順を推奨した結果として製品不良・作業事故が発生した場合の責任帰属は、法的に未解決な領域である。現行の製造物責任法・労働安全衛生法の枠組みは AI の中間的意思決定を想定しておらず、責任はシステム開発者・導入企業・個別の作業者のいずれに帰すかについて明確な判例・指針が存在しない (高橋, 2022)。EU AI Act は High Risk AI に対して「人間の監視 (Human Oversight)」を義務づけているが、その実務的な実装方法は未定義な部分が多い。製造現場への AI 実装においては、AI の推奨はあくまで「支援・提案」であり最終判断は人間が行うという設計原則を明確にし、UI レベルでその区別を明示することが法的リスク管理の観点から重要である。

### 10.4 Fine-tuning データの著作権と LLM のハルシネーションリスク

LLM を自社用途に Fine-tuning する際のトレーニングデータ著作権問題は、2023〜2024 年の複数の訴訟 (The New York Times v. OpenAI 等) によって法的論点として顕在化した。社内の作業標準書・設計図書には第三者 (設備メーカー・外部コンサルタント) が著作権を保有するコンテンツが含まれる可能性があり、Fine-tuning への使用前に権利確認が必要である (Samuelson, 2023)。RAG アーキテクチャは Fine-tuning と異なりモデルのパラメータに知識を埋め込まないため、この著作権問題を回避しやすいが、生成された回答が原文をほぼそのまま引用する場合は別途問題となりうる。

### 10.5 EU AI Act 規制コストと中小製造業への影響

EU AI Act の High Risk 要件への適合には、適合性評価・技術文書整備・ログ記録・人間監視体制の構築等の相当なコストが伴う。欧州市場向けに製品を供給する中小製造業が High Risk AI を導入する場合、これらのコストが ROI を圧迫する可能性がある。また、EU AI Act の域外適用規定は明確でない部分もあり、日本の製造業が EU 市場向け製品の製造に AI を活用する場合に適用されるかどうかについて、現時点では解釈の余地がある。AI 規制の不確実性は、製造業での AI 投資判断に抑制的に作用している側面があることも事実であり、規制環境の動向を注視した段階的投資が合理的な戦略となる。

### 10.6 データ主権・プライバシーと AI 学習のジレンマ

ML モデルの改善は継続的なデータ収集と学習に依存するが、作業者の行動データ (作業速度・エラー率・特定工程への対応時間) を AI 学習に用いることは、作業者のプライバシーと評価への影響という倫理的問題を孕む (24 章参照)。特に、個人識別可能な作業ログを外部の AI クラウドサービスに送信することはデータ主権の観点から重大なリスクとなり得る。本アプリが採用する社内 LAN 完結のオンプレミスアーキテクチャはこの問題を大きく緩和するが、社内でのデータ利用目的・保存期間・アクセス権限についての透明な方針策定と労使間の合意形成が、AI 機能を持続的に運用するための社会的基盤として不可欠である。

---

## 参考文献

- Kagermann, H., Wahlster, W., and Helbig, J. (Eds.). *Recommendations for Implementing the Strategic Initiative Industrie 4.0*. acatech, 2013.
- Lee, J., Bagheri, B., and Kao, H.-A. "A Cyber-Physical Systems Architecture for Industry 4.0-Based Manufacturing Systems." *Manufacturing Letters*, vol. 3, 2015, pp. 18–23.
- Shewhart, Walter A. *Economic Control of Quality of Manufactured Product*. Van Nostrand, 1931.
- Montgomery, Douglas C. *Introduction to Statistical Quality Control*. 8th ed., John Wiley & Sons, 2020.
- Liu, Fei Tony, Kai Ming Ting, and Zhi-Hua Zhou. "Isolation Forest." *Proceedings of the 8th IEEE International Conference on Data Mining (ICDM)*, 2008, pp. 413–422.
- Schölkopf, B., et al. "Estimating the Support of a High-Dimensional Distribution." *Neural Computation*, vol. 13, no. 7, 2001, pp. 1443–1471.
- Malhotra, Pankaj, et al. "LSTM-Based Encoder-Decoder for Multi-Sensor Anomaly Detection." *ICML 2016 Anomaly Detection Workshop*, 2016.
- He, Kaiming, et al. "Deep Residual Learning for Image Recognition." *Proceedings of the IEEE Conference on CVPR*, 2016, pp. 770–778.
- Lewis, Patrick, et al. "Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks." *Advances in Neural Information Processing Systems (NeurIPS)*, vol. 33, 2020, pp. 9459–9474.
- Hollnagel, Erik, and David D. Woods. *Joint Cognitive Systems: Foundations of Cognitive Systems Engineering*. CRC Press, 2005.
- Endsley, Mica R. "Toward a Theory of Situation Awareness in Dynamic Systems." *Human Factors*, vol. 37, no. 1, 1995, pp. 32–64.
- Parasuraman, Raja, Thomas B. Sheridan, and Christopher D. Wickens. "A Model for Types and Levels of Human Interaction with Automation." *IEEE Transactions on Systems, Man, and Cybernetics—Part A*, vol. 30, no. 3, 2000, pp. 286–297.
- Gunning, David, et al. "XAI—Explainable Artificial Intelligence." *Science Robotics*, vol. 4, no. 37, 2019.
- Sculley, David, et al. "Hidden Technical Debt in Machine Learning Systems." *Advances in Neural Information Processing Systems (NeurIPS)*, vol. 28, 2015.
- Chawla, N. V., et al. "SMOTE: Synthetic Minority Over-sampling Technique." *Journal of Artificial Intelligence Research*, vol. 16, 2002, pp. 321–357.
- Dettmers, Tim, et al. "QLoRA: Efficient Finetuning of Quantized LLMs." *NeurIPS*, 2023.
- Kusiak, Andrew. "Smart Manufacturing Must Embrace Big Data." *Nature*, vol. 544, 2017, pp. 23–25.
- Rudin, Cynthia. "Stop Explaining Black Box Machine Learning Models for High Stakes Decisions and Use Interpretable Models Instead." *Nature Machine Intelligence*, vol. 1, 2019, pp. 206–215.
- ISO/IEC. *ISO/IEC 42001:2023 — Information Technology — Artificial Intelligence — Management System*. ISO, 2023.
- 高橋和之. 「AI 推奨システムの法的責任帰属論」. *情報法制研究*, 第 11 号, 2022, pp. 45–62.

## 関連章

- [07_スマートファクトリーと作業のデジタル化](./07_スマートファクトリーと作業のデジタル化.md) — Industry 4.0・CPS・スマートファクトリーの技術基盤との接続
- [12_認知工学と状況認識](./12_認知工学と状況認識.md) — Endsley SA モデル・自動化バイアス・モードコンフュージョンとの理論的連接
- [21_作業ログ分析とプロセスマイニング](./21_作業ログ分析とプロセスマイニング.md) — AI 学習データとしての作業ログ活用とプロセス改善への接続
- [24_作業者プライバシー・データ倫理と労務監視](./24_作業者プライバシー・データ倫理と労務監視.md) — AI 学習データとしての作業者行動データの倫理的取り扱い
- [28_不適合と手順改訂のフィードバックループ](./28_不適合と手順改訂のフィードバックループ.md) — 異常検知 AI の出力を手順改善サイクルに統合するフィードバック設計
