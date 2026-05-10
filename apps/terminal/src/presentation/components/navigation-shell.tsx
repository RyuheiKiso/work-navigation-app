// 対応 §: ロードマップ §3.6 §3.6.2 §3.6.4 §9.2 §9.5 §11.2 §10.4 §10.6
// ナビゲーションシェルのオーケストレーター。
// 単一責務に保つため、ヘッダ／タスクドロワー／ステップフォーカス／ステップマップドロワー／
// アクションバー／アンドンを子コンポーネントに分割する。
// ドロワーは現場フォーカス（§3.6.2 ナビ 6 機能）優先のため既定で閉じる。
// テーマ／ショートカット／完了演出を統括する。

import { useMemo, useState } from 'react';
import { t, getLocale, setLocale, isRtl, type LocaleKey } from '../../i18n';
import { logout } from '../../adapter/api-client';
import { useTaskNavigation } from '../hooks/use-task-navigation';
import { useOnlineStatus } from '../hooks/use-online-status';
import { useTheme } from '../hooks/use-theme';
import { useKeyboardShortcuts, formatShortcut, type ShortcutSpec } from '../hooks/use-keyboard-shortcuts';
import { ConfirmDialog } from './confirm-dialog';
import { CelebrationOverlay } from './celebration-overlay';
import { NavigationHeader } from './navigation/navigation-header';
import { NavigationTaskDrawer } from './navigation/navigation-task-drawer';
import { NavigationStepFocus } from './navigation/navigation-step-focus';
import { NavigationStepMapDrawer } from './navigation/navigation-step-map-drawer';
import { NavigationActionBar } from './navigation/navigation-action-bar';
import { NavigationAndonBanner } from './navigation/navigation-andon-banner';
import { NavigationShortcutsHelp } from './navigation/navigation-shortcuts-help';
import { palette, fontStack, space } from '../../tokens/access';

export interface NavigationShellProps {
  user: { user_id: string; display_name: string };
  onLogout(): void;
}

