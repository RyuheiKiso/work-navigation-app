// 対応 §: ロードマップ §3.6.4 §9.5.2 §11.2.3
// ステップ完了時のごく短い視覚フィードバック (240ms decelerated)。
// チェックを画面中央にバースト表示し、外周にリングを 1 つ拡散する。
// `prefers-reduced-motion: reduce` 時は global.css 側で 0.01ms に短縮されるため自動で抑制される。
// 触覚／音は `triggerFeedback('success')` に任せ、ここは視覚等価物のみ担う。

import { useEffect, useState } from 'react';
import { palette } from '../../tokens/access';
import { Icon } from './icon/icon';

export interface CelebrationOverlayProps {
  // 単調増加のシード値。値が変化したタイミングで 1 度だけ発火する。
  trigger: number;
  // 表示時間 (ms)。既定 600ms。
  durationMs?: number;
}

export function CelebrationOverlay(props: CelebrationOverlayProps): JSX.Element | null {
  const [active, setActive] = useState(false);

  useEffect(() => {
    if (props.trigger === 0) return;
    setActive(true);
    const id = window.setTimeout(() => setActive(false), props.durationMs ?? 600);
    return () => window.clearTimeout(id);
  }, [props.trigger, props.durationMs]);

  if (!active) return null;

  return (
    <div
      aria-hidden="true"
      data-motion="decoration"
      style={{
        position: 'fixed',
        inset: 0,
        pointerEvents: 'none',
        display: 'grid',
        placeItems: 'center',
        zIndex: 1100
      }}
    >
      {/* 拡散リング: 成功色を薄く環状に展開 */}
      <span
        className="wna-celebrate-ring"
        style={{
          position: 'absolute',
          width: 240,
          height: 240,
          borderRadius: '50%',
          background: palette.success.subtle,
          border: `4px solid ${palette.success.default}`
        }}
      />
      {/* 中心バースト: 大きなチェックマークを成功円の上に */}
      <div
        className="wna-celebrate-burst"
        style={{
          width: 144,
          height: 144,
          borderRadius: '50%',
          background: palette.success.default,
          color: palette.white,
          display: 'grid',
          placeItems: 'center',
          boxShadow: '0 12px 32px rgba(0,0,0,0.25)'
        }}
      >
        <Icon name="check" size={48} strokeWidth={3} color={palette.white} />
      </div>
    </div>
  );
}
