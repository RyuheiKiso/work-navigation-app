# 現場HCIと作業者インターフェース

## 1. 製造現場における HCI の特殊性

**HCI (Human-Computer Interaction)** は人間とコンピュータシステムの相互作用を研究・設計する学際領域である。製造現場の HCI は一般的なオフィス向け UI 設計と本質的に異なる制約を持つ。

| 制約 | 内容 | 代表的影響 |
|---|---|---|
| 手袋・汚れ | 静電容量式タッチの反応低下 | ターゲットサイズ・素材選定 |
| 騒音環境 | 音声フィードバックの不達 | 触覚・視覚フィードバックへの代替 |
| 動きながらの操作 | 凝視・両手操作が困難 | Glanceable UI、ハンズフリー入力 |
| 照明変動 | 粉塵・逆光・暗所 | 画面輝度・コントラスト比の確保 |
| 認知負荷の重複 | 作業タスクに並行してUIを操作 | 情報の最小化・エラー防止設計 |

## 2. Norman の認知工学

**Donald Norman** は *The Psychology of Everyday Things* (1988) で、モノの設計が使い手の認知にどう影響するかを論じた。製造現場 UI に直結する概念:

### 2.1 アフォーダンスとシグニファイア

**アフォーダンス (Affordance)** は Gibson (1979) 由来の概念で「対象が行為者に提供する行為の可能性」を指す。Norman はこれを設計上の「知覚されたアフォーダンス」として再解釈した。**シグニファイア (Signifier)** はアフォーダンスを使用者に伝えるためのシグナル (ボタンの形状、下線など) であり、より設計に直結する概念として区別した (*The Design of Everyday Things* 改訂版, 2013)。

### 2.2 マッピングとフィードバック

**マッピング (Mapping)** はコントロールと結果の空間的・概念的対応関係。製造現場では「左がステップ前へ・右がステップ次へ」のような一貫したレイアウトがエラーを防ぐ。**フィードバック** は行為の結果を即座に使用者に返すこと。音声が届かない騒音環境ではバイブレーション・LED 点滅などのマルチモーダル設計が必要となる。

### 2.3 エラーの 2 種類とデザイン

Norman はエラーを **スリップ (Slip)**: 意図は正しいが実行で失敗、**ミステイク (Mistake)**: 意図そのものが誤りに分類した。スリップはハードウェア/UIの物理的干渉によって生じるため、ターゲットサイズの確保や確認ダイアログが有効である。

## 3. Fitts の法則と Hick の法則

### 3.1 Fitts の法則

**Paul Fitts (1954)** が提唱したポインティングの動作時間モデル:

```
MT = a + b × log₂(2A / W)

MT: 移動時間 (ms)
A:  ターゲットまでの距離
W:  ターゲットの幅
```

タッチ UI では指先の接触面積 (約 10mm) を考慮し、ターゲット最小サイズは **44px × 44px** (Apple HIG) / **48dp × 48dp** (Material Design) が推奨される。手袋着用時はさらに大きな目標が必要であり、産業用 UI では 72dp 以上が経験則とされる。

### 3.2 Hick の法則

**William Hick (1952)** と **Ray Hyman (1953)** が示した選択肢数と反応時間の関係:

```
RT = b × log₂(n + 1)

RT: 反応時間
n:  等確率の選択肢数
```

製造ナビゲーションアプリでは「次のステップへ進む」ボタンは 1 択に絞り、条件分岐が必要な場合も選択肢を 2〜4 に制限することで反応時間と誤操作を最小化できる。

## 4. Glanceable UI と Ambient Display

**Glanceable UI** は一瞬の視線 (グランス: 約 1〜2 秒) でクリティカルな情報を把握できる設計概念 (Matthews et al., 2004)。腕時計や計器盤の設計思想を UI に適用する。

設計原則:
- 情報の階層化: 最重要情報 (現在ステップ番号・完了率) を最大フォント・高コントラストで最上位に置く
- 色の意味符号化: 緑=OK / 赤=NG / 黄=要確認 を一貫して使用 (WCAG 2.1 AA 以上のコントラスト比 4.5:1)
- 冗長符号化: 色だけでなく形状・記号を併用してカラーユニバーサルデザインに対応

**Ambient Display** は Weiser & Brown (1996) が提案した、背景として情報を提示する手法。アンドン (Andon) は製造現場における Ambient Display の古典的な実装である。

## 5. ハンズフリー入力技術

| 技術 | 仕組み | 適用場面 | 課題 |
|---|---|---|---|
| 音声入力 (VUI) | 自動音声認識 (ASR) | 両手使用中の数値入力 | 騒音 SNR、誤認識 |
| ジェスチャ入力 | カメラ / 深度センサ | クリーンルーム、高電磁場 | 疲労 (Gorilla Arm) |
| 視線入力 | アイトラッキング | 重機オペレーター | キャリブレーション精度 |
| フットペダル | 物理スイッチ | 組立・縫製 | 姿勢固定が必要 |
| 近接センサ | ジェスチャ非接触 | 食品・医療クリーン | 精度・距離制約 |

音声入力の製造現場適用では、語彙制限コマンドセット方式 (コンフィデンス閾値で再確認要求) が誤認識による誤作業防止に有効である。

## 6. 工場耐環境 UI ハードウェア

### 6.1 耐環境規格

