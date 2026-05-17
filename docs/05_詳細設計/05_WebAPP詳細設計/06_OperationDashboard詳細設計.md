# 05 OperationDashboard 詳細設計

本章は MOD-FE-MC-001（OperationDashboard）の TypeScript インターフェース・SLI データ型定義・react-query ポーリングフック・コンポーネントツリーを確定する。OperationDashboard は OPS-036〜053 で要求される運用ダッシュボードを担い、SCR-MC-001 で system_admin/executive が SLI を監視するためのコア UI を提供する。

---

## 1. モジュール概要

| 項目 | 内容 |
|---|---|
| MOD-ID | MOD-FE-MC-001 |
| 物理名 | OperationDashboard |
| ファイルパス | `src/features/dashboard/` |
| 関連 NFR | OPS-036〜053（SLI/SLO 監視）|
| 関連 SCR | SCR-MC-001（運用ダッシュボード）|
| アクセスロール | system_admin・executive |

---

## 2. SLI データ型定義

```typescript
// SLI 測定値（OPS-036〜053 の指標に対応）
export interface SliMetrics {
  /** API P95 レイテンシ（ms）: SLO = 200 ms 以内（OPS-039）*/
  apiLatencyP95Ms: number;
  /** サービス可用性（%）: SLO = 99.5% 以上（OPS-040）*/
  availabilityPercent: number;
  /** オフライン同期成功率（%）: SLO = 99% 以上（OPS-043）*/
  offlineSyncRatePercent: number;
  /** Outbox キュー積算深度（件）: SLO = 100 件未満（OPS-046）*/
  outboxQueueDepth: number;
  /** DLQ 未処理件数（件）: SLO = 0 件（OPS-047）*/
  dlqCount: number;
  /** バックアップ経過時間（h）: SLO = 24 h 以内（OPS-050）*/
  backupAgeHours: number;
  /** 集計時刻 */
  measuredAt: Date;
}

// SLO 閾値定義（UI のゲージ色変化に使用）
export const SLO_THRESHOLDS = {
  apiLatencyP95Ms:        { warn: 150,  alert: 200  },
  availabilityPercent:    { warn: 99.9, alert: 99.5 },
  offlineSyncRatePercent: { warn: 99.5, alert: 99.0 },
  outboxQueueDepth:       { warn: 50,   alert: 100  },
  dlqCount:               { warn: 1,    alert: 5    },
  backupAgeHours:         { warn: 12,   alert: 24   },
} as const satisfies Record<keyof Omit<SliMetrics, 'measuredAt'>, { warn: number; alert: number }>;

export type SloStatus = 'ok' | 'warn' | 'alert';

// アンドン発報サマリ（最大 5 件表示）
export interface AndonAlertSummary {
  alertId: string;
  alertType: string;
  workcellName: string;
  raisedAt: Date;
  /** null = 未解決 */
  resolvedAt: Date | null;
}
```

---

## 3. コンポーネント Props 定義

```typescript
// OperationDashboard コンポーネント（SCR-MC-001 に対応）
export interface DashboardProps {
  /** メトリクスポーリング間隔（ms）デフォルト 30000（30 s）*/
  refreshIntervalMs: number;
  metrics: SliMetrics;
  activeAlerts: AndonAlertSummary[];
}
```

---

## 4. react-query ポーリングフック（FNC-FE-011）

```typescript
import { useQuery, UseQueryResult } from '@tanstack/react-query';

/**
 * FNC-FE-011: SLI メトリクスを定期ポーリングする react-query フック
 *
 * @param refreshIntervalMs - ポーリング間隔（デフォルト 30000 ms）
 * @returns SliMetrics と直近 5 件の AndonAlertSummary を含む結果
 *
 * staleTime: refreshIntervalMs（ポーリング間隔と一致させる）
 * refetchInterval: refreshIntervalMs
 * retry: 3（バックオフ: 指数関数的）
 */
export declare function useSliMetrics(
  refreshIntervalMs?: number,
): UseQueryResult<{ metrics: SliMetrics; activeAlerts: AndonAlertSummary[] }, Error>;

// デフォルト値
export const DEFAULT_REFRESH_INTERVAL_MS = 30_000 as const;
```

