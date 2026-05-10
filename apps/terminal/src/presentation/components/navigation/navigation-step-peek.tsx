// 対応 §: ロードマップ §3.6.2 §3.6.4
// 「次のステップ」を peek-card として常時提示する。
// SwipeGuide／Poka が単一ステップ集中で評価される一方、現場の作業者は
// 「次に何が来るか」の予告を強く要望する（先回り段取り）。
// peek-card は控えめなサイズ・低コントラストでフォーカス領域より弱い視覚優先度を保つ。

import type { StepDto } from '../../../adapter/api-client';
import { palette, fontSize, fontWeight, radius, space } from '../../../tokens/access';
import { Icon } from '../icon/icon';

export interface NavigationStepPeekProps {
  next: StepDto | null;
  index: number;
  total: number;
  title: string;
  endLabel: string;
}

export function NavigationStepPeek(props: NavigationStepPeekProps): JSX.Element {
  const empty = props.next === null;
  return (
    <aside
      aria-label={props.title}
      style={{
        padding: space[3],
        background: palette.surfaceAlt,
        border: `1px solid ${palette.border}`,
        borderRadius: radius.medium,
        display: 'flex',
        flexDirection: 'column',
        gap: space[2]
      }}
    >
      <header
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: space[2],
          fontSize: fontSize.caption,
          fontWeight: fontWeight.bold,
          color: palette.fgMuted,
          textTransform: 'uppercase',
          letterSpacing: '0.06em'
        }}
      >
        <Icon name="chevron-right" size={16} color={palette.fgMuted} />
        {props.title}
      </header>
      {empty ? (
        <span style={{ fontSize: fontSize.body, color: palette.fgMuted }}>{props.endLabel}</span>
      ) : (
        <>
          <span
            aria-hidden="true"
            style={{
              fontSize: '32px',
              fontWeight: fontWeight.bold,
              color: palette.info.default,
              lineHeight: 1
            }}
          >
            {String(props.index + 1).padStart(2, '0')}
            <span style={{ fontSize: fontSize.body, color: palette.fgMuted, marginLeft: 4 }}>
              / {String(props.total).padStart(2, '0')}
            </span>
          </span>
          <strong style={{ fontSize: fontSize.body, color: palette.fg, fontWeight: fontWeight.medium }}>
            {props.next!.label}
          </strong>
        </>
      )}
    </aside>
  );
}