**IP (Ingress Protection)** 規格 (IEC 60529) はデバイスの異物・水の侵入防護等級を定める。製造現場では IP54 (防塵・飛沫防水) 以上、食品・屋外では IP65〜67 が求められる。

**MIL-STD-810** (米国防総省) は落下・振動・温度衝撃への耐性を定める軍用規格で、産業用タブレット選定の参照基準として広く用いられる。

### 6.2 屋外・強光下の視認性

液晶パネルの輝度は通常 250〜400 nits だが、直射日光下 (〜100,000 lux) では 800 nits 以上が必要とされる。半透過型 (Transflective) ディスプレイは外光を反射して視認性を補う代替技術である。

### 6.3 手袋対応タッチ

静電容量式タッチスクリーンは導電性のある素材のみ反応する。工業用手袋対応には:
- パネルの静電容量感度を下げた「高感度モード」
- 導電性繊維をつまみ部に組み込んだ「タッチ対応手袋」
- 感圧式 (Resistive) パネルへの切り替え

## 7. AR / MR ヘッドマウントデバイス

**Microsoft HoloLens 2** や **RealWear HMT-1** は製造現場での作業支援に実用化されている。ハンズフリーで作業手順・寸法データを現実空間にオーバーレイする。

認知負荷の課題:
- 情報過多による **インフォメーション・オーバーレイ** (Mayer, 2009 マルチメディア学習理論で整理)
- 長時間着用による **視覚疲労・前庭障害 (VR Sickness)**
- 既存手順 UI との **モーダル整合性** (タブレットと HMD の切り替えコスト)

Boeing 社はワイヤハーネス組立に AR を適用し、作業時間 25% 短縮・誤配線ゼロを実証した (Funk et al., 2016) が、同社は並行して **UI の認知負荷定量評価** (NASA-TLX) を義務付けている。

## 8. ユーザビリティ評価手法

| 手法 | 内容 | 適用タイミング |
|---|---|---|
| ヒューリスティック評価 | Nielsen (1994) 10 原則に基づく専門家レビュー | プロトタイプ段階 |
| 思考発話法 (Think-Aloud) | 操作しながら思考を口述させる観察 | ユーザテスト |
| NASA-TLX | 精神負荷 6 次元の主観評定 (Hart & Staveland, 1988) | 作業後評価 |
| GOMS モデル | Goals-Operators-Methods-Selection Rules で操作時間を予測 | 設計フェーズ |

Nielsen (1994) の 10 ヒューリスティックスは製造 UI 評価でも基本フレームとなる。特に「エラーの防止 (Error Prevention)」と「柔軟性と効率性 (Flexibility and Efficiency)」は熟練作業者と新人が混在する現場に直結する。

## 9. 批判と限界

ユーザビリティ研究の多くはオフィス・消費者製品を前提としており、工場現場の過酷な制約条件を扱う研究は相対的に少ない。Fitts の法則はマウス・スタイラスで検証されたものであり、グローブ着用タッチ操作への適用には補正係数が必要との指摘がある (Parhi et al., 2006)。AR/HMD についても長期使用の人間工学的データは蓄積途上である。

---

## 参考文献

- Norman, D. A. *The Design of Everyday Things*. Basic Books, 1988 (revised 2013).
- Gibson, J. J. *The Ecological Approach to Visual Perception*. Houghton Mifflin, 1979.
- Fitts, P. M. "The information capacity of the human motor system in controlling the amplitude of movement." *Journal of Experimental Psychology*, 47(6), 1954.
- Hick, W. E. "On the rate of gain of information." *Quarterly Journal of Experimental Psychology*, 4(1), 1952.
- Matthews, T., Rattenbury, T., & Carter, S. "Defining, designing, and evaluating peripheral displays." *Human-Computer Interaction*, 22(1-2), 2007.
- Weiser, M., & Brown, J. S. "Designing Calm Technology." *PowerGrid Journal*, 1(1), 1996.
- Hart, S. G., & Staveland, L. E. "Development of NASA-TLX (Task Load Index)." *Advances in Psychology*, 52, 1988.
- Nielsen, J. *Usability Engineering*. Academic Press, 1993.
- Nielsen, J. "Enhancing the explanatory power of usability heuristics." *Proceedings of ACM CHI*, 1994.
- Parhi, P., Karlson, A. K., & Bederson, B. B. "Target size study for one-handed thumb use on small touchscreen devices." *Proceedings of MobileHCI*, 2006.
- Funk, M., et al. "Working with augmented reality? A long-term analysis of in-situ instructions at the assembly workplace." *ACM CHI*, 2016.
- Mayer, R. E. *Multimedia Learning* (2nd ed.). Cambridge University Press, 2009.
- ISO 9241-11:2018 *Ergonomics of human-system interaction — Part 11: Usability definitions and concepts*.
- IEC 60529:2013 *Degrees of protection provided by enclosures (IP Code)*.

## 関連章

- [08_人間工学と作業負荷](./08_人間工学と作業負荷.md) — 身体的人間工学と HCI の統合
- [12_認知工学と状況認識](./12_認知工学と状況認識.md) — SA 維持のための画面設計
- [32_自動認識技術とデータ収集設計](./32_自動認識技術とデータ収集設計.md) — 現場 UI とスキャン入力の統合
- [35_環境耐性と防爆・クリーンルーム設計](./35_環境耐性と防爆・クリーンルーム設計.md) — 環境制約下での UI 設計
