// 対応 §: ロードマップ §3.6.2 §11.2 §9.5.2 §10.4
// Bento レイアウトのフォーカス領域。
// - 主カード: 巨大ステップ番号 + ラベル + メタチップ + 次アクション帯
// - ゲージカード: 残ステップ／経過 vs 標準を 1 視野で
// - メディアカード: 写真／動画／図面のアフォーダンス（A5 軸）
// - Peek カード: 次のステップを控えめに常時提示
// 開始前 / 完了後 / 空はそれぞれ専用パネルに分岐し、フォーカスを 1 つに保つ。

import type { StepDto } from '../../../adapter/api-client';
import { palette, fontSize, fontWeight, radius, space, elevation } from '../../../tokens/access';
import { Icon } from '../icon/icon';
import { Button } from '../primitives/button';
import { StatusPill } from '../primitives/status-pill';
import { LoadingState } from '../../states/loading-state';
import { Skeleton } from '../../states/skeleton';
import { EmptyState } from '../../states/empty-state';
import { ErrorPanel } from '../../states/error-panel';
import { NavigationProgressGauge } from './navigation-progress-gauge';
import { NavigationStepMedia } from './navigation-step-media';
import { NavigationStepPeek } from './navigation-step-peek';

export interface NavigationStepFocusProps {
  // 入力状態
  stepsLoading: boolean;
  selectedTaskState: string;
  selectedTaskId: string | null;
  current: StepDto | null;
  cursor: number;
  totalSteps: number;
  steps: StepDto[];
  progress: number;
  remaining: number;
  elapsedSec: number;
  stdSec: number;
  overrun: boolean;
  error: string | null;
  busy: boolean;
  // 文言
  loadingLabel: string;
  selectTaskPrompt: string;
  startButtonLabel: string;
  allStepsDoneLabel: string;
  noStepsTitle: string;
  completionPrefix: string;
  standardTimePrefix: string;
  nextActionPrefix: string;
  nextActionLabel: string;
  overrunLabel: string;
  progressAriaLabel: string;
  imageLabel: string;
  videoLabel: string;
  diagramLabel: string;
  peekTitle: string;
  peekEndLabel: string;
  completionLabel(criteria: string): string;
  // 動作
  onStart(): void;
  onRetry(): void;
  onDismissError(): void;
}

export function NavigationStepFocus(props: NavigationStepFocusProps): JSX.Element {
  return (
    <main
      style={{
        gridArea: 'main',
        padding: `${space[5]} ${space[6]}`,
        overflowY: 'auto',
        display: 'flex',
        flexDirection: 'column',
        gap: space[4]
      }}
    >
      {props.error !== null && (
        <ErrorPanel message={props.error} onRetry={props.onRetry} onDismiss={props.onDismissError} />
      )}
      {props.stepsLoading ? (
        <LoadingShell loadingLabel={props.loadingLabel} />
      ) : props.selectedTaskState === 'Idle' || props.selectedTaskState === 'Ready' ? (
        <StartPanel
          prompt={props.selectTaskPrompt}
          startLabel={props.startButtonLabel}
          disabled={props.busy || props.selectedTaskId === null}
          onStart={props.onStart}
        />
      ) : props.current !== null ? (
        <BentoGrid {...props} />
      ) : props.totalSteps > 0 ? (
        <CompletedPanel allDoneLabel={props.allStepsDoneLabel} />
      ) : (
        <EmptyState iconName="note" title={props.noStepsTitle} />
      )}
    </main>
  );
}

function LoadingShell(props: { loadingLabel: string }): JSX.Element {
  return (
    <div
      role="status"
      aria-live="polite"
      aria-label={props.loadingLabel}
      style={{
        display: 'grid',
        gap: space[4],
        gridTemplateColumns: 'minmax(0, 2fr) minmax(0, 1fr)',
        gridTemplateRows: 'auto auto',
        flex: 1
      }}
    >
      <div
        style={{
          padding: space[6],
          background: palette.surface,
          borderRadius: radius.large,
          boxShadow: elevation[2],
          display: 'flex',
          flexDirection: 'column',
          gap: space[3]
        }}
      >
        <Skeleton height={32} width="40%" />
        <Skeleton height={48} width="80%" />
        <Skeleton height={20} width="50%" />
      </div>
      <div
        style={{
          padding: space[5],
          background: palette.surface,
          borderRadius: radius.large,
          boxShadow: elevation[2],
          display: 'grid',
          placeItems: 'center'
        }}
      >
        <Skeleton width={160} height={160} shape="pill" />
      </div>
      <div style={{ gridColumn: '1 / span 2', display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: space[3] }}>
        <Skeleton height={96} shape="medium" />
        <Skeleton height={96} shape="medium" />
        <Skeleton height={96} shape="medium" />
      </div>
    </div>
  );
}

interface StartPanelProps {
  prompt: string;
  startLabel: string;
  disabled: boolean;
  onStart(): void;
}

function StartPanel(props: StartPanelProps): JSX.Element {
  return (
    <section
      style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: space[5],
        padding: space[7],
        background: palette.surface,
        borderRadius: radius.large,
        boxShadow: elevation[2]
      }}
    >
      <h2 style={{ margin: 0, fontSize: fontSize.title, fontWeight: fontWeight.bold, color: palette.fg }}>
        {props.prompt}
      </h2>
      <Button
        variant="primary"
        size="xl"
        onClick={props.onStart}
        disabled={props.disabled}
        leadingIcon={<Icon name="play" size={32} />}
      >
        {props.startLabel}
      </Button>
    </section>
  );
}