export function NavigationShell(props: NavigationShellProps): JSX.Element {
  const [locale, setLocaleState] = useState<LocaleKey>(getLocale());
  const nav = useTaskNavigation();
  const online = useOnlineStatus();
  const theme = useTheme();
  const [confirmingAndon, setConfirmingAndon] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  // ドロワー既定値: 「次に何を押すか」だけが見える状態にする
  const [taskDrawerOpen, setTaskDrawerOpen] = useState(false);
  const [stepMapDrawerOpen, setStepMapDrawerOpen] = useState(false);

  const taskRunning = nav.selectedTaskState === 'Running';
  const blocked = nav.storage.status === 'blocked';

  // ショートカット定義（メモ化して useEffect の再登録を最小化）
  const shortcuts = useMemo<ShortcutSpec[]>(() => {
    const list: ShortcutSpec[] = [
      {
        key: 'Enter',
        description: t('shortcut.complete'),
        handler: () => {
          if (nav.busy || nav.current === null || !taskRunning) return;
          void nav.doCompleteCurrent();
        }
      },
      {
        key: ' ',
        description: t('shortcut.suspend_resume'),
        handler: () => {
          if (nav.busy || nav.selectedTaskId === null) return;
          void (nav.andon ? nav.doResume() : nav.doSuspend());
        }
      },
      {
        key: 'a',
        description: t('shortcut.andon'),
        handler: () => setConfirmingAndon(true)
      },
      {
        key: 't',
        description: t('shortcut.toggle_task_drawer'),
        handler: () => setTaskDrawerOpen((v) => !v)
      },
      {
        key: 'm',
        description: t('shortcut.toggle_step_map'),
        handler: () => setStepMapDrawerOpen((v) => !v)
      },
      {
        key: 'd',
        description: t('shortcut.cycle_theme'),
        handler: () => theme.cycle()
      },
      {
        key: '?',
        shift: true,
        description: t('shortcut.show_help'),
        handler: () => setShortcutsOpen(true)
      },
      {
        key: 'Escape',
        description: t('shortcut.close_dialogs'),
        handler: () => {
          setShortcutsOpen(false);
          setConfirmingAndon(false);
        }
      }
    ];
    return list;
  }, [nav, taskRunning, theme]);

  useKeyboardShortcuts(shortcuts);

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns:
          (taskDrawerOpen ? '280px ' : '') +
          'minmax(0, 1fr)' +
          (stepMapDrawerOpen ? ' 320px' : ''),
        gridTemplateRows: 'auto auto minmax(0, 1fr) auto',
        gridTemplateAreas: gridAreas(taskDrawerOpen, stepMapDrawerOpen),
        height: '100vh',
        fontFamily: fontStack,
        background: blocked ? palette.danger.subtle : palette.bg,
        color: palette.fg
      }}
      dir={isRtl(locale) ? 'rtl' : 'ltr'}
    >
      <NavigationHeader
        userDisplayName={props.user.display_name}
        userId={props.user.user_id}
        selectedTaskState={nav.selectedTaskState}
        selectedTaskId={nav.selectedTaskId}
        taskStatePrefix={t('shell.state_prefix')}
        taskIdPrefix={t('shell.task_id_prefix')}
        taskIdUnselectedLabel={t('shell.task_id_unselected')}
        online={online}
        onlineLabel={t('network.online')}
        offlineLabel={t('network.offline')}
        networkAriaLabel={t('network.aria_label')}
        locale={locale}
        onLocaleChange={(l) => {
          void setLocale(l);
          setLocaleState(l);
        }}
        onLogout={() => {
          logout();
          props.onLogout();
        }}
        logoutLabel={t('shell.logout')}
        taskDrawerOpen={taskDrawerOpen}
        stepMapDrawerOpen={stepMapDrawerOpen}
        onToggleTaskDrawer={() => setTaskDrawerOpen((v) => !v)}
        onToggleStepMapDrawer={() => setStepMapDrawerOpen((v) => !v)}
        taskDrawerLabel={t('shell.toggle_task_drawer')}
        stepMapDrawerLabel={t('shell.toggle_step_map_drawer')}
        themeMode={theme.mode}
        onCycleTheme={theme.cycle}
        themeLabel={t('shell.theme_label')}
        onShowShortcuts={() => setShortcutsOpen(true)}
        shortcutsLabel={t('shell.shortcuts_label')}
      />

      <div style={{ gridArea: 'andon', padding: `${space[2]} ${space[5]}` }}>
        <NavigationAndonBanner
          severity={nav.andon ? nav.andon.severity : null}
          message={nav.andon?.message ?? ''}
          noAndonLabel={t('task.no_andon')}
          severityPrefix={t('task.andon_severity_prefix')}
        />
      </div>

      <NavigationTaskDrawer
        open={taskDrawerOpen}
        tasks={nav.tasks}
        loading={nav.tasksLoading}
        selectedTaskId={nav.selectedTaskId}
        title={t('task.today_list_title')}
        loadingLabel={t('state_label.loading_tasks')}
        emptyTitle={t('state_label.no_tasks_title')}
        emptyDescription={t('state_label.no_tasks_description')}
        onSelect={nav.selectTask}
      />

      <NavigationStepFocus
        stepsLoading={nav.stepsLoading}
        selectedTaskState={nav.selectedTaskState}
        selectedTaskId={nav.selectedTaskId}
        current={nav.current}
        cursor={nav.cursor < 0 ? nav.steps.length - 1 : nav.cursor}
        totalSteps={nav.steps.length}
        steps={nav.steps}
        progress={nav.progress}
        remaining={nav.remaining}
        elapsedSec={nav.elapsedSec}
        stdSec={nav.stdSec}
        overrun={nav.overrun}
        error={nav.error}
        busy={nav.busy}
        loadingLabel={t('state_label.loading_steps')}
        selectTaskPrompt={t('task.select_task_prompt')}
        startButtonLabel={t('task.start_button_short')}
        allStepsDoneLabel={t('task.all_steps_done')}
        noStepsTitle={t('state_label.no_steps_title')}
        completionPrefix={t('task.completion_criteria_label')}
        standardTimePrefix={t('task.standard_time_prefix')}
        nextActionPrefix={t('task.next_action_prefix')}
        nextActionLabel={t('action.complete')}
        overrunLabel={t('task.overrun_label')}
        progressAriaLabel={t('task.progress_aria_label')}
        imageLabel={t('media.image')}
        videoLabel={t('media.video')}
        diagramLabel={t('media.diagram')}
        peekTitle={t('task.peek_next_title')}
        peekEndLabel={t('task.peek_next_end')}
        completionLabel={(c) => t(`completion.${c}`)}
        onStart={() => void nav.doStartTask()}
        onRetry={() => void nav.retryTasks()}
        onDismissError={nav.dismissError}
      />

      <NavigationStepMapDrawer
        open={stepMapDrawerOpen}
        steps={nav.steps}
        cursor={nav.cursor}
        storageStatus={nav.storage.status}
        storageUtilization={nav.storage.utilization}
        voiceInputRef={nav.voiceInputRef}
        onVoiceCommand={nav.handleVoiceCommand}
        stepMapTitle={t('task.step_map_title')}
        storageTitle={t('task.storage_title')}
        voiceTitle={t('task.voice_section_title')}
        voicePlaceholder={t('task.voice_input_placeholder')}
        voiceButtonLabel={t('task.voice_recognize_button')}
      />

      <NavigationActionBar
        busy={nav.busy}
        hasCurrentStep={nav.current !== null}
        taskRunning={taskRunning}
        selectedTaskId={nav.selectedTaskId}
        andonActive={nav.andon !== null}
        completeLabel={t('task.complete_button')}
        suspendLabel={t('task.suspend_button')}
        resumeLabel={t('task.resume_button')}
        andonLabel={t('task.andon_button')}
        completeShortcut={formatShortcut({ key: 'Enter' })}
        suspendShortcut={formatShortcut({ key: ' ' })}
        andonShortcut={formatShortcut({ key: 'a' })}
        onComplete={() => void nav.doCompleteCurrent()}
        onSuspendOrResume={() => void (nav.andon ? nav.doResume() : nav.doSuspend())}
        onAndon={() => setConfirmingAndon(true)}
      />

      <ConfirmDialog
        open={confirmingAndon}
        title={t('confirm.andon_title')}
        description={t('confirm.andon_description')}
        confirmLabel={t('confirm.andon_confirm')}
        cancelLabel={t('confirm.cancel')}
        variant="danger"
        onConfirm={() => {
          setConfirmingAndon(false);
          nav.fireAndon();
        }}
        onCancel={() => setConfirmingAndon(false)}
      />

      <NavigationShortcutsHelp
        open={shortcutsOpen}
        shortcuts={shortcuts}
        title={t('shortcut.help_title')}
        closeLabel={t('shortcut.close_label')}
        onClose={() => setShortcutsOpen(false)}
      />

      <CelebrationOverlay trigger={nav.successCounter} />
    </div>
  );
}

// `gridTemplateAreas` をドロワー開閉ごとに動的に組む。
// 全幅行 (header / andon / actions) は同名を 1〜3 列ぶん繰り返す。
// メイン行のみ taskDrawer | main | stepMapDrawer を実際に切替える。
function gridAreas(taskOpen: boolean, mapOpen: boolean): string {
  const wrap = (cells: string[]): string => '"' + cells.join(' ') + '"';
  const fill = (name: string): string =>
    wrap([taskOpen ? name : null, name, mapOpen ? name : null].filter((v): v is string => v !== null));
  const main = wrap(
    [taskOpen ? 'taskDrawer' : null, 'main', mapOpen ? 'stepMapDrawer' : null].filter(
      (v): v is string => v !== null
    )
  );
  return [fill('header'), fill('andon'), main, fill('actions')].join(' ');
}
