// 対応 §: ロードマップ §10.1 §3.6 §9.5 §11.2
// 端末作業ナビアプリの最小コンポーネント。
// §3.6.2 ナビゲーションの 6 基本機能のうち「現在地」「目的地」「進捗」の表示を担う。

// React の Hook
import { useState } from 'react';
// ドメイン
import type { Task } from '../../domain/task';

/** TaskCard プロパティ */
export interface TaskCardProps {
  // 表示対象の Task
  task: Task;
  // 完了ボタン押下時のコールバック（オプション）
  onComplete?: () => void;
}

/**
 * Task の現状を最小限のカード UI として表示する。
 *
 * §11.2 タッチターゲット 9mm 以上を満たすよう、ボタンに最小サイズを設定する。
 */
export function TaskCard(props: TaskCardProps): JSX.Element {
  // 完了済みフラグ（UI 状態のみ／本来は usecase 経由でドメインに反映）
  const [completedLocal, setCompletedLocal] = useState<boolean>(false);

  // 完了ボタン押下ハンドラ
  function handleComplete(): void {
    // ローカル状態を更新
    setCompletedLocal(true);
    // 親に通知
    props.onComplete?.();
  }

  // 描画
  return (
    <article
      // §9.5.1 surface medium 角丸／コンテナ余白
      style={{
        padding: '16px',
        borderRadius: '8px',
        boxShadow: '0 2px 4px rgba(13, 17, 23, 0.06)',
        background: '#FFFFFF'
      }}
      // ARIA ラベル（§11.2.2 アクセシビリティ）
      aria-labelledby={`task-${props.task.id.toString()}`}
    >
      {/* 現在地: タスク ID／状態 */}
      <header>
        <h2 id={`task-${props.task.id.toString()}`} style={{ fontSize: '20px', margin: 0 }}>
          作業: {props.task.id.toString()}
        </h2>
        <p style={{ margin: '4px 0', color: '#6C757D' }}>状態: {props.task.state}</p>
      </header>

      {/* 進捗: Lamport 値 */}
      <section>
        <p>
          進捗（Lamport）: <strong>{props.task.lamport.toBigInt().toString()}</strong>
        </p>
      </section>

      {/* 目的地: 完了条件 */}
      <section>
        <p>完了条件: {props.task.completionCriteria}</p>
      </section>

      {/* アクション: 完了ボタン（タッチターゲット 9mm 以上）*/}
      <footer>
        <button
          type="button"
          onClick={handleComplete}
          disabled={completedLocal}
          // §11.2 タッチターゲット最小 9mm（≒ 34px）／推奨 11mm（≒ 41px）
          style={{
            minHeight: '44px',
            minWidth: '44px',
            padding: '8px 16px',
            background: '#28A745',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: '8px',
            cursor: 'pointer'
          }}
          aria-label="作業を完了する"
        >
          {completedLocal ? '完了済み' : '完了する'}
        </button>
      </footer>
    </article>
  );
}