---

## 5. コンポーネントツリー

```
OperationDashboard (MOD-FE-MC-001)
  DashboardHeader（最終更新時刻・手動更新ボタン）
  SliGaugeGroup（3 ゲージ: P95 レイテンシ / 可用性 / 同期成功率）
    SliGauge (×3)
      GaugeArc（SloStatus に応じて green/yellow/red）
      MetricValue（数値・単位表示）
      SloThresholdLabel（SLO 閾値表示）
  SliCounterGroup（3 カウンター: Outbox 深度 / DLQ 件数 / バックアップ経過時間）
    SliCounter (×3)
  AlertList（上位 5 件のアクティブアンドン発報一覧）
    AlertRow (×N)
      AlertTypeBadge
      WorkcellLabel
      ElapsedTime（raisedAt からの経過時間）
  QuickLinks
    → SCR-MC-007（DLQ 監視）へのリンク（dlqCount > 0 時にバッジ表示）
    → SCR-MC-008（ハッシュチェーン検証）へのリンク
    → SCR-MC-006（バックアップ状況）へのリンク（backupAgeHours > 12 時にバッジ表示）
```

---

## 6. SloStatus 判定ロジック

```typescript
/**
 * SloStatus を判定する純粋関数
 * SLO_THRESHOLDS の warn/alert と実測値を比較する
 *
 * 判定ルール:
 * - availabilityPercent・offlineSyncRatePercent は値が「低いほど悪い」（逆方向）
 * - それ以外は値が「高いほど悪い」（順方向）
 */
export function computeSloStatus(
  metricKey: keyof Omit<SliMetrics, 'measuredAt'>,
  value: number,
): SloStatus {
  const thresholds = SLO_THRESHOLDS[metricKey];
  const isReverseMetric =
    metricKey === 'availabilityPercent' ||
    metricKey === 'offlineSyncRatePercent';

  if (isReverseMetric) {
    if (value < thresholds.alert) return 'alert';
    if (value < thresholds.warn)  return 'warn';
    return 'ok';
  } else {
    if (value >= thresholds.alert) return 'alert';
    if (value >= thresholds.warn)  return 'warn';
    return 'ok';
  }
}
```

---

## 7. エラーハンドリング

| エラーコード | 発生条件 | UI 対応 |
|---|---|---|
| ERR-SYS-001 | SLI API タイムアウト（3 回リトライ後）| ゲージをグレーアウト・「データ取得失敗」バナー |
| ERR-AUTH-003 | RBAC 不足（executive 以外のアクセス）| ダッシュボード非表示・403 ページへリダイレクト |

---

**本節で確定した方針**
- **SLI 指標の SloStatus 判定を `computeSloStatus` 純粋関数として定義し、availabilityPercent・offlineSyncRatePercent の逆方向メトリクスを明示的に分岐処理することを確定した。**
- **ポーリング間隔 30 s（DEFAULT_REFRESH_INTERVAL_MS）を Props でカスタマイズ可能とし、staleTime = refreshIntervalMs と一致させることで不要な再フェッチを防止することを確定した。**
- **QuickLinks は DLQ 件数・バックアップ経過時間が閾値超過時にバッジを表示し、system_admin が異常を発見した直後に該当画面へ遷移できるナビゲーションを提供することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/25_作業指示書とSOPの構造化・表現論.md`](../../90_業界分析/25_作業指示書とSOPの構造化・表現論.md)

### 関連
- [`90_業界分析/18_現場HCIと作業者インターフェース.md`](../../90_業界分析/18_現場HCIと作業者インターフェース.md)
