// 対応 §: ロードマップ §10.4 §3.6.2 §3.6.4
// メディアスロット: 写真 / 動画 / 図面の参照アフォーダンスを 3 タイル並列で示す。
// 端末がメディアを持たない時点では空のスロット (placeholder) を表示し、
// バックエンドが `step.media` を提供した瞬間に同レイアウトで実体に差し替えられる構造にする。
// 競合（Poka／tebiki／Dozuki）の核心 UX「動画／写真でステップを示す」軸 (A5) を確保する。

import { palette, fontSize, fontWeight, radius, space } from '../../../tokens/access';
import { Icon } from '../icon/icon';
import type { IconName } from '../icon/glyphs';

export interface NavigationStepMediaProps {
  imageLabel: string;
  videoLabel: string;
  diagramLabel: string;
}

interface SlotProps {
  iconName: IconName;
  label: string;
  hint: string;
}

function Slot(props: SlotProps): JSX.Element {
  return (
    <button
      type="button"
      style={{
        flex: 1,
        minHeight: 96,
        padding: space[3],
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        gap: space[1],
        background: palette.surfaceAlt,
        color: palette.fgMuted,
        border: `1px dashed ${palette.borderStrong}`,
        borderRadius: radius.medium,
        cursor: 'pointer',
        textAlign: 'left'
      }}
    >
      <Icon name={props.iconName} size={28} color={palette.info.default} />
      <strong style={{ fontSize: fontSize.body, fontWeight: fontWeight.medium, color: palette.fg }}>
        {props.label}
      </strong>
      <span style={{ fontSize: fontSize.caption }}>{props.hint}</span>
    </button>
  );
}

export function NavigationStepMedia(props: NavigationStepMediaProps): JSX.Element {
  return (
    <div
      role="group"
      aria-label="step media"
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(3, 1fr)',
        gap: space[3]
      }}
    >
      <Slot iconName="image" label={props.imageLabel} hint="—" />
      <Slot iconName="video" label={props.videoLabel} hint="—" />
      <Slot iconName="diagram" label={props.diagramLabel} hint="—" />
    </div>
  );
}
