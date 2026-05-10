// 対応 §: ロードマップ §9.2.2 §11.2 §11.2.3
// 破壊的・不可逆操作 (完了確定／アンドン発報) の誤タップを物理的閾値で抑止する Hook。
// 単押しでは手袋・濡れ手・揺れる搬送台車での誤発火を防げないため、
// `holdMs` ミリ秒の連続押下で初めて onComplete を発火する。
// ハプティックは段階的フィードバック (押下開始→刻み振動→確定で強振動) で、
// 屋外騒音 80dB 環境でも触覚で「決断の進行」を伝える。

import { useCallback, useEffect, useRef, useState } from 'react';

export type HoldState = 'idle' | 'pressing' | 'completed' | 'cancelled';

export interface UseHoldConfirmOptions {
  /** 確定までの保持時間 (ms)。屋外運用では 800〜1200ms を推奨 */
  holdMs: number;
  /** 閾値到達時に 1 回だけ呼ばれる */
  onComplete: () => void;
  /** 閾値未満で離した時に呼ばれる (任意) */
  onCancel?: () => void;
  /** ハプティック発火の有無。テスト・低スペック端末で抑止可能 */
  hapticEnabled?: boolean;
}

export interface PointerHandlers {
  onPointerDown: () => void;
  onPointerUp: () => void;
  onPointerCancel: () => void;
  onPointerLeave: () => void;
}

export interface UseHoldConfirmResult {
  /** idle→pressing→(completed|cancelled) の状態機械 */
  state: HoldState;
  /** 0..1 の進捗。SVG 円弧や横バーの描画に使う */
  progress: number;
  /** ボタン要素にスプレッドする pointer ハンドラ群 */
  pointerHandlers: PointerHandlers;
  /** 状態を idle / progress=0 に戻す。連続発火後の再アーム用 */
  reset: () => void;
}

// 16ms ≒ 60fps。setInterval ベースで RAF を避け、fake-timer での property test を可能にする
const TICK_MS = 16;
// 押下開始の合図 (短) と確定 (強)
const HAPTIC_START_MS = 15;
const HAPTIC_COMPLETE_MS = 80;
// 進行感を伝える刻みハプティックの間隔 (短)
const HAPTIC_TICK_MS = 8;
const HAPTIC_TICK_INTERVAL_MS = 100;

function vibrate(ms: number): void {
  if (typeof navigator === 'undefined') return;
  if (typeof navigator.vibrate !== 'function') return;
  try { navigator.vibrate(ms); } catch { /* 一部環境では throw する。視覚演出にフォールバック */ }
}

export function useHoldConfirm(options: UseHoldConfirmOptions): UseHoldConfirmResult {
  const { holdMs, onComplete, onCancel, hapticEnabled = true } = options;
  const [state, setState] = useState<HoldState>('idle');
  const [progress, setProgress] = useState(0);

  const startedAt = useRef<number | null>(null);
  const timerId = useRef<ReturnType<typeof setInterval> | null>(null);
  const completedFired = useRef(false);
  const lastHapticStep = useRef(0);

  // クロージャ越しに最新の onComplete/onCancel を呼ぶ
  const onCompleteRef = useRef(onComplete);
  const onCancelRef = useRef(onCancel);
  useEffect(() => { onCompleteRef.current = onComplete; }, [onComplete]);
  useEffect(() => { onCancelRef.current = onCancel; }, [onCancel]);

  const stopTimer = useCallback((): void => {
    if (timerId.current !== null) {
      clearInterval(timerId.current);
      timerId.current = null;
    }
  }, []);

  const haptic = useCallback((ms: number): void => {
    if (hapticEnabled) vibrate(ms);
  }, [hapticEnabled]);

  const reset = useCallback((): void => {
    stopTimer();
    startedAt.current = null;
    completedFired.current = false;
    lastHapticStep.current = 0;
    setState('idle');
    setProgress(0);
  }, [stopTimer]);

  const onPointerDown = useCallback((): void => {
    // 既に押下中・確定済みなら無視 (連続発火防止)
    if (timerId.current !== null) return;
    if (completedFired.current) return;
    startedAt.current = Date.now();
    lastHapticStep.current = 0;
    setState('pressing');
    setProgress(0);
    haptic(HAPTIC_START_MS);
    timerId.current = setInterval(() => {
      const start = startedAt.current;
      if (start === null) return;
      const elapsed = Date.now() - start;
      const p = Math.min(elapsed / holdMs, 1);
      setProgress(p);
      const stepIndex = Math.floor(elapsed / HAPTIC_TICK_INTERVAL_MS);
      if (stepIndex > lastHapticStep.current && p < 1) {
        lastHapticStep.current = stepIndex;
        haptic(HAPTIC_TICK_MS);
      }
      if (elapsed >= holdMs && !completedFired.current) {
        completedFired.current = true;
        stopTimer();
        startedAt.current = null;
        setState('completed');
        setProgress(1);
        haptic(HAPTIC_COMPLETE_MS);
        onCompleteRef.current();
      }
    }, TICK_MS);
  }, [holdMs, haptic, stopTimer]);

  const onPointerRelease = useCallback((): void => {
    // 既に確定済みなら何もしない (cancel として誤発火させない)
    if (completedFired.current) return;
    // 押下していない場合も何もしない
    if (timerId.current === null && startedAt.current === null) return;
    stopTimer();
    startedAt.current = null;
    setState('cancelled');
    setProgress(0);
    onCancelRef.current?.();
  }, [stopTimer]);

  // unmount 時に setInterval をリーク無く解放する
  useEffect(() => () => stopTimer(), [stopTimer]);

  return {
    state,
    progress,
    pointerHandlers: {
      onPointerDown,
      onPointerUp: onPointerRelease,
      onPointerCancel: onPointerRelease,
      onPointerLeave: onPointerRelease
    },
    reset
  };
}
