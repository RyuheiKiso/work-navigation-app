// 対応 §: ロードマップ §3.6.2 §10.6 §11.2.3 §9.5.4
// ステップマップ＋ストレージ＋音声コマンド。フォーカス領域を確保するため折りたたみ既定。
// ●○✓ などの記号文字を Icon 形状（circle-filled / circle-outline / check）に置換し、
// 色のみで状態を伝えない設計（§11.2.2）に揃える。
// 音声入力は騒音下でテキスト入力に頼らないよう、PTT (push-to-talk) 風の大型ボタンを主役にする。

import type { RefObject } from 'react';
import type { StepDto } from '../../../adapter/api-client';
import { palette, fontSize, fontWeight, radius, space } from '../../../tokens/access';
import { Icon } from '../icon/icon';
import { Button } from '../primitives/button';
import { tone, type SemanticTone } from '../../../tokens/access';

export interface NavigationStepMapDrawerProps {
  open: boolean;
  steps: StepDto[];
  cursor: number;
  storageStatus: 'normal' | 'warning' | 'blocked';
  storageUtilization: number;
  voiceInputRef: RefObject<HTMLInputElement>;
  onVoiceCommand(): void;
  stepMapTitle: string;
  storageTitle: string;
  voiceTitle: string;
  voicePlaceholder: string;
  voiceButtonLabel: string;
}

function statusToTone(status: NavigationStepMapDrawerProps['storageStatus']): SemanticTone {
  if (status === 'normal') return 'success';
  if (status === 'warning') return 'warning';
  return 'danger';
}

export function NavigationStepMapDrawer(props: NavigationStepMapDrawerProps): JSX.Element | null {
  if (!props.open) return null;
  const t = tone(statusToTone(props.storageStatus));
  return (
    <aside
      aria-label={props.stepMapTitle}
      style={{
        gridArea: 'stepMapDrawer',
        width: 320,
        background: palette.white,
        borderLeft: `1px solid ${palette.neutral[200]}`,
        overflowY: 'auto',
        padding: space[3],
        display: 'flex',
        flexDirection: 'column',
        gap: space[5]
      }}
    >
      <section>
        <h2 style={sectionTitle}>
          <Icon name="list" size={18} />
          {props.stepMapTitle}
        </h2>
        <ol style={{ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: space[1] }}>
          {props.steps.map((s, i) => {
            const done = s.done;
            const current = i === props.cursor;
            const fg = done ? palette.success.strong : current ? palette.info.strong : palette.neutral[700];
            const bg = current ? palette.info.subtle : 'transparent';
            return (
              <li
                key={s.id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: space[2],
                  padding: `${space[1]} ${space[2]}`,
                  background: bg,
                  borderRadius: radius.small,
                  color: fg,
                  fontSize: fontSize.caption
                }}
              >
                <Icon
                  name={done ? 'check' : current ? 'circle-filled' : 'circle-outline'}
                  size={16}
                  color={done ? palette.success.default : current ? palette.info.default : palette.neutral[400]}
                />
                <span style={{ flex: 1 }}>{s.label}</span>
              </li>
            );
          })}
        </ol>
      </section>

      <section>
        <h2 style={sectionTitle}>
          <Icon name="shield-check" size={18} />
          {props.storageTitle}
        </h2>
        <div
          style={{
            padding: space[2],
            background: t.bg,
            color: t.fg,
            border: `1px solid ${t.border}`,
            borderRadius: radius.small,
            fontSize: fontSize.caption,
            fontWeight: fontWeight.medium
          }}
        >
          {Math.round(props.storageUtilization * 100)}% — {props.storageStatus}
        </div>
      </section>

      <section>
        <h2 style={sectionTitle}>
          <Icon name="mic" size={18} />
          {props.voiceTitle}
        </h2>
        <input
          ref={props.voiceInputRef}
          placeholder={props.voicePlaceholder}
          aria-label={props.voiceTitle}
          onKeyDown={(e) => {
            if (e.key === 'Enter') props.onVoiceCommand();
          }}
          style={{
            width: '100%',
            padding: space[2],
            marginBottom: space[2],
            border: `1px solid ${palette.neutral[300]}`,
            borderRadius: radius.medium,
            fontSize: fontSize.caption
          }}
        />
        <Button
          variant="ghost"
          size="md"
          block
          onClick={props.onVoiceCommand}
          leadingIcon={<Icon name="mic" size={20} />}
        >
          {props.voiceButtonLabel}
        </Button>
      </section>
    </aside>
  );
}

const sectionTitle = {
  margin: 0,
  marginBottom: space[2],
  fontSize: fontSize.caption,
  fontWeight: fontWeight.bold,
  color: palette.neutral[700],
  display: 'flex',
  alignItems: 'center',
  gap: space[2],
  textTransform: 'uppercase' as const,
  letterSpacing: '0.06em'
};
