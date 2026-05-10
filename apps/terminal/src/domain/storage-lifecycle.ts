// 対応 §: ロードマップ §10.4.4 §10.4.6 §16
// メディアストレージライフサイクル: 80% で警告、90% で新規撮影をブロックする。
// `StorageMetrics` は端末側の実ストレージ照会（Tauri API 等）から得るが、
// 本ドメインは純粋関数で判定のみを担当する（§9.4 副作用は境界層）。

/** ストレージメトリクス（端末側スナップショット） */
export interface StorageMetrics {
  // 全体容量（バイト）
  readonly totalBytes: number;
  // 使用中容量（バイト）
  readonly usedBytes: number;
}

/** ストレージ状態（§10.4.4 既定の 3 段階） */
export type StorageStatus = 'normal' | 'warning' | 'blocked';

/** ストレージ判定結果 */
export interface StorageDecision {
  // 状態
  readonly status: StorageStatus;
  // 使用率（0.0〜1.0）
  readonly utilization: number;
  // 警告／ブロックの根拠メッセージ（§20.1 「人を責めない」原則）
  readonly message: string;
}

/** §10.4.4 既定しきい値 */
export const WARNING_RATIO = 0.8;
/** §10.4.4 既定しきい値 */
export const BLOCK_RATIO = 0.9;

/**
 * メトリクスから状態を判定する純粋関数。
 *
 * @throws totalBytes が 0 以下の場合
 */
export function evaluate(metrics: StorageMetrics): StorageDecision {
  // 容量検査（0 除算防止／不正値検出）
  if (metrics.totalBytes <= 0) {
    throw new Error('totalBytes は正の整数である必要があります');
  }
  // 使用率
  const utilization = metrics.usedBytes / metrics.totalBytes;
  // 状態判定
  if (utilization >= BLOCK_RATIO) {
    return {
      status: 'blocked',
      utilization,
      message:
        '端末ストレージの空き容量が不足しています。新規撮影は一時停止されました（§10.4.4）。'
    };
  }
  if (utilization >= WARNING_RATIO) {
    return {
      status: 'warning',
      utilization,
      message:
        '端末ストレージの空き容量が少なくなっています（§10.4.4）。同期完了済みの古いメディアの削除を検討してください。'
    };
  }
  return {
    status: 'normal',
    utilization,
    message: ''
  };
}

/** 新規撮影が許可されるかを判定する */
export function canCapture(metrics: StorageMetrics): boolean {
  // ブロック状態以外は撮影可能
  return evaluate(metrics).status !== 'blocked';
}
