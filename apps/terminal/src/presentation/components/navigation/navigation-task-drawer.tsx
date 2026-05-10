// 対応 §: ロードマップ §3.6.4 §11.2 §10.6
// 当日タスク一覧。フォーカス領域を最大化するため、既定で折りたたみ。
// 開時のみ 280px の縦リストを描画し、選択状態は色＋枠＋アイコンの三重で示す。

import type { TaskListItem } from '../../../adapter/api-client';
import { palette, fontSize, fontWeight, radius, space } from '../../../tokens/access';
import { Icon } from '../icon/icon';
import { EmptyState } from '../../states/empty-state';
import { SkeletonTaskCard } from '../../states/skeleton';

export interface NavigationTaskDrawerProps {
  open: boolean;
  tasks: TaskListItem[];
  loading: boolean;
  selectedTaskId: string | null;
  title: string;
  loadingLabel: string;
  emptyTitle: string;
  emptyDescription: string;
  onSelect(id: string, state: string): void;
}

export function NavigationTaskDrawer(props: NavigationTaskDrawerProps): JSX.Element | null {
  if (!props.open) return null;
  return (
    <aside
      aria-label={props.title}
      style={{
        gridArea: 'taskDrawer',
        width: 280,
        background: palette.white,
        borderRight: `1px solid ${palette.neutral[200]}`,
        overflowY: 'auto',
        padding: space[3]
      }}
    >
      <h2
        style={{
          margin: 0,
          marginBottom: space[3],
          fontSize: fontSize.body,
          fontWeight: fontWeight.bold,
          color: palette.neutral[800],
          display: 'inline-flex',
          alignItems: 'center',
          gap: space[2]
        }}
      >
        <Icon name="clipboard" size={20} />
        {props.title}
      </h2>
      {props.loading && (
        <div
          role="status"
          aria-live="polite"
          aria-label={props.loadingLabel}
          style={{ display: 'flex', flexDirection: 'column', gap: space[2] }}
        >
          <SkeletonTaskCard />
          <SkeletonTaskCard />
          <SkeletonTaskCard />
        </div>
      )}
      {!props.loading && props.tasks.length === 0 && (
        <EmptyState iconName="inbox-empty" title={props.emptyTitle} description={props.emptyDescription} inline />
      )}
      <ul style={{ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: space[2] }}>
        {props.tasks.map((task) => {
          const selected = props.selectedTaskId === task.id;
          return (
            <li key={task.id}>
              <button
                type="button"
                onClick={() => props.onSelect(task.id, task.state)}
                aria-pressed={selected}
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: space[1],
                  width: '100%',
                  textAlign: 'left',
                  padding: `${space[2]} ${space[3]}`,
                  background: selected ? palette.info.subtle : palette.white,
                  border: `1px solid ${selected ? palette.info.default : palette.neutral[200]}`,
                  borderLeft: `4px solid ${selected ? palette.info.default : palette.neutral[200]}`,
                  borderRadius: radius.medium,
                  cursor: 'pointer'
                }}
              >
                <strong style={{ fontSize: fontSize.body, color: palette.neutral[900], fontWeight: fontWeight.medium }}>
                  {task.title ?? task.id}
                </strong>
                <span style={{ fontSize: fontSize.caption, color: palette.neutral[600] }}>
                  {task.state} · {task.device_id}
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </aside>
  );
}
