// 対応 §: ロードマップ §11.2 §9.2.2 §3.6.4
// CTA の階層を可視化するため、完了 (xl, primary) を中央 60%、中断と Andon を補助で並べる。
// アンドンは danger variant で角丸を 0 に固定（剛性表現で誤タップ抑止、§9.2.2）。
// ボタン内に `<kbd>` チップでショートカット可視化（覚える前から目に入る）。
// 完了 CTA は HoldConfirmButton で 800ms 押し続け確定 — 手袋・揺れ環境での
// 誤タップを物理閾値で抑止しつつ、キーボード Enter は意図性が高いため即発火。

import { space } from '../../../tokens/access';
import { Icon } from '../icon/icon';
import { Button } from '../primitives/button';
import { HoldConfirmButton } from '../primitives/hold-confirm-button';

interface ButtonContentProps {
  icon: JSX.Element;
  label: string;
  shortcut?: string | undefined;
}

function ButtonContent(props: ButtonContentProps): JSX.Element {
  return (
    <>
      {props.icon}
      <span style={{ marginLeft: 8 }}>{props.label}</span>
      {props.shortcut !== undefined && (
        <kbd style={{ marginLeft: 12, opacity: 0.85 }}>{props.shortcut}</kbd>
      )}
    </>
  );
}

export interface NavigationActionBarProps {
  busy: boolean;
  hasCurrentStep: boolean;
  taskRunning: boolean;
  selectedTaskId: string | null;
  andonActive: boolean;
  completeLabel: string;
  suspendLabel: string;
  resumeLabel: string;
  andonLabel: string;
  // ショートカット文字列（"Enter" 等）。表示専用。実際の発火は shell 側で hook が担う。
  completeShortcut?: string;
  suspendShortcut?: string;
  andonShortcut?: string;
  onComplete(): void;
  onSuspendOrResume(): void;
  onAndon(): void;
}

export function NavigationActionBar(props: NavigationActionBarProps): JSX.Element {
  const completeDisabled = props.busy || !props.hasCurrentStep || !props.taskRunning;
  const suspendDisabled = props.busy || props.selectedTaskId === null;
  return (
    <footer
      style={{
        gridArea: 'actions',
        padding: `${space[3]} ${space[5]}`,
        display: 'grid',
        gridTemplateColumns: '1fr 3fr 1fr',
        gap: space[4],
        alignItems: 'center',
        background: 'transparent'
      }}
    >
      <Button
        variant={props.andonActive ? 'primary' : 'warning'}
        size="lg"
        block
        onClick={props.onSuspendOrResume}
        disabled={suspendDisabled}
      >
        <ButtonContent
          icon={<Icon name={props.andonActive ? 'play' : 'pause'} size={24} />}
          label={props.andonActive ? props.resumeLabel : props.suspendLabel}
          shortcut={props.suspendShortcut}
        />
      </Button>
      <HoldConfirmButton
        variant="primary"
        size="xl"
        block
        holdMs={800}
        onHoldComplete={props.onComplete}
        disabled={completeDisabled}
      >
        <ButtonContent
          icon={<Icon name="check" size={32} strokeWidth={3} />}
          label={props.completeLabel}
          shortcut={props.completeShortcut}
        />
      </HoldConfirmButton>
      <Button variant="danger" size="lg" block onClick={props.onAndon}>
        <ButtonContent
          icon={<Icon name="andon-tower" size={24} />}
          label={props.andonLabel}
          shortcut={props.andonShortcut}
        />
      </Button>
    </footer>
  );
}
