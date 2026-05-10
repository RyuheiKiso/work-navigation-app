// 対応 §: ロードマップ §3.6 §3.6.2 §3.6.4 §10.4 §10.6
// ナビゲーションシェルのドメイン状態 (tasks / steps / andon / 経過時間) と
// API 連携ハンドラを集約するカスタムフック。
// 表示層 (navigation-shell.tsx) からこのフックの戻り値だけを参照する。

import { useEffect, useMemo, useRef, useState } from 'react';
import { normalize } from '../../domain/voice-command';
import { evaluate as evaluateStorage } from '../../domain/storage-lifecycle';
import {
  listTasks,
  listSteps,
  startTask,
  completeTask,
  suspendTask,
  resumeTask,
  markStepDone,
  appendRecord,
  type StepDto,
  type TaskListItem
} from '../../adapter/api-client';
import { triggerFeedback } from '../utils/feedback';

export interface AndonState {
  severity: 1 | 2 | 3 | 4 | 5;
  message: string;
}

export interface TaskNavigation {
  tasks: TaskListItem[];
  selectedTaskId: string | null;
  selectedTaskState: string;
  steps: StepDto[];
  current: StepDto | null;
  cursor: number;
  progress: number;
  remaining: number;
  andon: AndonState | null;
  busy: boolean;
  error: string | null;
  elapsedSec: number;
  stdSec: number;
  overrun: boolean;
  storage: ReturnType<typeof evaluateStorage>;
  voiceInputRef: React.RefObject<HTMLInputElement>;
  selectTask(id: string, state: string): void;
  doStartTask(): Promise<void>;
  doCompleteCurrent(): Promise<void>;
  doSuspend(): Promise<void>;
  doResume(): Promise<void>;
  fireAndon(): void;
  handleVoiceCommand(): void;
}

export function useTaskNavigation(): TaskNavigation {
  const [tasks, setTasks] = useState<TaskListItem[]>([]);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [selectedTaskState, setSelectedTaskState] = useState<string>('Idle');
  const [steps, setSteps] = useState<StepDto[]>([]);
  const [andon, setAndon] = useState<AndonState | null>(null);
  const [stepStartedAt, setStepStartedAt] = useState<number>(Date.now());
  const [now, setNow] = useState<number>(Date.now());
  const [busy, setBusy] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const voiceInputRef = useRef<HTMLInputElement | null>(null);
  const lamportRef = useRef<number>(0);

  const storage = useMemo(
    () =>
      evaluateStorage({
        totalBytes: 64 * 1024 * 1024 * 1024,
        usedBytes: 30 * 1024 * 1024 * 1024
      }),
    []
  );

  useEffect(() => {
    void (async () => {
      try {
        const ts = await listTasks();
        setTasks(ts);
        if (ts.length > 0 && !selectedTaskId) {
          setSelectedTaskId(ts[0]!.id);
          setSelectedTaskState(ts[0]!.state);
        }
      } catch (e) {
        setError((e as Error).message);
      }
    })();
    const tick = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(tick);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!selectedTaskId) return;
    void (async () => {
      try {
        const s = await listSteps(selectedTaskId);
        setSteps(s);
        setStepStartedAt(Date.now());
      } catch (e) {
        setError((e as Error).message);
      }
    })();
  }, [selectedTaskId]);

  const cursor = steps.findIndex((s) => !s.done);
  const current = cursor >= 0 ? steps[cursor]! : null;
  const progress = steps.length === 0 ? 0 : steps.filter((s) => s.done).length / steps.length;
  const remaining = steps.length - steps.filter((s) => s.done).length;
  const elapsedSec = Math.floor((now - stepStartedAt) / 1000);
  const stdSec = current?.standard_time_seconds ?? 0;
  const overrun = stdSec > 0 && elapsedSec > stdSec;

  async function refreshTasks(): Promise<void> {
    try {
      const ts = await listTasks();
      setTasks(ts);
    } catch {
      /* 無視 */
    }
  }

  async function refreshSteps(): Promise<void> {
    if (!selectedTaskId) return;
    try {
      const s = await listSteps(selectedTaskId);
      setSteps(s);
    } catch {
      /* 無視 */
    }
  }

  function selectTask(id: string, state: string): void {
    setSelectedTaskId(id);
    setSelectedTaskState(state);
  }

  async function doStartTask(): Promise<void> {
    if (!selectedTaskId) return;
    setBusy(true);
    setError(null);
    try {
      const task = await startTask(selectedTaskId);
      setSelectedTaskState(task.state);
      lamportRef.current = task.lamport;
      triggerFeedback('success');
      await refreshTasks();
    } catch (e) {
      setError((e as Error).message);
      triggerFeedback('fail');
    } finally {
      setBusy(false);
    }
  }

  async function doCompleteCurrent(): Promise<void> {
    if (!current || !selectedTaskId) return;
    setBusy(true);
    setError(null);
    try {
      await markStepDone(selectedTaskId, current.id);
      lamportRef.current += 1;
      await appendRecord(selectedTaskId, 'browser-001', lamportRef.current, {
        step_id: current.id,
        completion_criteria: current.completion_criteria,
        elapsed_sec: Math.floor((Date.now() - stepStartedAt) / 1000)
      });
      triggerFeedback('success');
      await refreshSteps();
      setStepStartedAt(Date.now());
      const after = await listSteps(selectedTaskId);
      if (after.every((s) => s.done)) {
        const task = await completeTask(selectedTaskId, {
          manually_marked: true,
          photo_attached: false
        });
        setSelectedTaskState(task.state);
        await refreshTasks();
      }
    } catch (e) {
      setError((e as Error).message);
      triggerFeedback('fail');
    } finally {
      setBusy(false);
    }
  }

  async function doSuspend(): Promise<void> {
    if (!selectedTaskId) return;
    setBusy(true);
    setError(null);
    try {
      const task = await suspendTask(selectedTaskId);
      setSelectedTaskState(task.state);
      setAndon({ severity: 1, message: '一時中断中' });
      triggerFeedback('input');
      await refreshTasks();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  async function doResume(): Promise<void> {
    if (!selectedTaskId) return;
    setBusy(true);
    setError(null);
    try {
      const task = await resumeTask(selectedTaskId);
      setSelectedTaskState(task.state);
      setAndon(null);
      triggerFeedback('input');
      await refreshTasks();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  function fireAndon(): void {
    setAndon({ severity: 4, message: '部材切れ — 班長に応援要請しました' });
    triggerFeedback('warning');
  }

  function handleVoiceCommand(): void {
    const v = voiceInputRef.current?.value ?? '';
    const cmd = normalize(v);
    if (!cmd) {
      setError('音声コマンドが認識できませんでした');
      return;
    }
    if (voiceInputRef.current) voiceInputRef.current.value = '';
    if (cmd === 'complete') void doCompleteCurrent();
    else if (cmd === 'suspend') void doSuspend();
    else if (cmd === 'start') void doResume();
    else if (cmd === 'capture') void doCompleteCurrent();
  }

  return {
    tasks,
    selectedTaskId,
    selectedTaskState,
    steps,
    current,
    cursor,
    progress,
    remaining,
    andon,
    busy,
    error,
    elapsedSec,
    stdSec,
    overrun,
    storage,
    voiceInputRef,
    selectTask,
    doStartTask,
    doCompleteCurrent,
    doSuspend,
    doResume,
    fireAndon,
    handleVoiceCommand
  };
}
