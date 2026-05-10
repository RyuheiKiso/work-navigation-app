// 対応 §: ロードマップ §10.4.3 §13.1 §11.2.2
// 音声コマンド正規化の単体テスト。

import { describe, it, expect } from 'vitest';
import { normalize } from './voice-command';

describe('voice command normalization', () => {
  // 各既定コマンドの認識
  it.each([
    ['開始', 'start'],
    ['スタート', 'start'],
    ['完了', 'complete'],
    ['中断', 'suspend'],
    ['戻る', 'back'],
    ['メモ', 'memo'],
    ['撮影', 'capture']
  ])('recognizes Japanese: %s -> %s', (input, expected) => {
    expect(normalize(input)).toBe(expected);
  });

  // 英語
  it('recognizes English aliases', () => {
    expect(normalize('start')).toBe('start');
    expect(normalize('complete')).toBe('complete');
  });

  // 韓国語
  it('recognizes Korean aliases', () => {
    expect(normalize('시작')).toBe('start');
    expect(normalize('완료')).toBe('complete');
  });

  // 中国語
  it('recognizes Chinese aliases', () => {
    expect(normalize('开始')).toBe('start');
    expect(normalize('完成')).toBe('complete');
  });

  // 部分一致（後置丁寧語）
  it('matches partial recognition', () => {
    expect(normalize('開始してください')).toBe('start');
    expect(normalize('please start')).toBe('start');
  });

  // 未マッチ
  it('returns null for unrecognized', () => {
    expect(normalize('xyz')).toBeNull();
    expect(normalize('')).toBeNull();
  });
});
