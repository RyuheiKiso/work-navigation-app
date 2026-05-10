// 対応 §: ロードマップ §10.1 §3.6 §9.5 §11.2
// 端末作業ナビアプリの最小コンポーネント。
// §3.6.2 ナビゲーションの 6 基本機能のうち「現在地」「目的地」「進捗」の表示を担う。

import { useState } from 'react';
import type { Task } from '../../domain/task';
import { palette, fontSize, fontWeight, radius, space, elevation } from '../../tokens/access';
import { Icon } from './icon/icon';

export interface TaskCardProps {
  task: Task;
  onComplete?: () => void;
}

/**
 * Task の現状を最小限のカード UI として表示する。
 * §11.2 タッチターゲット 9mm 以上を満たすよう、ボタンに最小サイズを設定する。
 */
export function TaskCard(props: TaskCardProps): JSX.Element {
  const [completedLocal, setCompletedLocal] = useState<boolean>(false);

  function handleComplete(): void {
    setCompletedLocal(true);
    props.onComplete?.();
  }

  return (
    <article
      style={{
        padding: space[4],
        borderRadius: radius.medium,
        boxShadow: elevation[1],
        background: palette.white,
        display: 'flex',
        flexDirection: 'column',
        gap: space[2]
      }}
      aria-labelledby={`task-${props.task.id.toString()}`}
    >
      <header>
        <h2
          id={`task-${props.task.id.toString()}`}
          style={{ margin: 0, fontSize: fontSize.subtitle, fontWeight: fontWeight.bold, color: palette.neutral[900] }}
        >
          作業: {props.task.id.toString()}
        </h2>
        <p style={{ margin: 0, color: palette.neutral[600], fontSize: fontSize.caption }}>
          状態: {props.task.state}
        </p>
      </header>
      <p style={{ margin: 0, fontSize: fontSize.body, color: palette.neutral[800] }}>
        進捗（Lamport）: <strong>{props.task.lamport.toBigInt().toString()}</strong>
      </p>
      <p style={{ margin: 0, fontSize: fontSize.body, color: palette.neutral[800] }}>
        完了条件: {props.task.completionCriteria}
      </p>
      <footer>
        <button
          type="button"
          onClick={handleComplete}
          disabled={completedLocal}
          style={{
            minHeight: space.touchRecommended,
            minWidth: space.touchRecommended,
            padding: `${space[2]} ${space[4]}`,
            background: completedLocal ? palette.neutral[300] : palette.success.default,
            color: palette.white,
            border: 'none',
            borderRadius: radius.medium,
            cursor: completedLocal ? 'not-allowed' : 'pointer',
            fontSize: fontSize.body,
            fontWeight: fontWeight.medium,
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[2]
          }}
          aria-label="作業を完了する"
        >
          <Icon name="check" size={20} strokeWidth={3} />
          {completedLocal ? '完了済み' : '完了する'}
        </button>
      </footer>
    </article>
  );
}
