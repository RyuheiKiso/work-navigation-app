# デザイントークン

> 対応 §: ロードマップ §9.5 §11.2 §11.3 §28
> 形式: [Style Dictionary](https://amzn.github.io/style-dictionary/) (Amazon, 2018)
> 出力先: Tauri／React／Android／Windows 各プラットフォームへ自動変換
> 改訂サイクル: §22.1 半期サイクル（CI で実装と乖離していないことを検査）

§9.5「デザインシステムと感覚モダリティ設計」を **構造化された設計トークン** に固定するための Style Dictionary 形式の JSON ファイル群。製造現場特有の制約（騒音・低照度・手袋・粉塵）下では、視覚以外のチャンネル（聴覚・触覚）も第一級設計対象とするため、9 区分のトークンを **同等に** 版管理する。

## 1. ファイル一覧

| ファイル | 区分 | §参照 |
| --- | --- | --- |
| [`color.json`](./color.json) | カラー | §9.5.1 |
| [`typography.json`](./typography.json) | タイポ | §9.5.1 |
| [`spacing.json`](./spacing.json) | 余白 | §9.5.1／§11.2 タッチターゲット |
| [`radius.json`](./radius.json) | 角丸 | §9.5.1 |
| [`elevation.json`](./elevation.json) | 影 | §9.5.1 |
| [`motion.json`](./motion.json) | モーション | §9.5.1／§9.5.2 |
| [`sound.json`](./sound.json) | サウンド | §9.5.3 |
| [`haptic.json`](./haptic.json) | 触覚 | §9.5.4 |
| [`icon.json`](./icon.json) | アイコン | §9.5.1／§11.2.2 |

## 2. 命名規則（Style Dictionary 標準）

```
{category}.{type}.{item}.{subitem}.{state}
```

例:

- `color.semantic.success.strong`
- `font.size.body`
- `space.unit.4`
- `motion.duration.standard`
- `sound.feedback.success`
- `haptic.success`

## 3. ビルド（将来手順）

```bash
# scripts/build-tokens.sh で実行（未整備）
npx style-dictionary build --config ./style-dictionary.config.cjs
```

出力:

- `apps/terminal/src/tokens.ts`（Tauri＋React）
- `apps/config-ui/src/tokens.ts`（React）
- `apps/terminal/android/tokens.xml`（Android リソース）
- `apps/terminal/windows/tokens.json`（Windows）

## 4. 受入観点（§9.5.5）

- 9 トークン区分が本ディレクトリで版管理されていること。
- prefers-reduced-motion／prefers-color-scheme への対応が axe-core ＋ playwright で回帰検証されること（コード実装後）。
- 工場環境（80dB 騒音／低照度／手袋）を想定した検証手順が [`../../04_運用/環境別検証.md`](../../04_運用/環境別検証.md) に文書化されていること。
- §9.2 心理学アプローチ・§11.2 アクセシビリティと矛盾しないこと（変更時に双方向参照を更新）。
- §28 用語集の UI 表記用語と一致すること（同義語混入を CI 検出）。
