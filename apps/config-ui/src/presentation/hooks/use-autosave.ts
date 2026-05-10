// 対応 §: ロードマップ §10.2.2 §14.2
// 任意 state を debounce して localStorage に永続化する汎用 Hook。
// 連続編集中の書き込み回数を抑制しつつ、最終状態は必ず保存されることを保証する。
// 値の同一性は参照ではなく JSON シリアライズ後の文字列で判定する。
// 呼び出し側が毎レンダで新規オブジェクトを生成しても無駄な書き込みが発生しない。

import { useEffect, useRef, useState } from 'react';

export type AutosaveStatus = 'idle' | 'saving' | 'saved' | 'error';

export interface UseAutosaveOptions<T> {
  readonly value: T;
  readonly debounceMs?: number;
  readonly enabled?: boolean;
  readonly write: (value: T) => void;
}

export interface UseAutosaveResult {
  readonly status: AutosaveStatus;
  readonly lastSavedAt: number | null;
}

export function useAutosave<T>(opts: UseAutosaveOptions<T>): UseAutosaveResult {
  const debounceMs = opts.debounceMs ?? 800;
  const enabled = opts.enabled ?? true;
  const [status, setStatus] = useState<AutosaveStatus>('idle');
  const [lastSavedAt, setLastSavedAt] = useState<number | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  // 初回マウント時の値は永続化済みとみなし書き込みを抑制する
  const skipFirst = useRef(true);
  // write/value を ref 化して effect の再実行を JSON 同値比較に絞る
  const writeRef = useRef(opts.write);
  writeRef.current = opts.write;
  const valueRef = useRef(opts.value);
  valueRef.current = opts.value;
  const valueKey = JSON.stringify(opts.value);

  useEffect(() => {
    if (skipFirst.current) {
      skipFirst.current = false;
      return;
    }
    if (!enabled) return;
    setStatus('saving');
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      try {
        writeRef.current(valueRef.current);
        setLastSavedAt(Date.now());
        setStatus('saved');
      } catch {
        setStatus('error');
      }
    }, debounceMs);
    return () => {
      if (timer.current !== null) {
        clearTimeout(timer.current);
        timer.current = null;
      }
    };
  }, [valueKey, debounceMs, enabled]);

  return { status, lastSavedAt };
}
