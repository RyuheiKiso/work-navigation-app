// 対応 §: ロードマップ §17 §11.2.2 §3.6.4 §9.5.2
// アンドンの severity を「数値の羅列」ではなく、シグナルタワーの 5 段積層比喩で表す。
// `palette.andon[1..5]` に従い、severity 以下の段を点灯（彩度大）させ、上段は暗色の消灯ドットにする。
// 色＋形（タワー）＋テキスト（Lv.X / メッセージ）の三重提示で SC 1.4.1 に整合する。

import { palette, fontSize, fontWeight, radius, space, elevation, motion } from '../../../tokens/access';
import { Icon } from '../icon/icon';

export type AndonSeverity = 1 | 2 | 3 | 4 | 5;

export interface NavigationAndonBannerProps {
  severity: AndonSeverity | null;
  message: string;
  noAndonLabel: string;
  severityPrefix: string;
}

function bgFor(severity: AndonSeverity | null): { bg: string; fg: string } {
  if (severity === null) return { bg: palette.success.subtle, fg: palette.success.strong };
  if (severity >= 4) return { bg: palette.andon[severity], fg: palette.white };
  if (severity === 3) return { bg: palette.andon[3], fg: palette.neutral[900] };
  return { bg: palette.andon[severity], fg: palette.white };
}

export function NavigationAndonBanner(props: NavigationAndonBannerProps): JSX.Element {
  const { severity } = props;
  const { bg, fg } = bgFor(severity);
  // tower の点灯段
  const lit = severity ?? 0;
  return (
    <div
      role="region"
      aria-live="assertive"
      style={{
        background: bg,
        color: fg,
        padding: `${space[3]} ${space[5]}`,
        borderRadius: radius.medium,
        boxShadow: elevation[1],
        display: 'flex',
        alignItems: 'center',
        gap: space[4],
        transition: `background ${motion.durationStandard} ${motion.easeEmphasized}`
      }}
    >
      <div
        aria-hidden="true"
        style={{ display: 'inline-flex', flexDirection: 'column-reverse', gap: 3, padding: space[1], background: 'rgba(0,0,0,0.18)', borderRadius: radius.small }}
      >
        {[1, 2, 3, 4, 5].map((lvl) => (
          <span
            key={lvl}
            style={{
              width: 18,
              height: 6,
              background: lvl <= lit ? palette.andon[lvl as AndonSeverity] : palette.neutral[700],
              opacity: lvl <= lit ? 1 : 0.45,
              borderRadius: radius.small
            }}
          />
        ))}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: space[1], flex: 1 }}>
        <strong
          style={{
            fontSize: fontSize.subtitle,
            fontWeight: fontWeight.bold,
            display: 'inline-flex',
            alignItems: 'center',
            gap: space[2]
          }}
        >
          <Icon name={severity === null ? 'shield-check' : 'andon-tower'} size={24} />
          {severity === null ? props.noAndonLabel : `${props.severityPrefix}${severity}`}
        </strong>
        {severity !== null && (
          <span style={{ fontSize: fontSize.body, fontWeight: fontWeight.regular }}>{props.message}</span>
        )}
      </div>
    </div>
  );
}
