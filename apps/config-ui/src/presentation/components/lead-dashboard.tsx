// 対応 §: ロードマップ §3.2.1.2 §3.6 §10.1
// 班長監視ダッシュボード: 全 Task の状態を一覧表示。

import { useEffect, useState } from 'react';
import { listDashboardTasks, type DashboardTask } from '../../adapter/api-client';
import { toApiError } from '../../adapter/api-error';
import { t } from '../../i18n';
import { LoadingState } from '../states/loading-state';
import { EmptyState } from '../states/empty-state';
import { ErrorPanel } from '../states/error-panel';
import { palette, radius, fontSize, fontWeight, space, elevation } from '../../tokens/access';

export function LeadDashboard(): JSX.Element {
  const [tasks, setTasks] = useState<DashboardTask[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [auto, setAuto] = useState(true);
  const [initialLoaded, setInitialLoaded] = useState(false);

  async function refresh(): Promise<void> {
    setError(null);
    try {
      setTasks(await listDashboardTasks());
    } catch (e) {
      setError(t(toApiError(e).i18nKey()));
    } finally {
      setInitialLoaded(true);
    }
  }

  useEffect(() => {
    void refresh();
    if (!auto) return;
    const t = setInterval(() => void refresh(), 5000);
    return () => clearInterval(t);
  }, [auto]);

  // 状態ごとに集計
  const counts = tasks.reduce<Record<string, number>>((acc, t) => {
    acc[t.state] = (acc[t.state] ?? 0) + 1;
    return acc;
  }, {});

  // 状態色は意味の運搬手段。各状態を意味色トークンに割当てる
  const stateColor = (s: string): string => {
    switch (s) {
      case 'Idle': return palette.fgMuted;
      case 'Ready': return palette.info.default;
      case 'Running': return palette.success.default;
      case 'Suspended': return palette.warning.default;
      case 'Exception': return palette.andon[4]; // alert (橙)
      case 'Completed': return palette.info.strong;
      case 'Failed':
      case 'Aborted': return palette.danger.default;
      default: return palette.fg;
    }
  };

  return (
    <div style={{ padding: space[5], background: palette.bg, color: palette.fg }}>
      <h1>📊 班長監視ダッシュボード</h1>
      <p style={{ color: palette.fgMuted }}>5 秒ごとに自動更新（§3.2.1.2 認知負荷を抑える段階的開示）</p>

      {error && (
        <div style={{ marginBottom: space[3] }}>
          <ErrorPanel
            message={error}
            onRetry={() => void refresh()}
            onDismiss={() => setError(null)}
          />
        </div>
      )}

      {!initialLoaded && !error && <LoadingState label={t('state_label.loading_dashboard')} />}

      <section style={{ display: 'flex', gap: space[2], marginBottom: space[4] }}>
        <button type="button" onClick={() => setAuto(!auto)} style={{ padding: `${space[2]} ${space[4]}`, background: auto ? palette.success.default : palette.fgMuted, color: palette.white, border: 'none', borderRadius: radius.small, cursor: 'pointer' }}>
          {auto ? '⏸ 自動更新 ON' : '▶ 自動更新 OFF'}
        </button>
        <button type="button" onClick={() => void refresh()} style={{ padding: `${space[2]} ${space[4]}`, background: palette.info.default, color: palette.white, border: 'none', borderRadius: radius.small, cursor: 'pointer' }}>
          🔄 今すぐ更新
        </button>
      </section>

      {/* 状態サマリ */}
      <section style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: space[3], marginBottom: space[4] }}>
        {Object.entries(counts).map(([s, n]) => (
          <div key={s} style={{ background: palette.surface, padding: space[3], borderRadius: radius.medium, borderTop: `4px solid ${stateColor(s)}`, boxShadow: elevation[1] }}>
            <div style={{ fontSize: fontSize.caption, color: palette.fgMuted }}>{s}</div>
            <div style={{ fontSize: fontSize.display, fontWeight: fontWeight.bold, color: stateColor(s) }}>{n}</div>
          </div>
        ))}
      </section>

      {/* 詳細リスト */}
      <section style={{ background: palette.surface, padding: space[4], borderRadius: radius.medium, boxShadow: elevation[1] }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: fontSize.caption }}>
          <thead>
            <tr style={{ background: palette.surfaceAlt }}>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>タスク</th>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>状態</th>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>端末</th>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>担当</th>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>現在ステップ</th>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>更新</th>
            </tr>
          </thead>
          <tbody>
            {tasks.map((t) => (
              <tr key={t.id}>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}>{t.title ?? t.id}</td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}>
                  <span style={{ display: 'inline-block', padding: `2px ${space[2]}`, background: stateColor(t.state), color: palette.white, borderRadius: radius.small, fontSize: fontSize.caption }}>
                    {t.state}
                  </span>
                </td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}><code>{t.device_id}</code></td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}>{t.responsible_user ?? '—'}</td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}>{t.current_step_id ?? '—'}</td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}`, whiteSpace: 'nowrap', color: palette.fgMuted }}>
                  {new Date(t.updated_at).toLocaleString()}
                </td>
              </tr>
            ))}
            {tasks.length === 0 && initialLoaded && !error && (
              <tr>
                <td colSpan={6} style={{ padding: 0 }}>
                  <EmptyState icon="📊" title={t('state_label.no_dashboard_title')} inline />
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </section>
    </div>
  );
}
