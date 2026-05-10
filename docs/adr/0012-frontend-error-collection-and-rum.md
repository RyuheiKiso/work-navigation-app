# ADR-0012: フロントエンドのエラー収集と Real User Monitoring

> 提案日: 2026-05-10
> 採用日: -
> 提案者: @RyuheiKiso
> 関連 §: ロードマップ §11.4 §31.1 §31.2 §14.2 ／ Phase 2 修正計画 P2-15 P2-16

## Status

Proposed

## Type

Type 1（不可逆／§30.1）— SaaS 採否はベンダー切り替えコストが大きい。

## Context

監査（2026-05-10）の結果、両フロント（Tauri 端末・config-ui）に
エラー収集と Real User Monitoring (RUM) の仕組みが存在しないことが判明した。
本プロジェクトは「世界一を判断基準に妥協を選ばない」「沈黙の妥協禁止」を
基本姿勢としており、ユーザーが現場で遭遇したクラッシュや遅延を観測できない
状態でリリースに至るのは方針と矛盾する。

論点:

- **Tauri 端末アプリ** はオフライン動作前提（§10.6）。エラーは即時送信できず、
  端末ローカル DB へ蓄積し復旧時に flush する必要がある。
- **PII** が混入しうる。ユーザー ID・端末 ID・メッセージ本文・スクショは
  GMP / HACCP 監査の対象データを含むため、収集前にマスキング規約が要る。
- **コスト**: 商用 RUM は端末数 ×（イベント数）に比例して請求される。
  個人開発 OSS（§19.4.3）の月額予算は数千円台が現実的上限。
- **OSS 路線**: §19.5 に従い、再ライセンス制限のないツールを優先する。
- **P2-14 の Web Vitals (LCP/INP/CLS)** と統合先を揃えると運用が単純化する。

## Decision

以下を採用する:

1. **エラー収集の収集パイプは自前**: フロントは backend の
   `POST /telemetry/errors` と `POST /telemetry/vitals` に **問題ペイロード**
   （後述）を送る最小実装に留める。ベンダー SaaS には依存しない。
2. **永続化は端末側 SQLite (Tauri) と config-ui の IndexedDB**:
   オフライン中はローカルに蓄積し、`navigator.onLine` 復旧時に exponential
   backoff で flush する。
3. **PII マスキング規約 v1**:
   - `user_id` は SHA-256 で前 8 文字に短縮 (`hashed_user_id`)。
   - エラーメッセージ本文は最大 500 byte に切り詰め、`/[\d]{6,}/`（連続 6 桁
     以上の数字）と email-like パターンを `***` に置換。
   - スクリーンショットは収集しない（v1）。
4. **Web Vitals**: `web-vitals` の `onLCP` / `onINP` / `onCLS` / `onFCP` /
   `onTTFB` を購読し、同一エンドポイントへ送る。
5. **将来の SaaS 切替**: Sentry または Highlight への切替は可能性を残すが、
   v1 では自前パイプ + 構造化ログ（backend tracing）で十分とする。
   切替が必要になった時点で本 ADR を Superseded で書き換える。

## Alternatives

| 代替案 | 概要 | 却下理由 |
| --- | --- | --- |
| Sentry SaaS（self-hosted 含む） | 業界標準の RUM。豊富なサンプリング・トリアージ機能 | 月額費用が端末数で膨らむ。OSS 自己ホストは運用コスト大。PII の境界制御が config 依存で監査に弱い |
| OpenTelemetry + 自前バックエンド | 標準仕様で将来の選択肢が広い | OTel SDK は web 向けがまだ stabilization 段階。Tauri webview の互換性も自前検証が必要 |
| ログのみ（収集パイプ無し） | 端末ローカル `tracing` のみ | ユーザーがクラッシュを報告するまで検知できない。本 ADR の動機と矛盾 |

## Consequences

- **正の帰結**:
  - 端末／設定 UI でクラッシュと体感遅延が定量化され、SLO ダッシュボードに
    乗る（§31.1）。
  - Tauri のオフライン制約をネイティブに扱うため、現場で取りこぼしがない。
  - PII マスキングをサーバ受信前に強制するので、監査適合（§11.4）に有利。
- **負の帰結**:
  - 自前パイプの保守コスト（スキーマ進化・dedup・サンプリング）。
  - SaaS の高機能ダッシュボードは利用できない。検索・トリアージ UI は
    config-ui に薄く実装する必要がある。
- **影響範囲**: §11.4 §14.2 §31.1 §31.2、および P2-15 (Web Vitals 計測 Hook)
  の出口を本パイプに揃える。

## Implementation Plan

1. `services/backend` に `POST /telemetry/errors` と `POST /telemetry/vitals`
   を追加（v1 はストレージ保留＝tracing ログ吐きのみ）。
2. 両フロントに `useTelemetry` Hook を追加し、`web-vitals` をブリッジ。
3. Tauri 端末では `lifecycle on app_pause` と `online` イベントで flush。
4. PII マスキング層を `apps/*/src/adapter/telemetry.ts` に集約し
   property-based test で網羅検証する。

## Open Questions

- `POST /telemetry/errors` のスキーマ詳細（v1 vs v2）。問題分類のための
  `error_code` を ApiError と統一する案を要検討（P1-7 参照）。
- 端末 SQLite に蓄積する場合の最大サイズ。§14.2 の容量目標と要調整。