function CompletedPanel(props: { allDoneLabel: string }): JSX.Element {
  return (
    <section
      style={{
        flex: 1,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexDirection: 'column',
        gap: space[3],
        padding: space[7],
        background: palette.success.subtle,
        color: palette.success.strong,
        borderRadius: radius.large,
        boxShadow: elevation[1]
      }}
    >
      <Icon name="sparkle" size={48} color={palette.success.default} />
      <strong style={{ fontSize: fontSize.title, fontWeight: fontWeight.bold }}>{props.allDoneLabel}</strong>
    </section>
  );
}

function BentoGrid(props: NavigationStepFocusProps): JSX.Element {
  const c = props.current!;
  const stepNumber = props.cursor + 1;
  const next = props.steps[props.cursor + 1] ?? null;
  return (
    <div
      style={{
        display: 'grid',
        gap: space[4],
        gridTemplateColumns: 'minmax(0, 2fr) minmax(0, 1fr)',
        gridTemplateAreas: '"primary gauge" "media peek"',
        flex: 1
      }}
    >
      <PrimaryCard
        c={c}
        stepNumber={stepNumber}
        totalSteps={props.totalSteps}
        overrun={props.overrun}
        completionPrefix={props.completionPrefix}
        standardTimePrefix={props.standardTimePrefix}
        nextActionPrefix={props.nextActionPrefix}
        nextActionLabel={props.nextActionLabel}
        overrunLabel={props.overrunLabel}
        completionLabel={props.completionLabel}
      />
      <div
        style={{
          gridArea: 'gauge',
          background: palette.surface,
          borderRadius: radius.large,
          boxShadow: elevation[1],
          padding: space[5],
          display: 'grid',
          placeItems: 'center'
        }}
      >
        <NavigationProgressGauge
          progress={props.progress}
          remaining={props.remaining}
          elapsedSec={props.elapsedSec}
          stdSec={props.stdSec}
          overrun={props.overrun}
          ariaLabel={props.progressAriaLabel}
          diameter={180}
        />
      </div>
      <div style={{ gridArea: 'media' }}>
        <NavigationStepMedia
          imageLabel={props.imageLabel}
          videoLabel={props.videoLabel}
          diagramLabel={props.diagramLabel}
        />
      </div>
      <div style={{ gridArea: 'peek' }}>
        <NavigationStepPeek
          next={next}
          index={props.cursor + 1}
          total={props.totalSteps}
          title={props.peekTitle}
          endLabel={props.peekEndLabel}
        />
      </div>
    </div>
  );
}

interface PrimaryCardProps {
  c: StepDto;
  stepNumber: number;
  totalSteps: number;
  overrun: boolean;
  completionPrefix: string;
  standardTimePrefix: string;
  nextActionPrefix: string;
  nextActionLabel: string;
  overrunLabel: string;
  completionLabel(criteria: string): string;
}

function PrimaryCard(props: PrimaryCardProps): JSX.Element {
  return (
    <section
      style={{
        gridArea: 'primary',
        background: palette.surface,
        borderRadius: radius.large,
        boxShadow: elevation[2],
        padding: space[6],
        display: 'flex',
        flexDirection: 'column',
        gap: space[4],
        minWidth: 0
      }}
    >
      <div style={{ display: 'flex', alignItems: 'baseline', gap: space[3], flexWrap: 'wrap' }}>
        <span
          aria-label={`step ${props.stepNumber} of ${props.totalSteps}`}
          style={{ fontSize: '72px', fontWeight: fontWeight.bold, color: palette.info.default, lineHeight: 1 }}
        >
          {String(props.stepNumber).padStart(2, '0')}
        </span>
        <span style={{ fontSize: fontSize.subtitle, color: palette.fgMuted, fontWeight: fontWeight.medium }}>
          / {String(props.totalSteps).padStart(2, '0')}
        </span>
        {props.overrun && (
          <StatusPill toneName="danger" iconName="warning-triangle" size="sm" role="alert">
            {props.overrunLabel}
          </StatusPill>
        )}
      </div>
      <h1
        style={{
          margin: 0,
          fontSize: fontSize.display,
          fontWeight: fontWeight.bold,
          color: palette.fg,
          lineHeight: 1.2,
          wordBreak: 'break-word'
        }}
      >
        {props.c.label}
      </h1>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: space[3] }}>
        <StatusPill toneName="info" size="sm">
          {props.completionPrefix}: {props.completionLabel(props.c.completion_criteria)}
        </StatusPill>
        <StatusPill toneName="neutral" size="sm">
          {props.standardTimePrefix}: {props.c.standard_time_seconds}s
        </StatusPill>
      </div>
      <div
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: space[3],
          padding: `${space[3]} ${space[4]}`,
          background: palette.warning.subtle,
          color: palette.warning.strong,
          border: `1px solid ${palette.warning.default}`,
          borderRadius: radius.medium,
          fontSize: fontSize.subtitle,
          fontWeight: fontWeight.medium
        }}
      >
        <Icon name="chevron-right" size={24} color={palette.warning.strong} />
        {props.nextActionPrefix}: <strong>{props.nextActionLabel}</strong>
      </div>
    </section>
  );
}
