# ADR-0004. SPA フレームワーク: React

| 項目 | 内容 |
|---|---|
| Status | Accepted |
| Date | 2026-05-09 |
| Deciders | RyuheiKiso |
| 関連 | 企画書 §8.3（v0.7.1 で本決定）、上流制約 C-3 |

## Context and Problem Statement

メンテナンス Web は SPA + API 構成（企画書 §8.3）で実装する。端末アプリは Tauri ＋ Web（HTML／CSS／JavaScript）で構成され（企画書 §8.2、Q24-D）、メンテナンス Web の UI コードを共有する方針が確定している。SPA フレームワークの選定は、React／Vue／Svelte／SolidJS の 4 候補から 1 つを確定する必要があり、コンポーネント部品の流通量・採用層との親和性・Tauri との実績が判断材料となる。

## Decision Drivers

- **Tauri との UI コード共有**: 端末側 Tauri と Web 側で同じ React コンポーネントを再利用できること（企画書 §8.2／§8.3）。
- **採用層との親和性**: 企画書 §14.9 で最初に届けたい層は「OSS／自作ツールに馴染んだ現役 IE ・生産技術エンジニア」「中小製造業の経営者・情シス兼任者」「製造業向け自前ツール開発の社内エンジニア」。これらの層が触れた経験を持つフレームワークが優先される。
- **エコシステムの厚み**: アクセシビリティ部品（WAI-ARIA 準拠）・テーブル・フォーム・状態管理・i18n（本プロジェクトでは i18n は不要だが、副次的に発生するロケール処理のため）。
- **単一メンテナの保守性**: 学習リソース・LTS 期間・破壊的変更頻度。
- **物理制約 §6.3 への対応**: 大ボタン UI ／ 高コントラスト ／ ハプティック・視覚フラッシュ通知の実装に必要な部品が揃っていること。

## Considered Options

1. **React**
2. Vue 3
3. Svelte（SvelteKit）
4. SolidJS

## Decision Outcome

**選定: React**

採用層との親和性・Tauri との実績・エコシステムの厚みで他候補を上回る。Vue は採用層の馴染みが React より薄く、Svelte／SolidJS は単一メンテナでの長期保守時のリソース不安が残る。

### Consequences

- **Good**: アクセシビリティ部品（Radix UI ／ React Aria 等）が Apache-2.0／MIT で揃っており、企画書 §6.3 の物理制約数値を実装しやすい。
- **Good**: 採用層の馴染みが厚く、Issue／PR の参入障壁が下がる（企画書 §15.2 K8 外部 PR 数の達成に寄与）。
- **Good**: React Native 流用や、React Query／TanStack Query による同期エンジンの UI バインディング実装の前例豊富。
- **Bad**: ビルドサイズが Svelte／SolidJS より大きい。Tauri 端末側のバンドルサイズ最適化（コード分割・lazy load）が必要。
- **Bad**: 状態管理ライブラリの選択肢が多く、設計初期に標準を決めないと分散する。本決定では別 ADR（未採番）で状態管理ライブラリを別途確定する。

### 関連する後続 ADR

- 状態管理ライブラリの確定（候補: TanStack Query ＋ Zustand ／ Redux Toolkit ／ Jotai 等）
- ルーティングライブラリの確定（候補: React Router ／ TanStack Router 等）
- UI コンポーネントベースの確定（候補: Radix UI ＋ Tailwind ／ Material UI ／ shadcn/ui 等）
- テスト戦略の React 部分（候補: Testing Library ／ Vitest ／ Playwright Component Test 等）

## Pros and Cons of the Options

### React（採用）

- **Good**: エコシステム最大、採用層の馴染み、Tauri 連携の前例豊富。
- **Good**: アクセシビリティ部品（Radix UI ／ React Aria）が成熟。
- **Bad**: 状態管理・ルーティングの選択肢が分散。
- **Bad**: ビルドサイズが他候補より大きい。

### Vue 3（却下）

- **Good**: テンプレート構文が読みやすく、HTML/CSS/JS の知識で入りやすい。
- **Good**: シングルファイルコンポーネントが構造化されている。
- **Bad**: 採用層（製造業 IT エンジニア）での馴染みが React より薄い（企画書 §14.9）。
- **Bad**: Tauri との連携実績は十分だが、React と比べて部品流通量が少ない。

### Svelte（SvelteKit）（却下）

- **Good**: ビルドサイズが小さい（端末側に効く）。
- **Good**: 構文がシンプルで、単一メンテナの保守時に読みやすい。
- **Bad**: 採用層での馴染みが薄く、Issue／PR の参入障壁を上げる。
- **Bad**: アクセシビリティ部品の流通量が React より少なく、§6.3 物理制約への対応で再発明が必要になる。

### SolidJS（却下）

- **Good**: 細粒度リアクティビティで性能が高い。
- **Good**: API は React に近く、移行学習コストは低い。
- **Bad**: コミュニティ規模が小さく、長期保守でのリソース不安。
- **Bad**: 採用層での馴染みが薄い。

## Links

- 企画書 §8.2 端末アプリ ／ §8.3 メンテナンス Web
- 企画書 §6.3 物理制約数値
- 企画書 §14.9 採用獲得（最初に届けたい層）
