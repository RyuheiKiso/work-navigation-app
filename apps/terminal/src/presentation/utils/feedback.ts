// 対応 §: ロードマップ §3.6.4 §11.2
// 触覚 (Vibration API) と聴覚 (WebAudio) のフィードバックを発火する。
// ブラウザサポートのばらつきは握り潰す。

export type FeedbackKind = 'success' | 'fail' | 'warning' | 'input';

export function triggerFeedback(kind: FeedbackKind): void {
  try {
    if ('vibrate' in navigator) {
      const pattern =
        kind === 'success' ? [50] :
        kind === 'warning' ? [50, 80, 50] :
        kind === 'fail' ? [120] : [20];
      navigator.vibrate(pattern);
    }
  } catch { /* */ }
  try {
    const Ctx = (window as unknown as { AudioContext?: typeof AudioContext }).AudioContext;
    if (!Ctx) return;
    const ctx = new Ctx();
    const o = ctx.createOscillator();
    const g = ctx.createGain();
    o.connect(g); g.connect(ctx.destination);
    const t0 = ctx.currentTime;
    if (kind === 'success') { o.frequency.setValueAtTime(523, t0); o.frequency.setValueAtTime(659, t0 + 0.1); }
    else if (kind === 'fail') { o.frequency.setValueAtTime(330, t0); o.frequency.setValueAtTime(262, t0 + 0.15); }
    else if (kind === 'warning') { o.frequency.setValueAtTime(440, t0); }
    else { o.frequency.setValueAtTime(1047, t0); }
    g.gain.setValueAtTime(0.05, t0);
    g.gain.exponentialRampToValueAtTime(0.001, t0 + 0.2);
    o.start(t0); o.stop(t0 + 0.25);
  } catch { /* */ }
}
